use colored::Colorize;

use crate::parser::parser::ExecutionTechnique;
use crate::runtime::source_environment::source_environment::SourceEnv;
use crate::typecheck::typecheck::TypeChecker;
use crate::{
    parser::{nodetypes::Node, parser::Parser},
    runtime::interpreter::Interpreter,
    tokenizer::tokenizer::tokenize,
};
use std::fs::{self};
use std::process::Command;
use std::time::Instant;
use std::{env, process};

mod codegen;
mod parser;
mod runtime;
mod tokenizer;
mod typecheck;
#[macro_use]
mod stdlib_interp;
mod tests;

fn print_node(node: &Node, depth: usize) {
    let indent = "    ".repeat(depth);

    match node {
        Node::Block(b) => {
            for x in b.body.clone() {
                print_node(&Box::new(x), depth)
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
            println!(
                "{}{}->Binding: {}",
                indent,
                if decl.is_mutable { "Mutable" } else { "" },
                decl.var_type
            );
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
                print_node(&Box::new(sub_node.clone()), depth + 2);
            }
        }
        Node::IfStmt(i) => {
            println!("{}->if", indent);
            println!("{}    condition:", indent);
            print_node(&i.condition, depth + 2);
            println!("{}    if body:", indent);
            for sub_node in i.body.clone() {
                print_node(&Box::new(sub_node), depth + 2);
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
                print_node(&Box::new(arg), depth + 2);
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

    let inject_stdlib_snippets = !args
        .iter()
        .any(|p| *p.to_lowercase() == *"no_stdlib_snippets");

    let is_sandboxed = args.iter().any(|p| *p.to_lowercase() == *"sandbox");

    let compile_ir = args.iter().any(|p| *p.to_lowercase() == *"compile");

    let do_coerce = args.iter().any(|p| *p.to_lowercase() == *"cmp-do-coerce");

    let contents = fs::read_to_string(&file_path)
        .unwrap_or_else(|err| panic!("Unable to execute Velvet file: {:#?}", err));

    let mut parser = Parser::new(
        &contents,
        inject_stdlib_snippets,
        if compile_ir {
            ExecutionTechnique::Compilation
        } else {
            ExecutionTechnique::Interpretation
        },
    );
    let ast = parser.produce_ast();
    // println!("{:#?}", ast);
    let mut checker = TypeChecker::new();
    checker.enter_scope();
    for node in &ast {
        checker.check_expr(node, None);
    }

    // println!("{:#?}", checker.type_table);
    if !checker.errors.is_empty() {
        println!("Typechecking failed");
        println!("Typecheck errors;");
        for err in &checker.errors {
            eprintln!(
                "{}: {}",
                String::from("tc-err").red().bold(),
                err.message.bold()
            )
        }
        process::exit(1);
    }

    if compile_ir {
        println!("{}\n  {}", "Notice!".on_red(), "The `compile` flag is present, so Velvet will attempt to compile your input rather than interpret it.\n  The compiler is still in beta, and behavior may not be uniform between interpretation and compilation methods.".yellow());
        use crate::codegen::codegen::IRGenerator;
        let file_name = args.get(1).unwrap();
        let context = inkwell::context::Context::create();

        let compile_time_checker = TypeChecker::new();

        println!(
            "[typecheck] inferred reference table size: {} element(s)",
            checker.type_table.len()
        );
        let mut generator = IRGenerator::new(
            &context,
            file_name,
            do_coerce,
            compile_time_checker,
            checker.type_table,
        );
        let start = Instant::now();
        println!("Compiling `{}`...", file_name);

        let result = generator.generate_ir_for_nodes(ast);
        if !result {
            process::exit(1);
        }
        let finish_irgen = start.elapsed();
        let finished_irgen_t = Instant::now();

        if !fs::exists("./velvet_tmp").unwrap() {
            fs::create_dir("./velvet_tmp").expect("Failed to create temporary Velvet directory.");
            println!("write_dir -> velvet_tmp");
        }

        // let mut rng = rand::rng();
        let name = format!("velvet_raw_{}", "IR"); //rng.random_range(481..194801));
        let unoptimized_path = format!("./velvet_tmp/{}-unop.ll", name);

        generator
            .module
            .print_to_file(format!("./velvet_tmp/{}-unop.ll", name))
            .expect("Failed to generate LLVM IR file.");

        let finish_codegen = finished_irgen_t.elapsed();
        let finished_codegen_t = Instant::now();

        println!("write -> velvet_tmp/{}-unop.ll", name);

        let output = Command::new("opt")
            .args([
                "-O3",
                unoptimized_path.as_str(),
                "-S",
                "-o",
                format!("./velvet_tmp/{}-op.ll", name).as_str(),
            ])
            .output()
            .expect("Failed to run `opt`. Do you have LLVM installed?");
        let finish_optimizer = finished_codegen_t.elapsed();
        let finished_optimizer_t = Instant::now();
        println!("write -> velvet_tmp/{}-op.ll", name);

        if !output.status.success() {
            eprintln!(
                "`opt` [LLVM IR optimizer] error: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::process::exit(1);
        }

        // generator.module.print_to_stderr();

        // generator.emit_object_file("./velvet_tmp/ir.o");
        let output = Command::new("clang").args([format!("./velvet_tmp/{}-op.ll", name).as_str(), "-o", "out"]).output().expect("Failed to link your Velvet program! You may have to manually link the object file under `./velvet_tmp`.");
        let finish_linking = finished_optimizer_t.elapsed();

        if !output.status.success() {
            eprintln!("Linker error:\n{}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }

        // fs::remove_dir_all("./velvet_tmp").expect("Failed to remove temporary Velvet directory. You may have to manually remove it at `./velvet_tmp`.");

        let final_time = start.elapsed();
        println!(
            "\nCompilation finished\n| -> Intermedaite Generation  : {:?}\n| -> Code Generation          : {:?}\n| -> Optimization             : {:?}\n| -> Linking                  : {:?}\n| -> Finished in              : {:?}\n",
            finish_irgen, finish_codegen, finish_optimizer, finish_linking, final_time
        );
        println!("Executable generated to `./out`");

        std::process::exit(0);
    }

    if args.iter().any(|p| *p.to_lowercase() == *"do_dump_tokens") {
        let tokenizer_results = tokenize(
            &contents,
            inject_stdlib_snippets,
            ExecutionTechnique::Interpretation,
        );
        println!("[Stdlib Snippet Token Dump]");
        println!(
            "\t-> <ommitted {} snippet group(s): not useful for debugging; snippets available via source at https://www.github.com/iamlanjt/velvetlanguage src/stdlib/snippets>",
            tokenizer_results.snippet_tokens.len()
        );

        println!(
            "\n[Pure Token Dump]: {} item(s)",
            tokenizer_results.real_tokens.len()
        );
        let label_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| format!("Token {}", t.kind).len())
            .max()
            .unwrap_or(0)
            + 3;

        let value_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| t.literal_value.len())
            .max()
            .unwrap_or(0)
            + 3;

        let line_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| t.line.to_string().len())
            .max()
            .unwrap_or(0)
            + 2;

        let col_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| t.column.to_string().len())
            .max()
            .unwrap_or(0)
            + 2;

        let end_line_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| (t.line).to_string().len())
            .max()
            .unwrap_or(0)
            + 2;

        let end_col_width = tokenizer_results
            .real_tokens
            .iter()
            .map(|t| (t.column + t.real_size).to_string().len())
            .max()
            .unwrap_or(0)
            + 2;

        for t in tokenizer_results.real_tokens {
            let label = format!("Token {}", t.kind);

            println!(
                "    -> src {start_line:>line_w$}:{start_col:<col_w$} → {end_line:>end_line_w$}:{end_col:<end_col_w$}  {label:<label_w$} {value:<value_w$}",
                start_line = t.line,
                start_col = t.column,
                end_line = t.line,
                end_col = (t.column + t.real_size),
                label = label,
                value = t.literal_value,
                line_w = line_width,
                col_w = col_width,
                end_line_w = end_line_width,
                end_col_w = end_col_width,
                label_w = label_width,
                value_w = value_width
            );
        }
    }

    if args.iter().any(|p| *p.to_lowercase() == *"do_dump_ast") {
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
