#![allow(unused)]

pub mod symbol;

use std::collections::HashMap;

use sloth_bytecode::Opcode;

use self::symbol::{Function, Symbol, SymbolTable, SymbolTableStack, SymbolType};
use crate::parser::ast::{BinaryOp, Expr, Literal, Stmt, UnaryOp};

// Modules:
//   Symbols (Functions, Constants)
//
// Functions:
//   Symbols (Functions, Variables, Constants)

pub enum CompilerMode {
    Module,
    Function,
}

pub struct Compiler {
    symbols: SymbolTableStack,
    mode: CompilerMode,
}

pub struct CompileOrder {
    code: Vec<Stmt>,
}

impl Compiler {
    fn new() -> Self {
        Self {
            symbols: SymbolTableStack::default(),
            mode: CompilerMode::Module,
        }
    }

    fn compile(&mut self, code: Vec<Stmt>) {
        let mut queue = Vec::<CompileOrder>::new();

        for stmt in code {
            match stmt {
                Stmt::DefineFunction {
                    ident,
                    args,
                    body,
                    return_type,
                } => {
                    self.symbols.push_symbol(ident, Symbol {
                        typ: SymbolType::Function(Function {
                            arity: args.len() as u8,
                            returns_value: return_type.is_some(),
                        }),
                    });

                    todo!()
                }

                _ => panic!("Failed to compile module due to unexpected statement"),
            }
        }
    }

    fn compile_function(&mut self, code: Vec<Stmt>) -> Function {
        unimplemented!()
    }
}

pub fn generate_symbols() -> SymbolTable {
    let mut table = SymbolTable::default();
    //
    todo!()
}
