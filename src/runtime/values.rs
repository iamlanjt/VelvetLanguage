use std::rc::Rc;

use crate::parser::nodetypes::Node;

#[derive(Debug, Clone)]
pub enum RuntimeVal {
    NumberVal(NumberVal),
    NullVal(NullVal),
    FunctionVal(FunctionVal),
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

#[derive(Debug, Clone)]
pub struct StringVal {
    pub value: String
}

#[derive(Debug, Clone)]
pub struct ReturnVal {
    pub value: Box<Node>
}