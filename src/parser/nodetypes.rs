#[derive(Debug)]
pub enum Node {
    BinaryExpr(BinaryExpr),
    NumericLiteral(NumericLiteral),
    VarDeclaration(VarDeclaration),
    AssignmentExpr(AssignmentExpr),
    Comparator(Comparator),
    ListLiteral(ListLiteral),
    ObjectLiteral(ObjectLiteral),
    FunctionDefinition(FunctionDefinition),
    Identifier(Identifier)
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String
}

#[derive(Debug)]
pub struct AssignmentExpr {
    pub left: Box<Node>,
    pub value: Box<Node>
}

#[derive(Debug)]
pub struct FunctionDefinition {
    pub params: Vec<String>,
    pub name: String,
    pub body: Vec<Box<Node>>,
    pub return_type: String
}

#[derive(Debug)]
pub struct Comparator {
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: String
}

#[derive(Debug)]
pub struct Identifier {
    pub identifier_name: String
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

#[derive(Debug)]
pub struct ListLiteral {
    pub props: Vec<Box<Node>>
}

#[derive(Debug)]
pub struct ObjectLiteral {
    pub props: Vec<Box<Node>>
}