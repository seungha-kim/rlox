use crate::ast;
use crate::token::{Token, TokenKind};
use crate::value::Value;
use anyhow::bail;

type ParseExprResult = anyhow::Result<Box<ast::Expr>>;
type ParseStmtResult = anyhow::Result<ast::Statement>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> anyhow::Result<Vec<ast::Statement>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_declaration()?);
        }
        Ok(statements)
    }

    /// declaration    → varDecl
    //                 | statement ;
    fn parse_declaration(&mut self) -> ParseStmtResult {
        if self.match_(&[TokenKind::Var]) {
            self.parse_variable_decl()
        } else {
            self.parse_statement()
        }
    }

    fn parse_variable_decl(&mut self) -> ParseStmtResult {
        let id = self
            .consume(&TokenKind::Identifier, "Expect variable name.")?
            .lexeme
            .to_owned();
        let expr = if self.match_(&[TokenKind::Equal]) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(ast::Statement::Variable { id, expr })
    }

    pub fn parse_statement(&mut self) -> ParseStmtResult {
        if self.match_(&[TokenKind::Print]) {
            self.parse_print_statement()
        } else if self.match_(&[TokenKind::LeftBrace]) {
            self.parse_block_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    pub fn parse_print_statement(&mut self) -> ParseStmtResult {
        let value = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(ast::Statement::Print(value))
    }

    pub fn parse_expression_statement(&mut self) -> ParseStmtResult {
        let value = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(ast::Statement::Expression(value))
    }

    pub fn parse_block_statement(&mut self) -> ParseStmtResult {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(Box::new(self.parse_declaration()?));
        }
        self.consume(&TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(ast::Statement::Block(statements))
    }

    /*
    Each method for parsing a grammar rule produces a syntax tree
    for that rule and returns it to the caller.

    When the body of the rule contains a nonterminal
    (a reference to another rule)
    we call that other rule’s method.

    This is why left recursion is problematic for recursive descent.
    The function for a left-recursive rule immediately calls itself,
    which calls itself again, and so on, until the parser hits a stack overflow and dies.

    program        → declaration* EOF ;

    declaration    → varDecl
                   | statement ;

    varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;

    statement      → exprStmt
                   | printStmt
                   | block ;

    exprStmt       → expression ";" ;
    printStmt      → "print" expression ";" ;
    block          → "{" declaration* "}" ;

    expression     → assignment ;
    assignment     → IDENTIFIER "=" assignment
                   | equality ;
    equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term           → factor ( ( "-" | "+" ) factor )* ;
    factor         → unary ( ( "/" | "*" ) unary )* ;
    unary          → ( "!" | "-" ) unary
                   | primary ;
    primary        → NUMBER | STRING | "true" | "false" | "nil"
                   | "(" expression ")"
                   | IDENTIFIER ;
    */

    /// expression     → equality ;
    fn parse_expression(&mut self) -> ParseExprResult {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseExprResult {
        let expr = self.parse_equality()?;

        if self.match_(&[TokenKind::Equal]) {
            let equals = self.previous().clone();
            // Assign operator is right-associative
            let value = self.parse_assignment()?;

            if let ast::Variable(name) = *expr {
                return Ok(Box::new(ast::Expr::Assign(name, value)));
            }

            return Self::error(&equals, "Invalid assignment target.");
        }

        return Ok(expr);
    }

    /// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> ParseExprResult {
        let mut expr = self.parse_comparison()?;

        while self.match_(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().kind;
            let right = self.parse_comparison()?;
            expr = Box::new(ast::BinaryExpr {
                left: expr,
                operator,
                right,
            });
        }

        return Ok(expr);
    }

    /// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn parse_comparison(&mut self) -> ParseExprResult {
        let mut expr = self.parse_term()?;

        while self.match_(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
        ]) {
            let operator = self.previous().kind;
            let right = self.parse_term()?;
            expr = Box::new(ast::BinaryExpr {
                left: expr,
                operator,
                right,
            });
        }

        return Ok(expr);
    }

    /// term           → factor ( ( "-" | "+" ) factor )* ;
    fn parse_term(&mut self) -> ParseExprResult {
        let mut expr = self.parse_factor()?;

        while self.match_(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().kind;
            let right = self.parse_factor()?;
            expr = Box::new(ast::BinaryExpr {
                left: expr,
                operator,
                right,
            });
        }

        return Ok(expr);
    }

    /// factor         → unary ( ( "/" | "*" ) unary )* ;
    fn parse_factor(&mut self) -> ParseExprResult {
        let mut expr = self.parse_unary()?;

        while self.match_(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().kind;
            let right = self.parse_unary()?;
            expr = Box::new(ast::BinaryExpr {
                left: expr,
                operator,
                right,
            });
        }

        return Ok(expr);
    }

    /// unary          → ( "!" | "-" ) unary
    //                 | primary ;
    fn parse_unary(&mut self) -> ParseExprResult {
        if self.match_(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().kind;
            let right = self.parse_unary()?;
            Ok(Box::new(ast::UnaryExpr { operator, right }))
        } else {
            self.parse_primary()
        }
    }

    /// primary        → NUMBER | STRING | "true" | "false" | "nil"
    //                 | "(" expression ")" ;
    fn parse_primary(&mut self) -> ParseExprResult {
        let expr: Box<ast::Expr> = if self.match_(&[TokenKind::Number, TokenKind::String]) {
            Box::new(ast::LiteralExpr(self.previous().literal.clone().unwrap()))
        } else if self.match_(&[TokenKind::True]) {
            Box::new(ast::LiteralExpr(Value::Boolean(true)))
        } else if self.match_(&[TokenKind::False]) {
            Box::new(ast::LiteralExpr(Value::Boolean(false)))
        } else if self.match_(&[TokenKind::Nil]) {
            Box::new(ast::LiteralExpr(Value::Nil))
        } else if self.match_(&[TokenKind::LeftParen]) {
            let expr = self.parse_expression()?;
            self.consume(&TokenKind::RightParen, "Expect ')' after expression")?;
            Box::new(ast::GroupingExpr(expr))
        } else if self.match_(&[TokenKind::Identifier]) {
            Box::new(ast::Variable(self.previous().lexeme.to_owned()))
        } else {
            return Self::error(self.peek(), "Expect expression.");
        };

        Ok(expr)
    }

    fn error<T>(token: &Token, message: &str) -> anyhow::Result<T> {
        bail!("Line {}, at '{}', {}", token.line, token.lexeme, message)
    }

    fn match_(&mut self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.check(k) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        return self.peek().kind == *kind;
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: &TokenKind, message: &str) -> anyhow::Result<&Token> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        Self::error(self.peek(), message)
    }

    // TODO: synchronize
}
