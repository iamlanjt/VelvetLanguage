use std::cell::RefCell;
use std::rc::Rc;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::*;
use crate::stdlib::helpers::{internal_fn, object_val};

pub fn string_module() -> RuntimeVal {
    object_val([
        ("split", internal_fn("split", |args, env: Rc<RefCell<SourceEnv>>| {
            args![args;
                StringVal => input,
                Option<StringVal> => delim = StringVal { value: ",".to_string() }
            ];

            let parts = input
                .value
                .split(&delim.value)
                .map(|s| RuntimeVal::StringVal(StringVal { value: s.to_string() }))
                .collect();

            RuntimeVal::ListVal(ListVal { values: parts })
        }))
    ])
}