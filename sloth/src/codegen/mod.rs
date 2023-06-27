use std::collections::HashMap;
use std::io::Write;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};
use itertools::{Either, Itertools};

use crate::parser::ast::{
    BinaryOp, Expr, ExprKind, Function, FunctionKind, Literal, Stmt, StmtKind, UnaryOp,
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

        let mut this = Codegen {
            context,
            builder,
            module,

            current_func: None,
            current_func_void: false,

            references: Default::default(),
        };

        // Compiler intrinsic functions
        this.INTRINSIC_vlen();

        for (c, t) in [
            ("i", Type::Integer),
            ("f", Type::Float),
            ("b", Type::Boolean),
        ] {
            this.INTRINSIC_vpush(c, t.clone());
            this.INTRINSIC_vpop(c, t.clone());
            this.INTRINSIC_vget(c, t.clone());
            this.INTRINSIC_vset(c, t);
        }

        this
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

        builder.build_alloca(typ, name)
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
                    if let StmtKind::Block(code) = &body.kind {
                        // Make the block containing the code for the function
                        let func = self.current_func.unwrap();
                        let block = self.context.append_basic_block(func, "entrypoint");

                        // Position the builder to be at the block
                        self.builder.position_at_end(block);

                        // FIXME 🍝
                        // This code adds parameters to the body so they're accessible
                        let body_table = body.symtable.clone();
                        for (i, input) in function.inputs.iter().enumerate() {
                            let symbol = body_table.get_value(&input.identifier).unwrap();

                            let ptr = self.codegen_alloca(
                                self.type_as_basic_type(symbol.typ),
                                &input.identifier,
                            );
                            let init_value = func.get_nth_param(i as u32).unwrap();

                            self.builder.build_store(ptr, init_value);
                            self.references.insert(symbol.id, ptr);
                        }

                        // Codegen the function body
                        self.codegen_block(code);

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
            Type::Integer => self.context.i32_type().fn_type(&inputs_typ, false),
            Type::Float => self.context.f32_type().fn_type(&inputs_typ, false),
            Type::Boolean => self.context.bool_type().fn_type(&inputs_typ, false),
            Type::Array { ref typ, .. } => {
                let i32_type = self.context.i32_type().as_basic_type_enum();

                let typ = self
                    .type_as_basic_type(*typ.clone())
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum();

                let vector_type = self.context.struct_type(&[i32_type, i32_type, typ], false);
                let ptr_to_that = vector_type.ptr_type(AddressSpace::default());

                ptr_to_that.fn_type(&inputs_typ, false)
            }
            Type::String => {
                let i8_type = self.context.i8_type().as_basic_type_enum();
                let ptr_type = i8_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum();

                ptr_type.fn_type(&inputs_typ, false)
            }
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

                        BinaryOp::LogicalAnd => self.builder.build_and(l, r, "logand"),
                        BinaryOp::LogicalOr => self.builder.build_or(l, r, "logor"),

                        _ => panic!("{op:?}"),
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
            ExprKind::UnaryOp { op, value } => {
                //
                match value.typ {
                    Some(Type::Boolean) => {
                        let value = self.codegen_expr(value).unwrap().into_int_value();

                        match op {
                            UnaryOp::Not => {
                                todo!()
                            }
                            UnaryOp::Neg => todo!(),
                        }
                    }
                    Some(Type::Integer) => {
                        let value = self.codegen_expr(value).unwrap().into_int_value();

                        match op {
                            UnaryOp::Not => todo!(),
                            UnaryOp::Neg => self.builder.build_int_neg(value, "neg").into(),
                        }
                    }
                    Some(Type::Float) => {
                        let value = self.codegen_expr(value).unwrap().into_float_value();

                        match op {
                            UnaryOp::Not => todo!(),
                            UnaryOp::Neg => self.builder.build_float_neg(value, "neg").into(),
                        }
                    }
                    None => {
                        unreachable!("Critical Error: Type should never be null by this point!")
                    }
                    _ => todo!(),
                }
            }
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
                .i32_type()
                .const_int(value as u64, true)
                .as_basic_value_enum(),
            Literal::Float(value) => self
                .context
                .f32_type()
                .const_float(value as f64)
                .as_basic_value_enum(),
            Literal::Boolean(value) => self
                .context
                .bool_type()
                .const_int(if value { 1 } else { 0 }, false)
                .as_basic_value_enum(),
            Literal::Array(values) => {
                // FIXME: Allocating a new dynamic array for constants is really inefficient
                let element_type = self.type_as_basic_type(values[0].typ.clone().unwrap());
                let i32_type = self.context.i32_type();

                let inner_ptr = self
                    .builder
                    .build_array_malloc(
                        element_type,
                        i32_type.const_int(5000, false),
                        "vecinnerptr",
                    )
                    .unwrap();

                for (idx, value) in values.iter().enumerate() {
                    let value = self.codegen_expr(value).unwrap();
                    let value_ptr = unsafe {
                        self.builder.build_gep(
                            element_type,
                            inner_ptr,
                            &[i32_type.const_int(idx as u64, false)],
                            "",
                        )
                    };

                    self.builder.build_store(value_ptr, value);
                }

                let vector_type = self.context.struct_type(
                    &[
                        i32_type.as_basic_type_enum(),
                        i32_type.as_basic_type_enum(),
                        inner_ptr.get_type().as_basic_type_enum(),
                    ],
                    false,
                );

                let vector_ptr = self.builder.build_malloc(vector_type, "vecptr").unwrap();

                // Set the size and capacity values
                let size_ptr = self
                    .builder
                    .build_struct_gep(vector_type, vector_ptr, 0, "size")
                    .unwrap();
                let cap_ptr = self
                    .builder
                    .build_struct_gep(vector_type, vector_ptr, 1, "cap")
                    .unwrap();
                let inner = self
                    .builder
                    .build_struct_gep(vector_type, vector_ptr, 2, "inner")
                    .unwrap();

                self.builder
                    .build_store(size_ptr, i32_type.const_int(values.len() as u64, false));

                self.builder
                    .build_store(cap_ptr, i32_type.const_int(2500, false));

                self.builder.build_store(inner, inner_ptr);

                vector_ptr.as_basic_value_enum()
            }
            Literal::String(value) => {
                let i32_type = self.context.i32_type();
                let i8_type = self.context.i8_type();
                // let ptr_type = i8_type
                //     .ptr_type(AddressSpace::default())
                //     .as_basic_type_enum();
                //
                // ptr_type.fn_type(&inputs_typ, false)

                let mut values = value
                    .as_bytes()
                    .iter()
                    .map(|it| i8_type.const_int(*it as u64, false))
                    .collect_vec();
                values.push(i8_type.const_int(b'\0' as u64, false));

                let const_arr = i8_type.const_array(&values);

                let ptr = self
                    .builder
                    .build_array_malloc(
                        i8_type,
                        i32_type.const_int(values.len() as u64 + 5, false),
                        "str",
                    )
                    .unwrap();

                // self.builder.build_memcpy(
                //     ptr,
                //     0,
                //     const_arr,
                //     0,
                //     i32_type.const_int(values.len() as u64, false),
                // );
                self.builder.build_store(ptr, const_arr);

                ptr.as_basic_value_enum()
            }
            _ => unimplemented!(),
        }
    }

    fn type_as_basic_type(&self, typ: Type) -> BasicTypeEnum<'ctx> {
        // self.context.i64_type().ptr_type(Address)
        match typ {
            Type::Integer => self.context.i32_type().into(),
            Type::Float => self.context.f32_type().into(),
            Type::Boolean => self.context.bool_type().into(),
            Type::Array { typ, .. } => {
                let i32_type = self.context.i32_type().as_basic_type_enum();

                let typ = self
                    .type_as_basic_type(*typ)
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum();

                let vector_type = self.context.struct_type(&[i32_type, i32_type, typ], false);

                let ptr_to_that = vector_type.ptr_type(AddressSpace::default());

                ptr_to_that.as_basic_type_enum()
            }
            // Type::Array { typ, len } => self.type_as_basic_type(*typ).array_type(len).into(),
            Type::String => {
                let i8_type = self.context.i8_type().as_basic_type_enum();
                i8_type
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
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

#[allow(non_snake_case)]
impl<'ctx> Codegen<'ctx> {
    fn _setup(&mut self, name: &str, func_type: FunctionType<'ctx>) {
        // Preparing for function
        self.references.clear();

        // Making the function
        let func = self.module.add_function(name, func_type, None);

        self.current_func = Some(func);
        self.current_func_void = true;

        let block = self.context.append_basic_block(func, "entrypoint");
        self.builder.position_at_end(block);
    }

    fn _get_ptrs(
        &mut self,
        element_type: BasicTypeEnum<'ctx>,
        vector_ptr: PointerValue<'ctx>,
    ) -> (PointerValue<'ctx>, PointerValue<'ctx>, PointerValue<'ctx>) {
        // Types
        let i32_type = self.context.i32_type();
        let element_ptr_type = element_type.ptr_type(AddressSpace::default());

        let vector_type = self.context.struct_type(
            &[
                i32_type.as_basic_type_enum(),
                i32_type.as_basic_type_enum(),
                element_ptr_type.as_basic_type_enum(),
            ],
            false,
        );

        // Getting pointers
        let size_ptr = self
            .builder
            .build_struct_gep(vector_type, vector_ptr, 0, "sizegep")
            .unwrap();
        let cap_ptr = self
            .builder
            .build_struct_gep(vector_type, vector_ptr, 1, "capgep")
            .unwrap();
        let inner_ptr = self
            .builder
            .build_struct_gep(vector_type, vector_ptr, 2, "innergep")
            .unwrap();

        (size_ptr, cap_ptr, inner_ptr)
    }

    fn _get_values(
        &mut self,
        element_type: BasicTypeEnum<'ctx>,
        size_ptr: PointerValue<'ctx>,
        cap_ptr: PointerValue<'ctx>,
        inner_ptr: PointerValue<'ctx>,
    ) -> (IntValue<'ctx>, IntValue<'ctx>, PointerValue<'ctx>) {
        // Types
        let i32_type = self.context.i32_type();
        let element_ptr_type = element_type.ptr_type(AddressSpace::default());

        // Getting values
        let size = self
            .builder
            .build_load(i32_type, size_ptr, "size")
            .into_int_value();
        let cap = self
            .builder
            .build_load(i32_type, cap_ptr, "cap")
            .into_int_value();
        let inner = self
            .builder
            .build_load(element_ptr_type, inner_ptr, "inner")
            .into_pointer_value();

        (size, cap, inner)
    }

    fn INTRINSIC_vpush(&mut self, name: &str, typ: Type) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(typ.clone()),
        });
        let inputs = &[array_input.into(), self.type_as_metadata_type(typ.clone())];
        let func_type = self.context.void_type().fn_type(inputs, false);
        self._setup(&format!("vpush{name}"), func_type);
        let func = self.current_func.unwrap();

        // Types
        let element_type = self.type_as_basic_type(typ);
        let i32_type = self.context.i32_type();

        // Getting the pointers and values needed
        let vector_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
        let (size_ptr, cap_ptr, inner_ptr) = self._get_ptrs(element_type, vector_ptr);
        let (size, _cap, inner) = self._get_values(element_type, size_ptr, cap_ptr, inner_ptr);

        // FIXME: Array cant grow as of now

        // Put the new element into backing array
        let element = func.get_nth_param(1).unwrap();
        let slot_ptr = unsafe { self.builder.build_gep(element_type, inner, &[size], "slot") };
        self.builder.build_store(slot_ptr, element);

        // Increase size tracker
        let updated_size = self
            .builder
            .build_int_add(size, i32_type.const_int(1, false), "");
        self.builder.build_store(size_ptr, updated_size);

        // Function return
        self.builder.build_return(None);
    }

    fn INTRINSIC_vpop(&mut self, name: &str, typ: Type) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(typ.clone()),
        });
        let inputs = &[array_input.into()];
        let func_type = match typ {
            Type::Integer => self.context.i32_type().fn_type(inputs, false),
            Type::Float => self.context.f32_type().fn_type(inputs, false),
            Type::Boolean => self.context.bool_type().fn_type(inputs, false),
            _ => panic!(),
        };
        self._setup(&format!("vpop{name}"), func_type);
        let func = self.current_func.unwrap();

        // Types
        let element_type = self.type_as_basic_type(typ);
        let i32_type = self.context.i32_type();

        // Getting the pointers and values needed
        let vector_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
        let (size_ptr, cap_ptr, inner_ptr) = self._get_ptrs(element_type, vector_ptr);
        let (size, _cap, inner) = self._get_values(element_type, size_ptr, cap_ptr, inner_ptr);

        // FIXME: Array cant shrink as of now

        // Get the last element in the array
        let element_idx = self
            .builder
            .build_int_sub(size, i32_type.const_int(1, false), "");
        let element_ptr = unsafe {
            self.builder
                .build_gep(element_type, inner, &[element_idx], "elementptr")
        };
        let element = self
            .builder
            .build_load(element_type, element_ptr, "element");

        // Decrease the size tracker
        self.builder.build_store(size_ptr, element_idx);

        // Return element
        self.builder.build_return(Some(&element));
    }

    fn INTRINSIC_vget(&mut self, name: &str, typ: Type) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(typ.clone()),
        });
        // [Array, Index]
        let inputs = &[array_input.into(), self.context.i32_type().into()];
        let func_type = match typ {
            Type::Integer => self.context.i32_type().fn_type(inputs, false),
            Type::Float => self.context.f32_type().fn_type(inputs, false),
            Type::Boolean => self.context.bool_type().fn_type(inputs, false),
            _ => panic!(),
        };
        self._setup(&format!("vget{name}"), func_type);
        let func = self.current_func.unwrap();

        // Types
        let element_type = self.type_as_basic_type(typ);

        // Getting the pointers and values needed
        let vector_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
        let (size_ptr, cap_ptr, inner_ptr) = self._get_ptrs(element_type, vector_ptr);
        let (_, _, inner) = self._get_values(element_type, size_ptr, cap_ptr, inner_ptr);

        // Get the selected element in the array
        let element_idx = func.get_nth_param(1).unwrap().into_int_value();
        let element_ptr = unsafe {
            self.builder
                .build_gep(element_type, inner, &[element_idx], "elementptr")
        };
        let element = self
            .builder
            .build_load(element_type, element_ptr, "element");

        // Return element
        self.builder.build_return(Some(&element));
    }

    fn INTRINSIC_vset(&mut self, name: &str, typ: Type) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(typ.clone()),
        });
        // [Array, Index, Value]
        let inputs = &[
            array_input.into(),
            self.context.i32_type().into(),
            self.type_as_metadata_type(typ.clone()),
        ];
        let func_type = self.context.void_type().fn_type(inputs, false);
        self._setup(&format!("vset{name}"), func_type);
        let func = self.current_func.unwrap();

        // Types
        let element_type = self.type_as_basic_type(typ);

        // Getting the pointers and values needed
        let vector_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
        let (size_ptr, cap_ptr, inner_ptr) = self._get_ptrs(element_type, vector_ptr);
        let (_, _, inner) = self._get_values(element_type, size_ptr, cap_ptr, inner_ptr);

        // Get the selected element in the array
        let element_idx = func.get_nth_param(1).unwrap().into_int_value();
        let element_ptr = unsafe {
            self.builder
                .build_gep(element_type, inner, &[element_idx], "elementptr")
        };
        let element = func.get_nth_param(2).unwrap();
        self.builder.build_store(element_ptr, element);

        self.builder.build_return(None);
    }

    fn INTRINSIC_vremove(&mut self, name: &str, typ: Type) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(typ.clone()),
        });
        // [Array, Index]
        let inputs = &[array_input.into(), self.context.i32_type().into()];
        let func_type = self.context.void_type().fn_type(inputs, false);
        self._setup(&format!("vremove{name}"), func_type);
        let func = self.current_func.unwrap();

        // Types
        let element_type = self.type_as_basic_type(typ);

        // FIXME: Array cant shrink as of now
        // TODO: vremove
    }

    fn INTRINSIC_vlen(&mut self) {
        // Setup function
        let array_input = self.type_as_basic_type(Type::Array {
            typ: Box::new(Type::Integer),
        });
        // [Array, Index]
        let inputs = &[array_input.into(), self.context.i32_type().into()];
        let func_type = self.context.i32_type().fn_type(inputs, false);
        self._setup("vlen", func_type);
        let func = self.current_func.unwrap();

        // Getting the pointers and values needed
        let element_type = self.type_as_basic_type(Type::Integer); // Dummy - Not actually used
        let vector_ptr = func.get_nth_param(0).unwrap().into_pointer_value();
        let (size_ptr, cap_ptr, inner_ptr) = self._get_ptrs(element_type, vector_ptr);
        let (size, _, _) = self._get_values(element_type, size_ptr, cap_ptr, inner_ptr);

        // Return element
        self.builder.build_return(Some(&size));
    }
}
