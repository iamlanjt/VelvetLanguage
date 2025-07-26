use std::{collections::HashMap, path::Path};

use inkwell::{
    AddressSpace, IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    targets::TargetMachine,
    values::{BasicValueEnum, IntValue, PointerValue},
};

use crate::parser::nodetypes::Node;

pub struct IRGenerator<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    variables: Vec<HashMap<String, PointerValue<'ctx>>>,
    format_str: Option<PointerValue<'ctx>>,
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
            format_str: None,
        }
    }

    fn enter_scope(&mut self) {
        self.variables.push(HashMap::new())
    }

    fn exit_scope(&mut self) {
        self.variables.pop();
    }

    fn declare_variable(&mut self, name: &str, ptr: PointerValue<'ctx>) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), ptr);
        } else {
            panic!("No scope to declare var in");
        }
    }

    fn get_variable(&self, name: &str) -> Option<PointerValue<'ctx>> {
        for scope in self.variables.iter().rev() {
            if let Some(&ptr) = scope.get(name) {
                return Some(ptr);
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
            let printf_type = self.context.i32_type().fn_type(&[i8ptr_type.into()], true);
            self.module.add_function("printf", printf_type, None)
        }
    }

    pub fn generate_ir_for_nodes(&mut self, nodes: Vec<Node>) {
        self.format_str = Some(self.build_global_format_string("format_str", "%d\n"));
        self.enter_scope();
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        let mut last_val = i32_type.const_int(0, false);

        for node in nodes {
            last_val = self.generate_ir_for_expr(&node);
        }

        self.builder.build_return(Some(&last_val)).unwrap();
    }

    pub fn generate_ir_for_expr(&mut self, node: &Node) -> IntValue<'ctx> {
        match node {
            Node::NumericLiteral(n) => self
                .context
                .i32_type()
                .const_int(n.literal_value.parse::<i32>().unwrap() as u64, false),
            Node::BinaryExpr(bin_op) => {
                let left_val = self.generate_ir_for_expr(&bin_op.left);
                let right_val = self.generate_ir_for_expr(&bin_op.right);

                match bin_op.op.as_str() {
                    "+" => self
                        .builder
                        .build_int_add(left_val, right_val, "addtmp")
                        .unwrap(),
                    "-" => self
                        .builder
                        .build_int_sub(left_val, right_val, "subtmp")
                        .unwrap(),
                    "*" => self
                        .builder
                        .build_int_mul(left_val, right_val, "multmp")
                        .unwrap(),
                    "/" => self
                        .builder
                        .build_int_signed_div(left_val, right_val, "divtmp")
                        .unwrap(),
                    _ => unimplemented!(),
                }
            }
            Node::VarDeclaration(vdecl) => {
                let alloca = self
                    .builder
                    .build_alloca(self.context.i32_type(), &vdecl.var_identifier)
                    .unwrap();
                self.declare_variable(&vdecl.var_identifier, alloca);

                let init_val = self.generate_ir_for_expr(&vdecl.var_value);
                self.builder.build_store(alloca, init_val).unwrap();

                self.context.i32_type().const_int(0, false)
            }
            Node::Identifier(ident) => {
                if let Some(ptr) = self.get_variable(&ident.identifier_name) {
                    self.builder
                        .build_load(self.context.i32_type(), ptr, &ident.identifier_name)
                        .unwrap()
                        .into_int_value()
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
                        .map(|arg| self.generate_ir_for_expr(arg).into())
                        .collect();

                    let call_site = self.builder.build_call(func, &args, "calltmp").unwrap();

                    if let Some(ret_val) = call_site.try_as_basic_value().left() {
                        ret_val.into_int_value()
                    } else {
                        self.context.i32_type().const_int(0, false)
                    }
                } else {
                    match function_name.as_str() {
                        "print" => {
                            let val = self.generate_ir_for_expr(&cexpr.args[0]);
                            let print_fn = self.get_or_declare_printf();
                            let format_str =
                                self.format_str.expect("Format string not initialized");
                            self.builder
                                .build_call(print_fn, &[format_str.into(), val.into()], "printcall")
                                .unwrap();
                            self.context.i32_type().const_int(0, false)
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
                let condition = self.generate_ir_for_expr(&ws.condition);
                /*
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::NE, condition, zero, "wcnd_cmp")
                    .unwrap();
                */
                self.builder
                    .build_conditional_branch(condition, body_block, end_block)
                    .unwrap();

                self.builder.position_at_end(body_block); // <- FIXED

                self.enter_scope();
                for stmt in &ws.body {
                    self.generate_ir_for_expr(stmt);
                }
                self.exit_scope();

                self.builder.build_unconditional_branch(cond_block).unwrap();

                self.builder.position_at_end(end_block);

                self.context.i32_type().const_int(0, false)
            }
            Node::Comparator(comp) => {
                let left = self.generate_ir_for_expr(&comp.lhs);
                let right = self.generate_ir_for_expr(&comp.rhs);

                match comp.op.as_str() {
                    "==" => self
                        .builder
                        .build_int_compare(IntPredicate::EQ, left, right, "eqtmp")
                        .unwrap(),
                    "!=" => self
                        .builder
                        .build_int_compare(IntPredicate::NE, left, right, "netmp")
                        .unwrap(),
                    ">" => self
                        .builder
                        .build_int_compare(IntPredicate::SGT, left, right, "sgttmp")
                        .unwrap(),
                    "<" => self
                        .builder
                        .build_int_compare(IntPredicate::SLT, left, right, "slttmp")
                        .unwrap(),
                    _ => unimplemented!(),
                }
            }
            Node::AssignmentExpr(a) => {
                let left = match a.left.as_ref() {
                    Node::Identifier(ident) => self.get_variable(&ident.identifier_name).unwrap(),
                    _ => unimplemented!(),
                };
                let right = self.generate_ir_for_expr(&a.value);

                self.builder.build_store(left, right).unwrap();

                right
            }
            _ => unimplemented!("{:?}", node),
        }
    }
}
