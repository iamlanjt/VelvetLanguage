#[derive(Debug)]
pub enum Node {
    BinaryExpr(BinaryExpr),
    NumericLiteral(NumericLiteral)
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String
}

#[derive(Debug)]
pub struct NumericLiteral {
    pub literal_value: String
}