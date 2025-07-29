use std::collections::HashMap;

use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, IntType},
    values::{BasicValue, BasicValueEnum, GlobalValue, IntValue, PointerValue},
};

use crate::parser::nodetypes::Node;

#[derive(Clone)]
struct IRVar<'ctx> {
    pub ptr: PointerValue<'ctx>,
    pub ty: BasicTypeEnum<'ctx>,
    pub mutable: bool,
}

impl<'ctx> IRVar<'ctx> {
    pub fn new(ptr: PointerValue<'ctx>, ty: BasicTypeEnum<'ctx>, is_mutable: bool) -> Self {
        Self {
            ptr,
            ty,
            mutable: is_mutable,
        }
    }
}

pub struct IRGenerator<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    variables: Vec<HashMap<String, IRVar<'ctx>>>,
    format_str_int: Option<PointerValue<'ctx>>,
    format_str_str: Option<PointerValue<'ctx>>,
    coerce_types: bool,

    // call stack stuff
    call_stack: Option<GlobalValue<'ctx>>,
    call_stack_ptr: Option<GlobalValue<'ctx>>,
}

impl<'ctx> IRGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str, coerce_types: bool) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        IRGenerator {
            context,
            builder,
            module,
            variables: Vec::new(),
            format_str_int: None,
            format_str_str: None,
            coerce_types,
            call_stack: None,
            call_stack_ptr: None,
        }
    }

    fn enter_scope(&mut self) {
        self.variables.push(HashMap::new())
    }

    fn exit_scope(&mut self) {
        self.variables.pop();
    }

    fn declare_variable(
        &mut self,
        name: &str,
        ptr: PointerValue<'ctx>,
        ty: BasicTypeEnum<'ctx>,
        mutable: bool,
    ) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), IRVar::new(ptr, ty, mutable));
        } else {
            panic!("No scope to declare var in");
        }
    }

    fn get_variable(&self, name: &str) -> Option<IRVar<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var.clone());
            }
        }
        None
    }

    /*
    pub fn emit_object_file(&mut self, output_path: &str) {
        use inkwell::targets::{FileType, InitializationConfig, Target};

        Target::initialize_all(&InitializationConfig::default());

        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                inkwell::OptimizationLevel::Aggressive,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .expect("Failed to create target machine");

        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(output_path))
            .expect("Failed to write object file");
    }
    */

    fn build_global_string(&mut self, value: &str, name: &str) -> PointerValue<'ctx> {
        let global_str = self
            .builder
            .build_global_string_ptr(value, name)
            .expect("Failed to generate IR string");

        global_str.as_pointer_value()
    }

    fn build_global_format_string(
        &mut self,
        name: &str,
        value: &str,
    ) -> inkwell::values::PointerValue<'ctx> {
        let c_string = std::ffi::CString::new(value).unwrap();
        let str_len = c_string.as_bytes_with_nul().len() as u32;

        let str_type = self.context.i8_type().array_type(str_len);
        let global = self.module.add_global(str_type, None, name);
        global.set_initializer(
            &self.context.i8_type().const_array(
                &c_string
                    .as_bytes_with_nul()
                    .iter()
                    .map(|&c| self.context.i8_type().const_int(c as u64, false))
                    .collect::<Vec<_>>(),
            ),
        );
        global.set_constant(true);
        global.set_linkage(inkwell::module::Linkage::Private);

        unsafe {
            global.as_pointer_value().const_gep(
                self.context.i32_type(),
                &[
                    self.context.i32_type().const_zero(),
                    self.context.i32_type().const_zero(),
                ],
            )
        }
    }

    fn get_or_declare_printf(&mut self) -> inkwell::values::FunctionValue<'ctx> {
        if let Some(func) = self.module.get_function("printf") {
            func
        } else {
            let i8ptr_type = self.context.ptr_type(AddressSpace::default());
            let printf_type = self.context.i64_type().fn_type(&[i8ptr_type.into()], true);
            self.module.add_function("printf", printf_type, None)
        }
    }

    pub fn coerce_ints_to_common_type(
        &self,
        left: IntValue<'ctx>,
        right: IntValue<'ctx>,
    ) -> (IntValue<'ctx>, IntValue<'ctx>) {
        if !self.coerce_types {
            return (left, right);
        }
        let l_ty = left.get_type();
        let r_ty = right.get_type();

        if l_ty == r_ty {
            return (left, right);
        }

        let precedence = |ty: IntType<'ctx>| match ty.get_bit_width() {
            8 => 0,
            16 => 1,
            32 => 2,
            64 => 3,
            other => panic!("Unsupported int width: {other}"),
        };

        let common_ty = if precedence(l_ty) >= precedence(r_ty) {
            l_ty
        } else {
            r_ty
        };

        let current_block = self.builder.get_insert_block().expect("No insert block");

        self.builder.position_at_end(current_block);

        let left_casted = if l_ty != common_ty {
            self.builder
                .build_int_cast(left, common_ty, "cast_l")
                .unwrap()
        } else {
            left
        };

        let right_casted = if r_ty != common_ty {
            self.builder
                .build_int_cast(right, common_ty, "cast_r")
                .unwrap()
        } else {
            right
        };

        (left_casted, right_casted)
    }

    pub fn enforce_type_equality(
        &self,
        left: &BasicValueEnum<'ctx>,
        right: &BasicValueEnum<'ctx>,
        right_node: &Node,
        help: String,
    ) {
        if left.get_type() != right.get_type() {
            panic!(
                "\n[typecheck::equality] Expected > {} < to be `{}`, but it is of type `{}`\nhelp: {}\n",
                right_node,
                left.get_type().print_to_string().to_str().unwrap(),
                right.get_type().print_to_string().to_str().unwrap(),
                help
            )
        }
    }

    // Helper function used for converting type identifiers to LLVM types
    pub fn str_to_llvm_type(&self, str_type: &str) -> BasicTypeEnum<'ctx> {
        match str_type {
            // GP number that equates to the default compiler-defined number, currently i32
            "number" => self.context.i32_type().into(),
            "i8" => self.context.i8_type().into(),
            "i16" => self.context.i16_type().into(),
            "i32" => self.context.i32_type().into(),
            "i64" => self.context.i64_type().into(),
            "string" => self.context.ptr_type(AddressSpace::default()).into(),
            _ => panic!("Failed to match typestring `{}` to LLVM type.", str_type),
        }
    }

    pub fn generate_ir_for_nodes(&mut self, nodes: Vec<Node>) {
        // Generate globals
        self.format_str_int = Some(self.build_global_format_string("format_str_int", "%lld\n"));
        self.format_str_str = Some(self.build_global_format_string("format_str_str", "%s\n"));

        // Call stack
        // %CallFrame
        let i32_type = self.context.i32_type();
        let char_arr_type = self.context.i8_type().array_type(64);
        let call_frame_type = self
            .context
            .struct_type(&[i32_type.into(), char_arr_type.into()], false);

        // @__CALL_STACK
        let stack_array_type = call_frame_type.array_type(64);
        let call_stack_global = self
            .module
            .add_global(stack_array_type, None, "__CALL_STACK");
        call_stack_global.set_initializer(&stack_array_type.const_zero());
        call_stack_global.set_linkage(Linkage::Internal);

        // @__CALL_STACK_PTR
        let call_stack_ptr = self.module.add_global(i32_type, None, "__CALL_STACK_PTR");
        call_stack_ptr.set_initializer(&i32_type.const_zero());
        call_stack_global.set_linkage(Linkage::Internal);

        self.call_stack = Some(call_stack_global);
        self.call_stack_ptr = Some(call_stack_ptr);

        // Main entry function, start proeducing LLVM IR
        // vvvvvvvvvvvvvvvvvvv
        self.enter_scope();
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        let mut last_val = Some(i64_type.const_int(0, false).into());

        for node in nodes {
            last_val = self.generate_ir_for_expr(&node);
        }

        if let Some(x) = last_val {
            self.builder.build_return(Some(&x)).unwrap();
        } else {
            // there is no return value, so return zero (success) to the main function
            let zero = self.context.i64_type().const_int(0, false);
            self.builder.build_return(Some(&zero)).unwrap();
        }
    }

    pub fn generate_ir_for_expr(&mut self, node: &Node) -> Option<BasicValueEnum<'ctx>> {
        // println!("Generating IR for {:?}", node);
        match node {
            Node::NoOpNode(_) => None,
            Node::StringLiteral(slit) => Some(
                self.build_global_string(&slit.literal_value, "gstring")
                    .into(),
            ),
            Node::NumericLiteral(n) => {
                let parsed_val = n.literal_value.parse::<u64>().unwrap();
                let base_type = self.context.i32_type();
                Some(base_type.const_int(parsed_val, false).into())
            }
            Node::TypeCast(cast) => {
                let expr_val = self
                    .generate_ir_for_expr(&cast.left)
                    .expect("Expected value to cast");

                let target_llvm_type = self.str_to_llvm_type(&cast.target_type);
                let target_type_str = cast.target_type.as_str();

                match expr_val {
                    BasicValueEnum::IntValue(int_val) => {
                        let src_type = int_val.get_type();

                        if target_llvm_type == src_type.into() {
                            Some(int_val.into())
                        } else if target_llvm_type.is_int_type() {
                            let dst_int_type = target_llvm_type.into_int_type();

                            let casted = if dst_int_type.get_bit_width() > src_type.get_bit_width()
                            {
                                self.builder
                                    .build_int_s_extend(int_val, dst_int_type, "sext_cast")
                                    .unwrap()
                            } else {
                                self.builder
                                    .build_int_truncate(int_val, dst_int_type, "trunc_cast")
                                    .unwrap()
                            };

                            Some(casted.into())
                        } else {
                            panic!("Cannot cast int to non-int type: {:?}", target_type_str);
                        }
                    }
                    _ => panic!("Type casting is only supported for int values right now"),
                }
            }
            Node::Return(r) => {
                let sub = self
                    .generate_ir_for_expr(&r.return_statement)
                    .expect("Expeted return statement to return a value.");
                self.builder.build_return(Some(&sub)).unwrap();
                None
            }
            Node::FunctionDefinition(fd) => {
                let return_type = self.str_to_llvm_type(&fd.return_type);
                let param_types: Vec<BasicMetadataTypeEnum> = fd
                    .params
                    .iter()
                    .map(|param| self.str_to_llvm_type(&param.1).into())
                    .collect();

                self.enter_scope();
                let fn_type = return_type.fn_type(&param_types, false);
                let func = self.module.add_function(&fd.name, fn_type, None);
                let entry_block = self.context.append_basic_block(func, "entry");

                self.builder.position_at_end(entry_block);

                // Set params
                for (i, (name, ty)) in fd.params.iter().enumerate() {
                    let param = func.get_nth_param(i as u32).unwrap();
                    param.set_name(name);

                    let alloca_name = format!("{name}.addr");
                    let alloc = self
                        .builder
                        .build_alloca(param.get_type(), &alloca_name)
                        .unwrap();
                    self.builder.build_store(alloc, param).unwrap();

                    let llvm_type = self.str_to_llvm_type(ty);

                    self.variables.last_mut().unwrap().insert(
                        name.clone(),
                        IRVar {
                            ptr: alloc,
                            ty: llvm_type,
                            mutable: false,
                        },
                    );
                }

                let mut last_val = None;

                for node in fd.body.as_ref() {
                    let result = self.generate_ir_for_expr(node);
                    if let Node::Return(_) = node {
                        break;
                    }
                    last_val = result;
                }

                let main_fn = self.module.get_function("main").unwrap();
                let main_entry = main_fn.get_first_basic_block().unwrap();
                self.builder.position_at_end(main_entry);

                last_val
            }
            Node::NullishCoalescing(nc) => {
                let parent_func = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let lhs = self.generate_ir_for_expr(&nc.left).unwrap();

                let is_truthy = match lhs {
                    BasicValueEnum::IntValue(v) => self.builder.build_int_compare(
                        IntPredicate::NE,
                        v,
                        v.get_type().const_zero(),
                        "truthycmp",
                    ),
                    _ => panic!("Unsupported type for ! operator"),
                };

                let then_block = self.context.append_basic_block(parent_func, "ncthen");
                let else_block = self.context.append_basic_block(parent_func, "ncelse");
                let end_block = self.context.append_basic_block(parent_func, "ncend");

                self.builder
                    .build_conditional_branch(is_truthy.unwrap(), then_block, else_block)
                    .unwrap();

                self.builder.position_at_end(then_block);
                self.builder.build_unconditional_branch(end_block).unwrap();

                let then_block_val = lhs;
                let then_block_end = self.builder.get_insert_block().unwrap();

                self.builder.position_at_end(else_block);
                let rhs_val = self.generate_ir_for_expr(&nc.right).unwrap();
                self.builder.build_unconditional_branch(end_block).unwrap();
                let else_block_end = self.builder.get_insert_block().unwrap();

                self.builder.position_at_end(end_block);
                let phi = match lhs {
                    BasicValueEnum::IntValue(_) => {
                        let phi = self
                            .builder
                            .build_phi(lhs.get_type(), "null_coalesce_result")
                            .unwrap();
                        phi.add_incoming(&[
                            (&then_block_val, then_block_end),
                            (&rhs_val, else_block_end),
                        ]);
                        phi.as_basic_value()
                    }
                    BasicValueEnum::PointerValue(_) => {
                        let phi = self
                            .builder
                            .build_phi(lhs.get_type(), "null_coalesce_result")
                            .unwrap();
                        phi.add_incoming(&[
                            (&then_block_val, then_block_end),
                            (&rhs_val, else_block_end),
                        ]);
                        phi.as_basic_value()
                    }
                    _ => panic!("unsupported type for ??"),
                };

                Some(phi)
            }
            Node::Block(bl) => {
                let mut last_val = Some(self.context.i64_type().const_int(0, false).into());

                for node in &bl.body {
                    last_val = self.generate_ir_for_expr(node);
                }

                if let Some(x) = last_val {
                    Some(x)
                } else {
                    let zero = self.context.i64_type().const_int(0, false);
                    Some(zero.into())
                }
            }
            Node::BinaryExpr(bin_op) => {
                let left = self.generate_ir_for_expr(&bin_op.left);
                let right = self.generate_ir_for_expr(&bin_op.right);

                match (left, right) {
                    (Some(BasicValueEnum::IntValue(l)), Some(BasicValueEnum::IntValue(r))) => {
                        let (l_coerced, r_coerced) = self.coerce_ints_to_common_type(l, r);
                        self.enforce_type_equality(
                            &l_coerced.into(),
                            &r_coerced.into(),
                            &bin_op.right,
                            format!(
                                "try typecasting\n->\t{} {} {}@{}",
                                bin_op.left,
                                bin_op.op,
                                bin_op.right,
                                l_coerced.get_type().print_to_string().to_str().unwrap()
                            ),
                        );
                        let val = match bin_op.op.as_str() {
                            "+" => self.builder.build_int_add(l_coerced, r_coerced, "addtmp"),
                            "-" => self.builder.build_int_sub(l_coerced, r_coerced, "subtmp"),
                            "*" => self.builder.build_int_mul(l_coerced, r_coerced, "multmp"),
                            "/" => self.builder.build_int_signed_div(
                                l_coerced,
                                r_coerced,
                                "signed_divtmp",
                            ),
                            _ => unimplemented!(),
                        };
                        Some(val.unwrap().into())
                    }
                    _ => panic!(
                        "Cannot perform operation `{}` on `{:?}` and `{:?}`",
                        bin_op.op,
                        left.unwrap().get_type().to_string(),
                        right.unwrap().get_type().to_string()
                    ),
                }
            }
            Node::VarDeclaration(vdecl) => {
                let var_type = self.str_to_llvm_type(&vdecl.var_type);
                let init_val = self
                    .generate_ir_for_expr(&vdecl.var_value)
                    .unwrap_or_else(|| var_type.const_zero());

                let alloca = self
                    .builder
                    .build_alloca(var_type, &vdecl.var_identifier)
                    .unwrap();

                self.declare_variable(&vdecl.var_identifier, alloca, var_type, vdecl.is_mutable);
                self.builder.build_store(alloca, init_val).unwrap();

                None
            }

            Node::Identifier(ident) => {
                if let Some(var) = self.get_variable(&ident.identifier_name) {
                    let loaded = self
                        .builder
                        .build_load(var.ty, var.ptr, &ident.identifier_name)
                        .unwrap();
                    Some(loaded)
                } else {
                    // Handle would-be interpreter exceptions for stdlib constants in v1 of velvet
                    // Yes I know it's weird
                    match ident.identifier_name.as_str() {
                        "__CALL_STACK" => {
                            let buffer_size = 4096;
                            let buffer_type = self.context.i8_type().array_type(buffer_size);
                            let buffer_alloca = self
                                .builder
                                .build_alloca(buffer_type, "callstack_buf")
                                .unwrap();

                            let buffer_ptr = unsafe {
                                self.builder
                                    .build_in_bounds_gep(
                                        buffer_type,
                                        buffer_alloca,
                                        &[
                                            self.context.i32_type().const_zero(),
                                            self.context.i32_type().const_zero(),
                                        ],
                                        "callstack_buf_ptr",
                                    )
                                    .unwrap()
                            };

                            let csp = self.call_stack_ptr.unwrap().clone();
                            let cs = self.call_stack.unwrap().clone();

                            let stack_size_val = self
                                .builder
                                .build_load(
                                    self.context.i32_type(),
                                    csp.as_pointer_value(),
                                    "stack_size",
                                )
                                .unwrap()
                                .into_int_value();

                            let printf_fn = self.get_or_declare_printf();
                            let format_prefix =
                                self.build_global_string("%d. %s\n", "stack_format");

                            for i in (0..64).rev() {
                                let i_val = self.context.i32_type().const_int(i, false);

                                let in_bounds = self
                                    .builder
                                    .build_int_compare(
                                        IntPredicate::ULT,
                                        i_val,
                                        stack_size_val,
                                        &format!("stack_check_{}", i),
                                    )
                                    .unwrap();

                                let parent_func = self
                                    .builder
                                    .get_insert_block()
                                    .unwrap()
                                    .get_parent()
                                    .unwrap();
                                let cond_block = self
                                    .context
                                    .append_basic_block(parent_func, &format!("frame_{}", i));
                                let cont_block = self
                                    .context
                                    .append_basic_block(parent_func, &format!("cont_{}", i));
                                self.builder
                                    .build_conditional_branch(in_bounds, cond_block, cont_block)
                                    .unwrap();

                                self.builder.position_at_end(cond_block);

                                let stack_slot = unsafe {
                                    self.builder
                                        .build_in_bounds_gep(
                                            self.context
                                                .ptr_type(AddressSpace::default())
                                                .array_type(64),
                                            cs.as_pointer_value(),
                                            &[self.context.i32_type().const_zero(), i_val],
                                            &format!("slot_ptr_{}", i),
                                        )
                                        .unwrap()
                                };

                                let fn_ptr = self
                                    .builder
                                    .build_load(
                                        self.context.ptr_type(AddressSpace::default()),
                                        stack_slot,
                                        &format!("fnptr_{}", i),
                                    )
                                    .unwrap();

                                self.builder
                                    .build_call(
                                        printf_fn,
                                        &[format_prefix.into(), i_val.into(), fn_ptr.into()],
                                        &format!("print_frame_{}", i),
                                    )
                                    .unwrap();

                                self.builder.build_unconditional_branch(cont_block).unwrap();
                                self.builder.position_at_end(cont_block);
                            }

                            Some(
                                self.build_global_string("Call stack printed", "callstack_done")
                                    .into(),
                            )
                        }
                        _ => panic!("Variable {} not found", ident.identifier_name),
                    }
                }
            }

            Node::CallExpr(cexpr) => {
                let function_name = match *cexpr.caller {
                    Node::Identifier(ref ident) => ident.identifier_name.clone(),
                    _ => panic!(
                        "Unsupported caller node type in call expression: {:#?}",
                        cexpr.caller
                    ),
                };

                let csp = self.call_stack_ptr.expect("CSP not initialized correctly");
                let cs = self.call_stack.expect("CS not initialized correctly");

                let stack_index = self
                    .builder
                    .build_load(
                        self.context.i32_type(),
                        csp.as_pointer_value(),
                        "stack_index",
                    )
                    .unwrap();

                let stack_slot_ptr = unsafe {
                    self.builder
                        .build_in_bounds_gep(
                            self.context
                                .ptr_type(inkwell::AddressSpace::default())
                                .array_type(64),
                            cs.as_pointer_value(),
                            &[
                                self.context.i32_type().const_zero(),
                                stack_index.into_int_value(),
                            ],
                            "callstack_slot",
                        )
                        .unwrap()
                };

                let fn_name_ptr = self
                    .builder
                    .build_global_string_ptr(&function_name, "fnname");

                self.builder
                    .build_store(stack_slot_ptr, fn_name_ptr.unwrap().as_pointer_value())
                    .unwrap();

                let incremented = self
                    .builder
                    .build_int_add(
                        stack_index.into_int_value(),
                        self.context.i32_type().const_int(1, false),
                        "stack_index_plus1",
                    )
                    .unwrap();

                self.builder
                    .build_store(csp.as_pointer_value(), incremented)
                    .unwrap();

                let function = self.module.get_function(&function_name);

                if let Some(func) = function {
                    let args: Vec<_> = cexpr
                        .args
                        .iter()
                        .map(|arg| self.generate_ir_for_expr(arg).unwrap().into())
                        .collect();

                    let call_site = self.builder.build_call(func, &args, "calltmp").unwrap();

                    // Pop call stack
                    let curr_ptr_val = self
                        .builder
                        .build_load(self.context.i32_type(), csp.as_pointer_value(), "stack_ptr")
                        .unwrap()
                        .into_int_value();
                    let new_ptr_val = self
                        .builder
                        .build_int_sub(
                            curr_ptr_val,
                            self.context.i32_type().const_int(1, false),
                            "stack_ptr_dec",
                        )
                        .unwrap();
                    self.builder
                        .build_store(csp.as_pointer_value(), new_ptr_val)
                        .unwrap();

                    if let Some(ret_val) = call_site.try_as_basic_value().left() {
                        Some(ret_val)
                    } else {
                        Some(self.context.i64_type().const_int(0, false).into())
                    }
                } else {
                    match function_name.as_str() {
                        "print" => {
                            let val = self
                                .generate_ir_for_expr(&cexpr.args[0])
                                .unwrap_or(self.context.i64_type().const_zero().into());
                            let print_fn = self.get_or_declare_printf();

                            let format_str = match val {
                                BasicValueEnum::IntValue(_) => self.format_str_int.unwrap(),
                                BasicValueEnum::PointerValue(_) => self.format_str_str.unwrap(),
                                _ => panic!("Unsupported type for print"),
                            };

                            self.builder
                                .build_call(print_fn, &[format_str.into(), val.into()], "printcall")
                                .unwrap();

                            // Pop call stack
                            let curr_ptr_val = self
                                .builder
                                .build_load(
                                    self.context.i32_type(),
                                    csp.as_pointer_value(),
                                    "stack_ptr",
                                )
                                .unwrap()
                                .into_int_value();
                            let new_ptr_val = self
                                .builder
                                .build_int_sub(
                                    curr_ptr_val,
                                    self.context.i32_type().const_int(1, false),
                                    "stack_ptr_dec",
                                )
                                .unwrap();
                            self.builder
                                .build_store(csp.as_pointer_value(), new_ptr_val)
                                .unwrap();

                            None
                            // self.context.i32_type().const_int(0, false).into()
                        }
                        _ => panic!("Unknown function '{}'", function_name),
                    }
                }
            }
            Node::WhileStmt(ws) => {
                let parent_func = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let cond_block = self.context.append_basic_block(parent_func, "wcnd");
                let body_block = self.context.append_basic_block(parent_func, "wbdy");
                let end_block = self.context.append_basic_block(parent_func, "wend");

                self.builder.build_unconditional_branch(cond_block).unwrap();

                self.builder.position_at_end(cond_block);
                let condition = self.generate_ir_for_expr(&ws.condition).unwrap();
                /*
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::NE, condition, zero, "wcnd_cmp")
                    .unwrap();
                */
                self.builder
                    .build_conditional_branch(condition.into_int_value(), body_block, end_block)
                    .unwrap();

                self.builder.position_at_end(body_block); // <- FIXED

                self.enter_scope();
                for stmt in &ws.body {
                    self.generate_ir_for_expr(stmt);
                }
                self.exit_scope();

                self.builder.build_unconditional_branch(cond_block).unwrap();

                self.builder.position_at_end(end_block);

                None
            }
            Node::Comparator(comp) => {
                let left = self.generate_ir_for_expr(&comp.lhs);
                let right = self.generate_ir_for_expr(&comp.rhs);

                match (left, right) {
                    (Some(BasicValueEnum::IntValue(l)), Some(BasicValueEnum::IntValue(r))) => {
                        let (l_coerced, r_coerced) = self.coerce_ints_to_common_type(l, r);

                        let val = match comp.op.as_str() {
                            "==" => self
                                .builder
                                .build_int_compare(IntPredicate::EQ, l_coerced, r_coerced, "eqtmp")
                                .unwrap(),
                            "!=" => self
                                .builder
                                .build_int_compare(IntPredicate::NE, l_coerced, r_coerced, "netmp")
                                .unwrap(),
                            ">" => self
                                .builder
                                .build_int_compare(
                                    IntPredicate::SGT,
                                    l_coerced,
                                    r_coerced,
                                    "sgttmp",
                                )
                                .unwrap(),
                            "<" => self
                                .builder
                                .build_int_compare(
                                    IntPredicate::SLT,
                                    l_coerced,
                                    r_coerced,
                                    "slttmp",
                                )
                                .unwrap(),
                            _ => unimplemented!(),
                        };

                        Some(val.into())
                    }
                    _ => panic!(
                        "Cannot perform operation `{}` on `{:?}` and `{:?}`",
                        comp.op, left, right
                    ),
                }
            }
            Node::ListLiteral(llit) => {
                let i32_type = self.context.i64_type();

                let mut const_vals = vec![];

                for elem in &llit.props {
                    let val = self.generate_ir_for_expr(elem)?;
                    match val {
                        BasicValueEnum::IntValue(int_val) => const_vals.push(int_val),
                        _ => panic!("Only int arrays are supported for now"),
                    }
                }

                let llvm_array = i32_type.const_array(&const_vals);
                let global = self.module.add_global(llvm_array.get_type(), None, "arr");
                global.set_initializer(&llvm_array);
                global.set_constant(true);

                let gep = unsafe {
                    global.as_pointer_value().const_gep(
                        self.context.i64_type(),
                        &[
                            self.context.i64_type().const_zero(),
                            self.context.i64_type().const_zero(),
                        ],
                    )
                };

                Some(gep.into())
            }
            Node::AssignmentExpr(a) => {
                let var = match a.left.as_ref() {
                    Node::Identifier(ident) => {
                        let v = self
                            .get_variable(&ident.identifier_name)
                            .expect("Binding not found");
                        if !v.mutable {
                            panic!(
                                "Attempt to mutate an immutable binding `{}`",
                                ident.identifier_name
                            );
                        }
                        v.clone()
                    }
                    _ => unimplemented!(),
                };

                let right = self.generate_ir_for_expr(&a.value).unwrap();

                self.builder.build_store(var.ptr, right).unwrap();

                Some(right)
            }

            Node::MatchExpr(mexpr) => {
                let target_val = self.generate_ir_for_expr(&mexpr.target).unwrap();
                let parent_func = self
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();

                let end_block = self.context.append_basic_block(parent_func, "match_end");

                let mut cases = vec![];
                let mut case_blocks = vec![];

                for (i, (pat, _)) in mexpr.arms.iter().enumerate() {
                    let const_val = match pat {
                        Node::NumericLiteral(i) => self
                            .context
                            .i32_type()
                            .const_int(i.literal_value.parse::<u64>().unwrap(), false),
                        _ => panic!("Only integer patterns are supported in match"),
                    };

                    let block_name = format!("match_case_{}", i);
                    let case_block = self.context.append_basic_block(parent_func, &block_name);

                    cases.push((const_val, case_block));
                    case_blocks.push(case_block);
                }
                let mut incoming_vals = vec![];

                let default_block = self
                    .context
                    .append_basic_block(parent_func, "match_default");

                self.builder
                    .build_switch(target_val.into_int_value(), default_block, &cases)
                    .unwrap();

                self.builder.position_at_end(default_block);
                self.builder.build_unconditional_branch(end_block).unwrap();
                incoming_vals.push((
                    Some(self.context.i32_type().const_zero().as_basic_value_enum()),
                    default_block,
                ));

                for ((_, body), case_block) in mexpr.arms.iter().zip(case_blocks) {
                    self.builder.position_at_end(case_block);
                    self.enter_scope();
                    let val = self.generate_ir_for_expr(body);
                    self.exit_scope();

                    self.builder.build_unconditional_branch(end_block).unwrap();
                    incoming_vals.push((val, case_block));
                }

                self.builder.position_at_end(end_block);

                let phi_type = self
                    .generate_ir_for_expr(&mexpr.arms[0].1)
                    .unwrap()
                    .get_type();
                let phi = self.builder.build_phi(phi_type, "match_result").unwrap();

                for (val, block) in incoming_vals {
                    phi.add_incoming(&[(&val.unwrap(), block)]);
                }

                Some(phi.as_basic_value())
            }
            _ => unimplemented!("No compiler value yet: {:#?}", node),
        }
    }
}
