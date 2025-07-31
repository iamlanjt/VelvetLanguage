use core::fmt;
use std::collections::HashMap;

use colored::Colorize;

use crate::parser::nodetypes::{CallExpr, Node};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum T {
    Integer8,
    Integer16,
    Integer32,
    Integer64,
    Integer128,
    Boolean,
    Void,
    String,
    Any,
    Function {
        params: Vec<(String, T)>,
        return_type: Box<T>,
    },
    Unknown, // should only be used in case of inference failure or error propogation
}

impl T {
    pub fn resolve_with_hint(&self, hint: &T) -> T {
        match self {
            T::Any => hint.clone(),
            _ => self.clone(),
        }
    }
}

impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            T::Integer8 => write!(f, "i8"),
            T::Integer16 => write!(f, "i16"),
            T::Integer32 => write!(f, "i32"),
            T::Integer64 => write!(f, "i64"),
            T::Integer128 => write!(f, "i128"),
            T::Boolean => write!(f, "bool"),
            T::Void => write!(f, "void"),
            T::String => write!(f, "string"),
            T::Any => write!(f, "any"),
            T::Function {
                params,
                return_type,
            } => {
                let param_strs: Vec<String> = params.iter().map(|t| t.1.to_string()).collect();
                write!(f, "fn({}) -> {}", param_strs.join(", "), return_type)
            }
            T::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
}

pub struct TypeChecker {
    pub scopes: Vec<HashMap<String, T>>,
    pub errors: Vec<TypeError>,
    pub type_table: HashMap<usize, T>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
            type_table: HashMap::new(),
        }
    }

    fn type_error(&mut self, message: &str) {
        self.errors.push(TypeError {
            message: message.to_string(),
        });
    }

    fn can_coerce(&self, from: &T, to: &T) -> bool {
        use T::*;
        match (from, to) {
            (Integer8, Integer16)
            | (Integer8, Integer32)
            | (Integer8, Integer64)
            | (Integer8, Integer128)
            | (Integer16, Integer32)
            | (Integer16, Integer64)
            | (Integer16, Integer128)
            | (Integer32, Integer64)
            | (Integer32, Integer128)
            | (Integer64, Integer128) => true,

            (Integer32, Any) => true,
            (Any, Integer32) => false,

            (a, b) => a == b,
        }
    }

    pub fn enforce_equality(&mut self, given: &T, expected: &T) -> T {
        if (given == expected) || self.can_coerce(given, expected) {
            expected.clone()
        } else if self.can_coerce(expected, given) {
            self.type_error(&format!("Cannot coerce {} to {}", given, expected));
            T::Unknown
        } else {
            self.type_error(&format!("Expected {}, got {}", expected, given));
            T::Unknown
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop().expect("No scope to pop off");
    }

    pub fn lookup_variable_type(&self, name: &str) -> Option<T> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub fn generate_fn_signature_string(&self, cexpr: &CallExpr, f: &T) -> String {
        match f {
            T::Function { params, .. } => {
                format!(
                    "->       signature: {}({})",
                    cexpr.caller.to_string().cyan(),
                    params
                        .iter()
                        .map(|p| format!("{} as {}", p.0.magenta(), p.1.to_string().blue()))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            _ => String::from("<?>(...)"),
        }
    }

    pub fn check_expr(&mut self, node: &Node, expected: Option<&T>) -> T {
        let ty = match node {
            Node::BoolLiteral(_) => T::Boolean,
            Node::StringLiteral(_) => T::String,
            Node::NumericLiteral(n) => match expected {
                Some(ty)
                    if matches!(
                        ty,
                        T::Integer8 | T::Integer16 | T::Integer32 | T::Integer64 | T::Integer128
                    ) =>
                {
                    ty.clone()
                }
                _ => {
                    let val = n.literal_value.parse::<i128>().unwrap();
                    if val > i64::MAX.into() {
                        T::Integer128
                    } else if val > i32::MAX.into() {
                        T::Integer64
                    } else if val > i16::MAX.into() {
                        T::Integer32
                    } else if val > i8::MAX.into() {
                        T::Integer16
                    } else {
                        T::Integer32
                    }
                }
            },
            Node::CallExpr(cexpr) => {
                let callee_ty = self.check_expr(&cexpr.caller, None);
                let f_clone = callee_ty.clone();

                match callee_ty {
                    T::Function {
                        params,
                        return_type,
                    } => {
                        if params.len() != cexpr.args.len() {
                            self.type_error(&format!(
                                "call to fn `{}` expected {} arguments, received {}\n{}",
                                cexpr.caller,
                                params.len(),
                                cexpr.args.len(),
                                self.generate_fn_signature_string(cexpr, &f_clone)
                            ));
                            return T::Unknown;
                        }

                        for (index, (arg_expr, expected_ty)) in
                            cexpr.args.iter().zip(params.iter()).enumerate()
                        {
                            let arg_ty = self.check_expr(arg_expr, Some(&expected_ty.1));
                            if arg_ty != expected_ty.1 {
                                self.type_error(&format!(
                                    "call to fn `{}` arg #{} type mismatch: expected `{}`, got `{}`\n{}",
                                    cexpr.caller,
                                    index + 1,
                                    &expected_ty.1,
                                    arg_ty,
                                    self.generate_fn_signature_string(cexpr, &f_clone)
                                ));
                            }
                        }

                        *return_type.clone()
                    }

                    _ => {
                        /*
                        self.type_error(&format!(
                            "Attempted to call a non-function value of type `{}`",
                            callee_ty
                        ));
                        */
                        T::Unknown
                    }
                }
            }
            Node::FunctionDefinition(fdef) => {
                let return_type = fdef.return_type.clone();

                self.enter_scope();

                for param in &fdef.params {
                    let param_ty = param.1.clone();
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(param.0.clone(), param_ty);
                }

                let mut saw_return = false;
                for stmt in &*fdef.body {
                    if let Node::Return(ret) = stmt {
                        saw_return = true;
                        let val_ty = self.check_expr(&ret.return_statement, Some(&return_type));
                        if val_ty != return_type {
                            self.type_error(&format!(
                                "Function `{}` returns `{}`, expected `{}`",
                                fdef.name, val_ty, return_type
                            ));
                        }
                    } else {
                        self.check_expr(stmt, None);
                    }
                }

                if return_type != T::Unknown && !saw_return {
                    self.type_error(&format!(
                        "Function `{}` missing return of type `{}`",
                        fdef.name, return_type
                    ));
                }

                self.exit_scope();

                let f = T::Function {
                    params: fdef
                        .params
                        .iter()
                        .map(|p| (p.0.clone(), p.1.clone()))
                        .collect(),
                    return_type: Box::new(return_type),
                };
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(fdef.name.clone(), f.clone());

                f
            }
            Node::VarDeclaration(vdecl) => {
                let expected_ty = vdecl.var_type.clone();
                let expr_ty = self.check_expr(&vdecl.var_value, Some(&expected_ty));

                let resolved_ty = expected_ty.resolve_with_hint(&expr_ty);

                if !self.can_coerce(&expr_ty, &resolved_ty) {
                    self.type_error(&format!(
                        "Cannot assign value of type `{}` to variable of type `{}`",
                        expr_ty, resolved_ty
                    ));
                    T::Unknown
                } else {
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(vdecl.var_identifier.clone(), resolved_ty.clone());
                    resolved_ty
                }
            }
            Node::TypeCast(tc) => {
                let left_val = self.check_expr(&tc.left, None);
                let to_type = tc.target_type.clone();

                if !self.can_coerce(&left_val, &to_type) {
                    self.type_error(&format!("Cannot cast from {} to {}", left_val, to_type));
                }

                to_type
            }
            Node::BinaryExpr(bin) => {
                let lhs_ty = self.check_expr(&bin.left, None);
                let rhs_ty = self.check_expr(&bin.right, Some(&lhs_ty));

                if lhs_ty != rhs_ty {
                    self.type_error(&format!(
                        "Cannot use `{}` operation on {} and {}.",
                        bin.op, lhs_ty, rhs_ty
                    ));
                    T::Unknown
                } else {
                    lhs_ty
                }
            }
            Node::AssignmentExpr(a) => {
                let left_ty = self.check_expr(&a.left, None);
                let val_ty = self.check_expr(&a.value, Some(&left_ty));

                self.enforce_equality(&val_ty, &left_ty)
            }
            Node::Identifier(ident) => {
                let var_t = self.lookup_variable_type(&ident.identifier_name);
                if let Some(t) = var_t { t } else { T::Unknown }
            }
            _ => unimplemented!("{:?}", node),
        };
        let node_id = match node {
            Node::AssignmentExpr(v) => v.id,
            Node::BinaryExpr(v) => v.id,
            Node::Block(v) => v.id,
            Node::BoolLiteral(v) => v.id,
            Node::CallExpr(v) => v.id,
            Node::Comparator(v) => v.id,
            Node::Eof(v) => v.id,
            Node::FunctionDefinition(v) => v.id,
            Node::Identifier(v) => v.id,
            Node::IfStmt(v) => v.id,
            Node::InterpreterBlock(v) => v.id,
            Node::Iterator(v) => v.id,
            Node::ListLiteral(v) => v.id,
            Node::MatchExpr(v) => v.id,
            Node::MemberExpr(v) => v.id,
            Node::NoOpNode(v) => v.id,
            Node::NullLiteral(v) => v.id,
            Node::NullishCoalescing(v) => v.id,
            Node::NumericLiteral(v) => v.id,
            Node::ObjectLiteral(v) => v.id,
            Node::OptionalArg(v) => v.id,
            Node::Return(v) => v.id,
            Node::StringLiteral(v) => v.id,
            Node::TypeCast(v) => v.id,
            Node::VarDeclaration(v) => v.id,
            Node::WhileStmt(v) => v.id,
        };

        self.type_table.insert(node_id.unwrap(), ty.clone());

        ty
    }
}
