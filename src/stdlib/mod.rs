pub mod core;
pub mod string;
pub mod rand;
pub mod debug;
pub mod process;
pub mod network;
pub mod crypto;
#[macro_use]
pub mod helpers;

use crate::runtime::values::{RuntimeVal, StringVal, BoolVal, NullVal, InternalFunctionVal, ObjectVal, NumberVal};
use std::{collections::HashMap, rc::Rc};

pub fn standard_library_values(sandboxed: bool) -> HashMap<String, RuntimeVal> {
    let mut values = HashMap::new();

    values.insert("null".to_string(), RuntimeVal::NullVal(NullVal {  }));

    values.insert("__VELVET_VERSION".to_string(), RuntimeVal::StringVal(StringVal {
        value: env!("CARGO_PKG_VERSION").to_string(),
    }));

    values.insert("__IS_SANDBOXED".to_string(), RuntimeVal::BoolVal(BoolVal {
        value: sandboxed,
    }));

    values.insert("print".to_string(), core::print_fn());
    // values.insert("itypeof".to_string(), core::itypeof_fn());
    
    values.insert("string".to_string(), string::string_module());
    values.insert("debug".to_string(), debug::debug_module());
    values.insert("rand".to_string(), rand::rand_module());
    values.insert("process".to_string(), process::process_module());
    values.insert("network".to_string(), network::network_module());
    values.insert("crypto".to_string(), crypto::crypto_module());

    values
}
