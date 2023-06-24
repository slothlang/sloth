pub mod ast;
pub mod expr;
pub mod graph;
pub mod stmt;

use self::ast::{Literal, Stmt, StmtKind};
use crate::lexer::{Token, TokenType};
use crate::symtable::SymbolTable;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ParsingError {
    #[error("Invalid operation")]
    InvalidOp,
    #[error("Unexpected token")]
    UnexpectedToken,
}

#[derive(Debug)]
pub struct AstParser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
    id: i32,
    top: SymbolTable,
}

impl<'a> AstParser<'a> {
    pub fn parse(tokens: Vec<Token<'a>>, root: SymbolTable) -> Result<Stmt, ParsingError> {
        let mut parser = Self::new(tokens, root);

        let mut statements = Vec::new();
        while !parser.eof() {
            statements.push(parser.statement()?);
        }

        let root = Stmt::new(
            parser.reserve_id(),
            StmtKind::Block(statements),
            parser.top.clone(),
        );

        Ok(root)
    }
}

/// Implementation containing utilities used by the parsers internal components
impl<'a> AstParser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, root: SymbolTable) -> Self {
        Self {
            tokens,
            index: 0,
            id: 0,
            top: root,
        }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    pub fn peek2(&self) -> &Token {
        &self.tokens[self.index + 1]
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

    pub fn consume(&mut self, next: TokenType, error: &str) -> Result<&Token, ParsingError> {
        if std::mem::discriminant(&self.peek().tt) != std::mem::discriminant(&next) {
            println!("{error} at index {:?}", self.index);
            return Err(ParsingError::UnexpectedToken);
        }

        Ok(self.advance().unwrap())
    }

    pub fn consume_literal(&mut self) -> Result<Literal, ParsingError> {
        let Some(TokenType::Literal(literal)) = self.advance().map(|it| it.tt.clone()) else {
            return Err(ParsingError::UnexpectedToken);
        };

        Ok(literal.into())
    }

    pub fn consume_identifier(&mut self) -> Result<String, ParsingError> {
        let Some(TokenType::Identifier(identifier)) = self.advance().map(|it| it.tt.clone()) else {
            return Err(ParsingError::UnexpectedToken);
        };

        Ok(identifier)
    }

    pub fn reserve_id(&mut self) -> i32 {
        let id = self.id;
        self.id += 1;
        id
    }

    pub fn push_table(&mut self) {
        self.top = self.top.make_child();
    }

    pub fn pop_table(&mut self) {
        self.top = self.top.parent().unwrap();
    }

    pub fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}
