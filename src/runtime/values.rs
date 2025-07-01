use std::{collections::HashMap, fmt, rc::Rc};

use crate::parser::nodetypes::Node;

#[derive(Debug, Clone)]
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
    ObjectVal(ObjectVal)
}

impl RuntimeVal {
    pub fn compare(&self, other: &RuntimeVal, op: &str) -> Result<bool, String> {
        match (self, other) {
            (RuntimeVal::NumberVal(l), RuntimeVal::NumberVal(r)) => {
                let result = match op {
                    "==" => l.value == r.value,
                    "!=" => l.value != r.value,
                    "<"  => l.value < r.value,
                    "<=" => l.value <= r.value,
                    ">"  => l.value > r.value,
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
            _ => Err(format!(
                "Unsupported comparison between {:?} and {:?}",
                self, other
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NumberVal {
    pub value: i32
}

#[derive(Debug, Clone)]
pub struct NullVal {

}

#[derive(Debug, Clone)]
pub struct BoolVal {
    pub value: bool
}

#[derive(Debug, Clone)]
pub struct FunctionVal {
    pub params: Vec<String>,
    pub fn_name: String,
    pub execution_body: Rc<Vec<Box<Node>>>,
    pub is_internal: bool
}

#[derive(Debug, Clone)]
pub struct IteratorVal {
    pub to_name: String,
    pub target: Box<RuntimeVal>
}

#[derive(Clone)]
pub struct InternalFunctionVal{
    pub fn_name: String,
    pub internal_callback: Rc<dyn Fn(Vec<RuntimeVal>) -> RuntimeVal>
}

impl fmt::Debug for InternalFunctionVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternalFunctionVal {{ name: {} }}", self.fn_name)
    }
}

#[derive(Debug, Clone)]
pub struct StringVal {
    pub value: String
}

#[derive(Debug, Clone)]
pub struct ReturnVal {
    pub value: Box<Node>
}

#[derive(Debug, Clone)]
pub struct ListVal {
    pub values: Vec<RuntimeVal>
}

#[derive(Debug, Clone)]
pub struct ObjectVal {
    pub values: HashMap<String, RuntimeVal>
}

impl RuntimeVal {
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
                    if i > 0 { write!(f, ", ")?; }
                    val.fmt_with_indent(f, depth)?;
                }
                write!(f, "]")
            }
            RuntimeVal::ObjectVal(ov) => {
                write!(f, "{{\n")?;
                for (i, (key, val)) in ov.values.iter().enumerate() {
                    if i > 0 { write!(f, ",\n")?; }
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
        self.fmt_with_indent(f, 0)
    }
}