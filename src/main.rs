use crate::{parser::{nodetypes::Node, parser::Parser}, tokenizer::tokenizer::tokenize};
use std::fs;

mod tokenizer;
mod parser;

// this function is cringe and will be removed in the event this language is released
// consider it debug, and in place for actual Display fmt implementations
fn print_node(node: &Box<Node>, depth: usize) {
    let indent = "    ".repeat(depth);

    match node.as_ref() {
        Node::NumericLiteral(n) => {
            println!("{}->NumericLiteral: {}", indent, n.literal_value);
        }
        Node::BinaryExpr(b) => {
            println!("{}->BinaryExpr: op '{}'", indent, b.op);
            println!("{}  left:", indent);
            print_node(&b.left, depth + 1);
            println!("{}  right:", indent);
            print_node(&b.right, depth + 1);
        }
        Node::VarDeclaration(decl) => {
            println!("{}{}->Binding: {}", indent, if decl.is_mutable { "Mutable" } else {""}, decl.var_type);
            println!("{}    ident_name: \"{}\"", indent, decl.var_identifier);
            println!("{}    value:", indent);
            print_node(&decl.var_value, depth + 2);
        }
        Node::FunctionDefinition(f) => {
            println!("{}->function {}()", indent, f.name);
            println!("{}    param count:     {}", indent, f.params.len());
            println!("{}    body node count: {}", indent, f.body.len());
            println!("{}    body expanded:", indent);
            for sub_node in &f.body {
                print_node(&sub_node, depth + 2);
            }
        }
        Node::Return(r) => {
            println!("{}->return", indent);
            println!("{}    return stmt expanded:", indent);
            print_node(&r.return_statement, depth + 2);
        }
        _ => {
            println!("{}Unknown: {:?}", indent, node)
        }
    }
}

fn main() {
    let contents = fs::read_to_string("./src/testFile.vel")
        .expect("Should have been able to read the file");
    let tokenizer_result = tokenize(&contents);
    
    for this_token in &tokenizer_result {
        let label = format!("Token {}", this_token.kind);
        println!("{:width$} {}", label, this_token.literal_value, width = 25);
    }

    println!("\nStarting parser\nToken bucket size: {} Token(s)\n", tokenizer_result.len());

    let mut parser = Parser::new(&contents);

    let result = parser.produce_ast();
    for node in result {
        print_node(&node, 0);
    }
}
