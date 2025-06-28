#[derive(Debug)]
pub enum Node {
    BinaryExpr(BinaryExpr),
    NumericLiteral(NumericLiteral),
    VarDeclaration(VarDeclaration)
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String
}

#[derive(Debug)]
pub struct VarDeclaration {
    pub is_mutable: bool,
    pub var_identifier: String,
    pub var_type: String,
    pub var_value: Box<Node>
}

// Literals
#[derive(Debug)]
pub struct NumericLiteral {
    pub literal_value: String
}