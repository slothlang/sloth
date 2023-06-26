use std::collections::HashMap;
use std::io::Write;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue,
};
use inkwell::{FloatPredicate, IntPredicate, OptimizationLevel};
use itertools::{Either, Itertools};

use crate::parser::ast::{
    BinaryOp, Expr, ExprKind, Function, FunctionKind, Literal, Stmt, StmtKind,
};
use crate::symtable::{SymbolTable, Type};

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    current_func: Option<FunctionValue<'ctx>>,
    current_func_void: bool,

    references: HashMap<i32, PointerValue<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    pub fn new(context: &'ctx Context, module: &str) -> Self {
        let builder = context.create_builder();
        let module = context.create_module(module);

        Codegen {
            context,
            builder,
            module,

            current_func: None,
            current_func_void: false,

            references: Default::default(),
        }
    }

    pub fn codegen(&mut self, code: &Stmt) {
        let StmtKind::Block(ref stmts) = &code.kind else {
            panic!("Code root should be a block");
        };

        for stmt in stmts {
            self.codegen_stmt(stmt);
            self.current_func.unwrap().print_to_stderr();
        }
    }

    fn codegen_alloca(&mut self, typ: BasicTypeEnum<'ctx>, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.current_func.unwrap().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first) => builder.position_before(&first),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(typ, &format!("alloca {name}"))
    }

    fn codegen_stmt(&mut self, code: &Stmt) {
        match &code.kind {
            StmtKind::Block(stmts) => self.codegen_block(stmts),
            StmtKind::ExprStmt(expr) => {
                self.codegen_expr(expr);
            }
            StmtKind::IfStmt {
                condition,
                if_then,
                else_then,
            } => {
                // Get the current function
                let func = self.current_func.unwrap();

                // Get the condition
                let condition = self.codegen_expr(condition).unwrap().into_int_value();

                // Add the branching logic
                let then_bb = self.context.append_basic_block(func, "then");
                let else_bb = self.context.append_basic_block(func, "else");
                let continue_bb = self.context.append_basic_block(func, "continue");

                self.builder
                    .build_conditional_branch(condition, then_bb, else_bb);

                // Building the blocks for then
                self.builder.position_at_end(then_bb);
                self.codegen_stmt(if_then);
                self.builder.build_unconditional_branch(continue_bb);

                // Building the blocks for else
                self.builder.position_at_end(else_bb);
                if let Some(else_then) = else_then {
                    self.codegen_stmt(else_then);
                }
                self.builder.build_unconditional_branch(continue_bb);

                // Position the builder at the end of the continue block
                self.builder.position_at_end(continue_bb);
            }
            StmtKind::WhileStmt { condition, body } => {
                // Get the current function
                let func = self.current_func.unwrap();

                let loop_bb = self.context.append_basic_block(func, "loop");
                let body_bb = self.context.append_basic_block(func, "loop body");
                let after_bb = self.context.append_basic_block(func, "after loop");

                self.builder.build_unconditional_branch(loop_bb);

                // Building the blocks for the head of the loop
                self.builder.position_at_end(loop_bb);
                let condition = self.codegen_expr(condition).unwrap().into_int_value();
                self.builder
                    .build_conditional_branch(condition, body_bb, after_bb);

                // Building the blocks for the body of the loop
                self.builder.position_at_end(body_bb);
                self.codegen_stmt(body);
                self.builder.build_unconditional_branch(loop_bb);

                // Position the builder at the end of the loop
                self.builder.position_at_end(after_bb);
            }
            StmtKind::DefineVariable {
                identifier, value, ..
            } => {
                let table = code.symtable.clone();
                let symbol = table.get_value(identifier).unwrap();

                let ptr = self.codegen_alloca(self.type_as_basic_type(symbol.typ), identifier);
                let init_value = self.codegen_expr(value).unwrap();

                self.builder.build_store(ptr, init_value);
                self.references.insert(symbol.id, ptr);
            }
            StmtKind::AssignVariable { identifier, value } => {
                let table = code.symtable.clone();
                let symbol = table.get_value(identifier).unwrap();

                let ptr = self.references.get(&symbol.id).unwrap();
                let init_value = self.codegen_expr(value).unwrap();

                self.builder.build_store(*ptr, init_value);
            }
            StmtKind::DefineFunction(function) => {
                let table = code.symtable.clone();
                self.codegen_function(table, function.clone());

                // If the function is written in sloth (as opposed to an extern one) we generate
                // the block contents
                if let FunctionKind::Normal { body } = &function.kind {
                    if let StmtKind::Block(body) = &body.kind {
                        // Make the block containing the code for the function
                        let func = self.current_func.unwrap();
                        let block = self.context.append_basic_block(func, "entrypoint");

                        // Position the builder to be at the block
                        self.builder.position_at_end(block);
                        self.codegen_block(body);

                        if self.current_func_void {
                            self.builder.build_return(None);
                        }
                    }
                };
            }
            StmtKind::Return(expr) => {
                let res = self.codegen_expr(expr).unwrap();
                self.builder.build_return(Some(&res));
            }
        }
    }

    fn codegen_function(&mut self, table: SymbolTable, function: Function) -> FunctionValue {
        // Clear references to variables
        self.references.clear();

        let inputs = function.inputs;
        let inputs_typ = inputs
            .iter()
            .map(|it| table.get_type(&it.typ).unwrap())
            .map(|it| self.type_as_metadata_type(it))
            .collect_vec();

        let output = function.output;
        let output_typ = output
            .map(|it| table.get_type(&it))
            .unwrap_or(Some(Type::Void))
            .unwrap();

        let llvm_function_type = match output_typ {
            Type::Void => self.context.void_type().fn_type(&inputs_typ, false),
            Type::Integer => self.context.i64_type().fn_type(&inputs_typ, false),
            Type::Float => self.context.f64_type().fn_type(&inputs_typ, false),
            Type::Boolean => self.context.bool_type().fn_type(&inputs_typ, false),
            _ => todo!(),
        };

        let llvm_function =
            self.module
                .add_function(&function.identifier, llvm_function_type, None);

        self.current_func = Some(llvm_function);
        self.current_func_void = matches!(output_typ, Type::Void);

        llvm_function
    }

    fn codegen_block(&mut self, code: &[Stmt]) {
        for stmt in code {
            self.codegen_stmt(stmt);
        }
    }

    fn codegen_expr(&self, code: &Expr) -> Option<BasicValueEnum<'ctx>> {
        Some(match &code.kind {
            ExprKind::Literal(literal) => self.codegen_value(literal.clone()),
            ExprKind::Grouping(inner) => self.codegen_expr(inner)?,
            ExprKind::Identifier(ident) => {
                let table = code.symtable.clone();
                let symbol = table.get_value(ident).unwrap();
                let ptr = self.references.get(&symbol.id).unwrap();

                self.builder
                    .build_load(self.type_as_basic_type(symbol.typ.clone()), *ptr, "")
            }
            ExprKind::BinaryOp { op, lhs, rhs } => match lhs.typ {
                Some(Type::Integer) | Some(Type::Boolean) => {
                    use IntPredicate::*;

                    let l = self.codegen_expr(lhs).unwrap().into_int_value();
                    let r = self.codegen_expr(rhs).unwrap().into_int_value();

                    match op {
                        BinaryOp::Add => self.builder.build_int_add(l, r, "add"),
                        BinaryOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                        BinaryOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                        BinaryOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                        BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),

                        BinaryOp::Gt => self.builder.build_int_compare(SGT, l, r, "gt"),
                        BinaryOp::GtEq => self.builder.build_int_compare(SGE, l, r, ""),
                        BinaryOp::Lt => self.builder.build_int_compare(SLT, l, r, "lt"),
                        BinaryOp::LtEq => self.builder.build_int_compare(SLE, l, r, ""),

                        BinaryOp::EqEq => self.builder.build_int_compare(EQ, l, r, ""),
                        BinaryOp::NotEq => self.builder.build_int_compare(NE, l, r, ""),
                        _ => panic!(),
                    }
                    .into()
                }
                Some(Type::Float) => {
                    use FloatPredicate::*;

                    let l = self.codegen_expr(lhs).unwrap().into_float_value();
                    let r = self.codegen_expr(rhs).unwrap().into_float_value();

                    match op {
                        BinaryOp::Add => self.builder.build_float_add(l, r, "add").into(),
                        BinaryOp::Sub => self.builder.build_float_sub(l, r, "sub").into(),
                        BinaryOp::Mul => self.builder.build_float_mul(l, r, "mul").into(),
                        BinaryOp::Div => self.builder.build_float_div(l, r, "div").into(),
                        BinaryOp::Mod => self.builder.build_float_rem(l, r, "mod").into(),

                        BinaryOp::Gt => self.builder.build_float_compare(OGT, l, r, "gt").into(),
                        BinaryOp::GtEq => self.builder.build_float_compare(OGE, l, r, "gt").into(),
                        BinaryOp::Lt => self.builder.build_float_compare(OLT, l, r, "lt").into(),
                        BinaryOp::LtEq => self.builder.build_float_compare(OLE, l, r, "le").into(),

                        BinaryOp::EqEq => self.builder.build_float_compare(OEQ, l, r, "eq").into(),
                        BinaryOp::NotEq => self.builder.build_float_compare(ONE, l, r, "ne").into(),
                        _ => panic!(),
                    }
                }
                None => unreachable!("Critical Error: Type should never be null by this point"),
                _ => todo!(),
            },
            ExprKind::UnaryOp { .. } => todo!(),
            ExprKind::Call { callee, args } => {
                // FIXME: Callee is an expression but for now were just
                // extracting an identifier to it. Change this
                // so you can do for example `fn(){}()`.
                let ExprKind::Identifier(ident) = &callee.kind else { panic!() };
                let function = self.module.get_function(ident).expect("oh nooos");

                let args = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg))
                    .map(|arg| arg.unwrap().into())
                    .collect::<Vec<BasicMetadataValueEnum>>();

                let call = self
                    .builder
                    .build_call(function, &args, "")
                    .try_as_basic_value();

                match call {
                    Either::Left(left) => left,
                    Either::Right(_) => return None,
                }
            }
        })
    }

    fn codegen_value(&self, value: Literal) -> BasicValueEnum<'ctx> {
        match value {
            Literal::Integer(value) => self
                .context
                .i64_type()
                .const_int(value as u64, true)
                .as_basic_value_enum(),
            Literal::Float(value) => self
                .context
                .f64_type()
                .const_float(value)
                .as_basic_value_enum(),
            Literal::Boolean(value) => self
                .context
                .bool_type()
                .const_int(if value { 1 } else { 0 }, false)
                .as_basic_value_enum(),
            _ => unimplemented!(),
        }
    }

    fn type_as_basic_type(&self, typ: Type) -> BasicTypeEnum<'ctx> {
        match typ {
            Type::Integer => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Boolean => self.context.bool_type().into(),
            _ => todo!(),
        }
    }

    fn type_as_metadata_type(&self, typ: Type) -> BasicMetadataTypeEnum<'ctx> {
        self.type_as_basic_type(typ).into()
    }

    pub fn write_obj(&self, file: &mut impl Write, filetype: FileType) {
        Target::initialize_native(&InitializationConfig::default()).unwrap();

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();
        let machine = target
            .create_target_machine(
                &triple,
                "x86-64",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap();

        self.module.set_triple(&triple);

        let buffer = machine
            .write_to_memory_buffer(&self.module, filetype)
            .unwrap();

        file.write_all(buffer.as_slice()).unwrap();
    }
}
