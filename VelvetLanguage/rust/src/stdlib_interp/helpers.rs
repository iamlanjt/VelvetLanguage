use std::{cell::RefCell, collections::HashMap};
use std::rc::Rc;

use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::runtime::values::{InternalFunctionVal, ObjectVal, RuntimeVal};

pub fn internal_fn<F>(name: &str, callback: F) -> RuntimeVal
where
    F: Fn(Vec<RuntimeVal>, Rc<RefCell<SourceEnv>>) -> RuntimeVal + 'static,
{
    RuntimeVal::InternalFunctionVal(InternalFunctionVal {
        fn_name: name.to_string(),
        internal_callback: Rc::new(callback),
    })
}

pub fn object_val(items: impl IntoIterator<Item = (impl Into<String>, RuntimeVal)>) -> RuntimeVal {
    RuntimeVal::ObjectVal(ObjectVal {
        values: items.into_iter().map(|(k, v)| (k.into(), v)).collect::<HashMap<_, _>>(),
    })
}

#[macro_export]
macro_rules! args {
    // Base case: no arguments, done parsing
    ($args:ident;) => {};

    // Required argument (no default)
    ($args:ident; $typ:ident => $var:ident, $($rest:tt)*) => {
        let $var = match $args.get(0) {
            Some(RuntimeVal::$typ(val)) => val,
            Some(other) => panic!(
                "Argument `{}` expected to be {}, found {:?}",
                stringify!($var),
                stringify!($typ),
                other
            ),
            None => panic!("Argument `{}` is missing", stringify!($var)),
        };
        let $args = &$args[1..];
        args!($args; $($rest)*);
    };

    // Optional argument with default
    ($args:ident; Option<$typ:ident> => $var:ident = $default:expr, $($rest:tt)*) => {
        let $var = match $args.get(0) {
            Some(RuntimeVal::$typ(val)) => val.clone(),
            Some(other) => panic!(
                "Argument `{}` expected to be Option<{}>, found {:?}",
                stringify!($var),
                stringify!($typ),
                other
            ),
            None => $default,
        };
        let $args = if $args.is_empty() { $args } else { (&$args[1..]).to_vec() };
        args!($args; $($rest)*);
    };

    // Optional argument with default and no trailing comma
    ($args:ident; Option<$typ:ident> => $var:ident = $default:expr) => {
        let $var = match $args.get(0) {
            Some(RuntimeVal::$typ(val)) => val.clone(),
            Some(other) => panic!(
                "Argument `{}` expected to be Option<{}>, found {:?}",
                stringify!($var),
                stringify!($typ),
                other
            ),
            None => $default,
        };
    };

    // Required argument with no trailing comma
    ($args:ident; $typ:ident => $var:ident) => {
        let $var = match $args.get(0) {
            Some(RuntimeVal::$typ(val)) => val,
            Some(other) => panic!(
                "Argument `{}` expected to be {}, found {:?}",
                stringify!($var),
                stringify!($typ),
                other
            ),
            None => panic!("Argument `{}` is missing", stringify!($var)),
        };
    };
}