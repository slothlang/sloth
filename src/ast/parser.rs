use super::{Expr, Stmt};
use crate::lexer::{Literal, Token, TokenType};

pub struct AstParser<'a> {
    tokens: Vec<Token<'a>>,
    index: usize,
}

/// Implementation containing utilities used by the parsers internal components
impl<'a> AstParser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, index: 0 }
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.index - 1)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peek_nth(&self, nth: usize) -> Option<&Token> {
        self.tokens.get(self.index + nth)
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.eof() {
            return None;
        }

        self.index += 1;
        Some(&self.tokens[self.index - 1])
    }

    fn advance_if(&mut self, next: impl FnOnce(&Token) -> bool) -> bool {
        if self.eof() {
            return false;
        }

        if next(self.peek()) {
            self.advance();
            return true;
        }

        false
    }

    fn advance_if_eq(&mut self, next: &TokenType) -> bool {
        self.advance_if(|it| it.tt == *next)
    }

    fn advance_seq(&mut self, seq: &[TokenType]) -> bool {
        for token in seq {
            if !self.advance_if_eq(token) {
                return false;
            }
        }

        true
    }

    fn consume(&mut self, next: TokenType, error: &str) {
        if std::mem::discriminant(&self.peek().tt) != std::mem::discriminant(&next) {
            panic!("{error}");
        }
        self.advance();
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }
}

/// Implementation containing parsers internal components related to statements
impl<'a> AstParser<'a> {
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.eof() {
            statements.push(self.statement());
        }

        statements
    }

    fn block(&mut self) -> Vec<Stmt> {
        self.consume(TokenType::LeftBrace, "Expected '{' at beggining of block");

        let mut statements = Vec::new();

        while !self.eof() && self.peek().tt != TokenType::RightBrace {
            statements.push(self.statement());
        }

        self.consume(TokenType::RightBrace, "Expected '}' at end of block");
        statements
    }

    fn statement(&mut self) -> Stmt {
        if self.peek().tt == TokenType::LeftBrace {
            return Stmt::Block(self.block());
        }

        if self.advance_if_eq(&TokenType::Print) {
            return self.print_statement();
        }

        if self.advance_if_eq(&TokenType::Var) {
            return self.var_statement();
        }

        if self.advance_if_eq(&TokenType::Val) {
            return self.val_statement();
        }

        if self.advance_if_eq(&TokenType::If) {
            return self.if_statement();
        }

        if self.advance_if_eq(&TokenType::For) {
            return self.for_statement();
        }

        // If we couldn't parse a statement return an expression statement
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
        Stmt::Print { value }
    }

    fn var_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'var'");
        };

        self.advance(); // Advancing from the identifier
        self.consume(TokenType::Eq, "Expected '=' after identifier");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::Var { ident, value }
    }

    fn val_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'val'");
        };

        self.advance(); // Advancing from the identifier
        self.consume(TokenType::Eq, "Expected '=' after identifier");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::Val { ident, value }
    }

    fn if_statement(&mut self) -> Stmt {
        let condition = self.expression();
        let body = self.block();

        Stmt::If { condition, body }
    }

    fn for_statement(&mut self) -> Stmt {
        let binding = self.expression();
        let Expr::Variable(binding) = binding else {
            panic!("Left side of for statement must be identifier");
        };

        self.consume(
            TokenType::In,
            "Expected 'in' in between identifier and range",
        );

        let range_start = self.expression();
        self.consume(
            TokenType::DotDot,
            "Expected '..' denoting min and max of range",
        );
        let range_end = self.expression();

        let body = self.block();

        Stmt::For {
            binding,
            range: (range_start, range_end),
            body,
        }
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();

        // FIXME: Move assignment handling
        if self.advance_if_eq(&TokenType::Eq) {
            if let Expr::Variable(ident) = &expr {
                let value = self.expression();

                self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
                return Stmt::Assignment {
                    ident: ident.clone(),
                    value,
                };
            }
        }

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
        Stmt::Expr(expr)
    }
}

/// Implementation containing parsers internal components related to expressions
impl<'a> AstParser<'a> {
    // FIXME: Should probably avoid cloning token types

    fn expression(&mut self) -> Expr {
        self.logical_or()
    }

    fn unary(&mut self) -> Expr {
        if !self.eof()
            && matches!(
                self.peek().tt,
                TokenType::Bang | TokenType::Plus | TokenType::Minus
            )
        {
            let operator = self.advance().unwrap().tt.clone();
            let rhs = self.unary();
            return Expr::Unary {
                operator,
                expr: Box::new(rhs),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        match self.advance().unwrap().tt.clone() {
            TokenType::Literal(literal) => Expr::Literal(literal),
            TokenType::Identifier(ident) => Expr::Variable(ident),
            TokenType::LeftParen => {
                let expr = self.expression();
                self.consume(TokenType::RightParen, "Must end expression with ')'");
                Expr::Grouping(Box::new(expr))
            }
            _ => unimplemented!("{:?}", self.peek()),
        }
    }
}

// Macro to generate repetitive binary expressions. Things like addition,
// multiplication, exc.
macro_rules! binary_expr {
    ($name:ident, $parent:ident, $pattern:pat) => {
        fn $name(&mut self) -> Expr {
            let mut expr = self.$parent();

            while !self.eof() && matches!(self.peek().tt, $pattern) {
                let operator = self.advance().unwrap().tt.clone();
                let rhs = self.$parent();
                expr = Expr::Binary {
                    operator,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };
            }

            expr
        }
    };
}

#[rustfmt::skip]
impl<'a> AstParser<'a> {
    // Binary expressions in order of precedence from lowest to highest.
    binary_expr!(logical_or      , logical_and     , (TokenType::PipePipe));
    binary_expr!(logical_and     , equality        , (TokenType::AmpAmp));
    binary_expr!(equality        , comparison      , (TokenType::BangEq | TokenType::EqEq));
    binary_expr!(comparison      , bitwise_shifting, (TokenType::Lt     | TokenType::Gt    | TokenType::LtEq | TokenType::GtEq));
    binary_expr!(bitwise_shifting, additive        , (TokenType::LtLt   | TokenType::GtGt));
    binary_expr!(additive        , multiplicative  , (TokenType::Plus   | TokenType::Minus));
    binary_expr!(multiplicative  , unary           , (TokenType::Star   | TokenType::Slash | TokenType::Perc));
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::AstParser;
    use crate::ast::Expr;
    use crate::lexer::{Lexer, Literal, TokenType};

    #[test]
    fn basic_expression_a() {
        let lexer = Lexer::new("3 + 5 * 4");
        let tokens = lexer.collect_vec();

        let expected_ast = Expr::Binary {
            operator: TokenType::Plus,
            lhs: Box::new(Expr::Literal(Literal::Number(3))),
            rhs: Box::new(Expr::Binary {
                operator: TokenType::Star,
                lhs: Box::new(Expr::Literal(Literal::Number(5))),
                rhs: Box::new(Expr::Literal(Literal::Number(4))),
            }),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.expression();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_expression_b() {
        let lexer = Lexer::new("17 - (-5 + 5) / 6");
        let tokens = lexer.collect_vec();

        let expected_ast = Expr::Binary {
            operator: TokenType::Minus,
            lhs: Box::new(Expr::Literal(Literal::Number(17))),
            rhs: Box::new(Expr::Binary {
                operator: TokenType::Slash,
                lhs: Box::new(Expr::Grouping(Box::new(Expr::Binary {
                    operator: TokenType::Plus,
                    lhs: Box::new(Expr::Unary {
                        operator: TokenType::Minus,
                        expr: Box::new(Expr::Literal(Literal::Number(5))),
                    }),
                    rhs: Box::new(Expr::Literal(Literal::Number(5))),
                }))),
                rhs: Box::new(Expr::Literal(Literal::Number(6))),
            }),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.expression();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_expression_c() {
        let lexer = Lexer::new("9 > 6 && 5 + 7 == 32 || \"apple\" != \"banana\"");
        let tokens = lexer.collect_vec();

        // TODO:
    }
}
