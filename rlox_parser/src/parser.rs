use anyhow::bail;
use rlox_syntax::*;
use std::sync::{Arc, RwLock};

type ParseExprResult = anyhow::Result<Expr>;
type ParseStmtResult = anyhow::Result<Statement>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> anyhow::Result<Vec<Statement>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_declaration()?);
        }
        Ok(statements)
    }

    /// declaration    → funDecl
    //                 | varDecl
    //                 | statement ;
    fn parse_declaration(&mut self) -> ParseStmtResult {
        if self.match_(&[TokenKind::Var]) {
            self.parse_variable_decl()
        } else if self.match_(&[TokenKind::Fun]) {
            self.parse_function_decl()
        } else {
            self.parse_statement()
        }
    }

    fn parse_variable_decl(&mut self) -> ParseStmtResult {
        let name = self
            .consume(&TokenKind::Identifier, "Expect variable name.")?
            .lexeme
            .to_owned();
        let expr = if self.match_(&[TokenKind::Equal]) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;

        Ok(statement::VariableDecl::new_wrapped(name, expr))
    }

    fn parse_function_decl(&mut self) -> ParseStmtResult {
        // TODO: method
        let name = self
            .consume(&TokenKind::Identifier, "Expect function name.")?
            .lexeme
            .to_owned();
        self.consume(&TokenKind::LeftParen, "Expect '(' after function name.")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                if params.len() >= 255 {
                    Self::error(self.peek(), "Can't have more than 255 parameters")?;
                }

                params.push(
                    self.consume(&TokenKind::Identifier, "Expect parameter name.")?
                        .lexeme
                        .to_owned(),
                );

                if !self.match_(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RightParen, "Expect ')' after parameters.")?;

        self.consume(&TokenKind::LeftBrace, "Expect '{' before function body.")?;

        let body = Arc::new(RwLock::new(self.parse_block_statement()?));
        Ok(statement::Function::new_wrapped(name, params, body))
    }

    fn parse_statement(&mut self) -> ParseStmtResult {
        if self.match_(&[TokenKind::Print]) {
            self.parse_print_statement()
        } else if self.match_(&[TokenKind::LeftBrace]) {
            self.parse_block_statement()
        } else if self.match_(&[TokenKind::If]) {
            self.parse_if_statement()
        } else if self.match_(&[TokenKind::While]) {
            self.parse_while_statement()
        } else if self.match_(&[TokenKind::For]) {
            self.parse_for_statement()
        } else if self.match_(&[TokenKind::Return]) {
            self.parse_return_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_print_statement(&mut self) -> ParseStmtResult {
        let value = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(statement::Print::new_wrapped(value))
    }

    fn parse_expression_statement(&mut self) -> ParseStmtResult {
        let value = self.parse_expression()?;
        self.consume(&TokenKind::Semicolon, "Expect ';' after value.")?;
        Ok(statement::Expression::new_wrapped(value))
    }

    fn parse_block_statement(&mut self) -> ParseStmtResult {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_declaration()?);
        }
        self.consume(&TokenKind::RightBrace, "Expect '}' after block.")?;
        Ok(statement::Block::new_wrapped(statements))
    }

    fn parse_if_statement(&mut self) -> ParseStmtResult {
        self.consume(&TokenKind::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenKind::RightParen, "Expect ')' after if condition.")?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.match_(&[TokenKind::Else]) {
            Some(self.parse_statement()?)
        } else {
            None
        };
        Ok(statement::If::new_wrapped(
            condition,
            then_branch,
            else_branch,
        ))
    }

    fn parse_while_statement(&mut self) -> ParseStmtResult {
        self.consume(&TokenKind::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.parse_expression()?;
        self.consume(&TokenKind::RightParen, "Expect ')' after condition.")?;
        let body = self.parse_statement()?;

        Ok(statement::While::new_wrapped(condition, body))
    }

    fn parse_for_statement(&mut self) -> ParseStmtResult {
        self.consume(&TokenKind::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_(&[TokenKind::Semicolon]) {
            None
        } else if self.match_(&[TokenKind::Var]) {
            Some(self.parse_variable_decl()?)
        } else {
            Some(self.parse_expression_statement()?)
        };

        let condition = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenKind::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(&TokenKind::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.parse_statement()?;

        // Desugaring
        if let Some(increment) = increment {
            body = statement::Block::new_wrapped(vec![
                body,
                statement::Expression::new_wrapped(increment),
            ]);
        }

        let condition = condition.unwrap_or(expr::Literal::new_wrapped(Literal::Boolean(true)));
        body = statement::While::new_wrapped(condition, body);

        if let Some(initializer) = initializer {
            body = statement::Block::new_wrapped(vec![initializer, body]);
        }

        Ok(body)
    }

    fn parse_return_statement(&mut self) -> ParseStmtResult {
        let mut expr = None;
        if !self.check(&TokenKind::Semicolon) {
            expr = Some(self.parse_expression()?);
        }
        self.consume(&TokenKind::Semicolon, "Expect ';' after return value.")?;
        Ok(statement::Return::new_wrapped(expr))
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

    declaration    → funDecl
                   | varDecl
                   | statement ;

    varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;

    funDecl        → "fun" function ;
    function       → IDENTIFIER "(" parameters? ")" block ;
    parameters     → IDENTIFIER ( "," IDENTIFIER )* ;

    statement      → exprStmt
                   | forStmt
                   | ifStmt
                   | printStmt
                   | returnStmt
                   | whileStmt
                   | block ;

    returnStmt     → "return" expression? ";" ;
    forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
                     expression? ";"
                     expression? ")" statement ;
    whileStmt      → "while" "(" expression ")" statement ;
    ifStmt         → "if" "(" expression ")" statement
                   ( "else" statement )? ;
    exprStmt       → expression ";" ;
    printStmt      → "print" expression ";" ;
    block          → "{" declaration* "}" ;

    expression     → assignment ;
    assignment     → IDENTIFIER "=" assignment
                   | logic_or ;
    logic_or       → logic_and ( "or" logic_and )* ;
    logic_and      → equality ( "and" equality )* ;
    equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term           → factor ( ( "-" | "+" ) factor )* ;
    factor         → unary ( ( "/" | "*" ) unary )* ;
    unary          → ( "!" | "-" ) unary | call ;
    call           → primary ( "(" arguments? ")" )* ;
    arguments      → expression ( "," expression )* ;
    primary        → NUMBER | STRING | "true" | "false" | "nil"
                   | "(" expression ")"
                   | IDENTIFIER ;
    */

    /// expression     → equality ;
    fn parse_expression(&mut self) -> ParseExprResult {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> ParseExprResult {
        let expr = self.parse_or()?;

        if self.match_(&[TokenKind::Equal]) {
            let equals = self.previous().clone();
            // Assign operator is right-associative
            let value = self.parse_assignment()?;

            if let Expr::Variable(var) = expr {
                return Ok(expr::Assign::new_wrapped(var.name, value, 0));
            }

            return Self::error(&equals, "Invalid assignment target.");
        }

        return Ok(expr);
    }

    fn parse_or(&mut self) -> ParseExprResult {
        let mut expr = self.parse_and()?;

        while self.match_(&[TokenKind::Or]) {
            let operator = self.previous().kind;
            let right = self.parse_and()?;
            expr = expr::Logical::new_wrapped(expr, operator, right);
        }

        Ok(expr)
    }

    fn parse_and(&mut self) -> ParseExprResult {
        let mut expr = self.parse_equality()?;

        while self.match_(&[TokenKind::And]) {
            let operator = self.previous().kind;
            let right = self.parse_equality()?;
            expr = expr::Logical::new_wrapped(expr, operator, right);
        }

        Ok(expr)
    }

    /// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    fn parse_equality(&mut self) -> ParseExprResult {
        let mut expr = self.parse_comparison()?;

        while self.match_(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().kind;
            let right = self.parse_comparison()?;

            expr = expr::Binary::new_wrapped(expr, operator, right);
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
            expr = expr::Binary::new_wrapped(expr, operator, right);
        }

        return Ok(expr);
    }

    /// term           → factor ( ( "-" | "+" ) factor )* ;
    fn parse_term(&mut self) -> ParseExprResult {
        let mut expr = self.parse_factor()?;

        while self.match_(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous().kind;
            let right = self.parse_factor()?;
            expr = expr::Binary::new_wrapped(expr, operator, right);
        }

        return Ok(expr);
    }

    /// factor         → unary ( ( "/" | "*" ) unary )* ;
    fn parse_factor(&mut self) -> ParseExprResult {
        let mut expr = self.parse_unary()?;

        while self.match_(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous().kind;
            let right = self.parse_unary()?;
            expr = expr::Binary::new_wrapped(expr, operator, right);
        }

        return Ok(expr);
    }

    /// unary          → ( "!" | "-" ) unary | call ;
    fn parse_unary(&mut self) -> ParseExprResult {
        if self.match_(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().kind;
            let right = self.parse_unary()?;
            Ok(expr::Unary::new_wrapped(operator, right))
        } else {
            self.parse_call()
        }
    }

    /// call           → primary ( "(" arguments? ")" )* ;
    /// arguments      → expression ( "," expression )* ;
    fn parse_call(&mut self) -> ParseExprResult {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_(&[TokenKind::LeftParen]) {
                let mut arguments = Vec::new();
                if !self.check(&TokenKind::RightParen) {
                    loop {
                        if arguments.len() >= 255 {
                            Self::error(self.peek(), "Can't have more than 255 arguments.")?;
                        }

                        arguments.push(self.parse_expression()?);
                        if !self.match_(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(&TokenKind::RightParen, "Expect ')' after arguments")?;

                expr = expr::Call::new_wrapped(expr, arguments);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// primary        → NUMBER | STRING | "true" | "false" | "nil"
    //                 | "(" expression ")" ;
    fn parse_primary(&mut self) -> ParseExprResult {
        let expr: Expr = if self.match_(&[TokenKind::Number, TokenKind::String]) {
            expr::Literal::new_wrapped(self.previous().literal.clone().unwrap())
        } else if self.match_(&[TokenKind::True]) {
            expr::Literal::new_wrapped(Literal::Boolean(true))
        } else if self.match_(&[TokenKind::False]) {
            expr::Literal::new_wrapped(Literal::Boolean(false))
        } else if self.match_(&[TokenKind::Nil]) {
            expr::Literal::new_wrapped(Literal::Nil)
        } else if self.match_(&[TokenKind::LeftParen]) {
            let expr = self.parse_expression()?;
            self.consume(&TokenKind::RightParen, "Expect ')' after expression")?;
            expr::Grouping::new_wrapped(expr)
        } else if self.match_(&[TokenKind::Identifier]) {
            expr::Variable::new_wrapped(self.previous().lexeme.to_owned(), 0)
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
