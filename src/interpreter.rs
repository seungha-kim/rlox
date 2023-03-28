use crate::ast::{Expr, Statement};
use crate::func;
use crate::func::{Callable, FunctionObject};
use crate::token::TokenKind;
use crate::value::{Object, Value};
use anyhow::bail;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Environment {
    parent: Option<Arc<Mutex<Environment>>>,
    variables: HashMap<String, Value>,
}

impl Environment {
    fn new(parent: Option<Arc<Mutex<Environment>>>) -> Environment {
        Self {
            parent,
            variables: HashMap::new(),
        }
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
    pub environment: Arc<Mutex<Environment>>,
    printer: &'p mut dyn Printer,
}

impl<'p> Interpreter<'p> {
    pub fn new(printer: &'p mut dyn Printer) -> Self {
        let mut global_env = Environment {
            parent: None,
            variables: HashMap::new(),
        };

        for f in func::impls::ALL_FUNCS {
            global_env
                .variables
                .insert(f.name.to_owned(), Value::NativeFunction(f));
        }

        Self {
            environment: Arc::new(Mutex::new(global_env)),
            printer,
        }
    }

    pub fn evaluate_stmt(&mut self, stmt: &Statement) -> anyhow::Result<()> {
        match stmt {
            Statement::Expression(expr) => {
                self.evaluate_expr(expr)?;
            }
            Statement::Print(expr) => {
                let value = self.evaluate_expr(expr)?;
                self.printer.print(&format!("{:?}", value));
            }
            Statement::Variable { id, expr } => {
                let value = if let Some(expr) = expr {
                    self.evaluate_expr(expr)?
                } else {
                    Value::Nil
                };
                self.environment
                    .lock()
                    .unwrap()
                    .define_variable(id, value)?;
            }
            Statement::Block(ss) => {
                self.push_environment();
                let mut zelf = scopeguard::guard(self, |zelf| {
                    zelf.pop_environment();
                });

                for s in ss {
                    zelf.evaluate_stmt(s)?;
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate_expr(condition)?;
                if Self::is_truthy(&condition) {
                    self.evaluate_stmt(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.evaluate_stmt(else_branch)?;
                }
            }
            Statement::While { condition, body } => {
                while Self::is_truthy(&self.evaluate_expr(condition)?) {
                    self.evaluate_stmt(body)?;
                }
            }
            Statement::Function { name, params, body } => {
                let closure = self.environment.clone();
                self.environment.lock().unwrap().define_variable(
                    name,
                    Value::FunctionObject(Object::new(FunctionObject {
                        name: name.to_owned(),
                        parameters: params.to_owned(),
                        body: body.clone(),
                        closure,
                    })),
                )?;
            }
            Statement::Return(expr) => {
                let value = if let Some(expr) = expr {
                    self.evaluate_expr(expr)?
                } else {
                    Value::Nil
                };
                // Rewind stack until call statement, using this dirty way!
                return Err(ReturnError(value).into());
            }
        }
        Ok(())
    }

    pub fn evaluate_expr(&mut self, expr: &Expr) -> anyhow::Result<Value> {
        let result = match expr {
            Expr::BinaryExpr {
                left,
                operator,
                right,
            } => {
                let lval = self.evaluate_expr(left)?;
                let rval = self.evaluate_expr(right)?;

                match (lval, operator, rval) {
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
            Expr::GroupingExpr(expr) => self.evaluate_expr(expr)?,
            Expr::LiteralExpr(lit) => lit.clone(),
            Expr::UnaryExpr { operator, right } => {
                let rval = self.evaluate_expr(right)?;
                match (operator, rval) {
                    (TokenKind::Minus, Value::Number(n)) => Value::Number(-n),
                    (TokenKind::Bang, rval) => Value::Boolean(Self::is_truthy(&rval)),
                    (op, r) => {
                        bail!("Unsupported unary operator: {:?}{:?}", op, r);
                    }
                }
            }
            Expr::Variable(id) => self.environment.lock().unwrap().get_variable(id)?,
            Expr::Assign(name, expr) => {
                let value = self.evaluate_expr(expr)?;
                self.environment
                    .lock()
                    .unwrap()
                    .assign_variable(name, &value)?;
                value
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate_expr(left)?;
                match operator {
                    TokenKind::Or if Self::is_truthy(&left) => left,
                    TokenKind::And if !Self::is_truthy(&left) => left,
                    _ => self.evaluate_expr(right)?,
                }
            }
            Expr::Call { callee, arguments } => {
                let callable = self.evaluate_expr(callee)?;
                let mut arg_values = Vec::new();
                for arg in arguments {
                    arg_values.push(self.evaluate_expr(arg)?);
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

    pub fn push_environment(&mut self) {
        self.environment = Arc::new(Mutex::new(Environment::new(Some(self.environment.clone()))));
    }

    pub fn pop_environment(&mut self) {
        let parent = self.environment.lock().unwrap().parent.clone().unwrap();
        self.environment = parent;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    struct TestPrinter {
        messages: Vec<String>,
    }

    impl TestPrinter {
        fn new() -> Self {
            Self {
                messages: Vec::new(),
            }
        }
    }

    impl Printer for TestPrinter {
        fn print(&mut self, message: &str) {
            self.messages.push(message.to_owned());
        }
    }

    fn print_from(source: &str) -> anyhow::Result<Vec<String>> {
        let mut printer = TestPrinter::new();
        let tokens = Scanner::new(source).scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let mut interpreter = Interpreter::new(&mut printer);
        for s in statements {
            interpreter.evaluate_stmt(&s)?;
        }
        Ok(printer.messages)
    }

    #[test]
    fn test_source() {
        let source = r"
var a = 1;
var b = 2;
{
    var a = 3;
    var b = 4;
    print a + b;
}
        ";

        assert_eq!(vec!["Number(7.0)"], print_from(source).unwrap());
    }

    #[test]
    fn test_closure() {
        let source = r"
fun counter() {
    var c = 0;
    fun inc() {
        c = c + 1;
        return c;
    }
    return inc;
}
var c = counter();
c();
print c();
";
        assert_eq!(vec!["Number(2.0)"], print_from(source).unwrap());
    }
}
