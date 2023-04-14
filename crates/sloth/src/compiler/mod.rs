#![allow(unused)]

use std::collections::HashMap;

use sloth_bytecode::Opcode;
use sloth_vm::value::{Function, Object, ObjectType, Primitive};
use sloth_vm::{Chunk, ObjectMap};

use crate::parser::ast::{BinaryOp, Expr, Literal, Stmt, UnaryOp};

#[derive(Default, Clone)]
pub struct CompilerScope {
    objects: HashMap<String, u32>,
}

#[derive(Eq, PartialEq)]
pub enum CompilerType {
    Program,
    Function,
}

pub struct Compiler<'a> {
    variables: HashMap<String, u32>,

    objects: &'a mut ObjectMap,
    chunk: Chunk,
}

impl<'a> Compiler<'a> {
    pub fn compile(
        objects: &'a mut ObjectMap,
        variables: HashMap<String, u32>,
        stmts: Vec<Stmt>,
    ) -> Chunk {
        let mut me = Self {
            variables,
            objects,
            chunk: Chunk::default(),
        };

        for stmt in &stmts {
            me.parse_statement(stmt);
        }

        me.chunk
    }

    fn push_constant(&mut self, value: Primitive) -> u16 {
        let next = self.chunk.constants.len();
        self.chunk.constants.push(value);
        next as u16
    }

    #[inline(always)]
    fn write_opcode(&mut self, opcode: Opcode) {
        self.write_u8(opcode as u8);
    }

    #[inline(always)]
    fn write_u8(&mut self, value: u8) {
        self.chunk.code.push(value);
    }

    #[inline(always)]
    fn write_u16(&mut self, value: u16) {
        let bytes = ((value >> 8) as u8, value as u8);

        self.write_u8(bytes.0);
        self.write_u8(bytes.1);
    }
}

impl<'a> Compiler<'a> {
    fn parse_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::ExprStmt(expr) => {
                self.parse_expression(expr);
            }
            Stmt::DefineFunction {
                ident,
                args,
                body,
                return_type,
            } => {
                if args.len() > 255 {
                    panic!("Function can not have more than 255 arguments");
                }

                let internal =
                    Compiler::compile(self.objects, self.variables.clone(), body.clone());
                let function = Function {
                    name: Some(ident.to_owned()),
                    chunk: internal,
                    arity: args.len() as u8,
                    returns_value: return_type.is_some(),
                };

                // Allocate the object as a function
                let object = Object::new(ObjectType::Function(function));
                let ptr = self.objects.allocate(object);

                // Store it in variables
                // Whenever there is a need to reference it just make a new constant lol
                self.variables.insert(ident.to_owned(), ptr as u32);
            }
            Stmt::DefineVariable { name, value, typ } => {
                // Take the expression and put whatever it evaluates to onto the stack
                self.parse_expression(value);
                // Store that as a variable
            }
            Stmt::DefineValue { name, value, typ } => todo!(),
            Stmt::AssignVariable { name, value } => {
                self.parse_expression(value);

                //
            }
            Stmt::If {
                expr,
                body,
                else_if,
                els,
            } => todo!(),
            Stmt::For { name, iter, body } => todo!(),
            Stmt::While { condition, body } => todo!(),
            Stmt::Return { value } => {
                self.parse_expression(value);
                self.write_opcode(Opcode::Return);
            }
        }
    }
}

impl<'a> Compiler<'a> {
    fn parse_expression(&mut self, expr: &Expr) {
        match expr {
            Expr::Grouping(e) => self.parse_expression(e),
            Expr::BinaryOp { op, lhs, rhs } => {
                self.parse_expression(lhs);
                self.parse_expression(rhs);

                let opcode = match op {
                    BinaryOp::Add => Opcode::Add,
                    BinaryOp::Con => todo!(),
                    BinaryOp::Sub => Opcode::Sub,
                    BinaryOp::Mul => Opcode::Mul,
                    BinaryOp::Pow => todo!(),
                    BinaryOp::Div => Opcode::Div,
                    BinaryOp::Mod => todo!(),
                    BinaryOp::BWSftRight => todo!(),
                    BinaryOp::BWSftLeft => todo!(),
                    BinaryOp::BWAnd => todo!(),
                    BinaryOp::BWOr => todo!(),
                    BinaryOp::BWXor => todo!(),
                    BinaryOp::Lt => todo!(),
                    BinaryOp::Gt => todo!(),
                    BinaryOp::LtEq => todo!(),
                    BinaryOp::GtEq => todo!(),
                    BinaryOp::EqEq => Opcode::Eq,
                    BinaryOp::NotEq => Opcode::Ne,
                    BinaryOp::LogAnd => todo!(),
                    BinaryOp::LogOr => todo!(),
                    BinaryOp::Range => todo!(),
                };

                self.write_opcode(opcode);
            }
            Expr::UnaryOp { op, value } => {
                self.parse_expression(value);

                let opcode = match op {
                    UnaryOp::Not => todo!(),
                    UnaryOp::Neg => todo!(),
                    UnaryOp::BWComp => todo!(),
                };

                self.write_opcode(opcode);
            }
            Expr::Call { ident, args } => {
                for arg in args.iter().rev() {
                    self.parse_expression(arg);
                }

                self.parse_expression(ident);
                self.write_opcode(Opcode::Call);
            }
            Expr::Variable(var) => {
                // TODO: THIS IS AWFUL AND I HATE IT
                let a = self.variables.get(var).cloned().expect("Uh oh spaghettio");

                let pos = self.chunk.constants.len();
                self.chunk.constants.push(Primitive::Object(a));
                self.write_opcode(Opcode::Constant);
                self.write_u16(pos as u16);
            }
            Expr::Literal(lit) => {
                let pos = self.chunk.constants.len();

                match lit {
                    Literal::Integer(i) => self.chunk.constants.push(Primitive::Integer(*i)),
                    Literal::Float(f) => self.chunk.constants.push(Primitive::Float(*f)),
                    Literal::Bool(b) => self.chunk.constants.push(Primitive::Bool(*b)),
                    Literal::String(s) => todo!(),
                    _ => todo!(),
                }

                self.write_opcode(Opcode::Constant);
                self.write_u16(pos as u16);
            }
            Expr::Lambda => todo!(),
        }
    }
}
