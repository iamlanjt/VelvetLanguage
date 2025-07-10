#[cfg(test)]
#[allow(warnings)]

use std::{collections::HashMap, vec};

use crate::tokenizer::{token::{VelvetToken, VelvetTokenType}, tokenizer::tokenize};

use super::super::tokenizer;

#[test]
fn tokenizer_unit_single_char() {
    let test_characters = HashMap::from([
        ('+', VelvetTokenType::Plus),
        ('-', VelvetTokenType::Minus),
        ('*', VelvetTokenType::Asterisk),
        ('/', VelvetTokenType::Slash),
        ('=', VelvetTokenType::Eq),
        ('<', VelvetTokenType::Lt),
        ('>', VelvetTokenType::Gt),
        ('(', VelvetTokenType::LParen),
        (')', VelvetTokenType::RParen),
        (':', VelvetTokenType::Colon),
        ('{', VelvetTokenType::LBrace),
        ('}', VelvetTokenType::RBrace),
        ('!', VelvetTokenType::Exclaimation),
        (';', VelvetTokenType::Semicolon),
        (',', VelvetTokenType::Comma),
        ('[', VelvetTokenType::LBracket),
        (']', VelvetTokenType::RBracket),
        ('.', VelvetTokenType::Dot)
    ]);

    for unit in &test_characters {
        let results = tokenize(&unit.0.to_string(), false);

        assert_eq!(results.real_tokens.len(), 1);

        let first = results.real_tokens.first();

        assert!(first.is_some());
        assert_eq!(first.unwrap().kind, *unit.1)
    }
}

#[test]
fn tokenizer_unit_multi_char() {
    let test_phrases = HashMap::from([
        ("456", VelvetToken {
            kind: VelvetTokenType::Number,
            literal_value: String::from("456"),
            real_size: 3,
            line: 1,
            column: 1
        }),
        ("'single_.   123  str'", VelvetToken {
            kind: VelvetTokenType::Str,
            literal_value: String::from("single_.   123  str"),
            real_size: ("single_.   123  str").len(),
            line: 1,
            column: 1
        })
    ]);

    for unit in &test_phrases {
        let result = tokenize(unit.0, false);
        
        assert_eq!(result.real_tokens.len(), 1);
        assert!(result.real_tokens.first().is_some());
        
        assert_eq!(unit.1.kind, result.real_tokens.first().unwrap().kind);
        assert_eq!(unit.1.literal_value, result.real_tokens.first().unwrap().literal_value);
    }
}

#[test]
fn tokenizer_unit_reserved_keywords() {
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

    for tkn in &reserved_tokens {
        let result = tokenize(tkn.0, false);

        assert_eq!(result.real_tokens.len(), 1);
        assert!(result.real_tokens.first().is_some());

        assert_eq!(result.real_tokens.first().unwrap().kind, *tkn.1);
    }
}

#[test]
fn tokenizer_unit_combinations() {
    let combined_tokens: HashMap<&'static str, VelvetTokenType> = HashMap::from([
        ("->", VelvetTokenType::Arrow),
        ("=>", VelvetTokenType::EqArrow),
        ("==", VelvetTokenType::DoubleEq)
    ]);

    for tkn in &combined_tokens {
        let result = tokenize(tkn.0, false);

        assert_eq!(result.real_tokens.len(), 1);
        assert!(result.real_tokens.first().is_some());

        assert_eq!(result.real_tokens.first().unwrap().kind, *tkn.1);
    }
}

#[test]
#[should_panic]
fn tokenizer_unit_unexpected_eofs() {
    let test_cases = vec![
        "'hello world!",
        "\"hello world!",
        "hello world!'",
        "hello world!\"",
        "'hello world!\"",
    ];

    for case in &test_cases {
        tokenize(case, false);
    }
}