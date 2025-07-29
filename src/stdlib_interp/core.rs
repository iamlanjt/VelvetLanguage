use std::cell::RefCell;
use std::rc::Rc;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::{NullVal, RuntimeVal, StringVal};
use crate::stdlib_interp::helpers::internal_fn;

// TODO: Generate FFI for compiler instead of interpreter
pub fn print_fn() -> RuntimeVal {
    internal_fn("print", |args, env: Rc<RefCell<SourceEnv>>| {
        let output = args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", output);
        RuntimeVal::NullVal(NullVal {})
    })
}

pub fn infer_runtime_type(val: &RuntimeVal) -> String {
    match val {
        RuntimeVal::InternalFunctionVal(_) => "internal_fn",
        RuntimeVal::ObjectVal(_) => "internal_object",
        RuntimeVal::StringVal(_) => "string",
        RuntimeVal::NumberVal(_) => "number",
        RuntimeVal::BoolVal(_) => "bool",
        RuntimeVal::NullVal(_) => "null",
        RuntimeVal::ListVal(_) => "list",
        RuntimeVal::FunctionVal(_) => "function",
        RuntimeVal::ReturnVal(_) => "return",
        _ => "unknown",
    }
    .to_string()
}
