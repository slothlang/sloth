pub mod ast;
pub mod expr;
pub mod stmt;

use crate::lexer::{Token, TokenType};

pub struct AstParser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

/// Implementation containing utilities used by the parsers internal components
impl<'a> AstParser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, index: 0 }
    }
    pub fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn advance(&mut self) -> Option<&Token> {
        if self.eof() {
            return None;
        }

        self.index += 1;
        Some(&self.tokens[self.index - 1])
    }

    pub fn advance_if(&mut self, next: impl FnOnce(&Token) -> bool) -> bool {
        if self.eof() {
            return false;
        }

        if next(self.peek()) {
            self.advance();
            return true;
        }

        false
    }

    pub fn advance_if_eq(&mut self, next: &TokenType) -> bool {
        self.advance_if(|it| it.tt == *next)
    }

    pub fn consume(&mut self, next: TokenType, error: &str) {
        if std::mem::discriminant(&self.peek().tt) != std::mem::discriminant(&next) {
            panic!("{error}");
        }
        self.advance();
    }

    pub fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}
