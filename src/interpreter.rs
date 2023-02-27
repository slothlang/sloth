use std::collections::HashMap;
use std::fmt::{Debug, Display};

use itertools::Itertools;

use crate::ast::{AstVisitor, Expr, Stmt};
use crate::lexer::{Literal, TokenType};

#[derive(Default)]
pub struct AstInterpreter {
    pub callables: HashMap<String, Box<dyn SlothCallable>>,
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
            } => {
                let Value::Number(lower_range) = self.visit_expr(&range.0) else { panic!("Lower range must be number") };
                let Value::Number(upper_range) = self.visit_expr(&range.1) else { panic!("Upper range must be number") };

                for i in lower_range..upper_range {
                    self.memory
                        .insert(binding.clone(), (Value::Number(i), false));
                    self.interpret(body);
                }

                self.memory.remove(binding);
            }
            Stmt::Return { value } => todo!(),
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
            Expr::Call { ident, arguments } => {
                let argument_values = arguments.iter().map(|it| self.visit_expr(it)).collect_vec();
                let Some(callable) = self.callables.remove(ident) else {
                    panic!("Unkown callable '{ident}'");
                };

                let result = callable.call(self, &argument_values);
                self.callables.insert(ident.clone(), callable);
                result
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

#[derive(Clone, Eq, PartialEq)]
pub enum Value {
    Number(i32),
    String(String),
    Bool(bool),
    Nil,
}

pub trait SlothCallable {
    fn call(&self, interpreter: &mut AstInterpreter, args: &[Value]) -> Value;
}

pub struct InternalFunction<'a>(pub &'a dyn Fn(&[Value]) -> Value);

impl<'a> SlothCallable for InternalFunction<'a> {
    fn call(&self, interpreter: &mut AstInterpreter, args: &[Value]) -> Value {
        self.0(args)
    }
}

// pub struct SlothFunction(Vec<Stmt>);

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
