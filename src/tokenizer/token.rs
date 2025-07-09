use core::fmt;

use serde::{Deserialize, Serialize};


#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum VelvetTokenType {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Identifier,
    Number,
    Eq,
    Arrow,
    LParen,
    RParen,
    Colon,
    LBrace,
    RBrace,
    SingleQuote,
    DoubleQuote,
    Exclaimation,
    Semicolon,
    Str,
    EqArrow,
    Keywrd_Bindmutable,
    Keywrd_Bind,
    Keywrd_As,
    Lt,
    Gt,
    DoubleEq,
    Comma,
    LBracket,
    RBracket,
    Dot,
    EOF,
    Keywrd_While,
    Keywrd_Do,
    Keywrd_If,
    Keywrd_For,
    Keywrd_Of,
    WallArrow,
    Keywrd_Match,
    QuestionMark,
    DollarSign
}

const VTOK_EQUIV: [&str; 39] = [
    "Plus",
    "Minus",
    "Asterisk",
    "Slash",
    "Identifier",
    "Number",
    "Equals",
    "Arrow",
    "LParen",
    "RParen",
    "Colon",
    "LBrace",
    "RBrace",
    "SingleQuote",
    "DoubleQuote",
    "Exclaimation",
    "Semicolon",
    "Str",
    "EqArrow",
    "Keywrd:Bindmutable",
    "Keywrd:Bind",
    "Keywrd:As",
    "Lt",
    "Gt",
    "DoubleEq",
    "Comma",
    "LBracket",
    "RBracket",
    "Dot",
    "EOF",
    "Keywrd:While",
    "Keywrd:Do",
    "Keywrd:If",
    "Keywrd:For",
    "Keywrd:Of",
    "WallArrow",
    "Keywrd:Match",
    "QuestionMark",
    "DollarSign"
];

impl fmt::Display for VelvetTokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", VTOK_EQUIV[*self as usize])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VelvetToken {
    pub kind: VelvetTokenType,
    pub start_index: usize,
    pub end_index: usize,
    pub literal_value: String
}