use crate::{parser::{nodetypes::Node, parser::Parser}, tokenizer::tokenizer::tokenize};
use std::fs;

mod tokenizer;
mod parser;

fn print_node(node: &Box<Node>, depth: usize) {
    let indent = "  ".repeat(depth); // two spaces per depth level

    match node.as_ref() {
        Node::NumericLiteral(n) => {
            println!("{}NumericLiteral: {}", indent, n.literal_value);
        }
        Node::BinaryExpr(b) => {
            println!("{}BinaryExpr: op '{}'", indent, b.op);
            println!("{}  left:", indent);
            print_node(&b.left, depth + 1);
            println!("{}  right:", indent);
            print_node(&b.right, depth + 1);
        }
    }
}

fn main() {
    let contents = fs::read_to_string("./src/testFile.vel")
        .expect("Should have been able to read the file");
    let tokenizer_result = tokenize(&contents);
    
    for this_token in tokenizer_result {
        println!("Token {}  {}", this_token.kind, this_token.literal_value)
    }

    println!("Starting parser");
    let mut parser = Parser::new(&contents);

    let result = parser.produce_ast();
    for node in result {
        print_node(&node, 0);
    }
}
