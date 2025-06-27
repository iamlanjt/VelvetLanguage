use core::fmt;


#[derive(Debug, Copy, Clone, PartialEq)]
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
    Str
}

const VTOK_EQUIV: [&str; 18] = [
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
    "Str"
];

impl fmt::Display for VelvetTokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", VTOK_EQUIV[*self as usize])
    }
}

#[derive(Debug)]
pub struct VelvetToken {
    pub kind: VelvetTokenType,
    pub start_index: usize,
    pub end_index: usize,
    pub literal_value: String
}