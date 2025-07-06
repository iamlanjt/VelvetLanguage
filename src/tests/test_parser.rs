use crate::parser::{nodetypes::{BinaryExpr, Node}, parser::Parser};

#[cfg(test)]

use super::super::parser;

// Unit tests: test all nodes
#[test]
fn parser_unit_node_binary_expr() {
    let mut parser = Parser::new("15 * 15 / 15 + 15");
    let ast = parser.produce_ast();

    assert_eq!(ast.len(), 1, "BinaryExprs should be recursive; found multiple top-level nodes in one BinaryExpression unit test.");
}

// Test for correct expression types
#[test]
fn parser_integration_member_cases() {
    let cases = vec![
        "parent.property",
        "parent[computed_property]",
        "parent.property.sub_property",
        "parent[computed_property][sub_computed_property]",
        "parent[computed_property[sub_computed_property]]"
    ];
    for c in cases {
        let mut parser = Parser::new(c);
        parser.produce_ast();
    }
}