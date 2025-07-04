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
    Keywrd_Of
}

const VTOK_EQUIV: [&str; 35] = [
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
    "Keywrd:Of"
];

impl fmt::Display for VelvetTokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", VTOK_EQUIV[*self as usize])
    }
}

#[derive(Debug, Clone)]
pub struct VelvetToken {
    pub kind: VelvetTokenType,
    pub start_index: usize,
    pub end_index: usize,
    pub literal_value: String
}