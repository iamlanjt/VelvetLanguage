use std::{collections::HashMap, fmt::Pointer, path::Path};

use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    targets::TargetMachine,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValueEnum, PointerValue},
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
}

impl<'ctx> IRGenerator<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        IRGenerator {
            context,
            builder,
            module,
            variables: Vec::new(),
            format_str_int: None,
            format_str_str: None,
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

    fn is_variable_mutable(&self, name: &str) -> bool {
        for scope in self.variables.iter().rev() {
            if let Some(var) = scope.get(name) {
                return var.mutable;
            }
        }
        false
    }

    fn get_variable(&self, name: &str) -> Option<IRVar<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var.clone());
            }
        }
        None
    }

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

    pub fn generate_ir_for_nodes(&mut self, nodes: Vec<Node>) {
        self.format_str_int = Some(self.build_global_format_string("format_str_int", "%lld\n"));
        self.format_str_str = Some(self.build_global_format_string("format_str_str", "%s\n"));
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
        match node {
            Node::StringLiteral(slit) => Some(
                self.build_global_string(&slit.literal_value, "gstring")
                    .into(),
            ),
            Node::NumericLiteral(n) => {
                let parsed_val = n.literal_value.parse::<u64>().unwrap();
                let base_type = self.context.i64_type();
                Some(base_type.const_int(parsed_val, false).into())
            }
            Node::BinaryExpr(bin_op) => {
                let left = self.generate_ir_for_expr(&bin_op.left);
                let right = self.generate_ir_for_expr(&bin_op.right);

                match (left, right) {
                    (Some(BasicValueEnum::IntValue(l)), Some(BasicValueEnum::IntValue(r))) => {
                        let val = match bin_op.op.as_str() {
                            "+" => self.builder.build_int_add(l, r, "addtmp"),
                            "-" => self.builder.build_int_sub(l, r, "subtmp"),
                            "*" => self.builder.build_int_mul(l, r, "multmp"),
                            "/" => self.builder.build_int_signed_div(l, r, "signed_divtmp"),
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
                let init_val = self
                    .generate_ir_for_expr(&vdecl.var_value)
                    .unwrap_or_else(|| self.context.i64_type().const_zero().into());

                let llvm_type = match init_val {
                    BasicValueEnum::IntValue(_) => self.context.i64_type().as_basic_type_enum(),
                    BasicValueEnum::PointerValue(ptr) => ptr.get_type().as_basic_type_enum(),
                    _ => panic!("Unsupported initializer type"),
                };

                let alloca = self
                    .builder
                    .build_alloca(llvm_type, &vdecl.var_identifier)
                    .unwrap();

                self.declare_variable(&vdecl.var_identifier, alloca, llvm_type, vdecl.is_mutable);
                self.builder.build_store(alloca, init_val).unwrap();

                Some(self.context.i64_type().const_int(0, false).into())
            }

            Node::Identifier(ident) => {
                if let Some(var) = self.get_variable(&ident.identifier_name) {
                    let loaded = self
                        .builder
                        .build_load(var.ty, var.ptr, &ident.identifier_name)
                        .unwrap();
                    Some(loaded)
                } else {
                    panic!("Variable {} not found", ident.identifier_name);
                }
            }

            Node::CallExpr(cexpr) => {
                let function_name = match *cexpr.caller {
                    Node::Identifier(ref ident) => ident.identifier_name.clone(),
                    _ => panic!("Unsupported caller node type in call expression"),
                };

                let function = self.module.get_function(&function_name);

                if let Some(func) = function {
                    let args: Vec<_> = cexpr
                        .args
                        .iter()
                        .map(|arg| self.generate_ir_for_expr(arg).unwrap().into())
                        .collect();

                    let call_site = self.builder.build_call(func, &args, "calltmp").unwrap();

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

                Some(self.context.i64_type().const_int(0, false).into())
            }
            Node::Comparator(comp) => {
                let left = self.generate_ir_for_expr(&comp.lhs);
                let right = self.generate_ir_for_expr(&comp.rhs);

                match (left, right) {
                    (Some(BasicValueEnum::IntValue(l)), Some(BasicValueEnum::IntValue(r))) => {
                        let val = match comp.op.as_str() {
                            "==" => self
                                .builder
                                .build_int_compare(IntPredicate::EQ, l, r, "eqtmp")
                                .unwrap(),
                            "!=" => self
                                .builder
                                .build_int_compare(IntPredicate::NE, l, r, "netmp")
                                .unwrap(),
                            ">" => self
                                .builder
                                .build_int_compare(IntPredicate::SGT, l, r, "sgttmp")
                                .unwrap(),
                            "<" => self
                                .builder
                                .build_int_compare(IntPredicate::SLT, l, r, "slttmp")
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
                // First, get the var info by matching on a.left (must be Identifier)
                let var = match a.left.as_ref() {
                    Node::Identifier(ident) => {
                        // Get immutable borrow, then immediately clone to own it and release borrow
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

                // Now mutable borrow is free, call generate_ir_for_expr
                let right = self.generate_ir_for_expr(&a.value).unwrap();

                // Store the new value
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
                            .i64_type()
                            .const_int(i.literal_value.parse::<u64>().unwrap(), false),
                        _ => panic!("Only integer patterns are supported in match"),
                    };

                    let block_name = format!("match_case_{}", i);
                    let case_block = self.context.append_basic_block(parent_func, &block_name);

                    cases.push((const_val, case_block));
                    case_blocks.push(case_block);
                }

                // Now build the switch
                self.builder
                    .build_switch(target_val.into_int_value(), end_block, &cases)
                    .unwrap();

                let mut incoming_vals = vec![];

                for ((_, body), case_block) in mexpr.arms.iter().zip(case_blocks) {
                    self.builder.position_at_end(case_block);
                    self.enter_scope();
                    let val = self.generate_ir_for_expr(body);
                    self.exit_scope();

                    self.builder.build_unconditional_branch(end_block).unwrap();
                    incoming_vals.push((val, case_block));
                }

                self.builder.position_at_end(end_block);

                let phi = self
                    .builder
                    .build_phi(self.context.i64_type(), "match_result")
                    .unwrap();

                for (val, block) in incoming_vals {
                    println!("{:?}", val);
                    println!("{:?}", block);
                    phi.add_incoming(&[(&val.unwrap(), block)]);
                }

                Some(phi.as_basic_value())
            }
            _ => unimplemented!(),
        }
    }
}
