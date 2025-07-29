use std::cell::RefCell;
use std::rc::Rc;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::*;
use crate::stdlib_interp::helpers::{internal_fn, object_val};

pub fn debug_module() -> RuntimeVal {
    object_val([
        (
            "inspect",
            internal_fn("inspect", |args, env: Rc<RefCell<SourceEnv>>| {
                args![args;];

                let mut end_printstr = String::new();
                for arg in &args {
                    end_printstr += format!("{:#?}", arg).as_str();
                    end_printstr += " ";
                }
                RuntimeVal::StringVal(StringVal {
                    value: end_printstr,
                })
            }),
        ),
        (
            "typeof",
            internal_fn("typeof", |args, env| {
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
                    _ => "unknown",
                };

                RuntimeVal::StringVal(StringVal {
                    value: type_name.to_string(),
                })
            }),
        ),
    ])
}
