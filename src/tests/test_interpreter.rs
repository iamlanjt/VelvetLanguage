use std::rc::Rc;

#[cfg(test)]
use crate::runtime::values::RuntimeVal;
use crate::{parser::parser::Parser, runtime::{interpreter::Interpreter, source_environment::source_environment::SourceEnv}};

#[cfg(test)]

fn quick_setup(source: &str) -> Box<RuntimeVal> {
    return Interpreter::new(Parser::new(source).produce_ast()).evaluate_body(
        SourceEnv::create_global(false)
    );
}

/**
 * Test environment functionality
 */
#[test]
fn test_var_decl_immutable() {
    let env = SourceEnv::create_global(false);

    Interpreter::new(Parser::new(
        "bind test_var as number = 10"
    ).produce_ast()).evaluate_body(Rc::clone(&env));

    let env_var = env.borrow().fetch(&String::from("test_var"));

    assert!(env_var.is_some(), "Failed to find created variable");
    assert_eq!(env_var.clone().unwrap().is_mutable, false, "EnvVar Mutability Fault");
    assert_eq!(env_var.clone().unwrap().var_type, String::from("number"));

    match env_var.unwrap().value {
        RuntimeVal::NumberVal(nv) => {
            assert_eq!(nv.value, 10);
        }
        _ => panic!("EnvVar incorrect type")
    }
}

#[test]
fn test_var_decl_mutable() {
    let env = SourceEnv::create_global(false);

    Interpreter::new(Parser::new(
        "bindm test_var as number = 10"
    ).produce_ast()).evaluate_body(Rc::clone(&env));

    let env_var = env.borrow().fetch(&String::from("test_var"));

    assert!(env_var.is_some(), "Failed to find created variable");
    assert_eq!(env_var.clone().unwrap().is_mutable, true, "EnvVar Mutability Fault");
    assert_eq!(env_var.clone().unwrap().var_type, String::from("number"));

    match env_var.unwrap().value {
        RuntimeVal::NumberVal(nv) => {
            assert_eq!(nv.value, 10);
        }
        _ => panic!("EnvVar incorrect type")
    }
}

#[test]
fn test_var_mutation() {
    let env = SourceEnv::create_global(false);

    Interpreter::new(Parser::new(
        "bindm test_var as number = 10\ntest_var = 5"
    ).produce_ast()).evaluate_body(Rc::clone(&env));

    let env_var = env.borrow().fetch(&String::from("test_var"));

    assert!(env_var.is_some(), "Failed to find created variable");
    assert_eq!(env_var.clone().unwrap().is_mutable, true, "EnvVar Mutability Fault");
    assert_eq!(env_var.clone().unwrap().var_type, String::from("number"));

    match env_var.unwrap().value {
        RuntimeVal::NumberVal(nv) => {
            assert_eq!(nv.value, 5);
        }
        _ => panic!("EnvVar did not mutate.")
    }
}

#[test]
#[should_panic(expected = "Cannot assign to immutable variable 'test_var'")]
fn test_var_mutation_on_immutable_env_var() {
    let env = SourceEnv::create_global(false);

    Interpreter::new(Parser::new(
        "bind test_var as number = 10\ntest_var = 5"
    ).produce_ast()).evaluate_body(Rc::clone(&env));
}

/**
 * Runtime Values
 */

#[test]
fn test_interpreter_objectval() {
    let res = *quick_setup("{ a: { sub_object: true }, b: 5 * 2 }");
    
    match res {
        RuntimeVal::ObjectVal(obj) => {
            assert_eq!(obj.values.len(), 2);
            assert!(obj.values.get("a").is_some());
            match obj.values.get("a").unwrap() {
                RuntimeVal::ObjectVal(obj2) => {
                    assert_eq!(obj2.values.len(), 1);
                    assert!(obj2.values.get("sub_object").is_some())
                }
                _ => panic!("Incorrect transformation for sub-object: expected ObjectVal")
            }
            assert!(obj.values.get("b").is_some());
            match obj.values.get("b").unwrap() {
                RuntimeVal::NumberVal(num) => {
                    assert_eq!(num.value, 10);
                }
                _ => panic!("Expected NumberVal")
            }
        }
        _ => panic!("Incorrect transformation: expected ObjectVal")
    }
}