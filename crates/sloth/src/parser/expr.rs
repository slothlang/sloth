use super::ast::{AstParser, BinaryOp, Expr, Literal, UnaryOp};
use crate::lexer::TokenType;

/// Implementation containing parsers internal components related to expressions
impl<'a> AstParser<'a> {
    // FIXME: Should probably avoid cloning token types

    pub fn expression(&mut self) -> Expr {
        self.logical_or()
    }

    fn unary(&mut self) -> Expr {
        if !self.eof()
            && matches!(
                self.peek().tt,
                TokenType::Bang | TokenType::Plus | TokenType::Minus
            )
        {
            let operator = match self.advance().unwrap().tt.clone() {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Tilde => UnaryOp::BWComp,
                TokenType::Minus => UnaryOp::Neg,
                _ => UnaryOp::Neg, // TODO: Idk how to not have this shit
            };

            let rhs = self.unary();
            return Expr::UnaryOp {
                op: (operator),
                value: (Box::new(rhs)),
            };
        }

        self.call()
    }

    fn call(&mut self) -> Expr {
        let mut expr = self.primary();

        if self.advance_if_eq(&TokenType::OpeningParen) {
            let mut arguments = Vec::<Expr>::new();

            if self.peek().tt != TokenType::ClosingParen {
                loop {
                    arguments.push(self.expression());
                    if !self.advance_if_eq(&TokenType::Comma) {
                        break;
                    }
                }
            }

            self.consume(
                TokenType::ClosingParen,
                "Expected ')' to close off function call",
            );

            // let Expr::Variable(_ident) = expr else { panic!("uh oh spaghettio"); };

            expr = Expr::Call {
                ident: (Box::new(expr)),
                args: (arguments),
            }
        }

        expr
    }

    fn primary(&mut self) -> Expr {
        match self.advance().unwrap().tt.clone() {
            TokenType::Integer(literal) => Expr::Literal(Literal::Integer(literal)),
            TokenType::Float(literal) => Expr::Literal(Literal::Float(literal)),
            TokenType::Boolean(literal) => Expr::Literal(Literal::Bool(literal)),
            TokenType::Character(literal) => Expr::Literal(Literal::Char(literal)),
            TokenType::String(literal) => Expr::Literal(Literal::String(literal)),
            TokenType::Regex(literal) => Expr::Literal(Literal::Regex(literal)),
            TokenType::Identifier(ident) => Expr::Variable(ident),
            TokenType::OpeningParen => {
                let expr = self.expression();
                self.consume(TokenType::ClosingParen, "Must end expression with ')'");
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
                let operator = match self.advance().unwrap().tt.clone() {
                    TokenType::Plus => BinaryOp::Add,
                    TokenType::PlusPlus => BinaryOp::Con,
                    TokenType::Minus => BinaryOp::Sub,
                    TokenType::Star => BinaryOp::Mul,
                    TokenType::StarStar => BinaryOp::Pow,
                    TokenType::Slash => BinaryOp::Div,
                    TokenType::Perc => BinaryOp::Mod,

                    TokenType::LtLt => BinaryOp::BWSftRight,
                    TokenType::GtGt => BinaryOp::BWSftLeft,
                    TokenType::Amp => BinaryOp::BWAnd,
                    TokenType::Pipe => BinaryOp::BWOr,
                    TokenType::Caret => BinaryOp::BWXor,

                    TokenType::Lt => BinaryOp::Lt,
                    TokenType::Gt => BinaryOp::Gt,
                    TokenType::LtEq => BinaryOp::LtEq,
                    TokenType::GtEq => BinaryOp::GtEq,
                    TokenType::EqEq => BinaryOp::EqEq,
                    TokenType::BangEq => BinaryOp::NotEq,
                    TokenType::AmpAmp => BinaryOp::LogAnd,
                    TokenType::PipePipe => BinaryOp::LogOr,
                    _ => BinaryOp::Add, // TODO: Idk how to not have this shit
                };

                let rhs = self.$parent();
                expr = Expr::BinaryOp {
                    op: (operator),
                    lhs: (Box::new(expr)),
                    rhs: (Box::new(rhs)),
                }
            }

            expr
        }
    };
}

#[rustfmt::skip]
#[allow(unused_parens)]
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

    use super::{AstParser, BinaryOp, Expr, Literal};
    use crate::lexer::Lexer;
    use crate::parser::ast::UnaryOp;

    #[test]
    fn basic_expression_a() {
        let lexer = Lexer::new("3 + 5 * 4");
        let tokens = lexer.collect_vec();

        let expected_ast = Expr::BinaryOp {
            op: BinaryOp::Add,
            lhs: Box::new(Expr::Literal(Literal::Integer(3))),
            rhs: Box::new(Expr::BinaryOp {
                op: BinaryOp::Mul,
                lhs: Box::new(Expr::Literal(Literal::Integer(5))),
                rhs: Box::new(Expr::Literal(Literal::Integer(4))),
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

        let expected_ast = Expr::BinaryOp {
            op: BinaryOp::Sub,
            lhs: Box::new(Expr::Literal(Literal::Integer(17))),
            rhs: Box::new(Expr::BinaryOp {
                op: BinaryOp::Div,
                lhs: Box::new(Expr::Grouping(Box::new(Expr::BinaryOp {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expr::UnaryOp {
                        op: UnaryOp::Neg,
                        value: Box::new(Expr::Literal(Literal::Integer(5))),
                    }),
                    rhs: Box::new(Expr::Literal(Literal::Integer(5))),
                }))),
                rhs: Box::new(Expr::Literal(Literal::Integer(6))),
            }),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.expression();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
}
