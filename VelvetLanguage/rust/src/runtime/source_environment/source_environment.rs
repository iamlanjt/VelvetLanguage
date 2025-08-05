use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::values::RuntimeVal;

use crate::stdlib_interp::standard_library_values;

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub value: RuntimeVal,
    pub is_mutable: bool,
}

#[derive(Debug, Clone)]
pub struct SourceEnv {
    pub parent: Option<Rc<RefCell<SourceEnv>>>,
    pub variables: HashMap<String, EnvVar>,
}

/// An entity that facilitates the declaration, reassignment, and lookup of variables.
///
/// To create a sub-environment manually, you must `Rc::clone()` the current env, which you can pass into the `parent` of SourceEnv::new.
impl SourceEnv {
    pub fn new(parent: Option<Rc<RefCell<SourceEnv>>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent,
        }
    }

    /// Creates an environment with default Velvet standard library values pre-defined.
    pub fn create_global(do_sandbox_safety: bool) -> Rc<RefCell<Self>> {
        let mut this_env = Self {
            variables: HashMap::new(),
            parent: None,
        };
        this_env.variables = standard_library_values(do_sandbox_safety)
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    EnvVar {
                        value: v.clone(),
                        is_mutable: false,
                    },
                )
            })
            .collect();
        Rc::new(RefCell::new(this_env))
    }

    pub fn declare_var(&mut self, var_name: String, var_value: RuntimeVal, var_is_mutable: bool) {
        if self.variables.contains_key(&var_name) {
            panic!(
                "Attempt to redeclare binding {} to {:#?} (non-assignment)",
                var_name, var_value
            );
        }
        self.variables.insert(
            var_name,
            EnvVar {
                value: var_value,
                is_mutable: var_is_mutable,
            },
        );
    }

    pub fn attempt_assignment(&mut self, var_name: String, var_new_value: RuntimeVal) {
        if let Some(local_var) = self.variables.get_mut(&var_name) {
            if local_var.is_mutable {
                local_var.value = var_new_value;
                return;
            } else {
                panic!("Cannot assign to immutable variable '{}'", var_name);
            }
        }

        match &self.parent {
            Some(parent_rc) => {
                parent_rc
                    .borrow_mut()
                    .attempt_assignment(var_name, var_new_value);
            }
            None => {
                panic!("Undefined variable '{}'", var_name);
            }
        }
    }

    pub fn fetch_local(&self, var_name: &String) -> Option<&EnvVar> {
        let res = self.variables.get(var_name);
        // println!("FINDING {}: {:#?}", var_name, res);
        res
    }

    pub fn fetch(&self, var_name: &String) -> Option<EnvVar> {
        if let Some(local) = self.fetch_local(var_name) {
            return Some(local.clone());
        }

        match &self.parent {
            Some(parent_env) => parent_env.borrow().fetch(var_name),
            None => None,
        }
    }
}
