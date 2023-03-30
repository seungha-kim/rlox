use crate::token::TokenKind;
use std::sync::Arc;

#[derive(Debug)]
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
    Function {
        name: String,
        params: Vec<String>,
        body: Arc<Statement>,
    },
    Return(Option<Box<Expr>>),
}

#[derive(Debug)]
pub enum Expr {
    BinaryExpr {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
    },
    GroupingExpr(Box<Expr>),
    LiteralExpr(Literal),
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
    Call {
        callee: Box<Expr>,
        arguments: Vec<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

pub use Expr::*;