use crate::syntax_node::*;
use crate::token::TokenKind;
use std::sync::Arc;

#[derive(Debug)]
pub enum Statement {
    Expression(Ptr<statement::Expression>),
    Print(Ptr<statement::Print>),
    Variable(Ptr<statement::Variable>),
    Block(Ptr<statement::Block>),
    If(Ptr<statement::If>),
    While(Ptr<statement::While>),
    Function(Ptr<statement::Function>),
    Return(Ptr<statement::Return>),
}

pub mod statement {
    use super::*;

    #[syntax_node(Statement::Expression)]
    #[derive(Debug)]
    pub struct Expression {
        pub id: Uuid,
        pub expr: Expr,
    }

    #[syntax_node(Statement::Print)]
    #[derive(Debug)]
    pub struct Print {
        pub id: Uuid,
        pub expr: Expr,
    }

    #[syntax_node(Statement::Variable)]
    #[derive(Debug)]
    pub struct Variable {
        pub id: Uuid,
        pub name: String,
        pub expr: Option<Expr>,
    }

    #[syntax_node(Statement::Block)]
    #[derive(Debug)]
    pub struct Block {
        pub id: Uuid,
        pub statements: Vec<Statement>,
    }

    #[syntax_node(Statement::Function)]
    #[derive(Debug)]
    pub struct Function {
        pub id: Uuid,
        pub name: String,
        pub params: Vec<String>,
        pub body: Arc<Statement>,
    }

    #[syntax_node(Statement::If)]
    #[derive(Debug)]
    pub struct If {
        pub id: Uuid,
        pub condition: Expr,
        pub then_branch: Statement,
        pub else_branch: Option<Statement>,
    }

    #[syntax_node(Statement::While)]
    #[derive(Debug)]
    pub struct While {
        pub id: Uuid,
        pub condition: Expr,
        pub body: Statement,
    }

    #[syntax_node(Statement::Return)]
    #[derive(Debug)]
    pub struct Return {
        pub id: Uuid,
        pub value: Option<Expr>,
    }
}

pub mod expr {
    use super::*;

    #[syntax_node(Expr::Binary)]
    #[derive(Debug)]
    pub struct Binary {
        pub id: Uuid,
        pub left: Expr,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Grouping)]
    #[derive(Debug)]
    pub struct Grouping {
        pub id: Uuid,
        pub expr: Expr,
    }

    #[syntax_node(Expr::Literal)]
    #[derive(Debug)]
    pub struct Literal {
        pub id: Uuid,
        pub literal: super::Literal,
    }

    #[syntax_node(Expr::Unary)]
    #[derive(Debug)]
    pub struct Unary {
        pub id: Uuid,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Variable)]
    #[derive(Debug)]
    pub struct Variable {
        pub id: Uuid,
        pub name: String,
    }

    #[syntax_node(Expr::Assign)]
    #[derive(Debug)]
    pub struct Assign {
        pub id: Uuid,
        pub name: String,
        pub value: Expr,
    }

    #[syntax_node(Expr::Logical)]
    #[derive(Debug)]
    pub struct Logical {
        pub id: Uuid,
        pub left: Expr,
        pub operator: TokenKind,
        pub right: Expr,
    }

    #[syntax_node(Expr::Call)]
    #[derive(Debug)]
    pub struct Call {
        pub id: Uuid,
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
