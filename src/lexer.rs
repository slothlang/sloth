#![allow(dead_code)]

#[derive(Debug, Eq, PartialEq)]
pub enum TokenType {
    // Short
    Plus,  // +
    Minus, // -
    Slash, // /
    Star,  // *
    Perc,  // %

    PlusEq,  // +=
    MinusEq, // -=
    SlashEq, // /=
    StarEq,  // *=
    PercEq,  // %=

    Eq,     // =
    EqEq,   // ==
    Bang,   // !
    BangEq, // !=

    Gt,   // >
    GtEq, // >=
    Lt,   // <
    LtEq, // <=

    Amp,      // &
    AmpAmp,   // &&
    Pipe,     // |
    PipePipe, // ||

    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }

    Comma,     // ,
    Dot,       // .
    Colon,     // :
    SemiColon, // ;

    // Literals
    Literal(Literal),
    Identifier(String),

    // Keywords
    Val,
    Var,
    Fn,

    If,
    Else,

    For,
    In,

    While,

    Loop,
    Break,
    Continue,

    Print, // TODO: Change to std library function
}

#[derive(Debug, Eq, PartialEq)]
pub enum Literal {
    String(String),
    Character(char),
    Number(i32),
    Bool(bool),
    Nil,
}

#[derive(Debug)]
pub struct Token<'a> {
    pub tt: TokenType,
    pub lexeme: &'a str,

    start: usize,
    length: usize,
    line: u32,
}

pub struct Lexer<'a> {
    source: &'a [u8],
    start: usize, // Start of the lexme
    pos: usize,   // End of the lexme
    line: u32,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Lexer<'a> {
        Self {
            source: source.as_bytes(),
            start: 0,
            pos: 0,
            line: 0,
        }
    }

    fn peek(&self) -> char {
        self.source
            .get(self.pos)
            .map(|it| *it as char)
            .unwrap_or('\u{0000}')
    }

    fn peek_nth(&self, nth: usize) -> char {
        self.source
            .get(self.pos + nth)
            .map(|it| *it as char)
            .unwrap_or('\u{0000}')
    }

    fn advance(&mut self) -> char {
        self.pos += 1;
        self.source
            .get(self.pos - 1)
            .map(|it| *it as char)
            .unwrap_or('\u{0000}')
    }

    fn advance_if(&mut self, next: char) -> bool {
        if self.peek() != next {
            return false;
        }

        self.advance();
        true
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Ignore all whitespace & comments
        loop {
            match self.peek() {
                '#' => {
                    while self.peek() != '\n' {
                        self.advance();
                    }
                }
                '\n' => self.line += 1,
                ' ' | '\r' | '\t' => (),
                _ => break,
            }
            self.advance();
        }

        // Resetting the lexeme
        self.start = self.pos;

        // Parse the next lexeme
        let character = self.advance();
        let tt = match character {
            // Arithmetic
            '+' if self.advance_if('=') => TokenType::PlusEq,
            '-' if self.advance_if('=') => TokenType::MinusEq,
            '*' if self.advance_if('=') => TokenType::StarEq,
            '/' if self.advance_if('=') => TokenType::SlashEq,
            '%' if self.advance_if('=') => TokenType::PercEq,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Perc,

            '0'..='9' => {
                let mut value = String::new();
                value.push(character);
                while ('0'..='9').contains(&self.peek()) {
                    value.push(self.advance());
                }
                if self.advance_if('.') {
                    value.push('.');
                    while ('0'..='9').contains(&self.peek()) {
                        let c = self.advance();
                        value.push(c);
                    }
                }
                TokenType::Literal(Literal::Number(value.parse::<i32>().unwrap()))
            }

            // Logical & Bitwise
            '!' if self.advance_if('=') => TokenType::BangEq,
            '=' if self.advance_if('=') => TokenType::EqEq,
            '>' if self.advance_if('=') => TokenType::GtEq,
            '<' if self.advance_if('=') => TokenType::LtEq,
            '!' => TokenType::Bang,
            '=' => TokenType::Eq,
            '>' => TokenType::Gt,
            '<' => TokenType::Lt,

            '&' if self.advance_if('&') => TokenType::AmpAmp,
            '|' if self.advance_if('|') => TokenType::PipePipe,
            '&' => TokenType::Amp,
            '|' => TokenType::Pipe,

            // Scope
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '[' => TokenType::LeftBracket,
            ']' => TokenType::RightBracket,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            ':' => TokenType::Colon,
            ';' => TokenType::SemiColon,

            '"' => {
                let mut value = String::new();
                while self.peek() != '"' {
                    let character = self.advance();

                    if character == '\\' {
                        match self.advance() {
                            '\\' => value.push('\\'),
                            '"' => value.push('"'),
                            _ => panic!(),
                        }
                        continue;
                    }

                    value.push(character);
                }

                self.advance();
                TokenType::Literal(Literal::String(value))
            }

            // Keywords & Identifiers
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut value = String::new();
                value.push(character);
                while matches!(self.peek(), 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                    value.push(self.advance())
                }

                match value.as_str() {
                    "val" => TokenType::Val,
                    "var" => TokenType::Var,
                    "fn" => TokenType::Fn,
                    "if" => TokenType::If,
                    "else" => TokenType::Else,
                    "for" => TokenType::For,
                    "in" => TokenType::In,
                    "while" => TokenType::While,
                    "loop" => TokenType::Loop,
                    "break" => TokenType::Break,
                    "continue" => TokenType::Continue,
                    "print" => TokenType::Print,
                    _ => TokenType::Identifier(value),
                }
            }

            // Misc.
            '\u{0000}' => return None,
            _ => panic!("Failed to parse"),
        };

        let lexeme = unsafe {
            // If it got to this point we know the slice is valid UTF-8. The only area in
            // the language that UTF-8 characters are recognized is within strings.
            std::str::from_utf8_unchecked(&self.source[self.start..self.pos])
        };

        let token = Token {
            tt,
            lexeme,
            start: self.start,
            length: self.pos - self.start,
            line: self.line,
        };

        Some(token)
    }
}
