use core::fmt;
use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::{tokenizer::token::VelvetToken, typecheck::typecheck::T};

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
    Iterator(Iterator),
    MatchExpr(MatchExpr),
    BoolLiteral(BoolLiteral),
    OptionalArg(OptionalArg),
    NoOpNode(NoOpNode),
    NullishCoalescing(NullishCoalescing),
    Block(Block),
    NullLiteral(NullLiteral),
    InterpreterBlock(InterpreterBlock),
    TypeCast(TypeCast),
}
#[derive(Clone)]
pub struct AstSnippet {
    pub id: Option<usize>,
    pub name: String,
    pub args: Vec<SnippetParam>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct TypeCast {
    pub id: Option<usize>,
    pub left: Box<Node>,
    pub target_type: T,
}

#[derive(Debug, Clone)]
pub struct InterpreterBlock {
    pub id: Option<usize>,
    pub feature: String,
    pub body: Vec<Box<Node>>,
}

#[derive(Debug, Clone)]
pub struct NullLiteral {
    pub id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: Option<usize>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct NullishCoalescing {
    pub id: Option<usize>,
    pub left: Box<Node>,
    pub right: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct SnippetParam {
    pub id: Option<usize>,
    pub name: String,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub struct NoOpNode {
    pub id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct OptionalArg {
    pub id: Option<usize>,
    pub arg: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub id: Option<usize>,
    pub condition: Box<Node>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub id: Option<usize>,
    pub condition: Box<Node>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Eof {
    pub id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub id: Option<usize>,
    pub left: Box<Node>,
    pub right: Box<Node>,
    pub op: String,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub id: Option<usize>,
    pub left: Box<Node>,
    pub value: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct Iterator {
    pub id: Option<usize>,
    pub left: VelvetToken,
    pub right: Box<Node>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub id: Option<usize>,
    pub params: Vec<(String, T)>,
    pub name: String,
    pub body: Rc<Vec<Node>>,
    pub return_type: T,
}

#[derive(Debug, Clone)]
pub struct Comparator {
    pub id: Option<usize>,
    pub lhs: Box<Node>,
    pub rhs: Box<Node>,
    pub op: String,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub id: Option<usize>,
    pub identifier_name: String,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub id: Option<usize>,
    pub return_statement: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct VarDeclaration {
    pub id: Option<usize>,
    pub is_mutable: bool,
    pub var_identifier: String,
    pub var_type: T,
    pub var_value: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub id: Option<usize>,
    pub args: Vec<Node>,
    pub caller: Box<Node>,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub id: Option<usize>,
    pub target: Box<Node>,
    pub arms: Vec<(Node, Node)>,
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub id: Option<usize>,
    pub object: Box<Node>,
    pub property: Box<Node>,
    pub is_computed: bool,
}

// Literals
#[derive(Debug, Clone)]
pub struct NumericLiteral {
    pub id: Option<usize>,
    pub literal_value: String,
}

#[derive(Debug, Clone)]
pub struct BoolLiteral {
    pub id: Option<usize>,
    pub literal_value: bool,
}

#[derive(Debug, Clone)]
pub struct ListLiteral {
    pub id: Option<usize>,
    pub props: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct ObjectLiteral {
    pub id: Option<usize>,
    pub props: HashMap<String, Node>,
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub id: Option<usize>,
    pub literal_value: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::BinaryExpr(expr) => write!(f, "{} {} {}", expr.left, expr.op, expr.right),
            Node::NumericLiteral(n) => write!(f, "{}", n.literal_value),
            Node::StringLiteral(s) => write!(f, "\"{}\"", s.literal_value),
            Node::BoolLiteral(b) => write!(f, "{}", b.literal_value),
            Node::NullLiteral(_) => write!(f, "null"),
            Node::Identifier(ident) => write!(f, "{}", ident.identifier_name),
            Node::VarDeclaration(v) => {
                let mutability = if v.is_mutable { "bindm" } else { "bind" };
                write!(
                    f,
                    "{} {} as {} = {}",
                    mutability, v.var_identifier, v.var_type, v.var_value
                )
            }
            Node::AssignmentExpr(a) => write!(f, "{} = {}", a.left, a.value),
            Node::Comparator(c) => write!(f, "{} {} {}", c.lhs, c.op, c.rhs),
            Node::ListLiteral(ll) => {
                let items: Vec<String> = ll.props.iter().map(|n| n.to_string()).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Node::ObjectLiteral(obj) => {
                let props: Vec<String> = obj
                    .props
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", props.join(", "))
            }
            Node::FunctionDefinition(fd) => {
                let params: Vec<String> = fd
                    .params
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, ty))
                    .collect();
                write!(
                    f,
                    "-> {}({}) => {} {{ ... }}",
                    fd.name,
                    params.join(", "),
                    fd.return_type
                )
            }
            Node::Return(r) => write!(f, "return {}", r.return_statement),
            Node::CallExpr(call) => {
                let args: Vec<String> = call.args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", call.caller, args.join(", "))
            }
            Node::MemberExpr(m) => {
                if m.is_computed {
                    write!(f, "{}[{}]", m.object, m.property)
                } else {
                    write!(f, "{}.{}", m.object, m.property)
                }
            }
            Node::IfStmt(if_stmt) => write!(f, "if {} {{ ... }}", if_stmt.condition),
            Node::WhileStmt(ws) => write!(f, "while {} do {{ ... }}", ws.condition),
            Node::MatchExpr(me) => {
                let arms: Vec<String> = me
                    .arms
                    .iter()
                    .map(|(pat, body)| format!("{} => ...", pat))
                    .collect();
                write!(f, "match {} {{ {} }}", me.target, arms.join(", "))
            }
            Node::NullishCoalescing(nc) => write!(f, "({} ?? {})", nc.left, nc.right),
            Node::Block(b) => {
                let stmts: Vec<String> = b.body.iter().map(|n| n.to_string()).collect();
                write!(f, "{{ {} }}", stmts.join("; "))
            }
            Node::InterpreterBlock(ib) => write!(f, "#feature({}) {{ ... }}", ib.feature),
            Node::OptionalArg(o) => write!(f, "{}?", o.arg),
            Node::NoOpNode(_) => write!(f, "<no-op>"),
            Node::Eof(_) => write!(f, "<eof>"),
            Node::Iterator(it) => {
                write!(f, "for {} in {} {{ ... }}", it.left.literal_value, it.right)
            }
            Node::TypeCast(cast) => write!(f, "{}@{}", cast.left, cast.target_type),
        }
    }
}
