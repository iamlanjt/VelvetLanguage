use std::collections::HashMap;

use crate::runtime::values::RuntimeVal;

#[derive(Debug)]
pub struct EnvVar {
    pub value: RuntimeVal,
    pub var_type: String,
    pub is_mutable: bool
}

#[derive(Debug)]
pub struct SourceEnv {
    pub parent: Option<Box<SourceEnv>>,
    pub variables: HashMap<String, EnvVar>
}

impl SourceEnv {
    pub fn new(parent: Option<Box<SourceEnv>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: parent
        }
    }

    pub fn create_global() -> Self {
        let mut variables: HashMap<String, EnvVar> = HashMap::new();
        return Self {
            variables,
            parent: None
        }
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

    pub fn fetch_local(&self, var_name: &String) -> Option<&EnvVar> {
        self.variables.get(var_name)
    }

    pub fn fetch(&self, var_name: &String) -> Option<&EnvVar> {
        if let Some(local) = self.fetch_local(var_name) {
            return Some(local);
        }

        match self.parent.as_ref() {
            Some(parent_env) => parent_env.fetch(var_name),
            None => None
        }
    }
}