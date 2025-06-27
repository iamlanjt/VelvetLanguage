use crate::token::{VelvetToken, VelvetTokenType};

mod token;

// TODO: once done with all basic tokenizations, error on no end path reached & enable whitespace skipping
fn tokenize(input: &str) -> Vec<VelvetToken> {
    let input_characters: Vec<char> = input.chars().clone().collect();
    let mut tokenizer_index = 0;
    let mut first = true;
    let mut end_tokens: Vec<VelvetToken> = Vec::new();

    while tokenizer_index < input_characters.len() - 1 {
        if first == false { tokenizer_index = tokenizer_index + 1; }
        first = false;
        let mut current_char = input_characters[tokenizer_index];

        if current_char.is_whitespace() {continue}

        // Single char mapping
        let token_result: Option<VelvetTokenType> = match current_char {
            '+' => Some(VelvetTokenType::Plus),
            '-' => Some(VelvetTokenType::Minus),
            '*' => Some(VelvetTokenType::Asterisk),
            '/' => Some(VelvetTokenType::Slash),
            '=' => Some(VelvetTokenType::Eq),
            _   => None
        };
        if token_result.is_some() {
            end_tokens.push(VelvetToken {
                kind: token_result.expect(""),
                start_index: tokenizer_index,
                end_index: tokenizer_index,
                literal_value: current_char.to_string()
            });
            continue;
        }

        // Multi char processing
        // Numbers
        if current_char.is_numeric() && current_char != '0' {
            let mut final_number = "".to_owned();
            let start_index = tokenizer_index;
            
            while current_char.is_numeric() {
                final_number = final_number + &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    break
                }
                tokenizer_index += 1;
                current_char = input_characters[tokenizer_index]
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Number,
                start_index,
                end_index: tokenizer_index,
                literal_value: final_number
            })
        }

        // Identifiers
        if current_char.is_alphabetic() {
            let mut final_ident: String = "".to_owned();
            let start_index: usize = tokenizer_index;

            while current_char.is_alphanumeric() || current_char == '_' {
                final_ident = final_ident + &current_char.to_string();
                if tokenizer_index + 1 >= input_characters.len() {
                    break
                }
                tokenizer_index += 1;
                current_char = input_characters[tokenizer_index]
            }

            end_tokens.push(VelvetToken {
                kind: VelvetTokenType::Identifier,
                start_index,
                end_index: tokenizer_index,
                literal_value: final_ident
            })
        }
    }

    return end_tokens;
}

fn main() {
    let tokenizer_result = tokenize("const x = 1 + 2");
    for this_token in tokenizer_result {
        println!("Token {}  {}", this_token.kind, this_token.literal_value)
    }
}
