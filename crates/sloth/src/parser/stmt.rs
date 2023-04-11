use super::ast::{AstParser, Expr, Stmt};
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
    // } TODO: Fix this garbage

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
        if self.advance_if_eq(&TokenType::Eq) {
            if let Expr::Variable(ident) = &expr {
                let value = self.expression();

                self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
                return Stmt::DefineVariable {
                    name: (ident.clone()),
                    value: (value),
                    typ: (None),
                };
            }
        }

        self.consume(TokenType::SemiColon, "Expected ';' at end of statement");
        Stmt::ExprStmt(expr)
    }
}
