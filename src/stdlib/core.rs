use std::cell::RefCell;
use std::rc::Rc;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::{RuntimeVal, StringVal, BoolVal, NullVal};
use crate::stdlib::helpers::internal_fn;

pub fn print_fn() -> RuntimeVal {
    internal_fn("print", |args, env: Rc<RefCell<SourceEnv>>| {
        let output = args
            .iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        println!("{}", output);
        RuntimeVal::NullVal(NullVal {  })
    })
}

pub fn itypeof_fn() -> RuntimeVal {
    internal_fn("itypeof", |args, env: Rc<RefCell<SourceEnv>>| {
        args![args;];

        let val = args.get(0).expect("itypeof requires 1 argument");

        let type_name = match val {
            RuntimeVal::StringVal(_) => "string",
            RuntimeVal::BoolVal(_) => "bool",
            RuntimeVal::NullVal(_) => "null",
            RuntimeVal::ListVal(_) => "list",
            RuntimeVal::ObjectVal(_) => "object",
            RuntimeVal::FunctionVal(_) => "function",
            RuntimeVal::InternalFunctionVal(_) => "internal_function",
            RuntimeVal::NumberVal(_) => "number",
            _ => "unknown"
        };

        RuntimeVal::StringVal(StringVal {
            value: type_name.to_string(),
        })
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
        _ => "unknown"
    }.to_string()
}