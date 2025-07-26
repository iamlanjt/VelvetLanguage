use colored::*;
use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    parser::nodetypes::{
        AssignmentExpr, BinaryExpr, CallExpr, Comparator, FunctionDefinition, Identifier, IfStmt,
        Iterator, MatchExpr, MemberExpr, Node, NullishCoalescing, ObjectLiteral, VarDeclaration,
        WhileStmt,
    },
    runtime::{
        source_environment::source_environment::SourceEnv,
        values::{
            BoolVal, FunctionVal, InternalFunctionVal, ListVal, NullVal, NumberVal, ObjectVal,
            ReturnVal, RuntimeVal, StringVal,
        },
    },
};

#[macro_export]
macro_rules! velvet_error {
    ($ctx:expr, $($arg:tt)*) => {
        $ctx.interpreter_error(format_args!($($arg)*))
    };
}

#[derive(Clone, Debug)]
pub struct UserDefinedFn {
    function: FunctionVal,
    display: String,
}

#[derive(Clone, Debug)]
pub enum CallTarget {
    Internal(String),
    UserDefined(UserDefinedFn),
}

pub struct Interpreter {
    ast: Vec<Node>,
    call_stack: Vec<CallTarget>,
    do_profile: bool,
    profile_stats: HashMap<String, ProfilerItem>,
}

pub struct ProfilerItem {
    name: String,
}

impl Interpreter {
    pub fn new(ast: Vec<Node>, do_profile: bool) -> Self {
        Self {
            ast,
            call_stack: Vec::new(),
            do_profile,
            profile_stats: HashMap::new(),
        }
    }

    fn auto_reassign_methods() -> &'static [&'static str] {
        &["push"]
    }

    pub fn evaluate_body(&mut self, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        self.call_stack.push(CallTarget::Internal(String::from(
            "velvet::entry_point::evaluate_body(...)",
        )));
        let ast = self.ast.clone();
        let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
        for node in ast {
            last_result = self.evaluate(Box::new(node), Rc::clone(&env));
        }
        self.call_stack.pop();
        last_result
    }

    pub fn interpreter_error(&mut self, args: fmt::Arguments<'_>) -> ! {
        self.call_stack.push(CallTarget::Internal(String::from(
            "velvet::runtime_error::interpreter_error(...)",
        )));
        println!(
            "Velvet Runtime Error\n- {}",
            format!("{}", args).red().bold()
        );
        self.call_stack.push(CallTarget::Internal(String::from(
            "velvet::internal_identifier_exceptions::call_stack_getter",
        )));

        let mut end_stack_string = format!(
            "\n0 = latest call; {} = first call; % = Rust thread\n{}",
            self.call_stack.len() - 1,
            format!("{}", "velvet call stack").blue().bold().underline(),
        );

        for (index, call) in self.call_stack.iter().rev().enumerate() {
            let end_str: String = match call {
                CallTarget::Internal(i) => "% ".to_owned() + &i.clone(),
                CallTarget::UserDefined(u) => {
                    "  ".to_owned()
                        + &u.function.fn_name.clone()
                        + "("
                        + &u.function.params.join(", ")
                        + ")"
                }
            };
            end_stack_string += &format!(
                "\n {} → {}",
                format!("{}", index).blue().underline().bold(),
                end_str
            );
        }

        println!("{}", end_stack_string);
        self.call_stack.pop();
        std::process::exit(-1);
    }

    pub fn evaluate(&mut self, node: Box<Node>, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        match node.as_ref() {
            Node::NumericLiteral(nl) => {
                let numeric_value: isize = nl.literal_value.parse().unwrap();
                Box::new(RuntimeVal::NumberVal(NumberVal {
                    value: numeric_value,
                }))
            }
            Node::StringLiteral(slit) => Box::new(RuntimeVal::StringVal(StringVal {
                value: slit.literal_value.clone(),
            })),
            Node::BoolLiteral(bl) => Box::new(RuntimeVal::BoolVal(BoolVal {
                value: bl.literal_value,
            })),
            Node::Return(ret) => Box::new(RuntimeVal::ReturnVal(ReturnVal {
                value: ret.return_statement.clone(),
            })),
            Node::ListLiteral(ll) => {
                let mut results: Vec<RuntimeVal> = Vec::new();
                for inner_node in &ll.props {
                    results.push(*self.evaluate(Box::new(inner_node.clone()), Rc::clone(&env)));
                }
                Box::new(RuntimeVal::ListVal(ListVal { values: results }))
            }
            Node::NullLiteral(_) => Box::new(RuntimeVal::NullVal(NullVal {})),
            Node::BinaryExpr(binop) => self.evaluate_binary_expr(binop, env),
            Node::VarDeclaration(decl) => self.evaluate_var_declaration(decl, env),
            Node::Identifier(ident) => self.evaluate_identifier(ident, env),
            Node::WhileStmt(while_loop) => self.evaluate_while_stmt(while_loop, env),
            Node::IfStmt(if_stmt) => self.evaluate_if_stmt(if_stmt, env),
            Node::Comparator(comp) => self.evaluate_comparator_expr(comp, env),
            Node::AssignmentExpr(asexp) => self.evaluate_assignment_expr(asexp, env),
            Node::FunctionDefinition(fdef) => self.evaluate_function_definition(fdef, env),
            Node::CallExpr(cexpr) => self.evaluate_call_expr(cexpr, env),
            Node::Iterator(it) => self.evaluate_iterator_expr(it, env),
            Node::ObjectLiteral(ol) => self.evaluate_object_literal(ol, env),
            Node::MemberExpr(mem) => self.evaluate_member_expr(mem, env),
            Node::MatchExpr(mexpr) => self.evaluate_match_expr(mexpr, env),
            Node::NoOpNode(_) => Box::new(RuntimeVal::NullVal(NullVal {})),
            Node::NullishCoalescing(n) => self.evaluate_nullish_coalescing(n, env),
            Node::Block(block) => {
                let mut last = Box::new(RuntimeVal::NullVal(NullVal {}));
                let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));
                for sub_node in &block.body {
                    last = self.evaluate(Box::new(sub_node.clone()), Rc::clone(&sub_environment));
                    match *last {
                        RuntimeVal::ReturnVal(r) => {
                            last = self.evaluate(r.value, Rc::clone(&sub_environment));
                            break;
                        }
                        _ => {}
                    }
                }
                last
            }
            _ => {
                velvet_error!(
                    self,
                    "Evaluation match fault:\nThis node has not been set up for execution yet!\n\n{:#?}\n\n",
                    node
                )
            }
        }
    }

    fn evaluate_nullish_coalescing(
        &mut self,
        nc: &NullishCoalescing,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let left = self.evaluate(nc.left.clone(), Rc::clone(&env));

        if left.is_null() {
            self.evaluate(nc.right.clone(), Rc::clone(&env))
        } else {
            left
        }
    }

    fn evaluate_match_expr(
        &mut self,
        mexpr: &MatchExpr,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        // Target is the LHS of the match expr, or the expression directly after the `match` keyword and before the match body.
        let target = self.evaluate(mexpr.target.clone(), Rc::clone(&env));

        for arm in &mexpr.arms {
            let left = self.evaluate(Box::new(arm.0.clone()), Rc::clone(&env));

            match left.as_ref() {
                RuntimeVal::FunctionVal(_) => {
                    let left_result = self.evaluate_call_expr(
                        &CallExpr {
                            args: Vec::from([*mexpr.target.clone()]),
                            caller: Box::new(arm.0.clone()),
                        },
                        Rc::clone(&env),
                    );
                    if left_result
                        .compare(&RuntimeVal::BoolVal(BoolVal { value: true }), "==")
                        .unwrap()
                    {
                        return self.evaluate(Box::new(arm.1.clone()), Rc::clone(&env));
                    }
                }
                RuntimeVal::InternalFunctionVal(_) => {
                    let left_result = self.evaluate_call_expr(
                        &CallExpr {
                            args: Vec::from([*mexpr.target.clone()]),
                            caller: Box::new(arm.0.clone()),
                        },
                        Rc::clone(&env),
                    );
                    if left_result
                        .compare(&RuntimeVal::BoolVal(BoolVal { value: true }), "==")
                        .unwrap()
                    {
                        return self.evaluate(Box::new(arm.1.clone()), Rc::clone(&env));
                    }
                }
                _ => {
                    let comparison = target.compare(&left, "==");
                    if comparison.is_ok() && comparison.unwrap() == true {
                        return self.evaluate(Box::new(arm.1.clone()), Rc::clone(&env));
                    }
                }
            }
        }

        return Box::new(RuntimeVal::NullVal(NullVal {}));
    }

    fn evaluate_member_expr(
        &mut self,
        mem: &MemberExpr,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let base_val = match *mem.object {
            Node::Identifier(ref ident) => self.evaluate_identifier(ident, Rc::clone(&env)),
            Node::MemberExpr(ref inner) => self.evaluate_member_expr(inner, Rc::clone(&env)),
            _ => {
                velvet_error!(self, "Invalid object in member expression.");
            }
        };

        let property_key = match *mem.property {
            Node::Identifier(ref ident) => ident.identifier_name.clone(),
            Node::NumericLiteral(ref numlit) => numlit.literal_value.clone(),
            _ => "".into(),
        };

        match *base_val {
            RuntimeVal::ObjectVal(ref obj) => {
                if let Some(val) = obj.values.get(&property_key) {
                    return Box::new(val.clone());
                } else {
                    return Box::new(RuntimeVal::NullVal(NullVal {}));
                }
            }
            RuntimeVal::ListVal(ref list) => {
                let list_rc = Rc::new(RefCell::new(list.clone()));

                let list_for_closure = Rc::clone(&list_rc);

                match property_key.as_str() {
                    "push" => {
                        let list_ref = Rc::clone(&list_rc);

                        return Box::new(RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: "push".into(),
                            internal_callback: Rc::new(move |args, _| {
                                let mut borrowed = list_ref.borrow_mut();
                                borrowed.values.extend(args);
                                RuntimeVal::ListVal(ListVal {
                                    values: borrowed.to_owned().values,
                                })
                            }),
                        }));
                    }
                    "len" => {
                        return Box::new(RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: "len".into(),
                            internal_callback: Rc::new(move |_, _| {
                                RuntimeVal::NumberVal(NumberVal {
                                    value: list_for_closure.borrow().len(),
                                })
                            }),
                        }));
                    }
                    _ => {
                        if let Ok(idx) = property_key.parse::<usize>() {
                            if let Some(val) = list.values.get(idx) {
                                return Box::new(val.clone());
                            } else {
                                velvet_error!(self, "Index {} is out of bounds!", idx);
                            }
                        } else if mem.is_computed {
                            let computed_property =
                                self.evaluate(mem.property.clone(), Rc::clone(&env));

                            let index = match *computed_property {
                                RuntimeVal::NumberVal(n) => {
                                    if n.value < 0 {
                                        velvet_error!(
                                            self,
                                            "List index must be a non-negative integer."
                                        );
                                    }
                                    n.value as usize
                                }
                                _ => {
                                    velvet_error!(self, "Computed index must be a number.");
                                }
                            };

                            if let Some(val) = list.values.get(index) {
                                return Box::new(val.clone());
                            } else {
                                velvet_error!(self, "Index {} is out of bounds!", index);
                            }
                        } else {
                            velvet_error!(self, "Invalid index access on list: {}", property_key);
                        }
                    }
                }
            }
            RuntimeVal::StringVal(ref strvl) => {
                if let Ok(idx) = property_key.parse::<usize>() {
                    if idx > strvl.value.len() {
                        velvet_error!(self, "Index out-of-bounds on string");
                    } else {
                        return Box::new(RuntimeVal::StringVal(StringVal {
                            value: strvl.value.chars().nth(idx).unwrap().try_into().unwrap(),
                        }));
                    }
                } else {
                    velvet_error!(self, "Invalid index access on string: {}", property_key);
                }
            }
            _ => {
                velvet_error!(
                    self,
                    "Cannot access property '{}' on non-object value.",
                    property_key
                );
            }
        }
    }

    fn evaluate_object_literal(
        &mut self,
        ov: &ObjectLiteral,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let mut runtime_props: HashMap<String, RuntimeVal> = HashMap::new();
        for inner_prop in &ov.props {
            runtime_props.insert(
                inner_prop.0.to_string(),
                *self.evaluate(Box::new(inner_prop.1.clone()), Rc::clone(&env)),
            );
        }
        Box::new(RuntimeVal::ObjectVal(ObjectVal {
            values: runtime_props,
        }))
    }

    fn evaluate_iterator_expr(
        &mut self,
        it: &Iterator,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let loop_through = self.evaluate(it.right.clone(), Rc::clone(&env));

        match loop_through.as_ref() {
            RuntimeVal::ListVal(lv) => {
                let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
                for v in &lv.values {
                    let sub_environment =
                        Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));
                    sub_environment.borrow_mut().declare_var(
                        it.left.literal_value.clone(),
                        v.clone(),
                        "u".to_owned(),
                        false,
                    );
                    for sub_expr in &it.body {
                        last_result =
                            self.evaluate(Box::new(sub_expr.clone()), Rc::clone(&sub_environment));
                        match last_result.as_ref() {
                            RuntimeVal::ReturnVal(rt) => {
                                return self
                                    .evaluate(rt.clone().value, Rc::clone(&sub_environment));
                            }
                            _ => {}
                        }
                    }
                }
                last_result
            }
            _ => {
                velvet_error!(self, "Cannot loop through non-list type");
            }
        }
    }

    fn evaluate_call_expr(
        &mut self,
        cexpr: &CallExpr,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let caller = self.evaluate(cexpr.caller.clone(), Rc::clone(&env));
        let callstack_push = match &*caller {
            RuntimeVal::FunctionVal(f) => CallTarget::UserDefined(UserDefinedFn {
                function: f.clone(),
                display: format!("{}({})", f.fn_name.clone(), f.params.join(", ")),
            }),
            RuntimeVal::InternalFunctionVal(f) => {
                CallTarget::Internal(format!("velvet::internal_functions::{}(...)", f.fn_name))
            }
            _ => CallTarget::Internal(format!(
                ">>> ILLEGAL CALL -> Caller = \"{:?}\", arglen = {} arg(s)",
                caller,
                cexpr.args.len()
            )),
        };
        self.call_stack.push(callstack_push);
        if self.call_stack.len() > 100 {
            self.call_stack.remove(0);
        }
        let res = match caller.as_ref() {
            RuntimeVal::FunctionVal(r#fn) => {
                // create sub-environment
                let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));

                if cexpr.args.len() < r#fn.params.len() {
                    velvet_error!(
                        self,
                        "Invalid call expression: expected {} arg{} for function '{}', received {}",
                        r#fn.params.len(),
                        if r#fn.params.len() != 1 { "s" } else { "" },
                        r#fn.fn_name,
                        cexpr.args.len()
                    );
                }

                // set all the args for the sub environment that were supplied in the CallExpr
                let mut i = 0;
                let mut post_evaluate_args: Vec<String> = Vec::new();
                for arg in &cexpr.args {
                    let evaluated = self.evaluate(Box::new(arg.clone()), Rc::clone(&env));
                    post_evaluate_args.push(format!(
                        "{} = {:#?}",
                        r#fn.params.get(i).unwrap(),
                        evaluated
                    ));
                    sub_environment.borrow_mut().declare_var(
                        r#fn.params[i].clone(),
                        *evaluated,
                        "inferred_any".to_string(),
                        false,
                    );
                    i = i + 1;
                }
                let old = self.call_stack.pop().unwrap();
                self.call_stack.push(match old {
                    CallTarget::Internal(_) => CallTarget::Internal(format!(
                        "{}({})",
                        r#fn.fn_name,
                        post_evaluate_args.join(", ")
                    )),
                    CallTarget::UserDefined(_) => CallTarget::UserDefined(UserDefinedFn {
                        function: r#fn.clone(),
                        display: format!("{}({})", r#fn.fn_name, post_evaluate_args.join(", ")),
                    }),
                });

                let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
                for sub_expr in r#fn.execution_body.as_ref() {
                    last_result =
                        self.evaluate(Box::new(sub_expr.clone()), Rc::clone(&sub_environment));
                    match last_result.as_ref() {
                        RuntimeVal::ReturnVal(rt) => {
                            let result =
                                self.evaluate(rt.clone().value, Rc::clone(&sub_environment));
                            self.call_stack.pop();
                            return result;
                        }
                        _ => {}
                    }
                }
                last_result
            }
            RuntimeVal::InternalFunctionVal(r#fn) => {
                let mut new_args: Vec<RuntimeVal> = Vec::new();
                let mut post_evaluate_args: Vec<String> = Vec::new();
                for (index, arg) in cexpr.args.iter().enumerate() {
                    let evaluated_arg = self.evaluate(Box::new(arg.clone()), Rc::clone(&env));
                    post_evaluate_args.push(format!("param{} = {}", index, "unknown"));
                    new_args.push(*evaluated_arg);
                }
                let internal_result = (r#fn.internal_callback)(new_args, Rc::clone(&env));
                if Self::auto_reassign_methods().contains(&r#fn.fn_name.as_str()) {
                    match *cexpr.caller.clone() {
                        Node::Identifier(ident) => {
                            env.borrow_mut()
                                .attempt_assignment(ident.identifier_name, internal_result.clone());
                        }
                        Node::MemberExpr(member_expr) => {
                            if let Node::Identifier(obj_ident) = *member_expr.object {
                                env.borrow_mut().attempt_assignment(
                                    obj_ident.identifier_name,
                                    internal_result.clone(),
                                );
                            } else {
                                velvet_error!(
                                    self,
                                    "Auto-reassign for complex member expressions not implemented"
                                );
                            }
                        }
                        _ => {
                            velvet_error!(
                                self,
                                "Auto-reassign is only supported on identifiers or member expressions"
                            );
                        }
                    }
                }
                self.call_stack.pop();
                return Box::new(internal_result);
            }
            _ => {
                velvet_error!(self, "Cannot call type \"{:#?}\"", caller)
            }
        };
        self.call_stack.pop();
        res
    }

    fn evaluate_function_definition(
        &mut self,
        def: &FunctionDefinition,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let this_function_val = RuntimeVal::FunctionVal(FunctionVal {
            params: def.params.clone(),
            fn_name: def.name.clone(),
            execution_body: Rc::clone(&def.body), // Reference counter clone because deep cloning nodes is not cheap
            is_internal: false,
        });

        // Add to env
        env.borrow_mut().declare_var(
            def.name.clone(),
            this_function_val,
            String::from("function"),
            false,
        );

        // Definitions do not return anything; it is automatically added by name to the env
        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_assignment_expr(
        &mut self,
        asexp: &AssignmentExpr,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let assign_to = &asexp.left;
        let assign_value = self.evaluate(asexp.value.clone(), Rc::clone(&env));

        match assign_to.as_ref() {
            Node::Identifier(ident) => {
                env.borrow_mut()
                    .attempt_assignment(ident.identifier_name.clone(), *assign_value);
            }
            _ => {
                velvet_error!(self, "Cannot assign to left-hand non-identifier");
            }
        }

        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_comparator_expr(
        &mut self,
        comp: &Comparator,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let lhs = self.evaluate(comp.lhs.clone(), Rc::clone(&env));
        let rhs = self.evaluate(comp.rhs.clone(), Rc::clone(&env));
        //println!("COMPARE {:#?}, {:#?}", lhs, rhs);
        let result = lhs
            .compare(&rhs, &comp.op)
            .unwrap_or_else(|err| velvet_error!(self, "Comparator error: {}", err));

        Box::new(RuntimeVal::BoolVal(BoolVal { value: result }))
    }

    fn is_truthy(&mut self, rtv: &RuntimeVal, _: Rc<RefCell<SourceEnv>>) -> bool {
        match rtv {
            RuntimeVal::NullVal(_nv) => false,
            RuntimeVal::NumberVal(n) => n.value != 0,
            RuntimeVal::BoolVal(b) => b.value == true,
            RuntimeVal::StringVal(_) => true,
            RuntimeVal::FunctionVal(_) => true, // because why tf not
            RuntimeVal::ReturnVal(_) => true,
            RuntimeVal::InternalFunctionVal(_) => true, // because why tf not x2??
            RuntimeVal::IteratorVal(_) => true,
            RuntimeVal::ListVal(_) => true,
            RuntimeVal::ObjectVal(_) => true,
        }
    }

    fn evaluate_if_stmt(
        &mut self,
        if_stmt: &IfStmt,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let condition_node = if_stmt.condition.as_ref();

        // TODO: see about using self.is_truthy instead; add case to is_truthy to run itself on Comparators
        // this would allow for things like `if my_true_variable {}` instead of `if my_true_variable == true {}`
        let comparator = match condition_node {
            Node::Comparator(comp) => comp,
            _ => velvet_error!(self, "If condition must be a comparator expression"),
        };

        let condition_result = self.evaluate_comparator_expr(comparator, Rc::clone(&env));

        if self.is_truthy(&*condition_result, Rc::clone(&env)) {
            let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
            for sub_node in &if_stmt.body {
                last_result = self.evaluate(Box::new(sub_node.clone()), Rc::clone(&env));
            }
            return last_result;
        }
        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_while_stmt(
        &mut self,
        while_loop: &WhileStmt,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let condition_node = while_loop.condition.as_ref();

        let comparator = match condition_node {
            Node::Comparator(comp) => comp,
            _ => velvet_error!(self, "While loop condition must be a comparator expression"),
        };

        while {
            let condition_result = self.evaluate_comparator_expr(comparator, Rc::clone(&env));
            self.is_truthy(&*condition_result, Rc::clone(&env))
        } {
            let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));
            for sub_node in &while_loop.body {
                self.evaluate(Box::new(sub_node.clone()), Rc::clone(&sub_environment));
            }
        }

        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_identifier(
        &mut self,
        identifier: &Identifier,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        if &identifier.identifier_name == "__CALL_STACK" {
            self.call_stack.push(CallTarget::Internal(String::from(
                "velvet::internal_identifier_exceptions::call_stack_getter",
            )));
            let mut end_stack_string = format!(
                "\n0 = latest call; {} = first call; % = Rust thread\nvelvet call stack:",
                self.call_stack.len() - 1
            );
            let mut index = 0;
            for call in self.call_stack.iter().rev() {
                match call {
                    CallTarget::Internal(i) => {
                        end_stack_string =
                            end_stack_string + format!("\n {} → % {}", index, i).as_str();
                    }
                    CallTarget::UserDefined(u) => {
                        end_stack_string =
                            end_stack_string + format!("\n {} →   {}", index, u.display).as_str();
                    }
                }
                index = index + 1
            }
            self.call_stack.pop();

            return Box::new(RuntimeVal::StringVal(StringVal {
                value: end_stack_string,
            }));
        }
        let value = {
            let borrowed = env.borrow();
            borrowed
                .fetch(&identifier.identifier_name)
                .map(|v| v.value.clone())
        };

        match value {
            Some(v) => Box::new(v),
            None => {
                velvet_error!(
                    self,
                    "Unresolved identifier \"{}\" does not exist in this scope.",
                    identifier.identifier_name
                );
            }
        }
    }

    fn evaluate_var_declaration(
        &mut self,
        declaration: &VarDeclaration,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        if env
            .borrow()
            .fetch_local(&declaration.var_identifier)
            .is_some()
        {
            velvet_error!(
                self,
                "Attempt to redeclare local binding \"{}\"",
                declaration.var_identifier
            );
        }

        let rhs = self.evaluate(declaration.var_value.clone(), Rc::clone(&env));

        env.borrow_mut().declare_var(
            declaration.var_identifier.clone(),
            *rhs,
            declaration.var_type.clone(),
            declaration.is_mutable,
        );

        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_binary_expr(
        &mut self,
        binop: &BinaryExpr,
        env: Rc<RefCell<SourceEnv>>,
    ) -> Box<RuntimeVal> {
        let left_result = self.evaluate(binop.left.clone(), Rc::clone(&env));
        let right_result = self.evaluate(binop.right.clone(), Rc::clone(&env));

        match (&*left_result, &*right_result) {
            (RuntimeVal::NumberVal(left_num), RuntimeVal::NumberVal(right_num)) => {
                let mut end_result: isize = 0;

                match binop.op.as_str() {
                    "+" => end_result = left_num.value + right_num.value,
                    "-" => end_result = left_num.value - right_num.value,
                    "*" => end_result = left_num.value * right_num.value,
                    "/" => end_result = left_num.value / right_num.value,
                    _ => {}
                }

                return Box::new(RuntimeVal::NumberVal(NumberVal { value: end_result }));
            }
            (RuntimeVal::StringVal(left_str), RuntimeVal::StringVal(right_str)) => {
                let end_result: String;
                match binop.op.as_str() {
                    "+" => end_result = left_str.value.clone() + &right_str.value,
                    _ => {
                        velvet_error!(
                            self,
                            "Binary operator \"{}\" is not allowed on types String and String.",
                            binop.op.as_str()
                        )
                    }
                };

                return Box::new(RuntimeVal::StringVal(StringVal {
                    value: end_result.to_string(),
                }));
            }
            _ => {
                velvet_error!(
                    self,
                    "Binary expression operands must be numbers, received {} and {} with operator {}.",
                    left_result,
                    right_result,
                    binop.op
                );
            }
        }
    }
}
