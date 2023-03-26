use crate::ast::Expr;
use crate::token::TokenKind;
use crate::value::Value;
use anyhow::bail;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
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
