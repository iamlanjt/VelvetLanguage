use core::fmt;
use std::{
    collections::HashMap,
    fs,
    path::{self, Path, PathBuf},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

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
    Infer,
    ExternWildcard, // a wildcard similar to `any` only used in external functions
    Array {
        array_t: Box<T>,
        is_stack_alloca: bool,
        becomes_heap_at: usize,
        element_count: usize,
    },
    Function {
        params: Vec<(String, T)>,
        return_type: Box<T>,
    },
    Unknown, // should only be used in case of inference failure or error propogation
}

impl T {
    pub fn resolve_with_hint(&self, hint: &T) -> T {
        match (self, hint) {
            (
                T::Array {
                    array_t,
                    is_stack_alloca,
                    becomes_heap_at,
                    element_count: 0, // ← inferred
                },
                T::Array {
                    array_t: hint_ty,
                    element_count: hint_len,
                    ..
                },
            ) if array_t == hint_ty => T::Array {
                array_t: array_t.clone(),
                is_stack_alloca: *is_stack_alloca,
                becomes_heap_at: *becomes_heap_at,
                element_count: *hint_len,
            },

            (T::Infer, _) => hint.clone(),

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
            T::Infer => write!(f, "inferred"),
            T::ExternWildcard => write!(f, "::extern-wc::"),
            T::Array {
                array_t,
                is_stack_alloca,
                becomes_heap_at: _,
                element_count,
            } => {
                write!(
                    f,
                    "{}[{} x {}]",
                    if is_stack_alloca == &true {
                        "stack:"
                    } else {
                        "heap:"
                    },
                    array_t,
                    element_count
                )
            }
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

#[derive(Debug, Deserialize)]
pub struct MetaStdlibExternal {
    pub name: String,
    pub desc: String,
    pub submod: Vec<MetaSubmodule>,
}

#[derive(Debug, Deserialize)]
pub struct MetaSubmodule {
    pub name: String,
    pub desc: String,
    pub args: Vec<MetaSubmoduleArg>,
    pub return_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MetaSubmoduleArg {
    pub name: String,
    pub r#type: String,
}

#[derive(Debug)]
pub enum SubmoduleFetchResult {
    Valid {
        meta: MetaStdlibExternal,
        entry: PathBuf,
    },
    Invalid,
}

#[derive(Debug)]
pub enum PathParseErrors {
    MissingParent,
    InvalidSyntax,
    NotFound(PathBuf),
    Io(std::io::Error),
}

pub enum SubmodulePathKind {
    External(PathBuf),
    Local(PathBuf),
}

pub struct SubmodulePath {
    original: String,
    kind: SubmodulePathKind,
}

impl SubmodulePath {
    pub fn parse(pathstr: &str, importer_path: &Path) -> Result<Self, PathParseErrors> {
        if pathstr.starts_with("@/") {
            let relative = &pathstr[2..];

            let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib_comp/externals");
            let resolved = base.join(relative);

            Ok(SubmodulePath {
                original: pathstr.to_string(),
                kind: SubmodulePathKind::External(resolved),
            })
        } else {
            let parent_opt = importer_path.parent();
            if parent_opt.is_none() {
                return Err(PathParseErrors::MissingParent);
            }

            let parent = parent_opt.unwrap();
            let resolved = parent.join(pathstr).with_extension("vel");

            Ok(SubmodulePath {
                original: pathstr.to_string(),
                kind: SubmodulePathKind::Local(resolved),
            })
        }
    }
}

pub fn try_fetch_submodule(
    pathstr: &str,
    importer_path: &Path,
) -> Result<SubmoduleFetchResult, PathParseErrors> {
    let parsed = SubmodulePath::parse(pathstr, importer_path)?;

    match parsed.kind {
        SubmodulePathKind::External(dir) => {
            let meta_path = dir.join("meta.toml");
            let main = dir.join("main.rs");

            let meta_exists = fs::metadata(&meta_path).is_ok();
            let main_exists = fs::metadata(&main).is_ok();

            if meta_exists && main_exists {
                let contents = match fs::read_to_string(&meta_path) {
                    Ok(s) => s,
                    Err(e) => return Err(PathParseErrors::Io(e)),
                };

                let parsed_meta: MetaStdlibExternal = match toml::from_str(&contents) {
                    Ok(meta) => meta,
                    Err(_) => return Ok(SubmoduleFetchResult::Invalid),
                };

                Ok(SubmoduleFetchResult::Valid {
                    meta: parsed_meta,
                    entry: main,
                })
            } else {
                Ok(SubmoduleFetchResult::Invalid)
            }
        }

        SubmodulePathKind::Local(_) => {
            todo!()
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
    pub externals_used: Vec<String>,
    pub path_at: String,
}

impl TypeChecker {
    pub fn new(externals_used: &Vec<String>, path_at: String) -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
            type_table: HashMap::new(),
            externals_used: externals_used.clone(),
            path_at,
        }
    }

    fn type_error(&mut self, message: &str) {
        self.errors.push(TypeError {
            message: message.to_string(),
        });
    }

    pub fn identifier_to_type(&self, ident: &str) -> T {
        if let Some(base_type) = ident.strip_suffix("[]") {
            let inner = self.identifier_to_type(base_type);
            return T::Array {
                array_t: Box::new(inner),
                is_stack_alloca: true,
                becomes_heap_at: 0,
                element_count: 0,
            };
        }
        match ident.to_lowercase().as_str() {
            "i8" => T::Integer8,
            "i16" => T::Integer16,
            "i32" | "number" => T::Integer32,
            "i64" => T::Integer64,
            "i128" => T::Integer128,
            "bool" => T::Boolean,
            "string" => T::String,
            "inferred" => T::Infer,
            "wc" => T::ExternWildcard,
            _ => panic!("`{}` is not a valid type", ident),
        }
    }

    // Load all external values into the scope
    pub fn load_externs(&mut self) {
        for extern_path in &self.externals_used {
            let parsed_path =
                try_fetch_submodule(extern_path.as_str(), Path::new(&self.path_at)).unwrap();

            match parsed_path {
                SubmoduleFetchResult::Valid { meta, entry } => {
                    for sub_module in &meta.submod {
                        // add this function to scope
                        let v = String::from("inferred");
                        let return_type_str = match &sub_module.return_type {
                            Some(rt) => rt,
                            None => &v,
                        }
                        .clone();
                        let return_type = self.identifier_to_type(&return_type_str);
                        let mut param_types: Vec<(String, T)> = Vec::new();
                        for arg in &sub_module.args {
                            param_types
                                .push((arg.name.clone(), self.identifier_to_type(&arg.r#type)))
                        }

                        self.scopes.last_mut().unwrap().insert(
                            sub_module.name.clone(),
                            T::Function {
                                params: param_types,
                                return_type: Box::new(return_type),
                            },
                        );
                    }
                }
                SubmoduleFetchResult::Invalid => {
                    panic!("Failed to parse external path {}", extern_path);
                }
            }
        }
        /*
        let vals: Vec<(&str, T)> = vec![(
            "print",
            T::Function {
                params: vec![("_".to_string(), T::ExternWildcard)],
                return_type: Box::new(T::Void),
            },
        )];

        for v in &vals {
            self.scopes
                .last_mut()
                .unwrap()
                .insert(v.0.to_string(), v.1.clone());
        }
        */
    }

    fn types_match(&self, expected: &T, actual: &T) -> bool {
        match (expected, actual) {
            (
                T::Array {
                    array_t: e_ty,
                    is_stack_alloca: e_stack,
                    becomes_heap_at: e_heap,
                    element_count: e_len,
                },
                T::Array {
                    array_t: a_ty,
                    is_stack_alloca: a_stack,
                    becomes_heap_at: a_heap,
                    element_count: a_len,
                },
            ) => {
                e_ty == a_ty
                    && e_stack == a_stack
                    && e_heap == a_heap
                    && (*e_len == 0 || *e_len == *a_len)
            }

            _ => expected == actual,
        }
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

            (Integer32, Infer) => true,
            (Infer, Integer32) => false,

            (_, ExternWildcard) => true,

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

    fn display_type_hint(&self, expected: Option<&T>) -> String {
        match expected {
            Some(ty) => format!("[hint: {}]", ty),
            None => "".to_string(),
        }
    }

    pub fn check_expr(&mut self, node: &Node, expected: Option<&T>, vb: bool, ts: usize) -> T {
        if vb {
            println!(
                "{}├─{} {}...",
                "│  ".repeat(ts * 2),
                self.display_type_hint(expected),
                node
            );
        }
        let ty = match node {
            Node::BoolLiteral(_) => T::Boolean,
            Node::StringLiteral(_) => T::String,
            Node::ListLiteral(l) => {
                let mut end_type_array = T::Array {
                    array_t: Box::new(T::Unknown),
                    is_stack_alloca: true,
                    becomes_heap_at: 0,
                    element_count: l.props.len(),
                };
                let mut inferred_elem_ty: Option<T> = None;

                for (index, expr) in l.props.iter().enumerate() {
                    let expected = inferred_elem_ty.as_ref();
                    let actual = self.check_expr(expr, expected, vb, ts + 1);

                    if let Some(t_expected) = &inferred_elem_ty {
                        if !self.can_coerce(&actual, t_expected) {
                            self.type_error(&format!(
                                "Array element types must be uniform; expected {}, got {} at index {}",
                                t_expected, actual, index
                            ));
                        }
                    } else {
                        inferred_elem_ty = Some(actual);
                    }
                }

                if let Some(t) = inferred_elem_ty {
                    end_type_array = T::Array {
                        array_t: Box::new(t),
                        is_stack_alloca: true,
                        becomes_heap_at: 0,
                        element_count: l.props.len(),
                    };
                }

                end_type_array
            }
            Node::MemberExpr(m) => {
                let parent_type = self.check_expr(&m.object, None, vb, ts + 1);

                match &parent_type {
                    T::Array {
                        array_t,
                        is_stack_alloca,
                        becomes_heap_at,
                        element_count,
                    } => {
                        let index_ty =
                            self.check_expr(&m.property, Some(&T::Integer32), vb, ts + 1);
                        if index_ty != T::Integer32 {
                            self.type_error(&format!("Array index must be i32, got {}", index_ty));
                        }
                        if let Node::NumericLiteral(n) = m.property.as_ref() {
                            if *is_stack_alloca && *element_count > 0 {
                                let indexing_to = n.literal_value.parse::<usize>().unwrap();
                                if element_count < &(indexing_to + 1) {
                                    self.type_error(&format!("Stack-alloc array has {} elements; attempt to access element at index {} (out-of-bounds)", element_count, indexing_to));
                                }
                            }
                        }

                        *array_t.clone()
                    }
                    _ => T::Unknown,
                }
            }
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
                let callee_ty = self.check_expr(&cexpr.caller, None, vb, ts + 1);
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
                            let arg_ty =
                                self.check_expr(arg_expr, Some(&expected_ty.1), vb, ts + 1);

                            if !self.types_match(&expected_ty.1, &arg_ty)
                                && !self.can_coerce(&arg_ty, &expected_ty.1)
                            {
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
                        for arg_expr in cexpr.args.iter() {
                            // just for typechecking for now
                            self.check_expr(arg_expr, None, vb, ts + 1);
                        }
                        T::Infer
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
                        let val_ty =
                            self.check_expr(&ret.return_statement, Some(&return_type), vb, ts + 1);
                        if val_ty != return_type {
                            self.type_error(&format!(
                                "Function `{}` returns `{}`, expected `{}`",
                                fdef.name, val_ty, return_type
                            ));
                        }
                    } else {
                        self.check_expr(stmt, None, vb, ts + 1);
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
                let expr_ty = self.check_expr(&vdecl.var_value, Some(&expected_ty), vb, ts + 1);

                let resolved_ty = expected_ty.resolve_with_hint(&expr_ty);

                if !self.types_match(&resolved_ty, &expr_ty) {
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
                } else {
                    self.scopes
                        .last_mut()
                        .unwrap()
                        .insert(vdecl.var_identifier.clone(), resolved_ty.clone());
                    resolved_ty
                }
            }
            Node::TypeCast(tc) => {
                let left_val = self.check_expr(&tc.left, None, vb, ts + 1);
                let to_type = tc.target_type.clone();

                if !self.can_coerce(&left_val, &to_type) {
                    self.type_error(&format!("Cannot cast from {} to {}", left_val, to_type));
                }

                to_type
            }
            Node::BinaryExpr(bin) => {
                let lhs_ty = self.check_expr(&bin.left, None, vb, ts + 1);
                let rhs_ty = self.check_expr(&bin.right, Some(&lhs_ty), vb, ts + 1);

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
                let left_ty = self.check_expr(&a.left, None, vb, ts + 1);
                let val_ty = self.check_expr(&a.value, Some(&left_ty), vb, ts + 1);

                self.enforce_equality(&val_ty, &left_ty)
            }
            Node::Identifier(ident) => {
                let var_t = self.lookup_variable_type(&ident.identifier_name);
                if let Some(t) = var_t { t } else { T::Unknown }
            }
            Node::Block(b) => {
                let mut last_val = T::Unknown;
                for sub_node in &b.body {
                    last_val = self.check_expr(sub_node, None, vb, ts + 1)
                }
                last_val
            }
            Node::WhileStmt(ws) => {
                self.check_expr(&ws.condition, None, vb, ts + 1);
                let mut last_val = T::Unknown;
                for sub_node in &ws.body {
                    last_val = self.check_expr(sub_node, None, vb, ts + 1)
                }
                last_val
            }
            Node::Comparator(comp) => {
                let l = self.check_expr(&comp.lhs, None, vb, ts + 1);
                self.check_expr(&comp.rhs, Some(&l), vb, ts + 1);
                l
            }
            Node::IfStmt(i) => {
                self.check_expr(&i.condition, None, vb, ts + 1);
                for node in &i.body {
                    self.check_expr(node, None, vb, ts + 1);
                }
                T::Unknown
            }
            Node::Return(r) => self.check_expr(&r.return_statement, None, vb, ts + 1),
            Node::MatchExpr(mexpr) => {
                let target = self.check_expr(&mexpr.target, None, vb, ts + 1);
                let base_type =
                    self.check_expr(&mexpr.arms.first().unwrap().0, expected, vb, ts + 2);
                let mut last_right = T::Unknown;
                for arm in &mexpr.arms {
                    self.check_expr(&arm.0, Some(&target), vb, ts + 2);
                    last_right = self.check_expr(&arm.1, None, vb, ts + 2);
                }
                last_right
            }
            Node::NullishCoalescing(n) => self.check_expr(&n.left, None, vb, ts + 1),
            Node::NoOpNode(_) => T::Unknown,
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

        if vb {
            println!("{}└ typecheck resolution to `{}`", "│  ".repeat(ts * 2), ty);
        }

        self.type_table.insert(node_id.unwrap(), ty.clone());

        ty
    }
}
