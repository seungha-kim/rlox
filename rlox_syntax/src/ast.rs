use crate::syntax_node::*;
use crate::token::TokenKind;
use std::sync::Arc;

#[derive(Debug)]
pub enum Statement {
    Expression(Ptr<statement::Expression>),
    Print(Ptr<statement::Print>),
    VariableDecl(Ptr<statement::VariableDecl>),
    Block(Ptr<statement::Block>),
    If(Ptr<statement::If>),
    While(Ptr<statement::While>),
    Function(Ptr<statement::Function>),
    Return(Ptr<statement::Return>),
}

pub mod statement {
    use super::*;
    use std::sync::{Mutex, RwLock};

    #[syntax_node(Statement::Expression)]
    #[derive(Debug)]
    pub struct Expression {
        pub id: usize,
        pub expr: Expr,
    }

    #[syntax_node(Statement::Print)]
    #[derive(Debug)]
    pub struct Print {
        pub id: usize,
        pub expr: Expr,
    }

    #[syntax_node(Statement::VariableDecl)]
    #[derive(Debug)]
    pub struct VariableDecl {
        pub id: usize,
        pub name: String,
        pub expr: Option<Expr>,
    }

    #[syntax_node(Statement::Block)]
    #[derive(Debug)]
    pub struct Block {
        pub id: usize,
        pub statements: Vec<Statement>,
    }

    #[syntax_node(Statement::Function)]
    #[derive(Debug)]
    pub struct Function {
        pub id: usize,
        pub name: String,
        pub params: Vec<String>,
        pub body: Arc<RwLock<Statement>>,
    }

    #[syntax_node(Statement::If)]
    #[derive(Debug)]
    pub struct If {
        pub id: usize,
        pub condition: Expr,
        pub then_branch: Statement,
        pub else_branch: Option<Statement>,
    }

    #[syntax_node(Statement::While)]
    #[derive(Debug)]
    pub struct While {
        pub id: usize,
        pub condition: Expr,
        pub body: Statement,
    }

    #[syntax_node(Statement::Return)]
    #[derive(Debug)]
    pub struct Return {
        pub id: usize,
        pub value: Option<Expr>,
    }
}

pub mod expr {
    use super::*;

    #[syntax_node(Expr::Binary)]
    #[derive(Debug)]
    pub struct Binary {
        pub id: usize,
        pub left: Expr,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Grouping)]
    #[derive(Debug)]
    pub struct Grouping {
        pub id: usize,
        pub expr: Expr,
    }

    #[syntax_node(Expr::Literal)]
    #[derive(Debug)]
    pub struct Literal {
        pub id: usize,
        pub literal: super::Literal,
    }

    #[syntax_node(Expr::Unary)]
    #[derive(Debug)]
    pub struct Unary {
        pub id: usize,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Variable)]
    #[derive(Debug)]
    pub struct Variable {
        pub id: usize,
        pub name: String,
        // How many levels should be escalated to resolve this variable
        pub resolution: usize,
    }

    #[syntax_node(Expr::Assign)]
    #[derive(Debug)]
    pub struct Assign {
        pub id: usize,
        pub name: String,
        pub value: Expr,
        // TODO: There are more things to which values can be assigned
        // e.g. instance.method()
        // How many levels should be escalated to resolve this variable
        pub resolution: usize,
    }

    #[syntax_node(Expr::Logical)]
    #[derive(Debug)]
    pub struct Logical {
        pub id: usize,
        pub left: Expr,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Call)]
    #[derive(Debug)]
    pub struct Call {
        pub id: usize,
        pub callee: Expr,
        pub arguments: Vec<Expr>,
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary(Box<expr::Binary>),
    Grouping(Box<expr::Grouping>),
    Literal(Box<expr::Literal>),
    Unary(Box<expr::Unary>),
    Variable(Box<expr::Variable>),
    Assign(Box<expr::Assign>),
    Logical(Box<expr::Logical>),
    Call(Box<expr::Call>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}
