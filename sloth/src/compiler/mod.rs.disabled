#![allow(unused)]

use std::collections::HashMap;
use std::path::Path;
use std::vec;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::IntValue;
use inkwell::OptimizationLevel;

use crate::parser::ast::{BinaryOp, Expr, FuncArgs, Literal, Stmt, UnaryOp};

pub struct Compiler<'ctx> {
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("sloth");

        Self {
            context,
            builder,
            module,
        }
    }

    pub fn compile(&self, src: Vec<Stmt>) {
        for stmt in src {
            match stmt {
                Stmt::DefineFunction {
                    ident,
                    args,
                    body,
                    return_type,
                } => {
                    self.compile_function(&ident, &args, return_type.is_some(), body);
                }
                _ => panic!("You may only define a function top level"),
            }
        }

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
        machine
            .write_to_file(&self.module, FileType::Object, Path::new("output.o"))
            .unwrap();
    }

    fn compile_function(&self, identifier: &str, args: &[FuncArgs], returns: bool, src: Vec<Stmt>) {
        let void_type = self.context.void_type();
        let i64_type = self.context.i64_type();

        let function_type = if returns {
            i64_type.fn_type(&vec![i64_type.into(); args.len()], false)
        } else {
            void_type.fn_type(&vec![i64_type.into(); args.len()], false)
        };
        let function = self.module.add_function(identifier, function_type, None);

        let basic_block = self.context.append_basic_block(function, "body");

        self.builder.position_at_end(basic_block);

        let mut arg_values = HashMap::<String, IntValue>::new();
        for (i, arg) in args.iter().enumerate() {
            arg_values.insert(
                arg.name.clone(),
                function.get_nth_param(i as u32).unwrap().into_int_value(),
            );
        }

        for stmt in src {
            match stmt {
                Stmt::Return { value } => match value {
                    Expr::BinaryOp { op, lhs, rhs } => {
                        let lhs = match *lhs {
                            Expr::Variable(a) => arg_values[&a],
                            _ => unimplemented!(),
                        };

                        let rhs = match *rhs {
                            Expr::Variable(a) => arg_values[&a],
                            _ => unimplemented!(),
                        };

                        let res = match op {
                            BinaryOp::Add => self.builder.build_int_add(lhs, rhs, "addop"),
                            BinaryOp::Sub => self.builder.build_int_sub(lhs, rhs, "subop"),
                            _ => unimplemented!(),
                        };

                        self.builder.build_return(Some(&res));
                        return;
                    }
                    Expr::Variable(name) => {
                        let var = arg_values[&name];
                        self.builder.build_return(Some(&var));
                        return;
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }

        self.builder.build_return(None);
    }
}
