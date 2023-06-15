use inkwell::context::Context;
use thiserror::Error;

use crate::codegen::{self, ModuleCodegen};
use crate::parser::ast::StmtKind;
use crate::symbol::{Symbol, SymbolTable, SymbolTableStack, SymbolType};

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Unknown compiler error")]
    Unknown,
}

pub struct Compiler {
    symbol_table: SymbolTableStack,
}

impl Compiler {
    /// Take in a AST in the form of a vector of statements and compile the
    /// program.
    pub fn compile(code: Vec<StmtKind>) -> Result<(), CompilerError> {
        let mut compiler = Self {
            symbol_table: SymbolTableStack::new(),
        };

        // Resolve names
        compiler.resolve_globals(&code);

        // Compile each function
        let context = Context::create();
        let codegen = ModuleCodegen::new("root", &context, &mut compiler.symbol_table);

        for stmt in code.iter() {
            if let StmtKind::DefineFunction { body, .. } = stmt {
                // compiler.compile_function(body);
            }
        }

        Ok(())
    }

    fn resolve_globals(&mut self, code: &[StmtKind]) {
        for stmt in code.iter() {
            // if let Stmt::DefineFunction { ident, .. } = stmt {
            //     let symbol = Symbol {
            //         typ: Some(SymbolType::Function),
            //     };
            //
            //     self.symbol_table.insert(ident, symbol);
            // }
        }
    }
}

// Step 1: Name resolution
// Step 2: Type checking
// Step 3: Code generation
