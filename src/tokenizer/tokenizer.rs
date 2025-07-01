use std::{collections::HashMap, ops::BitAnd};

use super::token::*;

fn t_peek(characters: &Vec<char>, cur_idx: usize, amount: usize) -> Option<char> {
    if cur_idx + amount >= characters.len() {
        None
    } else {
        Some(characters[cur_idx + amount])
    }
}

// TODO: once done with all basic tokenizations, error on no end path reached & enable whitespace skipping
pub fn tokenize(input: &str) -> Vec<VelvetToken> {
    let input_characters: Vec<char> = input.chars().clone().collect();
    let mut tokenizer_index = 0;
    let mut first = true;
    let mut end_tokens: Vec<VelvetToken> = Vec::new();

    let reserved_tokens: HashMap<&'static str, VelvetTokenType> = HashMap::from([
        ("bind", VelvetTokenType::Keywrd_Bind),
        ("bindm", VelvetTokenType::Keywrd_Bindmutable),
        ("as", VelvetTokenType::Keywrd_As),
        ("while", VelvetTokenType::Keywrd_While),
        ("do", VelvetTokenType::Keywrd_Do),
        ("if", VelvetTokenType::Keywrd_If),
        ("for", VelvetTokenType::Keywrd_For),
        ("of", VelvetTokenType::Keywrd_Of)
    ]);

    while tokenizer_index < input_characters.len() {
        let mut current_char = input_characters[tokenizer_index];

        if current_char.is_whitespace() {
            tokenizer_index += 1;
            continue
        }

        // Single char mapping
        let mut prefix_literal_value = "".to_owned();
        let token_result: Option<VelvetTokenType> = match current_char {
            '+' => Some(VelvetTokenType::Plus),
            '-' => {
                match t_peek(&input_characters, tokenizer_index, 1) {
                    Some('>') => { tokenizer_index += 1; prefix_literal_value = "-".to_owned(); Some(VelvetTokenType::Arrow) },
                    _ => Some(VelvetTokenType::Minus)
                }
            },
            '*' => Some(VelvetTokenType::Asterisk),
            '/' => Some(VelvetTokenType::Slash),
            '=' => {
                match t_peek(&input_characters, tokenizer_index, 1) {
                    Some('>') => { tokenizer_index += 1; prefix_literal_value = "=".to_owned(); Some(VelvetTokenType::EqArrow)},
                    Some('=') => { tokenizer_index += 1; prefix_literal_value = "=".to_owned();Some(VelvetTokenType::DoubleEq)},
                    _ => Some(VelvetTokenType::Eq)
                }
            },
            '<' => Some(VelvetTokenType::Lt),
            '>' => Some(VelvetTokenType::Gt),
            '(' => Some(VelvetTokenType::LParen),
            ')' => Some(VelvetTokenType::RParen),
            ':' => Some(VelvetTokenType::Colon),
            '{' => Some(VelvetTokenType::LBrace),
            '}' => Some(VelvetTokenType::RBrace),
            '!' => Some(VelvetTokenType::Exclaimation),
            ';' => Some(VelvetTokenType::Semicolon),
            ',' => Some(VelvetTokenType::Comma),
            '[' => Some(VelvetTokenType::LBracket),
            ']' => Some(VelvetTokenType::RBracket),
            '.' => Some(VelvetTokenType::Dot),
            _   => None
        };
        if token_result.is_some() {
            end_tokens.push(VelvetToken {
                kind: token_result.expect(""),
                start_index: tokenizer_index,
                end_index: tokenizer_index,
                literal_value: prefix_literal_value + &input_characters[tokenizer_index].to_string()
            });
            tokenizer_index += 1;
            continue;
        }

        // Multi char processing
        // Numbers
        if current_char.is_numeric() {
            let mut final_number = "".to_owned();
            let start_index = tokenizer_index;
            
            while tokenizer_index < input_characters.len()
                && input_characters[tokenizer_index].is_numeric()
            {
                final_number.push(input_characters[tokenizer_index]);
                tokenizer_index += 1;
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Number,
                start_index,
                end_index: tokenizer_index,
                literal_value: final_number
            });
            continue
        }

        // Identifiers
        if current_char.is_alphabetic() || current_char == '_' {
            let mut final_ident: String = "".to_owned();
            let start_index: usize = tokenizer_index;

            while current_char.is_alphanumeric() || current_char == '_' {
                final_ident = final_ident + &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    tokenizer_index += 1;
                    break
                }
                tokenizer_index += 1;
                current_char = input_characters[tokenizer_index]
            }

            tokenizer_index -= 1;

            let reserved_check = reserved_tokens.get(final_ident.as_str());
            if let Some(x) = reserved_check {
                end_tokens.push(VelvetToken {
                    kind: x.to_owned(),
                    start_index,
                    end_index: tokenizer_index,
                    literal_value: final_ident
                });
                tokenizer_index += 1;
                continue
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Identifier,
                start_index,
                end_index: tokenizer_index,
                literal_value: final_ident
            });
            tokenizer_index += 1;
            continue
        }

        if current_char == '\'' || current_char == '"' {
            let end_quote_char = current_char;
            let si = tokenizer_index;
            let mut end_string = "".to_owned();

            if tokenizer_index + 1 >= input_characters.len() {
                panic!("Unexpected EOF: Expected string end sequence, got EOF.")
            }
            tokenizer_index += 1;
            current_char = input_characters[tokenizer_index];

            while current_char != end_quote_char {
                end_string += &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    panic!("Unexpected EOF: Expected string end sequence, got EOF.")
                }
                tokenizer_index += 1;
                current_char = input_characters[tokenizer_index];
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Str,
                start_index: si,
                end_index: tokenizer_index,
                literal_value: end_string
            });
            tokenizer_index += 1;
            continue
        }

        // No token consumed, assume bad syntax
        panic!("Tokenizer Syntax Error: Failed to associate character {} with a token.", current_char);
    }

    return end_tokens;
}