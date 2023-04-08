#![allow(dead_code)]

//! TODO: Lexing Regex Literals
//! TODO: Lexing Character Literals

use std::str::Chars;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected token")]
    UnexpectedToken,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Meta
    DocComment,
    Comment,

    // Brackets
    OpeningParen,   // (
    ClosingParen,   // )
    OpeningBracket, // [
    ClosingBracket, // ]
    OpeningBrace,   // {
    ClosingBrace,   // }

    // Operators
    Plus,     // +
    PlusPlus, // ++
    Minus,    // -
    Star,     // *
    StarStar, // **
    Slash,    // /
    Perc,     // %
    Tilde,    // ~

    PlusEq,     // +=
    PlusPlusEq, // ++=
    MinusEq,    // -=
    StarEq,     // *=
    StarStarEq, // **=
    SlashEq,    // /=
    PercEq,     // %=
    TildeEq,    // ~=

    Amp,      // &
    AmpAmp,   // &&
    Pipe,     // |
    PipePipe, // ||

    Eq,       // =
    EqEq,     // ==
    Bang,     // !
    BangBang, // !!
    BangEq,   // !=

    Lt,     // <
    LtLt,   // <<
    LtLtLt, // <<<
    LtEq,   // <=
    Gt,     // >
    GtGt,   // >>
    GtGtGt, // >>>
    GtEq,   // >=

    Comma,

    Question,         // ?
    QuestionDot,      // ?.
    QuestionQuestion, // ??
    Dot,              // .
    DotDot,           // ..

    Colon,      // :
    ColonColon, // ::
    SemiColon,  // ;

    Arrow,    // ->
    FatArrow, // =>

    // Keywords
    Val,
    Var,

    Fn,

    If,
    Else,

    While,
    For,
    In,

    Loop,
    Break,
    Continue,

    As,

    // Literals
    Integer(i128),
    Float(f64),
    Boolean(bool),
    Character(char),
    String(String),
    Regex(String),

    Identifier(String),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Location {
    index: usize,
    pub row: u32,
    pub col: u32,
}

impl Location {
    fn advance(&mut self, len: usize, newline: bool) {
        if newline {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        self.index += len;
    }
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
    window: [char; 3],
    chars: Chars<'a>,

    start: Location,
    current: Location,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        let mut chars = source.chars();
        let window = [
            chars.next().unwrap_or('\0'),
            chars.next().unwrap_or('\0'),
            chars.next().unwrap_or('\0'),
        ];

        Self {
            source: source.as_bytes(),
            window,
            chars,
            start: Default::default(),
            current: Default::default(),
        }
    }
}

impl<'a> Lexer<'a> {
    fn pos(&self) -> usize {
        self.current.index
    }

    fn peek(&self) -> char {
        self.window[0]
    }

    fn eof(&self) -> bool {
        self.peek() == '\0'
    }

    fn advance(&mut self) -> char {
        let current = self.window[0];
        self.window = [
            self.window[1],
            self.window[2],
            self.chars.next().unwrap_or('\0'),
        ];
        self.current.advance(current.len_utf8(), current == '\n');
        current
    }

    fn advance_with(&mut self, with: TokenType) -> TokenType {
        self.advance();
        with
    }

    fn advance_by(&mut self, amount: usize) {
        for _ in 0..amount {
            self.advance();
        }
    }

    fn advance_by_with(&mut self, amount: usize, with: TokenType) -> TokenType {
        self.advance_by(amount);
        with
    }

    fn advance_while(&mut self, predicate: impl Fn([char; 3]) -> bool) {
        while !self.eof() && predicate(self.window) {
            self.advance();
        }
    }
}

impl<'a> Lexer<'a> {
    fn lex_number(&mut self) -> TokenType {
        let mut value = self.advance().to_string();

        while self.peek().is_ascii_digit() {
            value.push(self.advance());
        }

        if self.peek() == '.' {
            value.push(self.advance());

            while self.peek().is_ascii_digit() {
                value.push(self.advance());
            }

            TokenType::Float(value.parse::<f64>().expect("Expected float"))
        } else {
            TokenType::Integer(value.parse::<i128>().expect("Expected integer"))
        }
    }

    fn lex_string(&mut self) -> TokenType {
        let mut value = String::new();

        self.advance();
        loop {
            match self.window {
                ['\\', '"', ..] => {
                    self.advance_by(2);
                    value.push('"');
                }
                ['\\', 't', ..] => {
                    self.advance_by(2);
                    value.push('\t');
                }
                ['\\', 'n', ..] => {
                    self.advance_by(2);
                    value.push('\n');
                }
                ['"', ..] => {
                    self.advance();
                    break;
                }
                _ => {
                    value.push(self.advance());
                    continue;
                }
            }
        }

        TokenType::String(value)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skipping whitespace
        self.advance_while(|it| it[0].is_whitespace());
        self.start = self.current;

        // If were at the end of the file return nothing
        if self.eof() {
            return None;
        }

        // Figuring out the token type
        let tt = match self.window {
            ['#', '#', ..] => {
                self.advance_while(|it| it[0] != '\n');
                TokenType::DocComment
            }

            ['#', ..] => {
                self.advance_while(|it| it[0] != '\n');
                TokenType::Comment
            }

            // Blocks
            ['(', ..] => self.advance_with(TokenType::OpeningParen),
            [')', ..] => self.advance_with(TokenType::ClosingParen),
            ['[', ..] => self.advance_with(TokenType::OpeningBracket),
            [']', ..] => self.advance_with(TokenType::ClosingBracket),
            ['{', ..] => self.advance_with(TokenType::OpeningBrace),
            ['}', ..] => self.advance_with(TokenType::ClosingBrace),

            // Operators
            ['-', '>', ..] => self.advance_by_with(2, TokenType::Arrow),
            ['=', '>', ..] => self.advance_by_with(2, TokenType::FatArrow),

            ['+', '+', '='] => self.advance_by_with(3, TokenType::PlusPlusEq),
            ['*', '*', '='] => self.advance_by_with(3, TokenType::StarStarEq),
            ['+', '+', ..] => self.advance_by_with(2, TokenType::PlusPlus),
            ['*', '*', ..] => self.advance_by_with(2, TokenType::StarStar),

            ['+', '=', ..] => self.advance_by_with(2, TokenType::PlusEq),
            ['-', '=', ..] => self.advance_by_with(2, TokenType::MinusEq),
            ['*', '=', ..] => self.advance_by_with(2, TokenType::StarEq),
            ['/', '=', ..] => self.advance_by_with(2, TokenType::SlashEq),
            ['%', '=', ..] => self.advance_by_with(2, TokenType::PercEq),
            ['~', '=', ..] => self.advance_by_with(2, TokenType::TildeEq),

            ['+', ..] => self.advance_with(TokenType::Plus),
            ['-', ..] => self.advance_with(TokenType::Minus),
            ['*', ..] => self.advance_with(TokenType::Star),
            ['/', ..] => self.advance_with(TokenType::Slash), // TODO: Check for regex literals
            ['%', ..] => self.advance_with(TokenType::Perc),
            ['~', ..] => self.advance_with(TokenType::Tilde),

            ['&', '&', ..] => self.advance_by_with(2, TokenType::AmpAmp),
            ['&', ..] => self.advance_with(TokenType::Amp),

            ['|', '|', ..] => self.advance_by_with(2, TokenType::PipePipe),
            ['|', ..] => self.advance_with(TokenType::Pipe),

            ['=', '=', ..] => self.advance_by_with(2, TokenType::EqEq),
            ['!', '=', ..] => self.advance_by_with(2, TokenType::BangEq),
            ['!', '!', ..] => self.advance_by_with(2, TokenType::BangBang),
            ['=', ..] => self.advance_with(TokenType::Eq),
            ['!', ..] => self.advance_with(TokenType::Bang),

            ['<', '<', '<'] => self.advance_by_with(3, TokenType::LtLtLt),
            ['<', '<', ..] => self.advance_by_with(2, TokenType::LtLt),
            ['<', '=', ..] => self.advance_by_with(2, TokenType::LtEq),
            ['<', ..] => self.advance_with(TokenType::Lt),

            ['>', '>', '>'] => self.advance_by_with(3, TokenType::GtGtGt),
            ['>', '>', ..] => self.advance_by_with(2, TokenType::GtGt),
            ['>', '=', ..] => self.advance_by_with(2, TokenType::GtEq),
            ['>', ..] => self.advance_with(TokenType::Gt),

            [',', ..] => self.advance_with(TokenType::Comma),

            ['.', '.', ..] => self.advance_by_with(2, TokenType::DotDot),
            ['.', ..] => self.advance_with(TokenType::Dot),
            ['?', '?', ..] => self.advance_by_with(2, TokenType::QuestionQuestion),
            ['?', '.', ..] => self.advance_by_with(2, TokenType::QuestionDot),
            ['?', ..] => self.advance_with(TokenType::Question),

            [';', ..] => self.advance_with(TokenType::SemiColon),
            [':', ':', ..] => self.advance_by_with(2, TokenType::ColonColon),
            [':', ..] => self.advance_with(TokenType::Colon),

            // Literals
            ['0'..='9', ..] => self.lex_number(),
            ['"', ..] => self.lex_string(),

            ['a'..='z' | 'A'..='Z' | '_', ..] => {
                let mut value = String::new();
                while matches!(self.peek(), 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                    value.push(self.advance());
                }

                match value.as_str() {
                    "val" => TokenType::Val,
                    "var" => TokenType::Var,
                    "fn" => TokenType::Fn,
                    "if" => TokenType::If,
                    "else" => TokenType::Else,
                    "while" => TokenType::While,
                    "for" => TokenType::For,
                    "in" => TokenType::In,
                    "loop" => TokenType::Loop,
                    "break" => TokenType::Break,
                    "continue" => TokenType::Continue,
                    "as" => TokenType::As,
                    "true" => TokenType::Boolean(true),
                    "false" => TokenType::Boolean(false),
                    _ => TokenType::Identifier(value),
                }
            }

            _ => panic!("Error while parsing"),
        };

        let lexeme = unsafe {
            // At this point it is already known that the string is valid UTF-8, might
            // aswell not check again
            std::str::from_utf8_unchecked(&self.source[self.start.index..self.pos()])
        };

        let token = Token {
            tt,
            lexeme,
            start: self.start,
            end: self.current,
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{Lexer, TokenType};

    #[test]
    fn lex_operators() {
        let source = "+ ++ - * ** / % ~ += ++= -= *= **= /= %= ~= & && | || = == ! !! != < << <<< \
                      <= > >> >>> >= , ? ?. ?? . .. : :: ; -> =>";
        let tokens = Lexer::new(source).map(|it| it.tt).collect_vec();

        assert_eq!(&tokens, &[
            TokenType::Plus,
            TokenType::PlusPlus,
            TokenType::Minus,
            TokenType::Star,
            TokenType::StarStar,
            TokenType::Slash,
            TokenType::Perc,
            TokenType::Tilde,
            TokenType::PlusEq,
            TokenType::PlusPlusEq,
            TokenType::MinusEq,
            TokenType::StarEq,
            TokenType::StarStarEq,
            TokenType::SlashEq,
            TokenType::PercEq,
            TokenType::TildeEq,
            TokenType::Amp,
            TokenType::AmpAmp,
            TokenType::Pipe,
            TokenType::PipePipe,
            TokenType::Eq,
            TokenType::EqEq,
            TokenType::Bang,
            TokenType::BangBang,
            TokenType::BangEq,
            TokenType::Lt,
            TokenType::LtLt,
            TokenType::LtLtLt,
            TokenType::LtEq,
            TokenType::Gt,
            TokenType::GtGt,
            TokenType::GtGtGt,
            TokenType::GtEq,
            TokenType::Comma,
            TokenType::Question,
            TokenType::QuestionDot,
            TokenType::QuestionQuestion,
            TokenType::Dot,
            TokenType::DotDot,
            TokenType::Colon,
            TokenType::ColonColon,
            TokenType::SemiColon,
            TokenType::Arrow,
            TokenType::FatArrow,
        ]);
    }

    #[test]
    fn lex_keywords() {
        let source = "val var fn if else while for in loop break continue as true false";
        let tokens = Lexer::new(source).map(|it| it.tt).collect_vec();

        assert_eq!(&tokens, &[
            TokenType::Val,
            TokenType::Var,
            TokenType::Fn,
            TokenType::If,
            TokenType::Else,
            TokenType::While,
            TokenType::For,
            TokenType::In,
            TokenType::Loop,
            TokenType::Break,
            TokenType::Continue,
            TokenType::As,
            TokenType::Boolean(true),
            TokenType::Boolean(false),
        ]);
    }

    #[test]
    fn lex_literals_a() {
        let source = "iden \"foo\" \"bar\" \"baz\" \"\\\"\" \"\\n\" \"\\t\" 93 3252 238 -382 -832 \
                      83 -25 52.9 83.7 12.4 35.2 3.3";
        let tokens = Lexer::new(source).map(|it| it.tt).collect_vec();

        assert_eq!(&tokens, &[
            TokenType::Identifier("iden".to_owned()),
            TokenType::String("foo".to_owned()),
            TokenType::String("bar".to_owned()),
            TokenType::String("baz".to_owned()),
            TokenType::String("\"".to_owned()),
            TokenType::String("\n".to_owned()),
            TokenType::String("\t".to_owned()),
            TokenType::Integer(93),
            TokenType::Integer(3252),
            TokenType::Integer(238),
            TokenType::Minus,
            TokenType::Integer(382),
            TokenType::Minus,
            TokenType::Integer(832),
            TokenType::Integer(83),
            TokenType::Minus,
            TokenType::Integer(25),
            TokenType::Float(52.9),
            TokenType::Float(83.7),
            TokenType::Float(12.4),
            TokenType::Float(35.2),
            TokenType::Float(3.3),
        ]);
    }
}
