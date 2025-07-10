use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::*;
use crate::stdlib::helpers::{internal_fn, object_val};

pub fn process_module() -> RuntimeVal {
    object_val([
        ("exit", internal_fn("exit", |args, env: Rc<RefCell<SourceEnv>>| {
            args![args;
                Option<NumberVal> => exit_code = NumberVal { value: 0 }
            ];

            std::process::exit(exit_code.value.try_into().unwrap())
        }))
    ])
}