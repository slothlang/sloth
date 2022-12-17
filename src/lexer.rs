#![allow(dead_code)]

#[derive(Debug, Eq, PartialEq)]
pub enum TokenType {
    // Utility
    Comment(String),

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

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).map(|it| *it as char)
    }

    fn peek_nth(&self, nth: usize) -> Option<char> {
        self.source.get(self.pos + nth).map(|it| *it as char)
    }

    fn advance(&mut self) -> Option<char> {
        self.pos += 1;
        self.source.get(self.pos - 1).map(|it| *it as char)
    }

    fn advance_if(&mut self, next: impl FnOnce(Option<char>) -> bool) -> bool {
        if next(self.peek()) {
            self.advance();
            return true;
        }

        false
    }

    fn advance_if_eq(&mut self, next: Option<char>) -> bool {
        self.advance_if(|it| it == next)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Ignore all whitespace
        loop {
            match self.peek() {
                Some('\n') => self.line += 1,
                Some(' ') | Some('\r') | Some('\t') => (),
                _ => break,
            }
            self.advance();
        }

        // Resetting the lexeme
        self.start = self.pos;

        // Parse the next lexeme- If it is EOF return nothing
        let Some(character) = self.advance() else {
            return None;
        };

        let tt = match character {
            // Whitespace & Comments
            '#' => {
                let mut value = String::new();
                while self.peek() != Some('\n') {
                    value.push(self.advance().unwrap());
                }

                TokenType::Comment(value)
            }

            // Arithmetic
            '+' if self.advance_if_eq(Some('=')) => TokenType::PlusEq,
            '-' if self.advance_if_eq(Some('=')) => TokenType::MinusEq,
            '*' if self.advance_if_eq(Some('=')) => TokenType::StarEq,
            '/' if self.advance_if_eq(Some('=')) => TokenType::SlashEq,
            '%' if self.advance_if_eq(Some('=')) => TokenType::PercEq,
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '%' => TokenType::Perc,

            '0'..='9' => {
                let mut value = String::new();
                value.push(character);
                while ('0'..='9').contains(&self.peek().unwrap()) {
                    value.push(self.advance().unwrap());
                }

                if self.advance_if_eq(Some('.')) {
                    value.push('.');
                    while ('0'..='9').contains(&self.peek().unwrap()) {
                        value.push(self.advance().unwrap());
                    }
                }
                TokenType::Literal(Literal::Number(value.parse::<i32>().unwrap()))
            }

            // Logical & Bitwise
            '!' if self.advance_if_eq(Some('=')) => TokenType::BangEq,
            '=' if self.advance_if_eq(Some('=')) => TokenType::EqEq,
            '>' if self.advance_if_eq(Some('=')) => TokenType::GtEq,
            '<' if self.advance_if_eq(Some('=')) => TokenType::LtEq,
            '!' => TokenType::Bang,
            '=' => TokenType::Eq,
            '>' => TokenType::Gt,
            '<' => TokenType::Lt,

            '&' if self.advance_if_eq(Some('&')) => TokenType::AmpAmp,
            '|' if self.advance_if_eq(Some('|')) => TokenType::PipePipe,
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
                while self.peek() != Some('"') {
                    let Some(character) = self.advance() else {
                        panic!("Syntax Error: String invalid");
                    };

                    if character == '\\' {
                        match self.advance().unwrap() {
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

                while let Some(character) = self.peek() && matches!(character, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                    value.push(self.advance().unwrap());
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

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use super::{Lexer, Literal, TokenType};

    const SAMPLE_PROGRAM: &str = r#"
val variable = 5;

if variable >= 7 {
    print "Hello World";
}

if variable < 52 {
    variable += 1;
    print "Hello ${variable}";
}

for person in ["Cody", "Johnny"] {
    print "Hello ${person}";
}
"#;

    #[test]
    fn simple_code() {
        let tokens = vec![
            // top
            TokenType::Val,
            TokenType::Identifier("variable".to_owned()),
            TokenType::Eq,
            TokenType::Literal(Literal::Number(5)),
            TokenType::SemiColon,
            // 1st block
            TokenType::If,
            TokenType::Identifier("variable".to_owned()),
            TokenType::GtEq,
            TokenType::Literal(Literal::Number(7)),
            TokenType::LeftBrace,
            TokenType::Print,
            TokenType::Literal(Literal::String("Hello World".to_owned())),
            TokenType::SemiColon,
            TokenType::RightBrace,
            // 2nd block
            TokenType::If,
            TokenType::Identifier("variable".to_owned()),
            TokenType::Lt,
            TokenType::Literal(Literal::Number(52)),
            TokenType::LeftBrace,
            TokenType::Identifier("variable".to_owned()),
            TokenType::PlusEq,
            TokenType::Literal(Literal::Number(1)),
            TokenType::SemiColon,
            TokenType::Print,
            TokenType::Literal(Literal::String("Hello ${variable}".to_owned())),
            TokenType::SemiColon,
            TokenType::RightBrace,
            // 3rd block
            TokenType::For,
            TokenType::Identifier("person".to_owned()),
            TokenType::In,
            TokenType::LeftBracket,
            TokenType::Literal(Literal::String("Cody".to_owned())),
            TokenType::Comma,
            TokenType::Literal(Literal::String("Johnny".to_owned())),
            TokenType::RightBracket,
            TokenType::LeftBrace,
            TokenType::Print,
            TokenType::Literal(Literal::String("Hello ${person}".to_owned())),
            TokenType::SemiColon,
            TokenType::RightBrace,
        ];

        let lexed_code = Lexer::new(SAMPLE_PROGRAM)
            .map(|it| it.tt)
            .collect::<Vec<_>>();

        assert_eq!(tokens, lexed_code);
    }

    #[bench]
    fn bench_lexer(b: &mut Bencher) {
        b.iter(|| {
            let _ = Lexer::new(SAMPLE_PROGRAM).collect::<Vec<_>>();
        });
    }
}
