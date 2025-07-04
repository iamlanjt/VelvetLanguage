use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::runtime::values::{InternalFunctionVal, NullVal, ObjectVal, RuntimeVal, StringVal};

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
                        let mut end_printstr = String::new();
                        for arg in &args {
                            end_printstr += &arg.to_string();
                            end_printstr += " ";
                        }
                        println!("{}", end_printstr);
                        RuntimeVal::NullVal(NullVal {})
                    })
                }),
                var_type: String::from("internal_fn"),
                is_mutable: false
            }),
            ("itypeof".to_string(), EnvVar {
                value: RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                    fn_name: "itypeof".to_string(),
                    internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                        if let Some(first) = args.first() {
                            let internal_type: &str = match first {
                                RuntimeVal::BoolVal(b) => "boolean",
                                RuntimeVal::FunctionVal(f) => "function",
                                RuntimeVal::InternalFunctionVal(f) => "internal_function",
                                RuntimeVal::NullVal(n) => "null",
                                RuntimeVal::NumberVal(n) => "number",
                                RuntimeVal::ReturnVal(r) => "return",
                                RuntimeVal::StringVal(s) => "string",
                                _ => "unknown"
                            };
                            RuntimeVal::StringVal(StringVal { value: internal_type.to_string() })
                        } else {
                            RuntimeVal::NullVal(NullVal {})
                        }
                    })
                }),
                var_type: String::from("internal_fn"),
                is_mutable: false
            }),
            ("network".to_string(), EnvVar {
                var_type: String::from("internal_object"),
                is_mutable: false,
                value: RuntimeVal::ObjectVal(ObjectVal {
                    values: HashMap::from([
                        ("get".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("get"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                return RuntimeVal::NullVal(NullVal {  });
                            })
                        }))
                    ])
                })
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