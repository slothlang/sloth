use super::ast::{Expr, FuncArgs, Stmt};
use super::AstParser;
use crate::lexer::TokenType;

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

        // if self.advance_if_eq(&TokenType::For) {
        //     return self.for_statement();
        // }

        if self.advance_if_eq(&TokenType::While) {
            return self.while_statement();
        }

        if self.advance_if_eq(&TokenType::Fn) {
            return self.function_statement();
        }

        if self.advance_if_eq(&TokenType::Return) {
            return self.return_statement();
        }

        // If we couldn't parse a statement return an expression statement
        self.expression_statement()
    }

    fn var_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'var'");
        };

        self.advance(); // Advancing from the identifier TODO: Check for type
        self.consume(TokenType::Eq, "Expected '=' after identifier");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::DefineVariable {
            name: (ident),
            value: (value),
            typ: (None),
        }
    }

    fn val_statement(&mut self) -> Stmt {
        let TokenType::Identifier(ident) = self.peek().tt.clone() else {
            panic!("Identifier expected after 'val'");
        };

        self.advance(); // Advancing from the identifier
        self.consume(TokenType::Eq, "Expected '=' after identifier");

        let value = self.expression();

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");

        Stmt::DefineValue {
            name: (ident),
            value: (value),
            typ: (None),
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

        Stmt::If {
            expr: (condition),
            body: (body),
            else_if: (Vec::new()),
            els: (None),
        } // TODO: implement else if and else
    }

    // fn for_statement(&mut self) -> Stmt {
    //     let binding = self.expression();
    //     let Expr::Variable(binding) = binding else {
    //         panic!("Left side of for statement must be identifier");
    //     };

    //     self.consume(
    //         TokenType::In,
    //         "Expected 'in' in between identifier and range",
    //     );

    //     let range_start = self.expression();
    //     self.consume(
    //         TokenType::DotDot,
    //         "Expected '..' denoting min and max of range",
    //     );
    //     let range_end = self.expression();

    //     let mut body = Vec::new();
    //     while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
    //         body.push(self.statement());
    //     }

    //     Stmt::For { name: (binding), iter: (), body: (body) }
    // } // TODO: Fix this garbage

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
                panic!("Identifier expected after '('");
            };

            self.advance_if_eq(&TokenType::Comma);

            let arg = FuncArgs {
                name: (name),
                typ: (None),
            };
            args.push(arg);
        }
        self.advance();
        self.consume(TokenType::OpeningBrace, "Expected '{' after parameters");
        let mut body = Vec::new();
        while !self.eof() && self.peek().tt != TokenType::ClosingBrace {
            body.push(self.statement());
        }

        Stmt::DefineFunction {
            ident: (ident),
            args: (Some(args)),
            body: (body),
            return_type: (None),
        }
    }

    fn return_statement(&mut self) -> Stmt {
        let expr = self.expression();
        Stmt::Return { value: (expr) }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::{AstParser, Expr, Stmt};
    use crate::lexer::Lexer;
    use crate::parser::ast::{BinaryOp, FuncArgs, Literal};

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
        let lexer = Lexer::new("fn test_c (a, b, c) {\nreturn (b + a * c);\n}");
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
                value: (Expr::BinaryOp {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expr::Variable("a".to_string())),
                    rhs: Box::new(Expr::BinaryOp {
                        op: BinaryOp::Mul,
                        lhs: Box::new(Expr::Variable("b".to_string())),
                        rhs: Box::new(Expr::Variable("c".to_string())),
                    }),
                }),
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
        let lexer = Lexer::new("while true {\nprint(\"Hello World\");}");
        let tokens = lexer.collect_vec();
        println!("{tokens:?}");

        let expected_ast = Stmt::While {
            condition: (Expr::Literal(Literal::Bool(true))),
            body: (vec![Stmt::ExprStmt(Expr::Call {
                ident: (Box::new(Expr::Literal(Literal::String("print".to_string())))),
                args: (vec![Expr::Literal(Literal::String("Hello World".to_string()))]),
            })]),
        };

        let mut parser = AstParser::new(tokens);
        let generated_ast = parser.statement();

        println!("Expected AST:\n{expected_ast:#?}\n\n");
        println!("Generated AST:\n{generated_ast:#?}\n\n");

        assert_eq!(expected_ast, generated_ast);
    }
}
