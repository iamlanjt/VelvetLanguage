pub enum VelvetTokenType {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Identifier,
    Number
}

pub struct VelvetToken {
    kind: VelvetTokenType,
    startIndex: i64,
    endIndex: i64,
    literalValue: String
}