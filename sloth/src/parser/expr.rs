use super::ast::{Expr, UnaryOp};
use super::AstParser;
use crate::lexer::TokenType;
use crate::parser::ast::{BinaryOp, ExprKind};
use crate::parser::ParsingError;

impl<'a> AstParser<'a> {
    pub(super) fn expression(&mut self) -> Result<Expr, ParsingError> {
        self.logical_or()
    }

    fn unary(&mut self) -> Result<Expr, ParsingError> {
        if matches!(self.peek().tt, TokenType::Bang | TokenType::Minus) {
            let oeprator_tt = self.advance().unwrap().tt.clone();
            let operator = UnaryOp::try_from(oeprator_tt)?;

            let value = self.unary()?;

            let kind = ExprKind::UnaryOp {
                op: operator,
                value: Box::new(value),
            };

            return Ok(Expr::new(self.reserve_id(), kind));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParsingError> {
        let kind = match self.advance().unwrap().tt.clone() {
            TokenType::Literal(literal) => ExprKind::Literal(literal.into()),
            TokenType::Identifier(identifier) => ExprKind::Identifier(identifier),

            TokenType::OpeningParen => {
                let expr = self.expression()?;
                self.consume(TokenType::ClosingParen, "Must end grouping with ')'")?;
                ExprKind::Grouping(Box::new(expr))
            }

            _ => return Err(ParsingError::UnexpectedToken),
        };

        Ok(Expr::new(self.reserve_id(), kind))
    }
}

// Macro to generate repetitive binary expressions. Things like addition,
// multiplication, exc.
macro_rules! binary_expr {
    ($name:ident, $parent:ident, $pattern:pat) => {
        fn $name(&mut self) -> Result<Expr, ParsingError> {
            let mut expr = self.$parent()?;

            while !self.eof() && matches!(self.peek().tt, $pattern) {
                let operator_tt = self.advance().unwrap().tt.clone();
                let operator = BinaryOp::try_from(operator_tt)?;

                let rhs = self.$parent()?;
                let kind = ExprKind::BinaryOp {
                    op: operator,
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                };

                expr = Expr::new(self.reserve_id(), kind);
            }

            Ok(expr)
        }
    };
}

#[rustfmt::skip]
#[allow(unused_parens)]
impl<'a> AstParser<'a> {    
    // Binary expressions in order of precedence from lowest to highest.
    binary_expr!(logical_or      , logical_and   , (TokenType::PipePipe));
    binary_expr!(logical_and     , range         , (TokenType::AmpAmp));
    binary_expr!(range           , equality      , (TokenType::DotDot));
    binary_expr!(equality        , comparison    , (TokenType::BangEq | TokenType::EqEq));
    binary_expr!(comparison      , additive      , (TokenType::Lt     | TokenType::Gt    | TokenType::LtEq | TokenType::GtEq));
    binary_expr!(additive        , multiplicative, (TokenType::Plus   | TokenType::Minus));
    binary_expr!(multiplicative  , unary         , (TokenType::Star   | TokenType::Slash | TokenType::Perc));
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::lexer::Lexer;
    use crate::parser::ast::{BinaryOp, Expr, ExprKind, Literal};
    use crate::AstParser;

    #[test]
    fn basic_expression() {
        let lexer = Lexer::new("3 + 5 * 4 - 9 / 3");
        let tokens = lexer.collect_vec();

        let expected_ast = Ok(Expr::new(8, ExprKind::BinaryOp {
            op: BinaryOp::Sub,
            lhs: Box::new(Expr::new(4, ExprKind::BinaryOp {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::new(0, ExprKind::Literal(Literal::Integer(3)))),
                rhs: Box::new(Expr::new(3, ExprKind::BinaryOp {
                    op: BinaryOp::Mul,
                    lhs: Box::new(Expr::new(1, ExprKind::Literal(Literal::Integer(5)))),
                    rhs: Box::new(Expr::new(2, ExprKind::Literal(Literal::Integer(4)))),
                })),
            })),
            rhs: Box::new(Expr::new(7, ExprKind::BinaryOp {
                op: BinaryOp::Div,
                lhs: Box::new(Expr::new(5, ExprKind::Literal(Literal::Integer(9)))),
                rhs: Box::new(Expr::new(6, ExprKind::Literal(Literal::Integer(3)))),
            })),
        }));

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.expression();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
}
