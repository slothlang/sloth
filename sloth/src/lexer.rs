#![allow(dead_code)]

//! TODO: Lexing Regex Literals

use std::fmt::Display;
use std::str::Chars;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
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
    Caret,    // ^

    Eq,       // =
    EqEq,     // ==
    Bang,     // !
    BangBang, // !!
    BangEq,   // !=

    Lt,     // <
    LtLt,   // <<
    LtEq,   // <=
    LtLtEq, // <<=
    Gt,     // >
    GtGt,   // >>
    GtEq,   // >=
    GtGtEq, // >>=

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
    Const,
    Val,
    Var,

    Fn,
    Return,

    If,
    Else,

    While,
    For,
    In,

    Loop,
    Break,
    Continue,

    As,

    Foreign,

    // Other
    Literal(Literal),
    Identifier(String),

    // Utility
    Error(LexerError),
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenType::DocComment => "##",
            TokenType::Comment => "#",
            TokenType::OpeningParen => "(",
            TokenType::ClosingParen => ")",
            TokenType::OpeningBracket => "[",
            TokenType::ClosingBracket => "]",
            TokenType::OpeningBrace => "{",
            TokenType::ClosingBrace => "}",
            TokenType::Plus => "+",
            TokenType::PlusPlus => "++",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::StarStar => "**",
            TokenType::Slash => "/",
            TokenType::Perc => "%",
            TokenType::Tilde => "~",
            TokenType::PlusEq => "+=",
            TokenType::PlusPlusEq => "++=",
            TokenType::MinusEq => "-=",
            TokenType::StarEq => "*=",
            TokenType::StarStarEq => "**=",
            TokenType::SlashEq => "/=",
            TokenType::PercEq => "%=",
            TokenType::TildeEq => "~=",
            TokenType::Amp => "&",
            TokenType::AmpAmp => "&&",
            TokenType::Pipe => "|",
            TokenType::PipePipe => "||",
            TokenType::Caret => "^",
            TokenType::Eq => "=",
            TokenType::EqEq => "==",
            TokenType::Bang => "!",
            TokenType::BangBang => "!!",
            TokenType::BangEq => "!=",
            TokenType::Lt => "<",
            TokenType::LtLt => "<<",
            TokenType::LtEq => "<=",
            TokenType::LtLtEq => "<<=",
            TokenType::Gt => ">",
            TokenType::GtGt => ">>",
            TokenType::GtEq => ">=",
            TokenType::GtGtEq => ">>=",
            TokenType::Comma => ",",
            TokenType::Question => "?",
            TokenType::QuestionDot => "?.",
            TokenType::QuestionQuestion => "??",
            TokenType::Dot => ".",
            TokenType::DotDot => "..",
            TokenType::Colon => ":",
            TokenType::ColonColon => "::",
            TokenType::SemiColon => ";",
            TokenType::Arrow => "->",
            TokenType::FatArrow => "=>",
            TokenType::Const => "const",
            TokenType::Val => "val",
            TokenType::Var => "var",
            TokenType::Fn => "fn",
            TokenType::Return => "return",
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::While => "while",
            TokenType::For => "for",
            TokenType::In => "in",
            TokenType::Loop => "loop",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::As => "as",
            TokenType::Foreign => "foreign",
            TokenType::Literal(_) => "literal",
            TokenType::Identifier(_) => "identifier",
            TokenType::Error(_) => "error",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Character(char),
    String(String),
}

impl From<Literal> for TokenType {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
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
    pub(crate) tt: TokenType,
    pub(crate) lexeme: &'a str,

    pub start: Location,
    pub end: Location,
}

pub struct Lexer<'a> {
    source: &'a [u8],
    window: [char; 3],
    chars: Chars<'a>,

    start: Location,
    current: Location,

    // Keep track if the lexer has encountered an error to stop lexing asap
    errored: bool,
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
            errored: false,
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

            Literal::Float(value.parse::<f32>().expect("Expected float")).into()
        } else {
            Literal::Integer(value.parse::<i32>().expect("Expected integer")).into()
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

        Literal::String(value).into()
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Skipping whitespace
        self.advance_while(|it| it[0].is_whitespace());
        self.start = self.current;

        // If were at the end of the file or an error has occurred return nothing
        if self.eof() || self.errored {
            return None;
        }

        // Figuring out the token type
        let tt = match self.window {
            ['#', '#', ..] => {
                self.advance_while(|it| it[0] != '\n');
                // TODO: TokenType::DocComment
                return self.next();
            }

            ['#', ..] => {
                self.advance_while(|it| it[0] != '\n');
                // TODO: okenType::Comment
                return self.next();
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

            ['^', ..] => self.advance_by_with(2, TokenType::Caret),

            ['=', '=', ..] => self.advance_by_with(2, TokenType::EqEq),
            ['!', '=', ..] => self.advance_by_with(2, TokenType::BangEq),
            ['!', '!', ..] => self.advance_by_with(2, TokenType::BangBang),
            ['=', ..] => self.advance_with(TokenType::Eq),
            ['!', ..] => self.advance_with(TokenType::Bang),

            ['<', '<', '='] => self.advance_by_with(3, TokenType::LtLtEq),
            ['<', '<', ..] => self.advance_by_with(2, TokenType::LtLt),
            ['<', '=', ..] => self.advance_by_with(2, TokenType::LtEq),
            ['<', ..] => self.advance_with(TokenType::Lt),

            ['>', '>', '='] => self.advance_by_with(3, TokenType::GtGtEq),
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
            ['\'', c, '\''] => self.advance_by_with(3, Literal::Character(c).into()),
            ['0'..='9', ..] => self.lex_number(),
            ['"', ..] => self.lex_string(),

            ['a'..='z' | 'A'..='Z' | '_' | '$', ..] => {
                let mut value = String::new();
                while matches!(self.peek(), 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '$') {
                    value.push(self.advance());
                }

                match value.as_str() {
                    "const" => TokenType::Const,
                    "val" => TokenType::Val,
                    "var" => TokenType::Var,
                    "fn" => TokenType::Fn,
                    "return" => TokenType::Return,
                    "if" => TokenType::If,
                    "else" => TokenType::Else,
                    "while" => TokenType::While,
                    "for" => TokenType::For,
                    "in" => TokenType::In,
                    "loop" => TokenType::Loop,
                    "break" => TokenType::Break,
                    "continue" => TokenType::Continue,
                    "as" => TokenType::As,
                    "foreign" => TokenType::Foreign,
                    "true" => Literal::Boolean(true).into(),
                    "false" => Literal::Boolean(false).into(),
                    _ => TokenType::Identifier(value),
                }
            }

            _ => {
                self.errored = true;
                TokenType::Error(LexerError::UnexpectedToken)
            }
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

    use super::{Lexer, Literal, TokenType};
    use crate::lexer::LexerError;

    #[test]
    fn lex_operators() {
        let source = "+ ++ - * ** / % ~ += ++= -= *= **= /= %= ~= & && | || ^ = == ! !! != < << \
                      <<= <= > >> >>= >= , ? ?. ?? . .. : :: ; -> =>";
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
            TokenType::Caret,
            TokenType::Eq,
            TokenType::EqEq,
            TokenType::Bang,
            TokenType::BangBang,
            TokenType::BangEq,
            TokenType::Lt,
            TokenType::LtLt,
            TokenType::LtLtEq,
            TokenType::LtEq,
            TokenType::Gt,
            TokenType::GtGt,
            TokenType::GtGtEq,
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
            Literal::Boolean(true).into(),
            Literal::Boolean(false).into(),
        ]);
    }

    #[test]
    fn lex_literals() {
        let source = "foo bar _foo __bar $0 $$1 \"foo\" \"bar\" \"baz\" \"\\\"\" \"\\n\" \"\\t\" \
                      'a' 'b' '\"' 93 3252 238 -382 -832 83 -25 52.9 83.7 12.4 35.2 3.3";
        let tokens = Lexer::new(source).map(|it| it.tt).collect_vec();

        assert_eq!(&tokens, &[
            TokenType::Identifier("foo".to_owned()),
            TokenType::Identifier("bar".to_owned()),
            TokenType::Identifier("_foo".to_owned()),
            TokenType::Identifier("__bar".to_owned()),
            TokenType::Identifier("$0".to_owned()),
            TokenType::Identifier("$$1".to_owned()),
            Literal::String("foo".to_owned()).into(),
            Literal::String("bar".to_owned()).into(),
            Literal::String("baz".to_owned()).into(),
            Literal::String("\"".to_owned()).into(),
            Literal::String("\n".to_owned()).into(),
            Literal::String("\t".to_owned()).into(),
            Literal::Character('a').into(),
            Literal::Character('b').into(),
            Literal::Character('"').into(),
            Literal::Integer(93).into(),
            Literal::Integer(3252).into(),
            Literal::Integer(238).into(),
            TokenType::Minus,
            Literal::Integer(382).into(),
            TokenType::Minus,
            Literal::Integer(832).into(),
            Literal::Integer(83).into(),
            TokenType::Minus,
            Literal::Integer(25).into(),
            Literal::Float(52.9).into(),
            Literal::Float(83.7).into(),
            Literal::Float(12.4).into(),
            Literal::Float(35.2).into(),
            Literal::Float(3.3).into(),
        ]);
    }

    #[test]
    fn lex_errors() {
        let source = "`";
        let tokens = Lexer::new(source).map(|it| it.tt).collect_vec();

        assert_eq!(&tokens, &[TokenType::Error(LexerError::UnexpectedToken)]);
    }
}
