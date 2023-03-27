use crate::token::TokenKind;
use crate::value::Value;

pub enum Statement {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Variable {
        id: String,
        expr: Option<Box<Expr>>,
    },
    Block(Vec<Box<Statement>>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Statement>,
    },
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
    // Short-circuit
    Logical {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
    },
}

pub use Expr::*;
