use crate::{parser::{nodetypes::Node, parser::Parser}, runtime::{interpreter::Interpreter, source_environment::source_environment::EnvVar}, tokenizer::tokenizer::tokenize};
use crate::{runtime::source_environment::source_environment::SourceEnv};
use std::{fs::{self, File}, io::Write, rc::Rc};
use std::time::Instant;
use std::env;

mod tokenizer;
mod parser;
mod runtime;
mod tests;

fn read_ast_from_str(s: &str) -> Vec<Box<Node>> {
    let ast: Vec<Box<Node>> = serde_json::from_str(&s).expect("Deserialization failed");
    ast
}

fn print_node(node: &Box<Node>, depth: usize) {
    let indent = "    ".repeat(depth);

    match node.as_ref() {
        Node::Block(b) => {
            for x in b.body.clone() {
                print_node(&x, depth)
            }
        }
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
        Node::IfStmt(i) => {
            println!("{}->if", indent);
            println!("{}    condition:", indent);
            print_node(&i.condition, depth + 2);
            println!("{}    if body:", indent);
            for sub_node in i.body.clone() {
                print_node(&sub_node, depth + 2);
            }
        }
        Node::Return(r) => {
            println!("{}->return", indent);
            println!("{}    return stmt expanded:", indent);
            print_node(&r.return_statement, depth + 2);
        }
        Node::CallExpr(cexpr) => {
            println!("{}->call", indent);
            println!("{}    target:", indent);
            print_node(&cexpr.caller, depth + 2);
            println!("{}    args:", indent);
            for arg in cexpr.args.clone() {
                print_node(&arg, depth + 2);
            }
        }
        Node::StringLiteral(strlit) => {
            println!("{}->stringliteral \"{}\"", indent, strlit.literal_value);
        }
        Node::Identifier(ident) => {
            println!("{}->identifier {}", indent, ident.identifier_name);
        }
        _ => {
            println!("{}Unknown: {:?}", indent, node)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("The Velvet REPL is not released yet! Please provide a file to execute.")
    }

    let file_path = args[1].clone();
    let contents = fs::read_to_string(&file_path);
    if contents.is_err() {
        panic!("Unable to execute Velvet file: {:#?}", contents)
    }

    let inject_stdlib_snippets = if args.iter().find(|p| {
        *p.to_lowercase() == *"no_stdlib_snippets"
    }).is_some() {
        false
    } else {
        true
    };

    let is_sandboxed = if args.iter().find(|p| {
        *p.to_lowercase() == *"sandbox"
    }).is_some() {
        true
    } else {
        false
    };

    let compile_json_ast = if args.iter().find(|p| {
        *p.to_lowercase() == *"compile_json_ast"
    }).is_some() {
        true
    } else {
        false
    };

    if file_path.ends_with(".imvel") {
        let ast = read_ast_from_str(&contents.unwrap());

        let mut interp = Interpreter::new(ast);
        interp.evaluate_body(SourceEnv::create_global(is_sandboxed));
        return
    };

    let mut parser = Parser::new(&contents.unwrap().as_ref(), inject_stdlib_snippets);
    let ast = parser.produce_ast();

    if compile_json_ast {
        let json_version = serde_json::to_string(&ast).expect("Serialization of AST failed.");

        let file = File::create("./json_ast.imvel");
        file.unwrap().write_all(json_version.as_bytes()).expect("Failed to write to file");
        println!("Wrote Intermediate Velvet File to ./json_ast.imvel.");
        std::process::exit(0);
    }

    if args.iter().find(|p| {
        *p.to_lowercase() == *"do_dump_ast"
    }).is_some() {
        println!("[AST Dump]");
        for inner_node in &ast {
            print_node(inner_node, 0);
        }
    }
    let mut interp = Interpreter::new(ast);
    interp.evaluate_body(SourceEnv::create_global(is_sandboxed));
}

/*
// The following code is commented out in production, but is used in development to debug all aspects of Velvet.

const DO_DUMP_TOKENS: bool = false;
const DO_DUMP_AST: bool = false;
const DO_DUMP_EVAL_RESULTS: bool = false;
const DO_DUMP_ENV: bool = false;

// this function is cringe and will be removed in the event this language is released
// consider it debug, and in place for actual Display fmt implementations

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
    
    if DO_DUMP_TOKENS {
        for this_token in &tokenizer_result {
            let label = format!("Token {}", this_token.kind);
            println!("{:width$} {}", label, this_token.literal_value, width = 25);
        }
    }

    println!("\n[Program Step 2] Starting Parser\n    * token bucket size: {} Token(s)", tokenizer_result.len());

    let parser_start_time = Instant::now();
    let mut parser = Parser::new(&contents);
    let result = parser.produce_ast();
    
    println!("[Program Step 2] AST Generation finished in {:.2?}\n", parser_start_time.elapsed());

    if DO_DUMP_AST {
        for node in &result {
            print_node(&node, 0);
        }
    }

    println!("\n[Program Step 3] Starting interpreter for AST evaluation\n    * using default initial global environment");
    println!("    * interpreter is running in early-version mode; the last expression evaluation will be printed to stdout");

    let mut interp = Interpreter::new(result);
    let mut this_env = SourceEnv::create_global();
    if DO_DUMP_EVAL_RESULTS {
        println!("\nBelow this line is any stdout from evaluation\n----");
    }
    println!("\n");
    let interp_result = interp.evaluate_body(Rc::clone(&this_env));
    println!("\n");
    if DO_DUMP_EVAL_RESULTS {
        println!("\n----\nAbove this line is any stdout from evaluation\n");
        println!("\n** EVALUATION RESULTS **\n  {:?}", interp_result);
    }

    if DO_DUMP_ENV {
        println!("\n\nDumping environment in debug mode");
        let env_ref = this_env.borrow(); // borrow lives long enough
        let variables: Vec<_> = env_ref.variables.iter().collect();

        for (name, var) in variables {
            print_env_var(name, var, 0);
        }
    }

    println!("Program took {:.2?} ({}ms) from Lexing to Evaluation", lexer_start_time.elapsed(), lexer_start_time.elapsed().as_millis());
}

*/