use std::io::Write;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue};
use itertools::Itertools;

use crate::parser::ast::{
    BinaryOp, Expr, ExprKind, Function, FunctionKind, Literal, Stmt, StmtKind,
};
use crate::symtable::{SymbolTable, Type};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    current_func: Option<FunctionValue<'ctx>>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn codegen(context: &'ctx Context, module: &str, code: &Stmt) {
        let builder = context.create_builder();
        let module = context.create_module(module);

        let mut this = Compiler {
            context,
            builder,
            module,

            current_func: None,
        };

        let StmtKind::Block(ref stmts) = &code.kind else {
            panic!("Code root should be a block");
        };

        for stmt in stmts {
            this.codegen_stmt(stmt);
            this.current_func.unwrap().print_to_stderr();
        }
    }

    fn codegen_stmt(&mut self, code: &Stmt) {
        match &code.kind {
            StmtKind::Block(stmts) => self.codegen_block(stmts),
            StmtKind::ExprStmt(expr) => {
                self.codegen_expr(expr);
            }
            StmtKind::Return(expr) => {
                let res = self.codegen_expr(expr);
                self.builder.build_return(Some(&res));
            }
            StmtKind::DefineFunction(function) => {
                let table = code.symtable.clone();
                self.codegen_function(table, function.clone());

                // If the function is written in sloth (as opposed to an extern one) we generate
                // the block contents
                if let FunctionKind::Normal { body } = &function.kind {
                    if let StmtKind::Block(body) = &body.kind {
                        self.codegen_block(body);
                    }
                };
            }
            _ => (),
        }
    }

    fn codegen_function(&mut self, table: SymbolTable, function: Function) -> FunctionValue {
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

        llvm_function
    }

    fn codegen_block(&mut self, code: &[Stmt]) {
        let Some(current_func) = self.current_func else {
            panic!("Block codegen requires function");
        };

        let block = self.context.append_basic_block(current_func, "block");

        self.builder.position_at_end(block);

        for stmt in code {
            self.codegen_stmt(stmt);
        }
    }

    fn codegen_expr(&self, code: &Expr) -> BasicValueEnum<'ctx> {
        // AnyValue
        match &code.kind {
            ExprKind::Literal(literal) => self.codegen_value(literal.clone()),
            ExprKind::Grouping(inner) => self.codegen_expr(inner),
            ExprKind::Identifier(ident) => {
                // FIXME: Do thsi
                todo!()
            }
            ExprKind::BinaryOp { op, lhs, rhs } => match lhs.typ {
                Some(Type::Integer) => {
                    let lhs_gen = self.codegen_expr(lhs).into_int_value();
                    let rhs_gen = self.codegen_expr(rhs).into_int_value();

                    match op {
                        BinaryOp::Add => self.builder.build_int_add(lhs_gen, rhs_gen, "add"),
                        BinaryOp::Sub => self.builder.build_int_sub(lhs_gen, rhs_gen, "sub"),
                        BinaryOp::Mul => self.builder.build_int_mul(lhs_gen, rhs_gen, "mul"),
                        BinaryOp::Div => self.builder.build_int_signed_div(lhs_gen, rhs_gen, "div"),
                        _ => panic!(),
                    }
                    .into()
                }
                Some(Type::Float) => {
                    let lhs_gen = self.codegen_expr(lhs).into_float_value();
                    let rhs_gen = self.codegen_expr(rhs).into_float_value();

                    match op {
                        BinaryOp::Add => self.builder.build_float_add(lhs_gen, rhs_gen, "add"),
                        BinaryOp::Sub => self.builder.build_float_sub(lhs_gen, rhs_gen, "sub"),
                        BinaryOp::Mul => self.builder.build_float_mul(lhs_gen, rhs_gen, "mul"),
                        BinaryOp::Div => self.builder.build_float_div(lhs_gen, rhs_gen, "div"),
                        _ => panic!(),
                    }
                    .into()
                }
                None => unreachable!("Critical Error: Type should never be null by this point"),
                _ => todo!(),
            },
            ExprKind::UnaryOp { op, value } => todo!(),
            ExprKind::Call { callee, args } => {
                // FIXME: Callee is an expression but for now were just
                // extracting an identifier to it. Change this
                // so you can do for example `fn(){}()`.
                let ExprKind::Identifier(ident) = &callee.kind else { panic!() };
                let function = self.module.get_function(ident).expect("oh nooos");

                let args = args
                    .iter()
                    .map(|arg| self.codegen_expr(arg))
                    .map(|arg| arg.into())
                    .collect::<Vec<BasicMetadataValueEnum>>();

                self.builder
                    .build_call(function, &args, "")
                    .try_as_basic_value()
                    .unwrap_left()
            }
        }
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
            _ => unimplemented!(),
        }
    }

    fn type_as_metadata_type(&self, typ: Type) -> BasicMetadataTypeEnum<'ctx> {
        match typ {
            Type::Integer => self.context.i64_type().into(),
            Type::Float => self.context.f64_type().into(),
            Type::Boolean => self.context.bool_type().into(),
            _ => todo!(), // Type::Function { inputs, output } => todo!(),
        }
    }

    fn write_obj(&self, file: &mut impl Write) {}
}
