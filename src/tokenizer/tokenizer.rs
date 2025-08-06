use std::collections::HashMap;

use colored::Colorize;

use crate::parser::parser::ExecutionTechnique;

use super::token::*;

use std::{env, fs};

fn t_peek(characters: &Vec<char>, cur_idx: usize, amount: usize) -> Option<char> {
    if cur_idx + amount >= characters.len() {
        None
    } else {
        Some(characters[cur_idx + amount])
    }
}

pub fn join_tokenizer_output(output: TokenizerOutput) -> Vec<VelvetToken> {
    let cloned_real = output.real_tokens.clone();
    let mut basin_snippets: Vec<VelvetToken> = Vec::new();

    for snippet_tgroup in output.snippet_tokens {
        for t in snippet_tgroup {
            basin_snippets.push(t)
        }
    }

    for joined_snippet_token in cloned_real {
        basin_snippets.push(joined_snippet_token);
    }

    basin_snippets
}

// Notice; fix from 0.1.15 and up, uses a static path based on EXE rather than CWD due to Bug 004.
// Jul 14 patch
pub fn load_snippet_sources(etech: ExecutionTechnique) -> Vec<String> {
    let mut sources = Vec::new();

    let exe_path = env::current_exe().expect("Failed to get current exe path");
    let snippets_path = match etech {
        ExecutionTechnique::Interpretation => exe_path
            .parent()
            .unwrap()
            .join("../../src/stdlib_interp/snippets"),
        ExecutionTechnique::Compilation => exe_path
            .parent()
            .unwrap()
            .join("../../src/stdlib_comp/snippets"),
    };
    let snippets_path = snippets_path.canonicalize().unwrap_or(snippets_path);

    if let Ok(entries) = fs::read_dir(&snippets_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("vel") {
                if let Ok(source) = fs::read_to_string(&path) {
                    sources.push(source);
                }
            }
        }
    }

    sources
}

pub struct TokenizerOutput {
    pub real_tokens: Vec<VelvetToken>,
    pub snippet_tokens: Vec<Vec<VelvetToken>>,
}

fn tokenizer_error(src: &str, err: &str, line: usize, column: usize) {
    let target_line = *src.split('\n').collect::<Vec<_>>().get(line).unwrap();
    println!("{}", String::from("Velvet Syntax Error\n").red());
    println!("{}: {}", String::from("error").red().bold(), err.bold());
    println!("{}", String::from("   |").bright_yellow());
    println!(
        "{}  {}",
        format!(
            "{}",
            String::from(" ".to_owned() + &line.to_string() + " |")
        )
        .bright_yellow(),
        target_line
    );
    println!(
        "{}  {}",
        String::from("   |").bright_yellow(),
        format!("{}^ {}", " ".repeat(column - 1), err).red().bold()
    );
    panic!("Unrecoverable syntax error. See above.");
}

pub fn tokenize(
    input: &str,
    inject_stdlib_snippets: bool,
    etech: ExecutionTechnique,
) -> TokenizerOutput {
    // Represent standard lib snippets as separate groups to not mess up idx/line/col source backtracking
    let mut snippet_tokens: Vec<Vec<VelvetToken>> = Vec::new();
    if inject_stdlib_snippets {
        let snippets = load_snippet_sources(etech.clone());

        for snippet in snippets {
            snippet_tokens.push(tokenize(&snippet, false, etech.clone()).real_tokens);
        }
    }
    let input_characters: Vec<char> = input.chars().clone().collect();
    let mut tokenizer_index = 0;
    let mut tokenizer_line: usize = 1;
    let mut tokenizer_column: usize = 0;
    let mut end_tokens: Vec<VelvetToken> = Vec::new();

    let reserved_tokens: HashMap<&'static str, VelvetTokenType> = HashMap::from([
        ("bind", VelvetTokenType::Keywrd_Bind),
        ("bindm", VelvetTokenType::Keywrd_Bindmutable),
        ("as", VelvetTokenType::Keywrd_As),
        ("while", VelvetTokenType::Keywrd_While),
        ("do", VelvetTokenType::Keywrd_Do),
        ("if", VelvetTokenType::Keywrd_If),
        ("for", VelvetTokenType::Keywrd_For),
        ("of", VelvetTokenType::Keywrd_Of),
        ("match", VelvetTokenType::Keywrd_Match),
        ("extern", VelvetTokenType::Keywrd_External),
        ("ext", VelvetTokenType::Keywrd_External),
    ]);

    while tokenizer_index < input_characters.len() {
        let mut current_char = input_characters[tokenizer_index];

        tokenizer_column += 1;

        let start_col = tokenizer_column;

        if current_char == '\n' || current_char == '\r' {
            tokenizer_line += 1;
            tokenizer_column = 0;
        }

        if current_char.is_whitespace() {
            tokenizer_index += 1;
            continue;
        }

        // Single char mapping
        let mut prefix_literal_value = "".to_owned();
        let token_result: Option<VelvetTokenType> = match current_char {
            '@' => Some(VelvetTokenType::At),
            '|' => match t_peek(&input_characters, tokenizer_index, 1) {
                Some('-') => match t_peek(&input_characters, tokenizer_index, 2) {
                    Some('>') => {
                        tokenizer_index += 2;
                        tokenizer_column += 2;
                        prefix_literal_value = "|-".to_owned();
                        Some(VelvetTokenType::WallArrow)
                    }
                    _ => None,
                },
                _ => None,
            },
            '+' => Some(VelvetTokenType::Plus),
            '-' => match t_peek(&input_characters, tokenizer_index, 1) {
                Some('>') => {
                    tokenizer_index += 1;
                    tokenizer_column += 1;
                    prefix_literal_value = "-".to_owned();
                    Some(VelvetTokenType::Arrow)
                }
                _ => Some(VelvetTokenType::Minus),
            },
            '*' => Some(VelvetTokenType::Asterisk),
            '/' => Some(VelvetTokenType::Slash),
            '=' => match t_peek(&input_characters, tokenizer_index, 1) {
                Some('>') => {
                    tokenizer_index += 1;
                    tokenizer_column += 1;
                    prefix_literal_value = "=".to_owned();
                    Some(VelvetTokenType::EqArrow)
                }
                Some('=') => {
                    tokenizer_index += 1;
                    tokenizer_column += 1;
                    prefix_literal_value = "=".to_owned();
                    Some(VelvetTokenType::DoubleEq)
                }
                _ => Some(VelvetTokenType::Eq),
            },
            '<' => Some(VelvetTokenType::Lt),
            '>' => Some(VelvetTokenType::Gt),
            '(' => Some(VelvetTokenType::LParen),
            ')' => Some(VelvetTokenType::RParen),
            ':' => Some(VelvetTokenType::Colon),
            '{' => Some(VelvetTokenType::LBrace),
            '}' => Some(VelvetTokenType::RBrace),
            '!' => Some(VelvetTokenType::Exclaimation),
            ';' => {
                if let Some(t) = t_peek(&input_characters, tokenizer_index, 1) {
                    if t == ';' {
                        // It's a comment, skip until newline
                        while current_char != '\n' {
                            tokenizer_index += 1;
                            tokenizer_column += 1;
                            current_char = input_characters[tokenizer_index];
                        }
                        Some(VelvetTokenType::NoOp)
                    } else {
                        Some(VelvetTokenType::Semicolon)
                    }
                } else {
                    Some(VelvetTokenType::Semicolon)
                }
            }

            ',' => Some(VelvetTokenType::Comma),
            '[' => Some(VelvetTokenType::LBracket),
            ']' => Some(VelvetTokenType::RBracket),
            '.' => Some(VelvetTokenType::Dot),
            '?' => Some(VelvetTokenType::QuestionMark),
            '$' => Some(VelvetTokenType::DollarSign),
            _ => None,
        };
        if token_result.is_some() {
            end_tokens.push(VelvetToken {
                kind: token_result.expect(""),
                literal_value: prefix_literal_value
                    + &input_characters[tokenizer_index].to_string(),
                real_size: 1,
                line: tokenizer_line,
                column: start_col,
            });
            tokenizer_index += 1;
            continue;
        }

        // Multi char processing
        // Numbers
        if current_char.is_numeric() {
            let mut final_number = "".to_owned();

            while tokenizer_index < input_characters.len()
                && input_characters[tokenizer_index].is_numeric()
            {
                final_number.push(input_characters[tokenizer_index]);
                tokenizer_index += 1;
                tokenizer_column += 1;
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Number,
                literal_value: final_number.clone(),
                real_size: final_number.len(),
                line: tokenizer_line,
                column: start_col,
            });
            continue;
        }

        // Identifiers
        if current_char.is_alphabetic() || current_char == '_' {
            let mut final_ident: String = "".to_owned();

            while current_char.is_alphanumeric() || current_char == '_' || current_char == '#' {
                final_ident = final_ident + &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    tokenizer_index += 1;
                    break;
                }
                tokenizer_index += 1;
                tokenizer_column += 1;
                current_char = input_characters[tokenizer_index]
            }

            tokenizer_index -= 1;
            tokenizer_column -= 1;

            let reserved_check = reserved_tokens.get(final_ident.as_str());
            if let Some(x) = reserved_check {
                end_tokens.push(VelvetToken {
                    kind: x.to_owned(),
                    literal_value: final_ident.clone(),
                    real_size: final_ident.len(),
                    line: tokenizer_line,
                    column: start_col,
                });
                tokenizer_index += 1;
                continue;
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Identifier,
                literal_value: final_ident.clone(),
                real_size: final_ident.len(),
                line: tokenizer_line,
                column: start_col,
            });
            tokenizer_index += 1;
            continue;
        }

        if current_char == '\'' || current_char == '"' {
            let end_quote_char = current_char;
            let mut end_string = "".to_owned();

            if tokenizer_index + 1 >= input_characters.len() {
                tokenizer_error(
                    input,
                    "Unexpected EOF: Expected string end sequence, got EOF.",
                    tokenizer_line - 1,
                    tokenizer_column,
                );
            }
            tokenizer_index += 1;
            tokenizer_column += 1;
            current_char = input_characters[tokenizer_index];

            while current_char != end_quote_char {
                if current_char == '\n' || current_char == '\r' {
                    tokenizer_error(
                        input,
                        "Unexpected Newline: Strings cannot span multiple lines.",
                        tokenizer_line - 1,
                        tokenizer_column,
                    );
                }
                end_string += &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    tokenizer_error(
                        input,
                        "Unexpected EOF: Expected string end sequence, got EOF.",
                        tokenizer_line - 1,
                        tokenizer_column,
                    );
                }
                tokenizer_index += 1;
                tokenizer_column += 1;
                current_char = input_characters[tokenizer_index];
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Str,
                literal_value: end_string.clone(),
                real_size: end_string.len() + 2, // +2 for start and end sequences
                line: tokenizer_line,
                column: start_col,
            });
            tokenizer_index += 1;
            tokenizer_column += 1;
            continue;
        }

        // No token consumed, assume bad syntax
        tokenizer_error(
            input,
            &format!(
                "Tokenizer Syntax Error: Failed to associate character {} with a token.",
                current_char
            ),
            tokenizer_line - 1,
            tokenizer_column,
        );
    }

    return TokenizerOutput {
        real_tokens: end_tokens,
        snippet_tokens: snippet_tokens,
    };
}
