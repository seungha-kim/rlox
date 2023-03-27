use crate::token::TokenKind;
use crate::value::Value;

pub enum Statement {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Variable { id: String, expr: Option<Box<Expr>> },
    Block(Vec<Box<Statement>>),
}

pub enum Expr {
    BinaryExpr {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
    },
    GroupingExpr(Box<Expr>),
    LiteralExpr(Value),
    UnaryExpr {
        operator: TokenKind,
        right: Box<Expr>,
    },
    Variable(String),
    Assign(String, Box<Expr>),
}

pub use Expr::*;
