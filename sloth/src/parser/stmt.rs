use std::collections::HashMap;

use super::ast::{Function, FunctionInput, FunctionKind, Stmt, StmtKind};
use super::{AstParser, ParsingError};
use crate::lexer::TokenType;

impl<'a> AstParser<'a> {
    pub(super) fn statement(&mut self) -> Result<Stmt, ParsingError> {
        // Checking what the next token is in order to figure out how we proceed. If
        // it's not one of these tokens we assume it's an expression statement
        // and parse the expression statement.
        match self.peek().tt {
            TokenType::OpeningBrace => self.block(),

            TokenType::Foreign => self.foreign(),

            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
            TokenType::Var => self.define_variable(),
            TokenType::Val => self.define_value(),
            TokenType::Struct => self.define_struct(),
            TokenType::Fn => self.define_function(false),
            TokenType::Return => self.return_stmt(),

            _ if self.peek2().tt == TokenType::Eq => self.assign_variable(),
            _ => self.expression_stmt(),
        }
    }

    fn foreign(&mut self) -> Result<Stmt, ParsingError> {
        // Consume the foreign token
        self.consume(TokenType::Foreign, "Expected foreign")?;

        // Foreign allows for you to interact with languages other than Sloth. When
        // Sloth sees a foreign keyword it expects something to follow
        // determining what from the other language you want to get, this is
        // similar to the "statement" function but more trimmed down.
        match &self.peek().tt {
            TokenType::Fn => self.define_function(true),

            tt => Err(ParsingError::UnexpectedToken(self.line, tt.clone(), "")),
        }
    }

    fn if_stmt(&mut self) -> Result<Stmt, ParsingError> {
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

        // Build the actual if statement kind
        let kind = StmtKind::IfStmt {
            condition,
            if_then: if_then.into(),
            else_then: else_then.map(|it| it.into()),
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn while_stmt(&mut self) -> Result<Stmt, ParsingError> {
        // Consume the while token
        self.consume(TokenType::While, "Expected while")?;

        let condition = self.expression()?;
        let body = self.block()?;

        let kind = StmtKind::WhileStmt {
            condition,
            body: body.into(),
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn for_stmt(&mut self) -> Result<Stmt, ParsingError> {
        // Consume the for token
        self.consume(TokenType::For, "Expected for")?;

        let identifier = self.consume_identifier()?;
        self.consume(TokenType::In, "Expected in")?;
        let iterator = self.expression()?;

        let body = self.block()?;

        let kind = StmtKind::ForStmt {
            iterator,
            identifier,
            body: body.into(),
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    // TODO: Make variable types optional
    fn define_variable(&mut self) -> Result<Stmt, ParsingError> {
        // Consume the var token
        self.consume(TokenType::Var, "Expected var")?;

        // Get the identifier and type
        let identifier = self.consume_identifier()?;
        let typ = if self.consume(TokenType::Colon, "Expected ':'").is_ok() {
            self.consume_type().ok()
        } else {
            None
        };

        // Get the default value
        self.consume(TokenType::Eq, "Expected '='")?;
        let value = self.expression()?;

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;

        let kind = StmtKind::DefineVariable {
            identifier,
            value,
            typ,
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn define_struct(&mut self) -> Result<Stmt, ParsingError> {
        let mut properties = HashMap::new();

        self.consume(TokenType::Struct, "Expected struct")?;

        let identifier = self.consume_identifier()?;

        while self.peek().tt != TokenType::ClosingBracket {
            self.consume(TokenType::Val, "Expected val in struct!")?;

            let ident = self.consume_identifier()?;
            let typ = self.consume_type()?;

            properties.insert(ident, typ).ok_or(0);
        }
        self.consume(TokenType::ClosingBracket, "Expected '}' at end of struct");

        let kind = StmtKind::DefineStruct {
            identifier,
            properties,
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn define_value(&mut self) -> Result<Stmt, ParsingError> {
        // Consume the val token
        self.consume(TokenType::Val, "Expected val")?;

        // Get the identifier and type
        let identifier = self.consume_identifier()?;
        let typ = if self.consume(TokenType::Colon, "Expected ':'").is_ok() {
            self.consume_type().ok()
        } else {
            None
        };

        // Get the default value
        self.consume(TokenType::Eq, "Expected '='")?;
        let value = self.expression()?;

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;

        let kind = StmtKind::DefineValue {
            identifier,
            value,
            typ,
        };

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }
    // TODO: Make argument types optional
    fn define_function(&mut self, is_foreign: bool) -> Result<Stmt, ParsingError> {
        // Consume the fn token
        self.consume(TokenType::Fn, "Expected fn")?;

        let identifier = self.consume_identifier()?;

        // Get the function inputs
        self.consume(TokenType::OpeningParen, "Expected '('")?;

        let mut inputs = Vec::new();
        while matches!(self.peek().tt, TokenType::Identifier(_)) {
            let input_identifier = self.consume_identifier()?;
            self.consume(TokenType::Colon, "Expected ':'")?;
            let input_type = self.consume_type()?;

            inputs.push(FunctionInput {
                identifier: input_identifier,
                typ: input_type,
            });

            if self.peek().tt != TokenType::Comma {
                break;
            }

            self.consume(TokenType::Comma, "Expected ','")?;
        }

        self.consume(TokenType::ClosingParen, "Expected ')'")?;

        // Get the function output
        let output = if matches!(
            self.peek().tt,
            TokenType::Identifier(_) | TokenType::OpeningBracket
        ) {
            Some(self.consume_type()?)
        } else {
            None
        };

        // Get the function kind
        let kind = if is_foreign {
            self.consume(TokenType::SemiColon, "Expected semicolon")?;
            FunctionKind::Foreign
        } else {
            FunctionKind::Normal {
                body: Box::new(self.block()?),
            }
        };

        let stmt = StmtKind::DefineFunction(Function {
            identifier,
            inputs,
            output,
            kind,
        });

        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            stmt,
            self.top.clone(),
        ))
    }

    fn return_stmt(&mut self) -> Result<Stmt, ParsingError> {
        self.consume(TokenType::Return, "Expected return")?;
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        let kind = StmtKind::Return(value);
        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn assign_variable(&mut self) -> Result<Stmt, ParsingError> {
        let identifier = self.consume_identifier()?;
        self.consume(TokenType::Eq, "Expected '='")?;
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        let kind = StmtKind::AssignVariable { identifier, value };
        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn expression_stmt(&mut self) -> Result<Stmt, ParsingError> {
        let expr = self.expression()?;
        self.consume(TokenType::SemiColon, "Expected ';' at end of statement")?;
        let kind = StmtKind::ExprStmt(expr);
        Ok(Stmt::new(
            self.reserve_id(),
            self.line,
            kind,
            self.top.clone(),
        ))
    }

    fn block(&mut self) -> Result<Stmt, ParsingError> {
        // This inner function exists to make cleanup of the pushed symbol table easier
        // in the case of a parsing error.
        fn inner(this: &mut AstParser) -> Result<Stmt, ParsingError> {
            // Consume the opening brace
            this.consume(TokenType::OpeningBrace, "Expected '{'")?;

            // Get the body of the block
            let mut body = Vec::new();
            while !this.eof() && this.peek().tt != TokenType::ClosingBrace {
                body.push(this.statement()?);
            }

            // Consume the closing brace
            this.consume(TokenType::ClosingBrace, "Expected '}'")?;

            let kind = StmtKind::Block(body);

            Ok(Stmt::new(
                this.reserve_id(),
                this.line,
                kind,
                this.top.clone(),
            ))
        }

        // Push a table, call the inner function and then pop that table
        self.push_table();
        let result = inner(self);
        self.pop_table();

        result
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{AstParser, StmtKind};
    use crate::lexer::Lexer;
    use crate::parser::ast::{
        BinaryOp, Expr, ExprKind, Function, FunctionInput, FunctionKind, Literal, Stmt,
        TypeIdentifier,
    };
    use crate::symtable::SymbolTable;

    #[test]
    fn standalone_blocks() {
        let tokens = Lexer::new("{{{ 0; }}}").collect_vec();

        let expected_ast = Ok(Stmt::without_table(
            4,
            StmtKind::Block(vec![Stmt::without_table(
                3,
                StmtKind::Block(vec![Stmt::without_table(
                    2,
                    StmtKind::Block(vec![Stmt::without_table(
                        1,
                        StmtKind::ExprStmt(Expr::without_table(0, Literal::Integer(0).into())),
                    )]),
                )]),
            )]),
        ));

        let mut parser = AstParser::new(tokens, SymbolTable::new());
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_variable_definition() {
        let tokens = Lexer::new("var foo: Int = 5 + 3;").collect_vec();

        let expected_ast = Ok(Stmt::without_table(3, StmtKind::DefineVariable {
            identifier: "foo".to_string(),
            value: Expr::without_table(2, ExprKind::BinaryOp {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::without_table(
                    0,
                    ExprKind::Literal(Literal::Integer(5)),
                )),
                rhs: Box::new(Expr::without_table(
                    1,
                    ExprKind::Literal(Literal::Integer(3)),
                )),
            }),
            typ: Some(TypeIdentifier {
                name: "Int".to_string(),
                is_list: false,
            }),
        }));

        let mut parser = AstParser::new(tokens, SymbolTable::new());
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_function() {
        let tokens = Lexer::new(
            r#"
            fn foo(bar: Int) Int {
                var baz: Int = bar + 1;
                baz = baz + 1;
                return baz;
            }
        "#,
        )
        .collect_vec();

        let expected_ast = Ok(Stmt::without_table(
            11,
            StmtKind::DefineFunction(Function {
                identifier: "foo".to_owned(),
                inputs: vec![FunctionInput {
                    identifier: "bar".to_owned(),
                    typ: TypeIdentifier {
                        name: "Int".to_owned(),
                        is_list: false,
                    },
                }],
                output: Some(TypeIdentifier {
                    name: "Int".to_owned(),
                    is_list: false,
                }),
                kind: FunctionKind::Normal {
                    body: Box::new(Stmt::without_table(
                        10,
                        StmtKind::Block(vec![
                            Stmt::without_table(3, StmtKind::DefineVariable {
                                identifier: "baz".to_owned(),
                                value: Expr::without_table(2, ExprKind::BinaryOp {
                                    op: BinaryOp::Add,
                                    lhs: Box::new(Expr::without_table(
                                        0,
                                        ExprKind::Identifier("bar".to_owned()),
                                    )),
                                    rhs: Box::new(Expr::without_table(
                                        1,
                                        Literal::Integer(1).into(),
                                    )),
                                }),
                                typ: Some(TypeIdentifier {
                                    name: "Int".to_owned(),
                                    is_list: false,
                                }),
                            }),
                            Stmt::without_table(7, StmtKind::AssignVariable {
                                identifier: "baz".to_owned(),
                                value: Expr::without_table(6, ExprKind::BinaryOp {
                                    op: BinaryOp::Add,
                                    lhs: Box::new(Expr::without_table(
                                        4,
                                        ExprKind::Identifier("baz".to_owned()),
                                    )),
                                    rhs: Box::new(Expr::without_table(
                                        5,
                                        Literal::Integer(1).into(),
                                    )),
                                }),
                            }),
                            Stmt::without_table(
                                9,
                                StmtKind::Return(Expr::without_table(
                                    8,
                                    ExprKind::Identifier("baz".to_owned()),
                                )),
                            ),
                        ]),
                    )),
                },
            }),
        ));

        let mut parser = AstParser::new(tokens, SymbolTable::new());
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_conditional() {
        let tokens = Lexer::new(
            r#"
            if foo {
                0;
            } else if bar {
                1;
            } else if baz {
                2;
            } else {
                3;
            }
        "#,
        )
        .collect_vec();

        let expected_ast = Ok(Stmt::without_table(17, StmtKind::IfStmt {
            condition: Expr::without_table(0, ExprKind::Identifier("foo".to_owned())),
            if_then: Box::new(Stmt::without_table(
                3,
                StmtKind::Block(vec![Stmt::without_table(
                    2,
                    StmtKind::ExprStmt(Expr::without_table(1, Literal::Integer(0).into())),
                )]),
            )),
            else_then: Some(Box::new(Stmt::without_table(16, StmtKind::IfStmt {
                condition: Expr::without_table(4, ExprKind::Identifier("bar".to_owned())),
                if_then: Box::new(Stmt::without_table(
                    7,
                    StmtKind::Block(vec![Stmt::without_table(
                        6,
                        StmtKind::ExprStmt(Expr::without_table(5, Literal::Integer(1).into())),
                    )]),
                )),
                else_then: Some(Box::new(Stmt::without_table(15, StmtKind::IfStmt {
                    condition: Expr::without_table(8, ExprKind::Identifier("baz".to_owned())),
                    if_then: Box::new(Stmt::without_table(
                        11,
                        StmtKind::Block(vec![Stmt::without_table(
                            10,
                            StmtKind::ExprStmt(Expr::without_table(9, Literal::Integer(2).into())),
                        )]),
                    )),
                    else_then: Some(Box::new(Stmt::without_table(
                        14,
                        StmtKind::Block(vec![Stmt::without_table(
                            13,
                            StmtKind::ExprStmt(Expr::without_table(12, Literal::Integer(3).into())),
                        )]),
                    ))),
                }))),
            }))),
        }));

        let mut parser = AstParser::new(tokens, SymbolTable::new());
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
}
