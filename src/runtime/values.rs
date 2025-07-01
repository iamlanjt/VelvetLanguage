use std::{fmt, rc::Rc};

use crate::parser::nodetypes::Node;

#[derive(Debug, Clone)]
pub enum RuntimeVal {
    NumberVal(NumberVal),
    NullVal(NullVal),
    FunctionVal(FunctionVal),
    InternalFunctionVal(InternalFunctionVal),
    BoolVal(BoolVal),
    StringVal(StringVal),
    ReturnVal(ReturnVal)
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
    pub value: usize
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

impl fmt::Display for RuntimeVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeVal::NumberVal(n) => write!(f, "{}", n.value),
            RuntimeVal::StringVal(s) => write!(f, "{}", s.value),
            RuntimeVal::BoolVal(b) => write!(f, "{}", b.value),
            RuntimeVal::NullVal(n) => write!(f, "null"),
            RuntimeVal::FunctionVal(func) => write!(f, "<function {}>", func.fn_name),
            RuntimeVal::InternalFunctionVal(func) => write!(f, "<internal fn {}>", func.fn_name),
            RuntimeVal::ReturnVal(r) => write!(f, "returned")
        }
    }
}