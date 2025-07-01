use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::values::{InternalFunctionVal, NullVal, RuntimeVal, StringVal};

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub value: RuntimeVal,
    pub var_type: String,
    pub is_mutable: bool
}

#[derive(Debug, Clone)]
pub struct SourceEnv {
    pub parent: Option<Rc<RefCell<SourceEnv>>>,
    pub variables: HashMap<String, EnvVar>
}

impl SourceEnv {
    pub fn new(parent: Option<Rc<RefCell<SourceEnv>>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: parent
        }
    }

    pub fn create_global() -> Rc<RefCell<Self>> {
        let variables: HashMap<String, EnvVar> = HashMap::from([
            ("__VELVET_VERSION".to_string(), EnvVar {
                value: RuntimeVal::StringVal(StringVal {
                    value: env!("CARGO_PKG_VERSION").to_string()
                }),
                var_type: "string".to_string(),
                is_mutable: false
            }),
            ("print".to_string(), EnvVar {
                value: RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                    fn_name: "print".to_string(),
                    internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                        for arg in &args {
                            print!("{}", arg.to_string());
                        }
                        RuntimeVal::NullVal(NullVal {})
                    })
                }),
                var_type: String::from("internal_fn"),
                is_mutable: false
            })
        ]);
        Rc::new(RefCell::new(Self {
            variables: variables,
            parent: None,
        }))
    }

    pub fn declare_var(&mut self, var_name: String, var_value: RuntimeVal, var_type: String, var_is_mutable: bool) {
        if self.variables.get(&var_name).is_some() {
            panic!("Attempt to redeclare binding {} to {:#?} (non-assignment)", var_name, var_value);
        }
        self.variables.insert(var_name, EnvVar {
            value: var_value,
            var_type,
            is_mutable: var_is_mutable
        });
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
            Some(parent_env) => {
                parent_env.borrow().fetch(var_name)
            }
            None => None,
        }
    }
}