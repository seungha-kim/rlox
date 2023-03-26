use crate::token::{Token, TokenKind};
use crate::value::Value;
use anyhow::bail;

pub struct Scanner {
    source: Vec<char>,
    start: usize,
    current: usize,
    line: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
            tokens: Vec::new(),
        }
    }

    pub fn scan_tokens(mut self) -> anyhow::Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });

        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> anyhow::Result<()> {
        let c = self.advance();
        match c {
            '(' => self.add_empty_token(TokenKind::LeftParen),
            ')' => self.add_empty_token(TokenKind::RightParen),
            '{' => self.add_empty_token(TokenKind::LeftBrace),
            '}' => self.add_empty_token(TokenKind::RightBrace),
            ',' => self.add_empty_token(TokenKind::Comma),
            '.' => self.add_empty_token(TokenKind::Dot),
            '-' => self.add_empty_token(TokenKind::Minus),
            '+' => self.add_empty_token(TokenKind::Plus),
            ';' => self.add_empty_token(TokenKind::Semicolon),
            '*' => self.add_empty_token(TokenKind::Star),
            '!' => {
                if self.match_('=') {
                    self.add_empty_token(TokenKind::BangEqual)
                } else {
                    self.add_empty_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_('=') {
                    self.add_empty_token(TokenKind::EqualEqual)
                } else {
                    self.add_empty_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_('=') {
                    self.add_empty_token(TokenKind::LessEqual)
                } else {
                    self.add_empty_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_('=') {
                    self.add_empty_token(TokenKind::GreaterEqual)
                } else {
                    self.add_empty_token(TokenKind::Greater)
                }
            }
            '/' => {
                if self.match_('/') {
                    // comment
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_empty_token(TokenKind::Slash);
                }
            }
            ' ' | '\r' | '\t' => {
                // skip
            }
            '\n' => {
                self.line += 1;
            }
            '"' => {
                self.string()?;
            }
            _ => {
                if c.is_digit(10) {
                    self.number();
                } else if c.is_alphabetic() {
                    self.identifier();
                } else {
                    bail!("Unsupported character");
                }
            }
        }
        Ok(())
    }

    fn string(&mut self) -> anyhow::Result<()> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            bail!("Unterminated string.");
        }

        self.advance(); // Closing "
        let value = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect::<String>();
        self.add_literal_token(TokenKind::String, Value::String(value));
        Ok(())
    }

    fn number(&mut self) {
        while self.peek().map(|c| c.is_digit(10)).unwrap_or(false) {
            self.advance();
        }

        if self.peek() == Some('.') && self.peek_next().map(|c| c.is_digit(10)).unwrap_or(false) {
            self.advance();

            while self.peek().map(|c| c.is_digit(10)).unwrap_or(false) {
                self.advance();
            }
        }

        let value_str: String = self.source[self.start..self.current].iter().collect();
        let value = value_str.parse::<f64>().unwrap();
        self.add_literal_token(TokenKind::Number, Value::Number(value));
    }

    fn identifier(&mut self) {
        while self.peek().map(|c| c.is_alphanumeric()).unwrap_or(false) {
            self.advance();
        }

        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.add_empty_token(Self::keyword_to_token(&lexeme).unwrap_or(TokenKind::Identifier));
        // TODO: not empty!
    }

    fn advance(&mut self) -> char {
        let result = self.source[self.current];
        self.current += 1;
        result
    }

    fn match_(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.current).cloned()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.current + 1).cloned()
    }

    fn add_empty_token(&mut self, kind: TokenKind) {
        self._add_token(kind, None);
    }

    fn add_literal_token(&mut self, kind: TokenKind, literal: Value) {
        self._add_token(kind, Some(literal));
    }

    fn _add_token(&mut self, kind: TokenKind, literal: Option<Value>) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens.push(Token {
            kind,
            lexeme: lexeme.iter().collect(),
            literal,
            line: self.line,
        })
    }

    fn keyword_to_token(candidate: &str) -> Option<TokenKind> {
        match candidate {
            "and" => Some(TokenKind::And),
            "class" => Some(TokenKind::Class),
            "else" => Some(TokenKind::Else),
            "false" => Some(TokenKind::False),
            "for" => Some(TokenKind::For),
            "fun" => Some(TokenKind::Fun),
            "if" => Some(TokenKind::If),
            "nil" => Some(TokenKind::Nil),
            "or" => Some(TokenKind::Or),
            "print" => Some(TokenKind::Print),
            "return" => Some(TokenKind::Return),
            "super" => Some(TokenKind::Super),
            "this" => Some(TokenKind::This),
            "true" => Some(TokenKind::True),
            "var" => Some(TokenKind::Var),
            "while" => Some(TokenKind::While),
            _ => None,
        }
    }
}
