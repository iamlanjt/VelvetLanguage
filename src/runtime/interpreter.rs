use crate::{parser::nodetypes::{BinaryExpr, Identifier, Node, VarDeclaration}, runtime::{source_environment::source_environment::SourceEnv, values::{NullVal, NumberVal, RuntimeVal}}};
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

    pub fn evaluate_body(&mut self, env: &mut SourceEnv) -> Box<RuntimeVal> {
        let ast = self.ast.clone();
        let mut last_result: Box<RuntimeVal> = Box::new(RuntimeVal::NullVal(NullVal {}));
        for node in ast {
            last_result = self.evaluate(node, env);
        }
        last_result
    }

    pub fn evaluate(&mut self, node: Box<Node>, env: &mut SourceEnv) -> Box<RuntimeVal> {
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
            _ => {
                panic!("Evaluation match fault:\nThis node has not been set up for execution yet!\nNode; {:?}", node)
            }
        }
    }

    fn evaluate_identifier(&mut self, identifier: &Identifier, env: &mut SourceEnv) -> Box<RuntimeVal> {
        if let Some(var) = env.fetch(&identifier.identifier_name) {
            return Box::new(var.value.clone())
        }
        panic!("Unresolved identifier \"{}\" does not exist in this scope.", identifier.identifier_name);
    }

    fn evaluate_var_declaration(&mut self, declaration: &VarDeclaration, env: &mut SourceEnv) -> Box<RuntimeVal> {
        if let Some(var) = env.fetch(&declaration.var_identifier) {
            panic!("Attempt to redeclare local binding \"{}\"", declaration.var_identifier);
        }

        let rhs = self.evaluate(declaration.var_value.clone(), env);

        env.declare_var(declaration.var_identifier.clone(), *rhs, declaration.var_type.clone(), declaration.is_mutable);
        Box::new(RuntimeVal::NullVal(NullVal {}))
    }

    fn evaluate_binary_expr(&mut self, binop: &BinaryExpr, env: &mut SourceEnv) -> Box<RuntimeVal> {
        let left_result = self.evaluate(binop.left.clone(), env);
        let right_result = self.evaluate(binop.right.clone(), env);

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