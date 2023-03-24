#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected token")]
    UnexpectedToken,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TokenType {
    // Meta
    DocComment,
    Comment,

    // Operatiors
    Plus,
    Minus,
    Star,
    Slash,
    Perc,

    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercEq,

    // Misc
    Literal(Literal),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Literal {
    Numeric,
    Boolean,
    Character,
    String,
    Regex,
}

#[derive(Debug, Default)]
pub struct Location {
    row: u32,
    column: u32,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub tt: TokenType,
    pub lexeme: &'a str,

    start: Location,
    end: Location,
}

pub struct Lexer<'a> {
    source: &'a [u8],

    start: Location,
    end: Location,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            start: Default::default(),
            end: Default::default(),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
