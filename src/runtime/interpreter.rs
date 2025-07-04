use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{parser::nodetypes::{AssignmentExpr, BinaryExpr, CallExpr, Comparator, FunctionDefinition, Identifier, IfStmt, Iterator, Node, ObjectLiteral, Return, VarDeclaration, WhileStmt}, runtime::{source_environment::source_environment::SourceEnv, values::{BoolVal, FunctionVal, IteratorVal, ListVal, NullVal, NumberVal, ObjectVal, ReturnVal, RuntimeVal, StringVal}}};
use crate::{runtime::source_environment::*};

pub struct Interpreter {
    ast: Vec<Box<Node>>,
    pointer: usize,
}

impl Interpreter {
    pub fn new(ast: Vec<Box<Node>>) -> Self {
        Self {
            ast,
            pointer: 0
        }
    }

    pub fn evaluate_body(&mut self, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let ast = self.ast.clone();
        let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
        for node in ast {
            last_result = self.evaluate(node, Rc::clone(&env));
        }
        last_result
    }

    pub fn evaluate(&mut self, node: Box<Node>, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        match node.as_ref() {
            Node::NumericLiteral(nl) => {
                let numeric_value: isize = nl.literal_value.parse().unwrap();
                Box::new(RuntimeVal::NumberVal(NumberVal {
                    value: numeric_value
                }))
            }
            Node::StringLiteral(slit) => {
                Box::new(RuntimeVal::StringVal(StringVal {
                    value: slit.literal_value.clone()
                }))
            }
            Node::Return(ret) => {
                Box::new(RuntimeVal::ReturnVal(ReturnVal {
                    value: ret.return_statement.clone()
                }))
            }
            Node::ListLiteral(ll) => {
                let mut results: Vec<RuntimeVal> = Vec::new();
                for inner_node in &ll.props {
                    results.push(*self.evaluate(inner_node.clone(), Rc::clone(&env)));
                }
                Box::new(RuntimeVal::ListVal(ListVal {
                    values: results
                }))
            }
            Node::BinaryExpr(binop) => {
                self.evaluate_binary_expr(binop, env)
            }
            Node::VarDeclaration(decl) => {
                self.evaluate_var_declaration(decl, env)
            }
            Node::Identifier(ident) => {
                self.evaluate_identifier(ident, env)
            }
            Node::WhileStmt(while_loop) => {
                self.evaluate_while_stmt(while_loop, env)
            }
            Node::IfStmt(if_stmt) => {
                self.evaluate_if_stmt(if_stmt, env)
            }
            Node::Comparator(comp) => {
                self.evaluate_comparator_expr(comp, env)
            }
            Node::AssignmentExpr(asexp) => {
                self.evaluate_assignment_expr(asexp, env)
            }
            Node::FunctionDefinition(fdef) => {
                self.evaluate_function_definition(fdef, env)
            }
            Node::CallExpr(cexpr) => {
                self.evaluate_call_expr(cexpr, env)
            }
            Node::Iterator(it) => {
                self.evaluate_iterator_expr(it, env)
            }
            Node::ObjectLiteral(ol) => {
                self.evaluate_object_literal(ol, env)
            }
            _ => {
                panic!("Evaluation match fault:\nThis node has not been set up for execution yet!\n\n{:#?}\n\n", node)
            }
        }
    }

    fn evaluate_object_literal(&mut self, ov: &ObjectLiteral, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let mut runtime_props: HashMap<String, RuntimeVal> = HashMap::new();
        for inner_prop in &ov.props {
            runtime_props.insert(inner_prop.0.to_string(), *self.evaluate(inner_prop.1.clone(), Rc::clone(&env)));
        }
        Box::new(RuntimeVal::ObjectVal(ObjectVal {
            values: runtime_props
        }))
    }

    fn evaluate_iterator_expr(&mut self, it: &Iterator, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let var_name = &it.left.literal_value;
        let loop_through = self.evaluate(it.right.clone(), Rc::clone(&env));

        match loop_through.as_ref() {
            RuntimeVal::ListVal(lv) => {
                let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {  }));
                for v in &lv.values {
                    let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));
                    sub_environment.borrow_mut().declare_var(
                        it.left.literal_value.clone(),
                        v.clone(),
                        "u".to_owned(),
                        false
                    );
                    for sub_expr in &it.body {
                        last_result = self.evaluate(sub_expr.clone(), Rc::clone(&sub_environment));
                        match last_result.as_ref() {
                            RuntimeVal::ReturnVal(rt) => {
                                return self.evaluate(rt.clone().value, Rc::clone(&sub_environment));
                            }
                            _ => {
                                
                            }
                        }
                    }
                }
                last_result
            }
            _ => {
                panic!("Cannot loop through non-list type");
            }
        }
    }

    fn evaluate_call_expr(&mut self, cexpr: &CallExpr, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let caller = self.evaluate(cexpr.caller.clone(), Rc::clone(&env));
        match caller.as_ref() {
            RuntimeVal::FunctionVal(r#fn) => {
                // create sub-environment
                let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));

                // set all the args for the sub environment that were supplied in the CallExpr
                let mut i = 0;
                for arg in &cexpr.args {
                    let evaluated = self.evaluate(arg.clone(), Rc::clone(&env));
                    sub_environment.borrow_mut().declare_var(r#fn.params[i].clone(), *evaluated, "inferred_any".to_string(), false);
                    i = i + 1;
                }

                let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {  }));
                for sub_expr in r#fn.execution_body.as_ref() {
                    last_result = self.evaluate(sub_expr.clone(), Rc::clone(&sub_environment));
                    match last_result.as_ref() {
                        RuntimeVal::ReturnVal(rt) => {
                            return self.evaluate(rt.clone().value, Rc::clone(&sub_environment));
                        }
                        _ => {
                            
                        }
                    }
                }
                last_result
            }
            RuntimeVal::InternalFunctionVal(r#fn) => {
                let mut new_args: Vec<RuntimeVal> = Vec::new();
                for arg in &cexpr.args {
                    new_args.push(*self.evaluate(Box::new(*arg.clone()), Rc::clone(&env)));
                }
                let internal_result = (r#fn.internal_callback)(new_args);
                return Box::new(internal_result);
            }
            _ => {
                panic!("Cannot call type \"{:#?}\"", caller)
            }
        }
    }

    fn evaluate_function_definition(&mut self, def: &FunctionDefinition, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let this_function_val = RuntimeVal::FunctionVal(FunctionVal {
            params: def.params.clone(),
            fn_name: def.name.clone(),
            execution_body: Rc::clone(&def.body), // Reference counter clone because deep cloning nodes is not cheap
            is_internal: false
        });

        // Add to env
        env.borrow_mut().declare_var(def.name.clone(), this_function_val, String::from("function"), false);

        // Definitions do not return anything; it is automatically added by name to the env
        Box::new(RuntimeVal::NullVal(NullVal {  }))
    }

    fn evaluate_assignment_expr(&mut self, asexp: &AssignmentExpr, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let assign_to = asexp.left.clone();
        let assign_value = self.evaluate(asexp.value.clone(), Rc::clone(&env));

        match assign_to.as_ref() {
            Node::Identifier(ident) => {
                env.borrow_mut().attempt_assignment(ident.identifier_name.clone(), *assign_value);
            }
            _ => {
                panic!("Cannot assign to left-hand non-identifier");
            }
        }

        Box::new(RuntimeVal::NullVal(NullVal {}))
    }


    fn evaluate_comparator_expr(&mut self, comp: &Comparator, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let lhs = self.evaluate(comp.lhs.clone(), Rc::clone(&env));
        let rhs = self.evaluate(comp.rhs.clone(), Rc::clone(&env));
        //println!("COMPARE {:#?}, {:#?}", lhs, rhs);
        let result = lhs
            .compare(&rhs, &comp.op)
            .unwrap_or_else(|err| panic!("Comparator error: {}", err));

        Box::new(RuntimeVal::BoolVal(BoolVal { value: result }))
    }

    fn is_truthy(&mut self, rtv: &RuntimeVal, env: Rc<RefCell<SourceEnv>>) -> bool {
        match rtv {
            RuntimeVal::NullVal(_nv) => false,
            RuntimeVal::NumberVal(n) => n.value != 0,
            RuntimeVal::BoolVal(b) => b.value == true,
            RuntimeVal::StringVal(s) => true,
            RuntimeVal::FunctionVal(f) => true, // because why tf not
            RuntimeVal::ReturnVal(rt) => true,
            RuntimeVal::InternalFunctionVal(rt) => true, // because why tf not x2??
            RuntimeVal::IteratorVal(it) => true,
            RuntimeVal::ListVal(lv) => true,
            RuntimeVal::ObjectVal(ov) => true
        }
    }

    fn evaluate_if_stmt(&mut self, if_stmt: &IfStmt, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let condition_node = if_stmt.condition.as_ref();

        // TODO: see about using self.is_truthy instead; add case to is_truthy to run itself on Comparators
        // this would allow for things like `if my_true_variable {}` instead of `if my_true_variable == true {}`
        let comparator = match condition_node {
            Node::Comparator(comp) => comp,
            _ => panic!("If condition must be a comparator expression")
        };

        let condition_result = self.evaluate_comparator_expr(comparator, Rc::clone(&env));

        if self.is_truthy(&*&condition_result, Rc::clone(&env)) {
            let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {  }));
            for sub_node in &if_stmt.body {
                last_result = self.evaluate(sub_node.clone(), Rc::clone(&env));
            }
            return last_result
        }
        Box::new(RuntimeVal::NullVal(NullVal {  }))
    }

    fn evaluate_while_stmt(&mut self, while_loop: &WhileStmt, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let condition_node = while_loop.condition.as_ref();

        let comparator = match condition_node {
            Node::Comparator(comp) => comp,
            _ => panic!("While loop condition must be a comparator expression"),
        };

        let sub_environment = Rc::new(RefCell::new(SourceEnv::new(Some(Rc::clone(&env)))));

        while {
            let condition_result = self.evaluate_comparator_expr(comparator, Rc::clone(&env));
            self.is_truthy(&*condition_result, Rc::clone(&env))
        } {
            for sub_node in &while_loop.body {
                self.evaluate(sub_node.clone(), Rc::clone(&sub_environment));
            }
        }

        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_identifier(&mut self, identifier: &Identifier, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let value = {
            let borrowed = env.borrow();
            borrowed.fetch(&identifier.identifier_name)
                .map(|v| v.value.clone())
        };

        match value {
            Some(v) => Box::new(v),
            None => panic!("Unresolved identifier \"{}\" does not exist in this scope.", identifier.identifier_name),
        }
    }

    fn evaluate_var_declaration(&mut self, declaration: &VarDeclaration, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        if env.borrow().fetch(&declaration.var_identifier).is_some() {
            panic!("Attempt to redeclare local binding \"{}\"", declaration.var_identifier);
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



    fn evaluate_binary_expr(&mut self, binop: &BinaryExpr, env: Rc<RefCell<SourceEnv>>) -> Box<RuntimeVal> {
        let left_result = self.evaluate(binop.left.clone(), Rc::clone(&env));
        let right_result = self.evaluate(binop.right.clone(), Rc::clone(&env));

        match (&*left_result, &*right_result) {
            (RuntimeVal::NumberVal(left_num), RuntimeVal::NumberVal(right_num)) => {
                let mut end_result: isize = 0;

                match binop.op.as_str() {
                    "+" => {
                        end_result = left_num.value + right_num.value
                    }
                    "-" => {
                        end_result = left_num.value - right_num.value
                    }
                    "*" => {
                        end_result = left_num.value * right_num.value
                    }
                    "/" => {
                        end_result = left_num.value / right_num.value
                    }
                    _ => {
                        
                    }
                }

                return Box::new(RuntimeVal::NumberVal(NumberVal { value: end_result  }))
            },
            (RuntimeVal::StringVal(left_str), RuntimeVal::StringVal(right_str)) => {
                let mut end_result = "".to_string();
                match binop.op.as_str() {
                    "+" => {
                        end_result = left_str.value.clone() + &right_str.value
                    }
                    _ => {
                        panic!("Binary operator \"{}\" is not allowed on types String and String.", binop.op.as_str())
                    }
                };
                
                return Box::new(RuntimeVal::StringVal(StringVal { value: end_result.to_string() }));
            }
            _ => {
                panic!("Binary expression operands must be numbers");
            }
        }
    }
}