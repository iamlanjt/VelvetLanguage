use crate::{parser::{nodetypes::Node, parser::Parser}, runtime::{interpreter::Interpreter, source_environment::source_environment::EnvVar}, tokenizer::tokenizer::tokenize};
use crate::{runtime::source_environment::source_environment::SourceEnv};
use std::{fs, rc::Rc};
use std::time::Instant;

mod tokenizer;
mod parser;
mod runtime;

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
            for sub_node in f.body.as_ref() {
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

fn print_env_var(name: &String, var: &EnvVar, depth: usize) {
    let indent = "    ".repeat(depth);

    println!("{} as {}{}  =  {:#?}", name, if var.is_mutable { "mutable " } else { "" }, var.var_type, var.value)
}

fn main() {
    let contents = fs::read_to_string("./src/testFile.vel")
        .expect("Should have been able to read the file");

    println!("[Program Step 1] Starting Lexer");
    let lexer_start_time = Instant::now();
    let tokenizer_result = tokenize(&contents);
    println!("[Program Step 1] Lexical analysis finished in {:.2?}\n", lexer_start_time.elapsed());
    
    for this_token in &tokenizer_result {
        let label = format!("Token {}", this_token.kind);
        println!("{:width$} {}", label, this_token.literal_value, width = 25);
    }

    println!("\n[Program Step 2] Starting Parser\n    * token bucket size: {} Token(s)", tokenizer_result.len());

    let parser_start_time = Instant::now();
    let mut parser = Parser::new(&contents);
    let result = parser.produce_ast();
    
    println!("[Program Step 2] AST Generation finished in {:.2?}\n", parser_start_time.elapsed());

    for node in &result {
        print_node(&node, 0);
    }

    println!("\n[Program Step 3] Starting interpreter for AST evaluation\n    * using default initial global environment");
    println!("    * interpreter is running in early-version mode; the last expression evaluation will be printed to stdout");

    let mut interp = Interpreter::new(result);
    let mut this_env = SourceEnv::create_global();
    println!("\nBelow this line is any stdout from evaluation\n----");
    let interp_result = interp.evaluate_body(Rc::clone(&this_env));
    println!("\n----\nAbove this line is any stdout from evaluation\n");
    println!("\n** EVALUATION RESULTS **\n  {:?}", interp_result);
    println!("\n\nDumping environment in debug mode");
    let env_ref = this_env.borrow(); // borrow lives long enough
    let variables: Vec<_> = env_ref.variables.iter().collect();

    for (name, var) in variables {
        print_env_var(name, var, 0);
    }


    println!("Program took {:.2?} ({}ms) from Lexing to Evaluation", lexer_start_time.elapsed(), lexer_start_time.elapsed().as_millis());
}
