use std::cell::RefCell;
use std::rc::Rc;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::*;
use crate::stdlib_interp::helpers::{internal_fn, object_val};

pub fn network_module() -> RuntimeVal {
    object_val([(
        "get",
        internal_fn("get", |args, env: Rc<RefCell<SourceEnv>>| {
            args![args;];

            RuntimeVal::NullVal(NullVal {})
        }),
    )])
}
