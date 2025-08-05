use std::cell::RefCell;
use std::rc::Rc;

use rand::Rng;

use crate::args;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::*;
use crate::stdlib_interp::helpers::{internal_fn, object_val};

pub fn rand_module() -> RuntimeVal {
    object_val([
        (
            "num",
            internal_fn("num", |args, env: Rc<RefCell<SourceEnv>>| {
                args![args;
                    Option<NumberVal> => min = NumberVal { value: 0 },
                    Option<NumberVal> => max = NumberVal { value: 1 }
                ];

                let mut rng = rand::rng();
                RuntimeVal::StringVal(StringVal {
                    value: rng
                        .random_range(min.value as usize..max.value as usize)
                        .to_string(),
                })
            }),
        ),
        (
            "bool",
            internal_fn("bool", |args, env: Rc<RefCell<SourceEnv>>| {
                let mut rng = rand::rng();
                RuntimeVal::BoolVal(BoolVal {
                    value: rng.random_bool(0.5),
                })
            }),
        ),
    ])
}
