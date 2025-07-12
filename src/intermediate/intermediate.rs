use std::{io::{self, Write, Read}};
use crate::parser::nodetypes::{AssignmentExpr, BinaryExpr, CallExpr, Identifier, Node, NumericLiteral, VarDeclaration};

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKindId {
    BinaryExpr = 0,
    NumericLiteral,
    VarDeclaration,
    AssignmentExpr,
    Comparator,
    ListLiteral,
    ObjectLiteral,
    FunctionDefinition,
    Identifier,
    Return,
    CallExpr,
    MemberExpr,
    Eof,
    WhileStmt,
    StringLiteral,
    IfStmt,
    Iterator,
    MatchExpr,
    BoolLiteral,
    OptionalArg,
    NoOpNode,
    NullishCoalescing,
    Block,
    NullLiteral,
}

impl From<&Node> for NodeKindId {
    fn from(node: &Node) -> Self {
        match node {
            Node::BinaryExpr(_) => NodeKindId::BinaryExpr,
            Node::NumericLiteral(_) => NodeKindId::NumericLiteral,
            Node::VarDeclaration(_) => NodeKindId::VarDeclaration,
            Node::AssignmentExpr(_) => NodeKindId::AssignmentExpr,
            Node::Comparator(_) => NodeKindId::Comparator,
            Node::ListLiteral(_) => NodeKindId::ListLiteral,
            Node::ObjectLiteral(_) => NodeKindId::ObjectLiteral,
            Node::FunctionDefinition(_) => NodeKindId::FunctionDefinition,
            Node::Identifier(_) => NodeKindId::Identifier,
            Node::Return(_) => NodeKindId::Return,
            Node::CallExpr(_) => NodeKindId::CallExpr,
            Node::MemberExpr(_) => NodeKindId::MemberExpr,
            Node::Eof(_) => NodeKindId::Eof,
            Node::WhileStmt(_) => NodeKindId::WhileStmt,
            Node::StringLiteral(_) => NodeKindId::StringLiteral,
            Node::IfStmt(_) => NodeKindId::IfStmt,
            Node::Iterator(_) => NodeKindId::Iterator,
            Node::MatchExpr(_) => NodeKindId::MatchExpr,
            Node::BoolLiteral(_) => NodeKindId::BoolLiteral,
            Node::OptionalArg(_) => NodeKindId::OptionalArg,
            Node::NoOpNode(_) => NodeKindId::NoOpNode,
            Node::NullishCoalescing(_) => NodeKindId::NullishCoalescing,
            Node::Block(_) => NodeKindId::Block,
            Node::NullLiteral(_) => NodeKindId::NullLiteral,
        }
    }
}

impl NodeKindId {
    pub fn from_u16(value: u16) -> Option<Self> {
        use NodeKindId::*;
        Some(match value {
            0 => BinaryExpr,
            1 => NumericLiteral,
            2 => VarDeclaration,
            3 => AssignmentExpr,
            4 => Comparator,
            5 => ListLiteral,
            6 => ObjectLiteral,
            7 => FunctionDefinition,
            8 => Identifier,
            9 => Return,
            10 => CallExpr,
            11 => MemberExpr,
            12 => Eof,
            13 => WhileStmt,
            14 => StringLiteral,
            15 => IfStmt,
            16 => Iterator,
            17 => MatchExpr,
            18 => BoolLiteral,
            19 => OptionalArg,
            20 => NoOpNode,
            21 => NullishCoalescing,
            22 => Block,
            23 => NullLiteral,
            _ => return None,
        })
    }

    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

pub fn write_node_kind<W: Write>(writer: &mut W, kind: NodeKindId) -> io::Result<()> {
    writer.write_all(&kind.to_u16().to_be_bytes())
}

pub fn read_node_kind<R: Read>(reader: &mut R) -> io::Result<NodeKindId> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    let val = u16::from_be_bytes(buf);
    NodeKindId::from_u16(val).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Unknown node kind: {}", val))
    })
}

pub fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut tag = [0u8];
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::LiteralStr.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected LiteralStr signifier"));
    }

    let mut len = [0u8];
    reader.read_exact(&mut len)?;
    let mut buf = vec![0u8; len[0] as usize];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

pub fn read_number<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut tag = [0u8];
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::LiteralNum.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected LiteralNum signifier"));
    }

    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_be_bytes(buf))
}

pub fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut tag = [0u8];
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::LiteralBool.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected LiteralBool signifier"));
    }

    let mut val = [0u8];
    reader.read_exact(&mut val)?;
    match val[0] {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid boolean value")),
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signifier {
    StartNode = 0x01,
    EndNode = 0x02,
    LiteralStr = 0x03,
    LiteralNum = 0x04,
    LiteralBool = 0x05,
}

impl Signifier {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Signifier::StartNode),
            0x02 => Some(Signifier::EndNode),
            0x03 => Some(Signifier::LiteralStr),
            0x04 => Some(Signifier::LiteralNum),
            0x05 => Some(Signifier::LiteralBool),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

pub fn write_string<W: Write>(writer: &mut W, value: &str) {
    let bytes = value.as_bytes();
    let len = bytes.len();

    if len > 255 {
        panic!("String too long to serialize ({} > 255)", len);
    }

    writer.write_all(&[Signifier::LiteralStr.to_byte()]).unwrap();
    writer.write_all(&[len as u8]).unwrap();
    writer.write_all(bytes).unwrap();
}

pub fn write_number<W: Write>(writer: &mut W, value: f64) -> io::Result<()> {
    writer.write_all(&[Signifier::LiteralNum.to_byte()])?;
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

pub fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[Signifier::LiteralBool.to_byte()])?;
    writer.write_all(&[value as u8])?;
    Ok(())
}

pub fn serialize_node<W: Write>(writer: &mut W, node: &Node) {
    // signify start of a new node
    writer.write_all(&[Signifier::StartNode.to_byte()]);

    let kind = NodeKindId::from(node);
    write_node_kind(writer, kind);
    match node {
        Node::BinaryExpr(b) => {
            // write left
            serialize_node(writer, &b.left);

            // write right
            serialize_node(writer, &b.right);

            // write operator
            write_string(writer, &b.op);
        }
        Node::NumericLiteral(n) => {
            write_number(writer, n.literal_value.parse::<f64>().unwrap());
        }
        Node::VarDeclaration(vdecl) => {
            // write mutability
            write_bool(writer, vdecl.is_mutable);
            
            // write identifier
            write_string(writer, &vdecl.var_identifier);
            
            // write type
            write_string(writer, &vdecl.var_type);

            // write var value
            serialize_node(writer, &vdecl.var_value);
        }
        Node::AssignmentExpr(assign) => {
            // write left
            serialize_node(writer, &assign.left);

            // write right / value
            serialize_node(writer, &assign.value);
        }
        Node::Comparator(comp) => {
            // write lhs
            serialize_node(writer, &comp.lhs);

            // write rhs
            serialize_node(writer, &comp.rhs);

            // write op
            write_string(writer, &comp.op);
        }
        Node::ListLiteral(ll) => {
            // write prop count for deserializer
            write_number(writer, ll.props.len() as f64);

            for prop in &ll.props {
                serialize_node(writer, &prop);
            }
        }
        Node::ObjectLiteral(obj) => {
            // write prop count for deserializer
            write_number(writer, obj.props.len() as f64);

            for prop in &obj.props {
                // write key name 
                write_string(writer, prop.0.as_str());
                
                // write actual prop
                serialize_node(writer, prop.1);
            }
        }
        Node::CallExpr(cexpr) => {
            // write arg count
            write_number(writer, cexpr.args.len() as f64);

            for arg in &cexpr.args {
                // write arg
                serialize_node(writer, &arg);
            }

            // write caller
            serialize_node(writer, &cexpr.caller);
        }
        Node::Identifier(ident) => {
            // write name
            write_string(writer, &ident.identifier_name);
        }
        _ => todo!("Match node {:?} in serializer.", node)
    }

    writer.write_all(&[Signifier::EndNode.to_byte()]);
}

pub fn deserialize_node<R: Read>(reader: &mut R) -> io::Result<Node> {
    // 1. Expect START_NODE
    let mut tag = [0u8];
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::StartNode.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected START_NODE"));
    }

    // 2. Read the node kind
    let kind = read_node_kind(reader)?;

    // 3. Match kind → decode fields
    let node = match kind {
        NodeKindId::BinaryExpr => {
            let left = Box::new(deserialize_node(reader)?);
            let right = Box::new(deserialize_node(reader)?);
            let op = read_string(reader)?;

            Node::BinaryExpr(BinaryExpr { left, right, op })
        }

        NodeKindId::NumericLiteral => {
            let value = read_number(reader)?;
            Node::NumericLiteral(NumericLiteral {
                literal_value: value.to_string(),
            })
        }

        NodeKindId::VarDeclaration => {
            let mutability = read_bool(reader)?;
            let ident = read_string(reader)?;
            let t = read_string(reader)?;
            let val = Box::new(deserialize_node(reader)?);

            Node::VarDeclaration(VarDeclaration {is_mutable: mutability, var_identifier: ident, var_type: t, var_value: val})
        }

        NodeKindId::AssignmentExpr => {
            let left = Box::new(deserialize_node(reader)?);
            let value = Box::new(deserialize_node(reader)?);

            Node::AssignmentExpr(AssignmentExpr { left, value })
        }

        NodeKindId::Identifier => {
            let name = read_string(reader)?;

            Node::Identifier(Identifier { identifier_name: name })
        }

        NodeKindId::CallExpr => {
            let arg_count = read_number(reader)?;
            let mut args: Vec<Box<Node>> = Vec::new();
            
            while args.len() < arg_count as usize {
                args.push(Box::new(deserialize_node(reader)?));
            };

            let caller = Box::new(deserialize_node(reader)?);
            
            Node::CallExpr(CallExpr { args, caller })
        }

        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unimplemented node kind: {:?}", kind))),
    };

    // 4. Expect END_NODE
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::EndNode.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected END_NODE"));
    }

    Ok(node)
}

