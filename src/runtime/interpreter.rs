use std::{cell::RefCell, rc::Rc};

use crate::{parser::nodetypes::{AssignmentExpr, BinaryExpr, Comparator, Identifier, Node, VarDeclaration, WhileStmt}, runtime::{source_environment::source_environment::SourceEnv, values::{BoolVal, NullVal, NumberVal, RuntimeVal}}};
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
                let numeric_value: usize = nl.literal_value.parse().unwrap();
                Box::new(RuntimeVal::NumberVal(NumberVal {
                    value: numeric_value
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
            Node::Comparator(comp) => {
                self.evaluate_comparator_expr(comp, env)
            }
            Node::AssignmentExpr(asexp) => {
                self.evaluate_assignment_expr(asexp, env)
            }
            _ => {
                panic!("Evaluation match fault:\nThis node has not been set up for execution yet!\nNode; {:?}", node)
            }
        }
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
        }
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
                let mut end_result: usize = 0;

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
            _ => {
                panic!("Binary expression operands must be numbers");
            }
        }
    }
}