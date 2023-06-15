use super::ast::{FunctionInput, StmtKind};
use super::{AstParser, ParsingError};
use crate::lexer::TokenType;

impl<'a> AstParser<'a> {
    pub(super) fn statement(&mut self) -> Result<StmtKind, ParsingError> {
        match self.peek().tt {
            TokenType::OpeningBrace => self.block(),

            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::Var => self.define_variable(),
            TokenType::Fn => self.define_function(),
            TokenType::Return => self.return_stmt(),

            _ if self.peek2().tt == TokenType::Eq => self.assign_variable(),
            _ => self.expression_stmt(),
        }
    }

    fn if_stmt(&mut self) -> Result<StmtKind, ParsingError> {
        // Consume the if token
        self.consume(TokenType::If, "Expected if")?;

        // Get the condition and if_then of the if statement
        let condition = self.expression()?;
        let if_then = self.block()?;

        // Check if there is an else
        let mut else_then = None;
        if self.advance_if_eq(&TokenType::Else) {
            if self.peek().tt == TokenType::If {
                else_then = Some(self.if_stmt()?);
            } else {
                else_then = Some(self.block()?);
            }
        }

        Ok(StmtKind::IfStmt {
            condition,
            if_then: if_then.into(),
            else_then: else_then.map(|it| it.into()),
        })
    }

    fn while_stmt(&mut self) -> Result<StmtKind, ParsingError> {
        // Consume the while token
        self.consume(TokenType::While, "Expected while")?;

        let condition = self.expression()?;
        let body = self.block()?;

        Ok(StmtKind::WhileStmt {
            condition,
            body: body.into(),
        })
    }

    // TODO: Make variable types optional
    fn define_variable(&mut self) -> Result<StmtKind, ParsingError> {
        // Consume the var token
        self.consume(TokenType::Var, "Expected var")?;

        // Get the identifier and type
        let identifier = self.consume_identifier()?;
        self.consume(TokenType::Colon, "Expected ':'")?;
        let typ = self.consume_identifier()?;

        // Get the default value
        self.consume(TokenType::Eq, "Expected '='")?;
        let value = self.expression()?;

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;

        Ok(StmtKind::DefineVariable {
            identifier,
            value,
            typ,
        })
    }

    // TODO: Make argument types optional
    fn define_function(&mut self) -> Result<StmtKind, ParsingError> {
        // Consume the fn token
        self.consume(TokenType::Fn, "Expected fn")?;

        let identifier = self.consume_identifier()?;

        // Get the function inputs
        self.consume(TokenType::OpeningParen, "Expected '('")?;

        let mut inputs = Vec::new();
        while matches!(self.peek().tt, TokenType::Identifier(_)) {
            let input_identifier = self.consume_identifier()?;
            self.consume(TokenType::Colon, "Expected ':'")?;
            let input_type = self.consume_identifier()?;

            inputs.push(FunctionInput {
                identifier: input_identifier,
                typ: input_type,
            });
        }

        self.consume(TokenType::ClosingParen, "Expected ')'")?;

        // Get the function output
        let output = if matches!(self.peek().tt, TokenType::Identifier(_)) {
            Some(self.consume_identifier()?)
        } else {
            None
        };

        // Get the function body
        let body = self.block()?;

        Ok(StmtKind::DefineFunction {
            identifier,
            inputs,
            output,
            body: body.into(),
        })
    }

    fn return_stmt(&mut self) -> Result<StmtKind, ParsingError> {
        self.consume(TokenType::Return, "Expected return")?;
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        Ok(StmtKind::Return(value))
    }

    fn assign_variable(&mut self) -> Result<StmtKind, ParsingError> {
        let identifier = self.consume_identifier()?;
        self.consume(TokenType::Eq, "Expected '='")?;
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        Ok(StmtKind::AssignVariable { identifier, value })
    }

    fn expression_stmt(&mut self) -> Result<StmtKind, ParsingError> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        Ok(StmtKind::ExprStmt(expr))
    }

    fn block(&mut self) -> Result<StmtKind, ParsingError> {
        // Consume the opening brace
        self.consume(TokenType::OpeningBrace, "Expected '{'")?;

        // Get the body of the block
        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement()?);
        }

        // Consume the closing brace
        self.consume(TokenType::ClosingBrace, "Expected '}'")?;

        Ok(StmtKind::Block(body))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{AstParser, StmtKind};
    use crate::lexer::Lexer;
    use crate::parser::ast::{BinaryOp, ExprKind, FunctionInput, Literal};

    // #[test]
    // fn standalone_blocks() {
    //     let tokens = Lexer::new("{{{ 0; }}}").collect_vec();
    //
    //     let expected_ast =
    // Ok(Stmt::Block(vec![Stmt::Block(vec![Stmt::Block(vec![
    //         Stmt::ExprStmt(Literal::Integer(0).into()),
    //     ])])]));
    //
    //     let mut parser = AstParser::new(tokens);
    //     let generated_ast = parser.statement();
    //
    //     println!("Expected AST:\n{expected_ast:#?}\n\n");
    //     println!("Generated AST:\n{generated_ast:#?}\n\n");
    //
    //     assert_eq!(expected_ast, generated_ast);
    // }
    //
    // #[test]
    // fn basic_variable_definition() {
    //     let tokens = Lexer::new("var foo: Int = 5 + 3;").collect_vec();
    //
    //     let expected_ast = Ok(Stmt::DefineVariable {
    //         identifier: "foo".to_string(),
    //         value: ExprKind::BinaryOp {
    //             op: BinaryOp::Add,
    //             lhs: Box::new(ExprKind::Literal(Literal::Integer(5))),
    //             rhs: Box::new(ExprKind::Literal(Literal::Integer(3))),
    //         },
    //         typ: "Int".to_string(),
    //     });
    //
    //     let mut parser = AstParser::new(tokens);
    //     let generated_ast = parser.statement();
    //
    //     println!("Expected AST:\n{expected_ast:#?}\n\n");
    //     println!("Generated AST:\n{generated_ast:#?}\n\n");
    //
    //     assert_eq!(expected_ast, generated_ast);
    // }
    //
    // #[test]
    // fn basic_function() {
    //     let tokens = Lexer::new(
    //         r#"
    //         fn foo(bar: Int) Int {
    //             var baz: Int = bar + 1;
    //             baz = baz + 1;
    //             return baz;
    //         }
    //     "#,
    //     )
    //     .collect_vec();
    //
    //     let expected_ast = Ok(Stmt::DefineFunction {
    //         identifier: "foo".to_owned(),
    //         inputs: vec![FunctionInput {
    //             identifier: "bar".to_owned(),
    //             typ: "Int".to_owned(),
    //         }],
    //         output: Some("Int".to_owned()),
    //         body: Box::new(Stmt::Block(vec![
    //             Stmt::DefineVariable {
    //                 identifier: "baz".to_owned(),
    //                 value: ExprKind::BinaryOp {
    //                     op: BinaryOp::Add,
    //                     lhs:
    // Box::new(ExprKind::Identifier("bar".to_owned())),
    // rhs: Box::new(Literal::Integer(1).into()),                 },
    //                 typ: "Int".to_owned(),
    //             },
    //             Stmt::AssignVariable {
    //                 identifier: "baz".to_owned(),
    //                 value: ExprKind::BinaryOp {
    //                     op: BinaryOp::Add,
    //                     lhs:
    // Box::new(ExprKind::Identifier("baz".to_owned())),
    // rhs: Box::new(Literal::Integer(1).into()),                 },
    //             },
    //             Stmt::Return(ExprKind::Identifier("baz".to_owned())),
    //         ])),
    //     });
    //
    //     let mut parser = AstParser::new(tokens);
    //     let generated_ast = parser.statement();
    //
    //     println!("Expected AST:\n{expected_ast:#?}\n\n");
    //     println!("Generated AST:\n{generated_ast:#?}\n\n");
    //
    //     assert_eq!(expected_ast, generated_ast);
    // }
    //
    // #[test]
    // fn basic_conditional() {
    //     let tokens = Lexer::new(
    //         r#"
    //         if foo {
    //             0;
    //         } else if bar {
    //             1;
    //         } else if baz {
    //             2;
    //         } else {
    //             3;
    //         }
    //     "#,
    //     )
    //     .collect_vec();
    //
    //     let expected_ast = Ok(Stmt::IfStmt {
    //         condition: ExprKind::Identifier("foo".to_owned()),
    //         if_then: Box::new(Stmt::Block(vec![Stmt::ExprStmt(
    //             Literal::Integer(0).into(),
    //         )])),
    //         else_then: Some(Box::new(Stmt::IfStmt {
    //             condition: ExprKind::Identifier("bar".to_owned()),
    //             if_then: Box::new(Stmt::Block(vec![Stmt::ExprStmt(
    //                 Literal::Integer(1).into(),
    //             )])),
    //             else_then: Some(Box::new(Stmt::IfStmt {
    //                 condition: ExprKind::Identifier("baz".to_owned()),
    //                 if_then: Box::new(Stmt::Block(vec![Stmt::ExprStmt(
    //                     Literal::Integer(2).into(),
    //                 )])),
    //                 else_then: Some(Box::new(Stmt::Block(vec![Stmt::ExprStmt(
    //                     Literal::Integer(3).into(),
    //                 )]))),
    //             })),
    //         })),
    //     });
    //
    //     let mut parser = AstParser::new(tokens);
    //     let generated_ast = parser.statement();
    //
    //     println!("Expected AST:\n{expected_ast:#?}\n\n");
    //     println!("Generated AST:\n{generated_ast:#?}\n\n");
    //
    //     assert_eq!(expected_ast, generated_ast);
    // }
}
