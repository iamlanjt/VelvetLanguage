#[derive(Debug, Clone)]
pub enum RuntimeVal {
    NumberVal(NumberVal),
    NullVal(NullVal),
    BoolVal(BoolVal)
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