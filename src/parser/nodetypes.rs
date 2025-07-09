use std::{collections::HashMap, rc::Rc};
use serde::{Serialize, Deserialize};

use crate::tokenizer::token::VelvetToken;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Iterator(Iterator),
    MatchExpr(MatchExpr),
    BoolLiteral(BoolLiteral),
    OptionalArg(OptionalArg),
    NoOpNode(NoOpNode),
    NullishCoalescing(NullishCoalescing),
    Block(Block),
    NullLiteral(NullLiteral)
}
#[derive(Clone)]
pub struct AstSnippet {
    pub name: String,
    pub args: Vec<SnippetParam>, // <-- snippet *parameters*
    pub body: Vec<Box<Node>>,    // or whatever your AST body type is
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullLiteral;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullishCoalescing {
    pub left: Box<Node>,
    pub right: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnippetParam {
    pub name: String,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoOpNode {

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionalArg {
    pub arg: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhileStmt {
    pub condition: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStmt {
    pub condition: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eof {
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentExpr {
    pub left: Box<Node>,
    pub value: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Iterator {
    pub left: VelvetToken,
    pub right: Box<Node>,
    pub body: Vec<Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub params: Vec<String>,
    pub name: String,
    pub body: Rc<Vec<Box<Node>>>,
    pub return_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparator {
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub identifier_name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Return {
    pub return_statement: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarDeclaration {
    pub is_mutable: bool,
    pub var_identifier: String,
    pub var_type: String,
    pub var_value: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpr {
    pub args: Vec<Box<Node>>,
    pub caller: Box<Node>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExpr {
    pub target: Box<Node>,
    pub arms: Vec<(Box<Node>, Box<Node>)>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberExpr {
    pub object: Box<Node>,
    pub property: Box<Node>,
    pub is_computed: bool
}

// Literals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericLiteral {
    pub literal_value: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoolLiteral {
    pub literal_value: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListLiteral {
    pub props: Vec<Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectLiteral {
    pub props: HashMap<String, Box<Node>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteral {
    pub literal_value: String
}