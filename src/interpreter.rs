use std::collections::HashMap;
use std::fmt::Display;

use crate::ast::{AstVisitor, Expr, Stmt};
use crate::lexer::{Literal, TokenType};

#[derive(Default)]
pub struct AstInterpreter {
    memory: HashMap<String, (Value, bool)>,
}

impl AstVisitor<Value> for AstInterpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Value {
        match stmt {
            Stmt::Block(stmts) => {
                self.interpret(stmts);
            }
            Stmt::Expr(expr) => {
                self.visit_expr(expr);
            }
            Stmt::Val { ident, value } => {
                let value = self.visit_expr(value);
                self.memory.insert(ident.clone(), (value, false));
            }
            Stmt::Var { ident, value } => {
                let value = self.visit_expr(value);
                self.memory.insert(ident.clone(), (value, true));
            }
            Stmt::Assignment { ident, value } => {
                if !self.memory.contains_key(ident) {
                    panic!("Cannot assign to variable that doesn't exist");
                }

                if !self.memory[ident].1 {
                    panic!("Cannot mutate value '{ident}'");
                }

                let value = self.visit_expr(value);
                self.memory.insert(ident.clone(), (value, true));
            }
            Stmt::Function {
                ident: name,
                arguments,
                return_type,
                body,
            } => todo!(),
            Stmt::If { condition, body } => {
                let result = self.visit_expr(condition);
                if result == Value::Bool(true) {
                    self.interpret(body);
                }
            }
            Stmt::For {
                binding,
                range,
                body,
            } => todo!(),
            Stmt::Return { value } => todo!(),
            Stmt::Print { value } => {
                println!("{}", self.visit_expr(value));
            }
        };

        // FIXME: Honestly should probably abandon this "visitor" pattern. 2 functions
        // with these match statements would work better
        Value::Nil
    }

    fn visit_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Literal(literal) => match literal {
                Literal::String(value) => Value::String(value.clone()),
                Literal::Character(value) => Value::String(value.to_string()),
                Literal::Number(value) => Value::Number(*value),
                Literal::Bool(value) => Value::Bool(*value),
                Literal::Nil => Value::Nil,
            },
            Expr::Variable(ident) => self.memory.get(ident).unwrap().clone().0,
            Expr::Grouping(child) => self.visit_expr(child),
            Expr::Binary { operator, lhs, rhs } => {
                let lhs = self.visit_expr(lhs);
                let rhs = self.visit_expr(rhs);

                if let Value::Number(lhs) = lhs && let Value::Number(rhs) = rhs {
                    match operator {
                        TokenType::Plus => Value::Number(lhs + rhs),
                        TokenType::Minus => Value::Number(lhs - rhs),
                        TokenType::Star => Value::Number(lhs * rhs),
                        TokenType::Slash => Value::Number(lhs / rhs),
                        TokenType::Perc => Value::Number(lhs % rhs),

                        TokenType::Gt => Value::Bool(lhs > rhs),
                        TokenType::GtEq => Value::Bool(lhs >= rhs),
                        TokenType::Lt => Value::Bool(lhs < rhs),
                        TokenType::LtEq => Value::Bool(lhs <= rhs),

                        TokenType::BangEq => Value::Bool(lhs != rhs),
                        TokenType::EqEq => Value::Bool(lhs == rhs),

                        _ => panic!(),
                    }
                } else if let Value::Bool(lhs) = lhs && let Value::Bool(rhs) = rhs {
                    match operator {
                        TokenType::AmpAmp => Value::Bool(lhs && rhs),
                        TokenType::PipePipe => Value::Bool(lhs || rhs),
                        _ => panic!()
                    }
                } else if let Value::String(lhs) = lhs && let Value::String(rhs) = rhs {
                    match operator {
                        TokenType::Plus => {
                            let mut value = lhs;
                            value.push_str(&rhs);
                            Value::String(value)
                        },
                        TokenType::BangEq => Value::Bool(lhs != rhs),
                        TokenType::EqEq => Value::Bool(lhs == rhs),
                        _ => panic!()
                    }
                } else {
                    panic!("Invalid operations for types");
                }
            }
            Expr::Unary { operator, expr } => {
                let value = self.visit_expr(expr);

                match operator {
                    TokenType::Bang => {
                        let Value::Bool(value) = value else {
                            panic!("Invalid operations for types");
                        };

                        Value::Bool(!value)
                    }
                    TokenType::Plus => value,
                    TokenType::Minus => {
                        let Value::Number(value) = value else {
                            panic!("Invalid operations for types");
                        };

                        Value::Number(-value)
                    }
                    _ => panic!(),
                }
            }
            _ => unimplemented!("{:?}", expr),
        }
    }
}

impl AstInterpreter {
    pub fn interpret(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Number(i32),
    String(String),
    Bool(bool),
    Nil,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(value) => write!(f, "{value}")?,
            Value::String(value) => write!(f, "{value}")?,
            Value::Bool(value) => write!(f, "{value}")?,
            Value::Nil => write!(f, "nil")?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    //
}
