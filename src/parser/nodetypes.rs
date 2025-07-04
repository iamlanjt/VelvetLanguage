use std::{collections::HashMap, rc::Rc};

use crate::tokenizer::token::VelvetToken;

#[derive(Debug, Clone)]
pub enum Node {
    BinaryExpr(BinaryExpr),
    NumericLiteral(NumericLiteral),
    VarDeclaration(VarDeclaration),
    AssignmentExpr(AssignmentExpr),
    Comparator(Comparator),
    ListLiteral(ListLiteral),
    ObjectLiteral(ObjectLiteral),
    FunctionDefinition(FunctionDefinition),
    Identifier(Identifier),
    Return(Return),
    CallExpr(CallExpr),
    MemberExpr(MemberExpr),
    Eof(Eof),
    WhileStmt(WhileStmt),
    StringLiteral(StringLiteral),
    IfStmt(IfStmt),
    Iterator(Iterator)
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone)]
pub struct Eof {
    
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub left: Box<Node>,
    pub value: Box<Node>
}

#[derive(Debug, Clone)]
pub struct Iterator {
    pub left: VelvetToken,
    pub right: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub params: Vec<String>,
    pub name: String,
    pub body: Rc<Vec<Box<Node>>>,
    pub return_type: String
}

#[derive(Debug, Clone)]
pub struct Comparator {
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: String
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub identifier_name: String
}

#[derive(Debug, Clone)]
pub struct Return {
    pub return_statement: Box<Node>
}

#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub is_mutable: bool,
    pub var_identifier: String,
    pub var_type: String,
    pub var_value: Box<Node>
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub args: Vec<Box<Node>>,
    pub caller: Box<Node>
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub object: Box<Node>,
    pub property: Box<Node>,
    pub is_computed: bool
}

// Literals
#[derive(Debug, Clone)]
pub struct NumericLiteral {
    pub literal_value: String
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub props: Vec<Box<Node>>
}

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub props: HashMap<String, Box<Node>>
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub literal_value: String
}