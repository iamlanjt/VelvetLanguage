use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{parser::nodetypes::Node, runtime::source_environment::source_environment::SourceEnv};

pub type NativeMethod = fn(&RuntimeVal, Vec<RuntimeVal>) -> RuntimeVal;

pub trait HasMethods {
    fn get_methods(&self) -> HashMap<String, NativeMethod>;
}

#[derive(Clone)]
pub enum RuntimeVal {
    NumberVal(NumberVal),
    NullVal(NullVal),
    FunctionVal(FunctionVal),
    InternalFunctionVal(InternalFunctionVal),
    BoolVal(BoolVal),
    StringVal(StringVal),
    ReturnVal(ReturnVal),
    IteratorVal(IteratorVal),
    ListVal(ListVal),
    ObjectVal(ObjectVal),
}

impl RuntimeVal {
    pub fn is_null(&self) -> bool {
        matches!(self, RuntimeVal::NullVal(_))
    }

    pub fn compare(&self, other: &RuntimeVal, op: &str) -> Result<bool, String> {
        match (self, other) {
            (RuntimeVal::NumberVal(l), RuntimeVal::NumberVal(r)) => {
                let result = match op {
                    "==" => l.value == r.value,
                    "!=" => l.value != r.value,
                    "<" => l.value < r.value,
                    "<=" => l.value <= r.value,
                    ">" => l.value > r.value,
                    ">=" => l.value >= r.value,
                    _ => return Err(format!("Unknown operator: {}", op)),
                };
                Ok(result)
            }
            (RuntimeVal::BoolVal(l), RuntimeVal::BoolVal(r)) => {
                let result = match op {
                    "==" => l.value == r.value,
                    "!=" => l.value != r.value,
                    _ => return Err(format!("Operator '{}' not supported for booleans", op)),
                };
                Ok(result)
            }
            (RuntimeVal::StringVal(s), RuntimeVal::StringVal(s2)) => {
                let result = match op {
                    "==" => &s.value == &s2.value,
                    _ => return Err(format!("Operator '{}' not supported for strings", op)),
                };
                Ok(result)
            }
            (RuntimeVal::NullVal(_), RuntimeVal::BoolVal(_)) => Ok(false),
            _ => Err(format!(
                "Unsupported comparison between {:?} and {:?}",
                self, other
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberVal {
    pub value: isize,
}

#[derive(Debug, Clone)]
pub struct NullVal {}

#[derive(Debug, Clone)]
pub struct BoolVal {
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionVal {
    pub params: Vec<String>,
    pub fn_name: String,
    pub execution_body: Rc<Vec<Node>>,
    pub is_internal: bool,
}

#[derive(Debug, Clone)]
pub struct IteratorVal {
    pub to_name: String,
    pub target: Box<RuntimeVal>,
}

#[derive(Clone)]
pub struct InternalFunctionVal {
    pub fn_name: String,
    pub internal_callback: Rc<dyn Fn(Vec<RuntimeVal>, Rc<RefCell<SourceEnv>>) -> RuntimeVal>,
}

impl fmt::Debug for InternalFunctionVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternalFunctionVal {{ name: {} }}", self.fn_name)
    }
}

#[derive(Debug, Clone)]
pub struct StringVal {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ReturnVal {
    pub value: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct ListVal {
    pub values: Vec<RuntimeVal>,
}

impl ListVal {
    pub fn push(&mut self, value: RuntimeVal) {
        self.values.push(value);
    }

    pub fn len(&self) -> isize {
        self.values.len().try_into().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct ObjectVal {
    pub values: HashMap<String, RuntimeVal>,
}

impl RuntimeVal {
    pub fn fmt_nondebug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeVal::NumberVal(n) => write!(f, "{}", n.value),
            RuntimeVal::StringVal(s) => write!(f, "{}", s.value),
            RuntimeVal::BoolVal(b) => write!(f, "{}", b.value),
            RuntimeVal::NullVal(_) => write!(f, "null"),
            RuntimeVal::FunctionVal(func) => write!(f, "<function {}>", func.fn_name),
            RuntimeVal::InternalFunctionVal(func) => write!(f, "<function {}>", func.fn_name),
            RuntimeVal::ReturnVal(r) => write!(f, "{:#?}", r.value),
            RuntimeVal::IteratorVal(i) => write!(f, "<iterator {}>", i.to_name),
            RuntimeVal::ListVal(lv) => {
                write!(f, "[")?;
                for (i, val) in lv.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    val.fmt_debug(f)?;
                }
                write!(f, "]")
            }
            RuntimeVal::ObjectVal(ov) => {
                write!(f, "{{\n")?;
                for (i, (key, val)) in ov.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",\n")?;
                    }
                    write!(f, "{}{}: ", "    ", key)?;
                    val.fmt_with_indent(f, 1)?;
                }
                write!(f, "\n}}")
            }
        }
    }

    pub fn fmt_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeVal::NumberVal(n) => write!(f, "{}", n.value),
            RuntimeVal::StringVal(s) => write!(f, "\"{}\"", s.value),
            RuntimeVal::BoolVal(b) => write!(f, "{}", b.value),
            RuntimeVal::NullVal(_) => write!(f, "null"),
            RuntimeVal::FunctionVal(func) => write!(
                f,
                "<function {} ({})>",
                func.fn_name,
                func.params.join(", ")
            ),
            RuntimeVal::InternalFunctionVal(func) => {
                write!(f, "<function::internal {}>", func.fn_name)
            }
            RuntimeVal::ReturnVal(r) => write!(f, "{:#?}", r.value),
            RuntimeVal::IteratorVal(i) => write!(f, "<iterator {}>", i.to_name),
            RuntimeVal::ListVal(lv) => {
                write!(f, "[")?;
                for (i, val) in lv.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    val.fmt_with_indent(f, 0)?;
                }
                write!(f, "]")
            }
            RuntimeVal::ObjectVal(ov) => {
                write!(f, "{{\n")?;
                for (i, (key, val)) in ov.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",\n")?;
                    }
                    write!(f, "{}{}: ", "    ", key)?;
                    val.fmt_with_indent(f, 1)?;
                }
                write!(f, "\n}}")
            }
        }
    }

    fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = "    ".repeat(depth); // 4 spaces per indent level

        match self {
            RuntimeVal::NumberVal(n) => write!(f, "{}", n.value),
            RuntimeVal::StringVal(s) => write!(f, "{}", s.value),
            RuntimeVal::BoolVal(b) => write!(f, "{}", b.value),
            RuntimeVal::NullVal(_) => write!(f, "null"),
            RuntimeVal::FunctionVal(func) => write!(f, "<function {}>", func.fn_name),
            RuntimeVal::InternalFunctionVal(func) => write!(f, "<internal fn {}>", func.fn_name),
            RuntimeVal::ReturnVal(_) => write!(f, "return"),
            RuntimeVal::IteratorVal(_) => write!(f, "iterator"),
            RuntimeVal::ListVal(lv) => {
                write!(f, "[")?;
                for (i, val) in lv.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    val.fmt_with_indent(f, depth)?;
                }
                write!(f, "]")
            }
            RuntimeVal::ObjectVal(ov) => {
                write!(f, "{{\n")?;
                for (i, (key, val)) in ov.values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",\n")?;
                    }
                    write!(f, "{}{}: ", indent.clone() + "    ", key)?;
                    val.fmt_with_indent(f, depth + 1)?;
                }
                write!(f, "\n{}}}", indent)
            }
        }
    }
}

impl fmt::Display for RuntimeVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_nondebug(f)
    }
}

impl fmt::Debug for RuntimeVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_debug(f)
    }
}

pub struct Pretty<'a>(pub &'a RuntimeVal);

impl<'a> fmt::Display for Pretty<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt_nondebug(f)
    }
}
