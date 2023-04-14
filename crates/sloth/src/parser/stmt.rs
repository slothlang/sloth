use super::ast::{Expr, FuncArgs, Stmt};
use super::AstParser;
use crate::lexer::TokenType;
use crate::parser::ast::Literal;

impl<'a> AstParser<'a> {
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.eof() {
            statements.push(self.statement());
        }

        statements
    }

    fn statement(&mut self) -> Stmt {
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

        if self.advance_if_eq(&TokenType::While) {
            return self.while_statement();
        }

        if self.advance_if_eq(&TokenType::Fn) {
            return self.function_statement();
        }

        if self.advance_if_eq(&TokenType::Return) {
            return self.return_statement();
        }

        self.mut_statement()

        // If we couldn't parse a statement return an expression statement
        // self.expression_statement()
    }

    fn mut_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("uh oh {:?}", self.peek());
        };

        self.advance();
        let next = self.advance().unwrap().tt.clone();
        if next == TokenType::Eq {
            let value = self.expression();
            self.consume(TokenType::SemiColon, "No semi colon for me i guess");
            return Stmt::AssignVariable {
                name: (ident),
                value: (value),
            };
        } else if next == TokenType::OpeningParen {
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

            self.consume(TokenType::SemiColon, "No semi colon for me i guess");
            return Stmt::ExprStmt(Expr::Call {
                ident: (Box::new(Expr::Literal(Literal::String(ident)))),
                args: (arguments),
            });
        }
        self.expression_statement()
    }

    fn var_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'var', not {:?}", self.peek());
        };

        self.advance();

        let mut typ: Option<String> = None;
        if self.peek().tt.clone() == TokenType::Colon {
            self.consume(TokenType::Colon, "How did you even get this error?");
            let TokenType::Identifier(name) = self.peek().tt.clone() else {
                panic!("Type expected after identifier, not {:?}", self.peek());
            };
            self.advance();
            typ = Some(name);
        }

        self.consume(TokenType::Eq, "Expected '=' after identifier at ");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::DefineVariable {
            name: (ident),
            value: (value),
            typ: (typ),
        }
    }

    fn val_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'val'");
        };

        self.advance(); // Advancing from the identifier

        let mut typ: Option<String> = None;
        if self.peek().tt.clone() == TokenType::Colon {
            self.consume(TokenType::Colon, "How did you even get this error?");
            let TokenType::Identifier(name) = self.peek().tt.clone() else {
                panic!("Type expected after identifier, not {:?}", self.peek());
            };
            self.advance();
            typ = Some(name);
        }

        self.consume(TokenType::Eq, "Expected '=' after identifier");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::DefineValue {
            name: (ident),
            value: (value),
            typ: (typ),
        }
    }

    fn if_statement(&mut self) -> Stmt {
        let condition = self.expression();

        self.consume(
            TokenType::OpeningBrace,
            "Expected '{' at beggining of block",
        );
        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement());
        }
        self.advance();
        Stmt::If {
            expr: (condition),
            body: (body),
            else_if: (Vec::new()),
            els: (None),
        } // TODO: implement else if and else
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

        // let range_start = self.expression();
        // self.consume(
        //     TokenType::DotDot,
        //     "Expected '..' denoting min and max of range",
        // );
        // let range_end = self.expression();

        let expr = self.expression();

        self.consume(TokenType::OpeningBrace, "Expected '{' after iterator");

        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement());
        }

        Stmt::For {
            name: (binding),
            iter: (expr),
            body: (body),
        }
    } // TODO: Fix this garbage

    fn while_statement(&mut self) -> Stmt {
        let condition = self.expression();

        self.consume(
            TokenType::OpeningBrace,
            "Expected '{' at beggining of block",
        );
        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement());
        }
        self.advance();
        Stmt::While { condition, body }
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();

        // FIXME: Move assignment handling
        // if self.advance_if_eq(&TokenType::Eq) {
        //     if let Expr::Literal(_ident) = &expr {
        //         let value = self.expression();

        //         self.consume(TokenType::SemiColon, "Expected ';' at end of
        // statement");         // return Stmt::DefineVariable {
        //         //     name: (ident.clone()),
        //         //     value: (value),
        //         //     typ: (None),
        //         // };
        //         return Stmt::ExprStmt(expr);
        //     }
        // }

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
        Stmt::ExprStmt(expr)
    }

    fn function_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.advance().unwrap().tt.clone() else {
            panic!("Identifier expected after 'fn'");
        };

        self.consume(TokenType::OpeningParen, "Expected '(' after identifier");
        let mut args: Vec<FuncArgs> = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingParen {
            let TokenType::Identifier(name) = self.advance().unwrap().tt.clone() else {
                panic!("parameter expected after '('");
            };

            let mut typ: Option<String> = None;

            if self.peek().tt.clone() == TokenType::Colon {
                self.consume(TokenType::Colon, "How did you even get this error?");
                let TokenType::Identifier(name) = self.peek().tt.clone() else {
                    panic!("Type expected after ':', not {:?}", self.peek());
                };
                self.advance();
                typ = Some(name);
            }

            self.advance_if_eq(&TokenType::Comma);

            let arg = FuncArgs {
                name: (name),
                typ: (typ),
            };
            args.push(arg);
        }
        self.advance();
        let mut typ: Option<String> = None;
        if self.peek().tt.clone() == TokenType::Arrow {
            self.advance();
            let TokenType::Identifier(name) = self.peek().tt.clone() else {
                panic!("Type expected after ':', not {:?}", self.peek());
            };
            typ = Some(name);
            self.advance();
        }
        self.consume(TokenType::OpeningBrace, "Expected '{' after parameters");
        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement());
        }

        Stmt::DefineFunction {
            ident: (ident),
            args: (Some(args)),
            body: (body),
            return_type: (typ),
        }
    }

    fn return_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.consume(TokenType::SemiColon, "Expected ';' after return statement");
        Stmt::Return { value: (expr) }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{AstParser, Stmt};
    use crate::lexer::Lexer;
    use crate::parser::ast::{BinaryOp, Expr, FuncArgs, Literal, UnaryOp};

    #[test]
    fn basic_statement_a() {
        let lexer = Lexer::new("var test_a = 5 + 3;");
        let tokens = lexer.collect_vec();

        let expected_ast = Stmt::DefineVariable {
            name: ("test_a".to_string()),
            value: (Expr::BinaryOp {
                op: (BinaryOp::Add),
                lhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                rhs: (Box::new(Expr::Literal(Literal::Integer(3)))),
            }),
            typ: (None),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_statement_b() {
        let lexer = Lexer::new("val test_b = \"Hello World\";");
        let tokens = lexer.collect_vec();

        let expected_ast = Stmt::DefineValue {
            name: ("test_b".to_string()),
            value: (Expr::Literal(Literal::String("Hello World".to_string()))),
            typ: (None),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_statement_c() {
        let lexer = Lexer::new(
            "\
        fn test_c (a, b, c) {\nreturn (a + b * c);\n}",
        );
        let tokens = lexer.collect_vec();
        println!("{tokens:?}");

        let expected_ast = Stmt::DefineFunction {
            ident: ("test_c".to_string()),
            args: Some(vec![
                FuncArgs {
                    name: ("a".to_string()),
                    typ: None,
                },
                FuncArgs {
                    name: ("b".to_string()),
                    typ: None,
                },
                FuncArgs {
                    name: ("c".to_string()),
                    typ: None,
                },
            ]),
            body: (vec![Stmt::Return {
                value: (Expr::Grouping(Box::new(Expr::BinaryOp {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expr::Variable("a".to_string())),
                    rhs: Box::new(Expr::BinaryOp {
                        op: BinaryOp::Mul,
                        lhs: Box::new(Expr::Variable("b".to_string())),
                        rhs: Box::new(Expr::Variable("c".to_string())),
                    }),
                }))),
            }]),
            return_type: (None),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
    #[test]
    fn basic_statement_d() {
        let lexer = Lexer::new(
            "\
            while true {\nprint(\"Hello World\");\nprintln(5 + 7/-3);\n}",
        );
        let tokens = lexer.collect_vec();
        println!("{tokens:?}");

        let expected_ast = Stmt::While {
            condition: (Expr::Literal(Literal::Bool(true))),
            body: (vec![
                Stmt::ExprStmt(Expr::Call {
                    ident: (Box::new(Expr::Literal(Literal::String("print".to_string())))),
                    args: (vec![Expr::Literal(Literal::String("Hello World".to_string()))]),
                }),
                Stmt::ExprStmt(Expr::Call {
                    ident: (Box::new(Expr::Literal(Literal::String("println".to_string())))),
                    args: (vec![Expr::BinaryOp {
                        op: (BinaryOp::Add),
                        lhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                        rhs: (Box::new(Expr::BinaryOp {
                            op: (BinaryOp::Div),
                            lhs: (Box::new(Expr::Literal(Literal::Integer(7)))),
                            rhs: (Box::new(Expr::UnaryOp {
                                op: (UnaryOp::Neg),
                                value: (Box::new(Expr::Literal(Literal::Integer(3)))),
                            })),
                        })),
                    }]),
                }),
            ]),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
    #[test]
    fn basic_statement_e() {
        let lexer = Lexer::new(
            "\
            if a+5 > 10 {\nprint(a);\n}\nif a+5 < 10 {\nprintln(10);\n}\nif a+5 == 10 \
             {\nprint(toString(10));\na = true;\n}",
        );
        let tokens = lexer.collect_vec();
        // println!("{tokens:?}");

        let expected_ast = vec![
            Stmt::If {
                expr: (Expr::BinaryOp {
                    op: (BinaryOp::Gt),
                    lhs: (Box::new(Expr::BinaryOp {
                        op: (BinaryOp::Add),
                        lhs: (Box::new(Expr::Variable("a".to_string()))),
                        rhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                    })),
                    rhs: (Box::new(Expr::Literal(Literal::Integer(10)))),
                }),
                body: (vec![Stmt::ExprStmt(Expr::Call {
                    ident: (Box::new(Expr::Literal(Literal::String("print".to_string())))),
                    args: (vec![Expr::Variable("a".to_string())]),
                })]),
                else_if: (Vec::new()),
                els: (None),
            },
            Stmt::If {
                expr: (Expr::BinaryOp {
                    op: (BinaryOp::Lt),
                    lhs: (Box::new(Expr::BinaryOp {
                        op: (BinaryOp::Add),
                        lhs: (Box::new(Expr::Variable("a".to_string()))),
                        rhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                    })),
                    rhs: (Box::new(Expr::Literal(Literal::Integer(10)))),
                }),
                body: (vec![Stmt::ExprStmt(Expr::Call {
                    ident: (Box::new(Expr::Literal(Literal::String("println".to_string())))),
                    args: (vec![Expr::Literal(Literal::Integer(10))]),
                })]),
                else_if: (Vec::new()),
                els: (None),
            },
            Stmt::If {
                expr: (Expr::BinaryOp {
                    op: (BinaryOp::EqEq),
                    lhs: (Box::new(Expr::BinaryOp {
                        op: (BinaryOp::Add),
                        lhs: (Box::new(Expr::Variable("a".to_string()))),
                        rhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                    })),
                    rhs: (Box::new(Expr::Literal(Literal::Integer(10)))),
                }),
                body: (vec![
                    Stmt::ExprStmt(Expr::Call {
                        ident: (Box::new(Expr::Literal(Literal::String("print".to_string())))),
                        args: (vec![Expr::Call {
                            ident: Box::new(Expr::Literal(Literal::String("toString".to_string()))),
                            args: vec![Expr::Literal(Literal::Integer(10))],
                        }]),
                    }),
                    Stmt::AssignVariable {
                        name: ("a".to_string()),
                        value: (Expr::Literal(Literal::Bool(true))),
                    },
                ]),

                else_if: (Vec::new()),
                els: (None),
            },
        ];

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.parse();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_statement_f() {
        let lexer = Lexer::new("test_a = 5 + 3;");
        let tokens = lexer.collect_vec();

        let expected_ast = Stmt::AssignVariable {
            name: ("test_a".to_string()),
            value: (Expr::BinaryOp {
                op: (BinaryOp::Add),
                lhs: (Box::new(Expr::Literal(Literal::Integer(5)))),
                rhs: (Box::new(Expr::Literal(Literal::Integer(3)))),
            }),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
    #[test]
    fn basic_statement_g() {
        let lexer = Lexer::new(
            "\
        fn times_two(x: int) -> int {\nval y: int = x*2;\nreturn y;\n}",
        );
        let tokens = lexer.collect_vec();

        let expected_ast = Stmt::DefineFunction {
            ident: ("times_two".to_string()),
            args: (Some(vec![FuncArgs {
                name: ("x".to_string()),
                typ: (Some("int".to_string())),
            }])),
            body: (vec![
                Stmt::DefineValue {
                    name: "y".to_string(),
                    value: (Expr::BinaryOp {
                        op: (BinaryOp::Mul),
                        lhs: (Box::new(Expr::Variable("x".to_string()))),
                        rhs: (Box::new(Expr::Literal(Literal::Integer(2)))),
                    }),
                    typ: Some("int".to_string()),
                },
                Stmt::Return {
                    value: (Expr::Variable("y".to_string())),
                },
            ]),

            return_type: Some("int".to_string()),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }

    #[test]
    fn basic_statement_h() {
        let lexer = Lexer::new("for i in 1 .. 3 {\nfor j in [1, 2, 3] {\nprint(j*i);}}");
        let tokens = lexer.collect_vec();

        let expected_ast = Stmt::For {
            name: ("i".to_string()),
            iter: (Expr::BinaryOp {
                op: (BinaryOp::Range),
                lhs: (Box::new(Expr::Literal(Literal::Integer(1)))),
                rhs: (Box::new(Expr::Literal(Literal::Integer(3)))),
            }),
            body: (vec![Stmt::For {
                name: ("j".to_string()),
                iter: (Expr::Literal(Literal::List(vec![
                    Expr::Literal(Literal::Integer(1)),
                    Expr::Literal(Literal::Integer(2)),
                    Expr::Literal(Literal::Integer(3)),
                ]))),
                body: (vec![Stmt::ExprStmt(Expr::Call {
                    ident: Box::new(Expr::Literal(Literal::String("print".to_string()))),
                    args: (vec![Expr::BinaryOp {
                        op: (BinaryOp::Mul),
                        lhs: (Box::new(Expr::Variable("j".to_string()))),
                        rhs: (Box::new(Expr::Variable("i".to_string()))),
                    }]),
                })]),
            }]),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
}
