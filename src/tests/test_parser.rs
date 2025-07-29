use crate::parser::parser::ExecutionTechnique;
#[cfg(test)]
use crate::parser::{nodetypes::Node, parser::Parser};

// Unit tests: test all nodes
#[test]
fn parser_unit_node_binary_expr() {
    let mut parser = Parser::new("15 * 11 / 1 + 2", false, ExecutionTechnique::Interpretation);
    let ast = parser.produce_ast();

    assert_eq!(
        ast.len(),
        1,
        "BinaryExprs should be recursive; found multiple top-level nodes in one BinaryExpression unit test."
    );

    match &ast[0] {
        Node::BinaryExpr(top_expr) => {
            assert_eq!(
                top_expr.op, "+",
                "BinOp Precedence issue: expected top expression operator to be a plus."
            );

            match &*top_expr.left {
                Node::BinaryExpr(left_expr) => {
                    assert_eq!(
                        left_expr.op, "/",
                        "BinOp Precedence issue: expected left expr operator to be a division."
                    );

                    match &*left_expr.left {
                        Node::BinaryExpr(mul_expr) => {
                            assert_eq!(
                                mul_expr.op, "*",
                                "BinOp Precedence issue: expected lowest-left expr operator to be multiplicative."
                            );

                            match (&*mul_expr.left, &*mul_expr.right) {
                                (Node::NumericLiteral(l1), Node::NumericLiteral(l2)) => {
                                    assert_eq!(l1.literal_value.parse::<i32>().unwrap(), 15);
                                    assert_eq!(l2.literal_value.parse::<i32>().unwrap(), 11);
                                }
                                _ => panic!("Expected literals on both sides of multiplication"),
                            }
                        }
                        _ => panic!("Expected multiplication as left side of division"),
                    }

                    match &*left_expr.right {
                        Node::NumericLiteral(lit) => {
                            assert_eq!(lit.literal_value.parse::<i32>().unwrap(), 1)
                        }
                        _ => panic!("Expected literal on right side of division"),
                    }
                }
                _ => panic!("Expected BinaryExpr on left side of addition"),
            }

            match &*top_expr.right {
                Node::NumericLiteral(lit) => {
                    assert_eq!(lit.literal_value.parse::<i32>().unwrap(), 2)
                }
                _ => panic!("Expected literal on right side of addition"),
            }
        }
        _ => panic!("Expected a BinaryExpr as the top-level node"),
    }
}

#[test]
fn parser_unit_node_while_stmt_comparator() {
    let ast = Parser::new(
        "while CONDITION_LEFT == CONDITION_RIGHT do {}",
        false,
        ExecutionTechnique::Interpretation,
    )
    .produce_ast();

    assert_eq!(ast.len(), 1);
    match &*ast.first().unwrap() {
        Node::WhileStmt(stmt) => match stmt.condition.as_ref() {
            Node::Comparator(comp) => {
                match &*comp.lhs {
                    Node::Identifier(ident) => {
                        assert_eq!(
                            ident.identifier_name, "CONDITION_LEFT",
                            "LHS of comparator has incorrect precedence"
                        )
                    }
                    _ => panic!("Expected a CONDITION_LEFT Identifier node for LHS of comparator"),
                }
                match &*comp.rhs {
                    Node::Identifier(ident) => {
                        assert_eq!(
                            ident.identifier_name, "CONDITION_RIGHT",
                            "RHS of comparator has incorrect precedence"
                        )
                    }
                    _ => panic!("Expected a CONDITION_RIGHT Identifier node for RHS of comparator"),
                }
                assert_eq!(
                    comp.op, "==",
                    "Operator for comparator expected to be DoubleEq"
                )
            }
            _ => panic!("Expected condition to be a comparator node."),
        },
        _ => panic!("Expected a while statement"),
    }
}

#[test]
fn parser_unit_node_while_stmt_truthy() {
    let ast = Parser::new(
        "while CONDITION_LEFT do {}",
        false,
        ExecutionTechnique::Interpretation,
    )
    .produce_ast();

    assert_eq!(
        ast.len(),
        1,
        "Expected AST size to be of 1 node for while statement"
    );
    match &*ast.first().unwrap() {
        Node::WhileStmt(w) => match &*w.condition {
            Node::Identifier(ident) => {
                assert_eq!(
                    ident.identifier_name, "CONDITION_LEFT",
                    "Expected identifier name for while condition expr"
                );
            }
            _ => panic!("Expected identifier for while condition expr"),
        },
        _ => panic!("Expected while statement node"),
    }
}

#[test]
fn parser_unit_node_while_stmt_body() {
    let ast = Parser::new(
        "while CONDITION_LEFT do { print(CONDITION_LEFT) }",
        false,
        ExecutionTechnique::Interpretation,
    )
    .produce_ast();

    assert_eq!(ast.len(), 1);
    match &*ast.first().unwrap() {
        Node::WhileStmt(w) => {
            assert_eq!(w.body.len(), 1);
            match w.body.first().unwrap() {
                Node::CallExpr(_cexpr) => {}
                _ => panic!("Expected body for while statement to be a call expression"),
            }
        }
        _ => panic!("Expected while statement node"),
    }
}

#[test]
fn parser_unit_node_string_literal() {
    let ast = Parser::new("'Hello World'", false, ExecutionTechnique::Interpretation).produce_ast();

    assert_eq!(ast.len(), 1);
    match &*ast.first().unwrap() {
        Node::StringLiteral(s) => {
            assert_eq!(
                s.literal_value, "Hello World",
                "String literal inner parsing failure: incorrect characterset"
            )
        }
        _ => panic!("Expected string literal node"),
    }
}

#[test]
fn parser_unit_node_object_lieral() {
    let ast = Parser::new(
        "{ a: { sub_object: true }, b: 2 * 3 }",
        false,
        ExecutionTechnique::Interpretation,
    )
    .produce_ast();

    assert_eq!(ast.len(), 1);
    match &*ast.first().unwrap() {
        Node::ObjectLiteral(obj) => {
            assert_eq!(obj.props.len(), 2);
            assert!(obj.props.contains_key("a"));
            assert!(obj.props.contains_key("b"));

            match obj.props.get("a").unwrap() {
                Node::ObjectLiteral(obj2) => {
                    assert!(obj2.props.contains_key("sub_object"));
                    match obj2.props.get("sub_object").unwrap() {
                        Node::BoolLiteral(_b) => {}
                        _ => panic!("Incorrect sub_object value"),
                    }
                }
                _ => panic!("Expected sub-object literal node"),
            }

            match obj.props.get("b").unwrap() {
                Node::BinaryExpr(_binexp) => {}
                _ => panic!("Incorrect B value; expected binexp"),
            }
        }
        _ => panic!("Expected object literal"),
    }
}

#[test]
fn parser_unit_node_list_literal() {
    let ast = Parser::new(
        "[1, 2, ['sub', 5]]",
        false,
        ExecutionTechnique::Interpretation,
    )
    .produce_ast();

    assert_eq!(ast.len(), 1);
    match &*ast.first().unwrap() {
        Node::ListLiteral(listlit) => {
            assert_eq!(listlit.props.len(), 3);
            match listlit.props.first().unwrap() {
                Node::NumericLiteral(num) => {
                    assert_eq!(num.literal_value, "1");
                }
                _ => panic!("Expected numeric literal"),
            }
            match listlit.props.get(2).unwrap() {
                Node::ListLiteral(listlit2) => {
                    assert_eq!(listlit2.props.len(), 2);
                    match listlit2.props.first().unwrap() {
                        Node::StringLiteral(_s) => {}
                        _ => panic!("Failed to parse correct type for sub-list lit"),
                    }
                }
                _ => panic!("Failed to parse sub-list literal"),
            }
        }
        _ => panic!("Expected a list literal"),
    }
}

// Test for correct expression types
#[test]
fn parser_integration_member_cases() {
    let cases = vec![
        "parent.property",
        "parent[computed_property]",
        "parent.property.sub_property",
        "parent[computed_property][sub_computed_property]",
        "parent[computed_property[sub_computed_property]]",
    ];
    for c in cases {
        let mut parser = Parser::new(c, false, ExecutionTechnique::Interpretation);
        parser.produce_ast();
    }
}
