use crate::func;
use crate::func::{Callable, FunctionObject};
use crate::value::{Object, Value};
use anyhow::bail;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};
use syntax_tree::{Expr, Statement, TokenKind};

#[derive(Debug)]
pub struct Environment {
    parent: Option<Arc<Mutex<Environment>>>,
    variables: HashMap<String, Value>,
}

pub type EnvironmentPtr = Arc<Mutex<Environment>>;

impl Environment {
    pub fn new_ptr(parent: EnvironmentPtr) -> EnvironmentPtr {
        Arc::new(Mutex::new(Self::new(Some(parent), false)))
    }

    pub fn new_globals_ptr() -> EnvironmentPtr {
        Arc::new(Mutex::new(Self::new(None, true)))
    }

    fn new(parent: Option<EnvironmentPtr>, fill_global: bool) -> Environment {
        let mut zelf = Self {
            parent,
            variables: HashMap::new(),
        };

        if fill_global {
            for f in func::impls::ALL_FUNCS {
                zelf.variables
                    .insert(f.name.to_owned(), Value::NativeFunction(f));
            }
        }

        zelf
    }

    pub fn get_variable(&self, name: &str) -> anyhow::Result<Value> {
        let value = if let Some(value) = self.variables.get(name) {
            value.clone()
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().get_variable(name)?
        } else {
            bail!("Undefined variable '{}'.", name);
        };

        Ok(value)
    }

    pub fn define_variable(&mut self, name: &str, value: Value) -> anyhow::Result<()> {
        self.variables.insert(name.to_string(), value);
        Ok(())
    }

    pub fn assign_variable(&mut self, name: &str, value: &Value) -> anyhow::Result<()> {
        // TODO: fun counter() { var c = 1; fun inc() { c = c + 1; return c; } return inc; }
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value.clone());
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().assign_variable(name, value)?;
        } else {
            bail!("Undefined variable '{name}'.");
        }
        Ok(())
    }
}

pub struct Interpreter<'p> {
    printer: &'p mut dyn Printer,
}

impl<'p> Interpreter<'p> {
    pub fn new(printer: &'p mut dyn Printer) -> Self {
        Self { printer }
    }

    pub fn evaluate_stmt(
        &mut self,
        environment: &Arc<Mutex<Environment>>,
        stmt: &Statement,
    ) -> anyhow::Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate_expr(environment, &expr.expr)?;
            }
            Statement::Print(expr) => {
                let value = self.evaluate_expr(environment, &expr.expr)?;
                self.printer.print(&format!("{:?}", value));
            }
            Statement::Variable(var) => {
                let value = if let Some(expr) = &var.expr {
                    self.evaluate_expr(environment, &expr)?
                } else {
                    Value::Nil
                };
                environment
                    .lock()
                    .unwrap()
                    .define_variable(&var.name, value)?;
            }
            Statement::Block(block) => {
                let environment = Environment::new_ptr(environment.clone());

                for s in &block.statements {
                    self.evaluate_stmt(&environment, s)?;
                }
            }
            Statement::If(s) => {
                let condition = self.evaluate_expr(environment, &s.condition)?;
                if Self::is_truthy(&condition) {
                    self.evaluate_stmt(environment, &s.then_branch)?;
                } else if let Some(else_branch) = &s.else_branch {
                    self.evaluate_stmt(environment, else_branch)?;
                }
            }
            Statement::While(s) => {
                while Self::is_truthy(&self.evaluate_expr(environment, &s.condition)?) {
                    self.evaluate_stmt(environment, &s.body)?;
                }
            }
            Statement::Function(s) => {
                // identifier resolution 을 별도 pass 없이 여기에서 해도 되지 않나
                let closure = environment.clone();
                environment.lock().unwrap().define_variable(
                    &s.name,
                    Value::FunctionObject(Object::new(FunctionObject {
                        name: s.name.to_owned(),
                        parameters: s.params.to_owned(),
                        body: s.body.clone(),
                        closure,
                    })),
                )?;
            }
            Statement::Return(expr) => {
                let value = if let Some(expr) = &expr.value {
                    self.evaluate_expr(environment, expr)?
                } else {
                    Value::Nil
                };
                // Rewind stack until call statement, using this dirty way!
                return Err(ReturnError(value).into());
            }
        }
        Ok(())
    }

    pub fn evaluate_expr(
        &mut self,
        environment: &EnvironmentPtr,
        expr: &Expr,
    ) -> anyhow::Result<Value> {
        let result = match expr {
            Expr::Binary(expr) => {
                let lval = self.evaluate_expr(environment, &expr.left)?;
                let rval = self.evaluate_expr(environment, &expr.right)?;

                match (lval, expr.operator, rval) {
                    (Value::Number(l), TokenKind::Plus, Value::Number(r)) => Value::Number(l + r),
                    (Value::String(mut l), TokenKind::Plus, Value::String(r)) => {
                        l.push_str(&r);
                        Value::String(l)
                    }
                    (Value::Number(l), TokenKind::Minus, Value::Number(r)) => Value::Number(l - r),
                    (Value::Number(l), TokenKind::Star, Value::Number(r)) => Value::Number(l * r),
                    (Value::Number(l), TokenKind::Slash, Value::Number(r)) => {
                        if r == 0.0 {
                            bail!("Divided by zero");
                        }
                        Value::Number(l / r)
                    }

                    (Value::Number(l), TokenKind::Greater, Value::Number(r)) => {
                        Value::Boolean(l > r)
                    }
                    (Value::Number(l), TokenKind::GreaterEqual, Value::Number(r)) => {
                        Value::Boolean(l >= r)
                    }
                    (Value::Number(l), TokenKind::Less, Value::Number(r)) => Value::Boolean(l < r),
                    (Value::Number(l), TokenKind::LessEqual, Value::Number(r)) => {
                        Value::Boolean(l <= r)
                    }
                    (lval, TokenKind::EqualEqual, rval) => Value::Boolean(lval == rval),
                    (lval, TokenKind::BangEqual, rval) => Value::Boolean(lval != rval),
                    (l, op, r) => {
                        bail!("Unsupported binary operator: {:?} {:?} {:?}", l, op, r);
                    }
                }
            }
            Expr::Grouping(expr) => self.evaluate_expr(environment, &expr.expr)?,
            Expr::Literal(expr) => expr.literal.clone().into(),
            Expr::Unary(expr) => {
                let rval = self.evaluate_expr(environment, &expr.right)?;
                match (expr.operator, rval) {
                    (TokenKind::Minus, Value::Number(n)) => Value::Number(-n),
                    (TokenKind::Bang, rval) => Value::Boolean(Self::is_truthy(&rval)),
                    (op, r) => {
                        bail!("Unsupported unary operator: {:?}{:?}", op, r);
                    }
                }
            }
            Expr::Variable(expr) => environment.lock().unwrap().get_variable(&expr.name)?,
            Expr::Assign(expr) => {
                let value = self.evaluate_expr(environment, &expr.value)?;
                environment
                    .lock()
                    .unwrap()
                    .assign_variable(&expr.name, &value)?;
                value
            }
            Expr::Logical(expr) => {
                let left = self.evaluate_expr(environment, &expr.left)?;
                match expr.operator {
                    TokenKind::Or if Self::is_truthy(&left) => left,
                    TokenKind::And if !Self::is_truthy(&left) => left,
                    _ => self.evaluate_expr(environment, &expr.right)?,
                }
            }
            Expr::Call(expr) => {
                let callable = self.evaluate_expr(environment, &expr.callee)?;
                let mut arg_values = Vec::new();
                for arg in &expr.arguments {
                    arg_values.push(self.evaluate_expr(environment, arg)?);
                }

                let result = if let Value::NativeFunction(f) = callable {
                    f.call(self, &arg_values)
                } else if let Value::FunctionObject(f) = callable {
                    f.call(self, &arg_values)
                } else {
                    bail!("Only function types can be called.");
                };

                match result {
                    Ok(value) => value,
                    Err(e) => match e.downcast::<ReturnError>() {
                        Ok(re) => re.0,
                        Err(e) => {
                            return Err(e);
                        }
                    },
                }
            }
        };

        Ok(result)
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

pub trait Printer {
    fn print(&mut self, message: &str);
}

pub struct StdOutPrinter;

impl Printer for StdOutPrinter {
    fn print(&mut self, message: &str) {
        println!("{}", message);
    }
}

#[derive(Debug)]
struct ReturnError(Value);

impl std::fmt::Display for ReturnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ReturnError {}
