use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{parser::nodetypes::NumericLiteral, runtime::values::{BoolVal, InternalFunctionVal, ListVal, NullVal, NumberVal, ObjectVal, RuntimeVal, StringVal}};

use rand::Rng;
use std::process;

#[derive(Debug, Clone)]
pub struct EnvVar {
    pub value: RuntimeVal,
    pub var_type: String,
    pub is_mutable: bool
}

#[derive(Debug, Clone)]
pub struct SourceEnv {
    pub parent: Option<Rc<RefCell<SourceEnv>>>,
    pub variables: HashMap<String, EnvVar>,
    pub is_sandboxed: bool
}

/// An entity that facilitates the declaration, reassignment, and lookup of variables.
/// 
/// To create a sub-environment manually, you must `Rc::clone()` the current env, which you can pass into the `parent` of SourceEnv::new.
impl SourceEnv {
    pub fn new(parent: Option<Rc<RefCell<SourceEnv>>>) -> Self {
        Self {
            variables: HashMap::new(),
            parent: parent,
            is_sandboxed: false
        }
    }

    /// Creates an environment with default Velvet standard library values pre-defined.
    pub fn create_global(do_sandbox_safety: bool) -> Rc<RefCell<Self>> {
        let is_sandboxed = do_sandbox_safety.clone();
        let mut this_env = Self {
            variables: HashMap::new(),
            parent: None,
            is_sandboxed: is_sandboxed
        };
        this_env.variables = HashMap::from([
            ("__VELVET_VERSION".to_string(), EnvVar {
                value: RuntimeVal::StringVal(StringVal {
                    value: env!("CARGO_PKG_VERSION").to_string()
                }),
                var_type: "string".to_string(),
                is_mutable: false
            }),
            ("__IS_SANDBOXED".to_string(), EnvVar {
                value: RuntimeVal::BoolVal(BoolVal { value: do_sandbox_safety }),
                var_type: "bool".to_string(),
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
            /*
            ("list_len".to_string(), EnvVar {
                value: RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                    fn_name: "list_len".to_string(),
                    internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                        // if let Some()
                    }),
                }),
                var_type: "internal_fn".to_string(),
                is_mutable: false
            }),
            */
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
                                RuntimeVal::ListVal(l) => "list",
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
            }),
            ("string".to_string(), EnvVar {
                var_type: String::from("internal_object"),
                is_mutable: false,
                value: RuntimeVal::ObjectVal(ObjectVal {
                    values: HashMap::from([
                        ("split".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("split"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let first = args.first().expect("split expects at least one argument");
                                
                                let to_split = match first {
                                    RuntimeVal::StringVal(s) => {
                                        s.value.clone()
                                    }
                                    _ => {
                                        panic!("First argument must be a string")
                                    }
                                };

                                let mut splitter = String::new();
                                
                                if args.len() >= 2 {
                                    match &args[1] {
                                        RuntimeVal::StringVal(s) => {
                                            splitter = s.value.clone();
                                        }
                                        _ => {
                                            panic!("Second argument must be a string or null")
                                        }
                                    }
                                }

                                let actual_split = to_split.split(&splitter);
                                return RuntimeVal::ListVal(ListVal {
                                    values: actual_split.map(|og_str| 
                                        RuntimeVal::StringVal(StringVal {
                                            value: og_str.to_owned()
                                        })
                                    ).collect()
                                })
                            })
                        })),
                        ("len".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("len"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let first = args.first().expect("len expects one argument");

                                match first {
                                    RuntimeVal::StringVal(s) => {
                                        return RuntimeVal::NumberVal(NumberVal {
                                            value: s.value.len().try_into().unwrap()
                                        })
                                    }
                                    _ => {
                                        panic!("Expected string type as an argument");
                                    }
                                }
                            })
                        })),
                        ("is_numeric".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("is_numeric"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let first = args.first().expect("is_numeric expects one argument");

                                match first {
                                    RuntimeVal::StringVal(s) => {
                                        return RuntimeVal::BoolVal(BoolVal { value: s.value.parse::<f64>().is_ok() })
                                    }
                                    _ => {
                                        panic!("Expected string type as an argument");
                                    }
                                }
                            })
                        }))
                    ])
                })
            }),
            ("debug".to_string(), EnvVar {
                var_type: String::from("internal_object"),
                is_mutable: false,
                value: RuntimeVal::ObjectVal(ObjectVal {
                    values: HashMap::from([
                        ("inspect".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("inspect"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let mut end_printstr = String::new();
                                for arg in &args {
                                    end_printstr += format!("{:#?}", arg).as_str();
                                    end_printstr += " ";
                                }
                                RuntimeVal::StringVal(StringVal { value: end_printstr })
                            })
                        })),
                    ])
                })
            }),
            ("rand".to_string(), EnvVar {
                var_type: String::from("internal_object"),
                is_mutable: false,
                value: RuntimeVal::ObjectVal(ObjectVal {
                    values: HashMap::from([
                        ("num".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("num"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let mut lower_bound: i32 = 0;
                                let mut upper_bound: i32 = 100;

                                let mut set = false;
                                if let Some(x) = args.get(0) {
                                    match x {
                                        RuntimeVal::NumberVal(n) => {
                                            lower_bound = n.value.try_into().unwrap();
                                        }
                                        _ => {}
                                    }
                                    set = true;
                                }
                                if let Some(x) = args.get(1) {
                                    match x {
                                        RuntimeVal::NumberVal(n) => {
                                            upper_bound = n.value.try_into().unwrap();
                                        }
                                        _ => {}
                                    }
                                } else if set {
                                    upper_bound = lower_bound;
                                    lower_bound = 0;
                                }
                                
                                let mut rng = rand::rng();
                                let rand = rng.random_range(lower_bound..upper_bound).try_into().unwrap();
                                return RuntimeVal::NumberVal(NumberVal { value: rand });
                            })
                        })),
                        ("bool".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("bool"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let mut rng = rand::rng();
                                return RuntimeVal::BoolVal(BoolVal { value: rng.random_bool(50.0) });
                            })
                        }))
                    ])
                })
            }),
            ("process".to_string(), EnvVar {
                var_type: String::from("internal_object"),
                is_mutable: false,
                value: RuntimeVal::ObjectVal(ObjectVal {
                    values: HashMap::from([
                        ("exit".to_string(), RuntimeVal::InternalFunctionVal(InternalFunctionVal {
                            fn_name: String::from("exit"),
                            internal_callback: Rc::new(|args: Vec<RuntimeVal>| {
                                let first = args.first();
                                let mut exit_code: i32 = 0;
                                if let Some(argf) = first {
                                    match argf {
                                        RuntimeVal::NumberVal(n) => {
                                            exit_code = n.value.try_into().unwrap()
                                        }
                                        _ => {}
                                    };
                                };
                                process::exit(exit_code)
                            })
                        })),
                    ])
                })
            }),
        ]);
        Rc::new(RefCell::new(this_env))
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