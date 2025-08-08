#[cfg(test)]
use crate::typecheck::typecheck::T;
use crate::{parser::parser::Parser, typecheck::typecheck::TypeChecker};

#[test]
fn test_single_expr_basic_types() {
    let cases = vec![
        ("1", T::Integer32),
        ("10", T::Integer32),
        ("999999999999", T::Integer64),
        ("9223372036854775808", T::Integer128),
        ("\"Hello World\"", T::String),
        ("true", T::Boolean),
        ("false", T::Boolean),
    ];

    for case in cases {
        let case_string = case.0;
        let should_equate_to = case.1;

        let mut tc = TypeChecker::new(&vec![], String::new());
        let ast = Parser::new(
            case_string,
            false,
            crate::parser::parser::ExecutionTechnique::Compilation,
        )
        .produce_ast();

        let t = tc.check_expr(ast.nodes.first().unwrap(), None, false, 0);

        assert_eq!(t, should_equate_to);
    }
}

#[test]
fn test_typecast() {
    let cases = vec![
        ("1@i8", T::Integer8),
        ("1@i16", T::Integer16),
        ("1@i32", T::Integer32),
        ("1@i64", T::Integer64),
        ("1@i128", T::Integer128),
        ("2@inferred", T::Infer),
    ];

    for case in cases {
        let case_string = case.0;
        let should_equate_to = case.1;

        let mut tc = TypeChecker::new(&vec![], String::new());
        let ast = Parser::new(
            case_string,
            false,
            crate::parser::parser::ExecutionTechnique::Compilation,
        )
        .produce_ast();

        let t = tc.check_expr(ast.nodes.first().unwrap(), None, false, 0);

        assert_eq!(t, should_equate_to);
    }
}
