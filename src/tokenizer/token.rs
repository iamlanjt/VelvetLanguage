use core::fmt;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct TryFromPrimitiveError {
    pub value: u8,
}

impl std::fmt::Display for TryFromPrimitiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "No matching token for discriminant {}", self.value)
    }
}

impl std::error::Error for TryFromPrimitiveError {}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    DollarSign,
    At,
    NoOp,
}

const VTOK_EQUIV: [&str; 41] = [
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
    "DollarSign",
    "At",
    "NoOp",
];

impl fmt::Display for VelvetTokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", VTOK_EQUIV[*self as usize])
    }
}

#[derive(Debug, Clone)]
pub struct VelvetToken {
    pub kind: VelvetTokenType,
    pub real_size: usize,
    pub line: usize,
    pub column: usize,
    pub literal_value: String,
}

impl TryFrom<u8> for VelvetTokenType {
    type Error = TryFromPrimitiveError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use VelvetTokenType::*;
        match value {
            0 => Ok(Plus),
            1 => Ok(Minus),
            2 => Ok(Asterisk),
            3 => Ok(Slash),
            4 => Ok(Identifier),
            5 => Ok(Number),
            6 => Ok(Eq),
            7 => Ok(Arrow),
            8 => Ok(LParen),
            9 => Ok(RParen),
            10 => Ok(Colon),
            11 => Ok(LBrace),
            12 => Ok(RBrace),
            13 => Ok(SingleQuote),
            14 => Ok(DoubleQuote),
            15 => Ok(Exclaimation),
            16 => Ok(Semicolon),
            17 => Ok(Str),
            18 => Ok(EqArrow),
            19 => Ok(Keywrd_Bindmutable),
            20 => Ok(Keywrd_Bind),
            21 => Ok(Keywrd_As),
            22 => Ok(Lt),
            23 => Ok(Gt),
            24 => Ok(DoubleEq),
            25 => Ok(Comma),
            26 => Ok(LBracket),
            27 => Ok(RBracket),
            28 => Ok(Dot),
            29 => Ok(EOF),
            30 => Ok(Keywrd_While),
            31 => Ok(Keywrd_Do),
            32 => Ok(Keywrd_If),
            33 => Ok(Keywrd_For),
            34 => Ok(Keywrd_Of),
            35 => Ok(WallArrow),
            36 => Ok(Keywrd_Match),
            37 => Ok(QuestionMark),
            38 => Ok(DollarSign),
            _ => Err(TryFromPrimitiveError { value }),
        }
    }
}
