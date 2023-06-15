use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{AnyTypeEnum, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::FunctionValue;
use itertools::Itertools;

use crate::parser::ast::{FunctionInput, Stmt};
use crate::symbol::SymbolTableStack;

/// A module codegen is a struct designated to compiling a single module
pub struct ModuleCodegen<'ctx, 'a> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    symbol_table: &'a mut SymbolTableStack,
}

impl<'ctx, 'a> ModuleCodegen<'ctx, 'a> {
    pub fn new(
        module: &str,
        context: &'ctx Context,
        symbol_table: &'a mut SymbolTableStack,
    ) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module);

        Self {
            context,
            builder,
            module,

            symbol_table,
        }
    }

    pub fn compile_function(
        &mut self,
        ident: String,
        args: Vec<FunctionInput>,
        return_type: Option<String>,
        body: &[Stmt],
    ) {
        let llvm_function_type = self.compile_function_type(&args, return_type.as_deref());
        let llvm_function = self.module.add_function(&ident, llvm_function_type, None);

        let entry_block = self.context.append_basic_block(llvm_function, "entry");

        self.block(body);
    }

    fn compile_function_type(
        &self,
        args: &[FunctionInput],
        return_type: Option<&str>,
    ) -> FunctionType<'ctx> {
        let args = args
            .iter()
            .map(|it| self.compile_basic_metadata_type(&it.typ).unwrap())
            .collect_vec();

        match return_type {
            None => self.context.void_type().fn_type(&args, false),
            Some("int") => self.context.i64_type().fn_type(&args, false),
            Some("float") => self.context.f64_type().fn_type(&args, false),
            _ => panic!(),
        }
    }

    fn compile_basic_metadata_type(&self, typ: &str) -> Option<BasicMetadataTypeEnum<'ctx>> {
        match typ {
            "int" => Some(self.context.i64_type().into()),
            "float" => Some(self.context.f64_type().into()),
            _ => None,
        }
    }

    fn block(&mut self, body: &[Stmt]) {
        self.symbol_table.push();
        self.symbol_table.pop();
    }

    // fn compile(symbol_table: &'a mut SymbolTableStack, code: Vec<Stmt>) {
    //     //
    // }
}
