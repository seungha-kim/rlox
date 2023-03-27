use crate::ast::{Expr, Statement};
use crate::token::TokenKind;
use crate::value::Value;
use anyhow::bail;
use std::collections::HashMap;

pub struct Interpreter<'p> {
    environment_stack: Vec<HashMap<String, Value>>,
    printer: &'p mut dyn Printer,
}

impl<'p> Interpreter<'p> {
    pub fn new(printer: &'p mut dyn Printer) -> Self {
        let environment_stack = vec![HashMap::new()];
        Self {
            environment_stack,
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
                self.define_variable(id, value)?;
            }
            Statement::Block(ss) => {
                self.push_environment(); // TODO: scopeguard
                for s in ss {
                    self.evaluate_stmt(s)?;
                }
                self.pop_environment();
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
            Expr::Variable(id) => self.get_variable(id)?.clone(),
            Expr::Assign(name, expr) => {
                let value = self.evaluate_expr(expr)?;
                self.assign_variable(name, value.clone())?;
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
        self.environment_stack.push(HashMap::new());
    }

    pub fn pop_environment(&mut self) {
        self.environment_stack.pop().unwrap();
    }

    pub fn define_variable(&mut self, name: &str, value: Value) -> anyhow::Result<()> {
        if let Some(env) = self.environment_stack.last_mut() {
            env.insert(name.to_string(), value);
            Ok(())
        } else {
            bail!("No environment found");
        }
    }

    pub fn assign_variable(&mut self, name: &str, value: Value) -> anyhow::Result<()> {
        for env in self.environment_stack.iter_mut().rev() {
            if env.contains_key(name) {
                env.insert(name.to_string(), value);
                return Ok(());
            }
        }

        bail!("Undefined variable '{name}'.");
    }

    pub fn get_variable(&self, name: &str) -> anyhow::Result<&Value> {
        for env in self.environment_stack.iter().rev() {
            if let Some(v) = env.get(name) {
                return Ok(v);
            }
        }

        bail!("No key '{}' in any environment", name);
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

    #[test]
    fn test_source() -> anyhow::Result<()> {
        let source = r"
var a = 1;
var b = 2;
{
    var a = 3;
    var b = 4;
    print a + b;
}
        ";
        let mut printer = TestPrinter::new();
        let tokens = Scanner::new(source).scan_tokens()?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;
        let mut interpreter = Interpreter::new(&mut printer);
        for s in statements {
            interpreter.evaluate_stmt(&s)?;
        }
        assert_eq!(vec!["Number(7.0)"], printer.messages);
        Ok(())
    }
}
