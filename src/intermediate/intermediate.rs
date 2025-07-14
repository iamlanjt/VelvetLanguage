use std::{collections::HashMap, io::{self, Read, Write}, rc::Rc};
use crate::{parser::nodetypes::{AssignmentExpr, BinaryExpr, Block, BoolLiteral, CallExpr, Comparator, FunctionDefinition, Identifier, IfStmt, Iterator, ListLiteral, MatchExpr, MemberExpr, Node, NullLiteral, NullishCoalescing, NumericLiteral, ObjectLiteral, OptionalArg, Return, StringLiteral, VarDeclaration, WhileStmt}, tokenizer::token::{VelvetToken, VelvetTokenType}};

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

    match tag[0] {
        x if x == Signifier::LiteralStrShort.to_byte() => {
            let mut len_buf = [0u8];
            reader.read_exact(&mut len_buf)?;
            let len = len_buf[0] as usize;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            Ok(String::from_utf8(buf).unwrap())
        }

        x if x == Signifier::LiteralStrLong.to_byte() => {
            let mut len_buf = [0u8; 4];
            reader.read_exact(&mut len_buf)?;
            let len = u32::from_be_bytes(len_buf) as usize;
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf)?;
            Ok(String::from_utf8(buf).unwrap())
        }

        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid string signifier")),
    }
}


pub fn read_number<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut tag_buf = [0u8; 1];
    reader.read_exact(&mut tag_buf)?;
    let tag = tag_buf[0];

    match Signifier::from_byte(tag) {
        Some(Signifier::NumU8) => {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            Ok(buf[0] as f64)
        }
        Some(Signifier::NumU16) => {
            let mut buf = [0u8; 2];
            reader.read_exact(&mut buf)?;
            Ok(u16::from_be_bytes(buf) as f64)
        }
        Some(Signifier::NumU32) => {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            Ok(u32::from_be_bytes(buf) as f64)
        }
        Some(Signifier::NumF64) => {
            let mut buf = [0u8; 8];
            reader.read_exact(&mut buf)?;
            Ok(f64::from_be_bytes(buf))
        }
        other => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid number tag: {:?}", other),
        )),
    }
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

pub fn write_token<W: Write>(writer: &mut W, token: &VelvetToken) -> io::Result<()> {
    writer.write_all(&[Signifier::StartToken.to_byte()])?;

    writer.write_all(&(token.kind as u16).to_be_bytes())?;

    write_string(writer, &token.literal_value)?;

    writer.write_all(&[Signifier::EndToken.to_byte()])?;
    Ok(())
}

pub fn read_token<R: Read>(reader: &mut R) -> io::Result<VelvetToken> {
    let mut byte = [0u8; 1];

    reader.read_exact(&mut byte)?;
    if byte[0] != Signifier::StartToken.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected StartToken"));
    }

    let mut kind_buf = [0u8; 2];
    reader.read_exact(&mut kind_buf)?;
    let kind_val = u16::from_be_bytes(kind_buf);
    let kind = VelvetTokenType::try_from(kind_val as u8)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Unknown token kind {}", kind_val)))?;

    let literal_value = read_string(reader)?;

    reader.read_exact(&mut byte)?;
    if byte[0] != Signifier::EndToken.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected EndToken"));
    }

    Ok(VelvetToken {
        kind,
        literal_value,
        real_size: 0,
        line: 0,
        column: 0,
    })
}



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signifier {
    StartNode = 0x01,
    EndNode = 0x02,
    LiteralStrShort = 0x03,
    NumU8 = 0x04,
    LiteralBool = 0x05,
    LiteralStrLong = 0x06,
    StartToken = 0x07,
    EndToken = 0x08,
    Null = 0x09,
    NumU16 = 0x10,
    NumU32 = 0x11,
    NumF64 = 0x12,
}

impl Signifier {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Signifier::StartNode),
            0x02 => Some(Signifier::EndNode),
            0x03 => Some(Signifier::LiteralStrShort),
            0x04 => Some(Signifier::NumU8),
            0x05 => Some(Signifier::LiteralBool),
            0x06 => Some(Signifier::LiteralStrLong),
            0x07 => Some(Signifier::StartToken),
            0x08 => Some(Signifier::EndToken),
            0x09 => Some(Signifier::Null),
            0x10 => Some(Signifier::NumU16),
            0x11 => Some(Signifier::NumU32),
            0x12 => Some(Signifier::NumF64),
            _ => None,
        }
    }

    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

pub fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let bytes = value.as_bytes();
    let len = bytes.len();

    if len <= u8::MAX as usize {
        writer.write_all(&[Signifier::LiteralStrShort.to_byte()])?;
        writer.write_all(&[len as u8])?;
    } else {
        writer.write_all(&[Signifier::LiteralStrLong.to_byte()])?;
        writer.write_all(&(len as u32).to_be_bytes())?;
    }

    writer.write_all(bytes)?;
    Ok(())
}


pub fn write_number<W: Write>(writer: &mut W, value: f64) -> io::Result<()> {
    if value.fract() != 0.0 || value < 0.0 {
        writer.write_all(&[Signifier::NumF64.to_byte()])?;
        writer.write_all(&value.to_be_bytes())?;
    } else if value <= u8::MAX as f64 {
        writer.write_all(&[Signifier::NumU8.to_byte()])?;
        writer.write_all(&[value as u8])?;
    } else if value <= u16::MAX as f64 {
        writer.write_all(&[Signifier::NumU16.to_byte()])?;
        writer.write_all(&(value as u16).to_be_bytes())?;
    } else if value <= u32::MAX as f64 {
        writer.write_all(&[Signifier::NumU32.to_byte()])?;
        writer.write_all(&(value as u32).to_be_bytes())?;
    } else {
        // Fallback to f64
        writer.write_all(&[Signifier::NumF64.to_byte()])?;
        writer.write_all(&value.to_be_bytes())?;
    }

    Ok(())
}

pub fn write_null<W: Write>(writer: &mut W) {
    writer.write_all(&[Signifier::Null.to_byte()]).unwrap();
}


pub fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[Signifier::LiteralBool.to_byte()])?;
    writer.write_all(&[value as u8])?;
    Ok(())
}

pub fn serialize_node<W: Write>(writer: &mut W, node: &Node) {
    // signify start of a new node
    writer.write_all(&[Signifier::StartNode.to_byte()]).unwrap();

    let kind = NodeKindId::from(node);
    write_node_kind(writer, kind).unwrap();
    match node {
        Node::BinaryExpr(b) => {
            // write left
            serialize_node(writer, &b.left);

            // write right
            serialize_node(writer, &b.right);

            // write operator
            write_string(writer, &b.op).unwrap();
        }
        Node::NumericLiteral(n) => {
            write_number(writer, n.literal_value.parse::<f64>().unwrap()).unwrap();
        }
        Node::VarDeclaration(vdecl) => {
            // write mutability
            write_bool(writer, vdecl.is_mutable).unwrap();
            
            // write identifier
            write_string(writer, &vdecl.var_identifier).unwrap();
            
            // write type
            write_string(writer, &vdecl.var_type).unwrap();

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
            write_string(writer, &comp.op).unwrap();
        }
        Node::ListLiteral(ll) => {
            // write prop count for deserializer
            write_number(writer, ll.props.len() as f64).unwrap();

            for prop in &ll.props {
                serialize_node(writer, &prop);
            }
        }
        Node::ObjectLiteral(obj) => {
            // write prop count for deserializer
            write_number(writer, obj.props.len() as f64).unwrap();

            for prop in &obj.props {
                // write key name 
                write_string(writer, prop.0.as_str()).unwrap();
                
                // write actual prop
                serialize_node(writer, prop.1);
            }
        }
        Node::Identifier(ident) => {
            // write name
            write_string(writer, &ident.identifier_name).unwrap();
        }
        Node::Return(r) => {
            serialize_node(writer, &r.return_statement);
        }

        Node::FunctionDefinition(fdef) => {
            write_number(writer, fdef.params.len() as f64).unwrap();
            for b in &fdef.params {
                write_string(writer, b).unwrap();
            }

            write_string(writer, &fdef.name).unwrap();

            write_number(writer, fdef.body.len() as f64).unwrap();
            let bodyclone = Rc::clone(&fdef.body);
            for b in bodyclone.iter() {
                serialize_node(writer, &*b);
            }

            write_string(writer, &fdef.return_type).unwrap();
        }
        
        Node::CallExpr(cexpr) => {
            // write arg count
            write_number(writer, cexpr.args.len() as f64).unwrap();

            for arg in &cexpr.args {
                // write arg
                serialize_node(writer, &arg);
            }

            // write caller
            serialize_node(writer, &cexpr.caller);
        }
        Node::MemberExpr(mem) => {
            serialize_node(writer, &mem.object);
            serialize_node(writer, &mem.property);
            write_bool(writer, mem.is_computed).unwrap();
        }
        Node::WhileStmt(w) => {
            serialize_node(writer, &w.condition);
            write_number(writer, w.body.len() as f64).unwrap();
            for b in &w.body {
                serialize_node(writer, b);
            }
        }
        Node::StringLiteral(slit) => {
            write_string(writer, &slit.literal_value).unwrap();
        }
        Node::IfStmt(i) => {
            serialize_node(writer, &i.condition);
            write_number(writer, i.body.len() as f64).unwrap();
            for b in &i.body {
                serialize_node(writer, b);
            }
        }
        Node::Iterator(it) => {
            write_token(writer, &it.left).unwrap();
            serialize_node(writer, &it.right);
            write_number(writer, it.body.len() as f64).unwrap();
            for b in &it.body {
                serialize_node(writer, b);
            }
        }
        Node::MatchExpr(mexpr) => {
            serialize_node(writer, &mexpr.target);
            write_number(writer, mexpr.arms.len() as f64).unwrap();
            for arm in &mexpr.arms {
                serialize_node(writer, &arm.0);
                serialize_node(writer, &arm.1);
            }
        }
        Node::BoolLiteral(bool) => {
            write_bool(writer, bool.literal_value).unwrap();
        }
        Node::OptionalArg(opt) => {
            serialize_node(writer, &opt.arg);
        }
        Node::NullishCoalescing(n) => {
            serialize_node(writer, &n.left);
            serialize_node(writer, &n.right);
        }
        Node::Block(b) => {
            write_number(writer, b.body.len() as f64).unwrap();
            for b in &b.body {
                serialize_node(writer, b);
            }
        }
        Node::NullLiteral(n) => {
            write_null(writer);
        }
        _ => todo!("Match node {:?} in serializer.", node)
    }

    writer.write_all(&[Signifier::EndNode.to_byte()]).unwrap();
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

        NodeKindId::Comparator => {
            Node::Comparator(Comparator {
                lhs: Box::new(deserialize_node(reader).unwrap()),
                rhs: Box::new(deserialize_node(reader).unwrap()),
                op: read_string(reader).unwrap()
            })
        }

        NodeKindId::ListLiteral => {
            let mut props = Vec::new();

            let amount = read_number(reader).unwrap() as usize;

            while props.len() < amount {
                props.push(Box::new(deserialize_node(reader).unwrap()));
            }

            Node::ListLiteral(ListLiteral {
                props
            })
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

        NodeKindId::MemberExpr => {
            let object = Box::new(deserialize_node(reader).unwrap());
            let prop = Box::new(deserialize_node(reader).unwrap());
            let computed = read_bool(reader).unwrap();

            Node::MemberExpr(MemberExpr {
                object,
                property: prop,
                is_computed: computed
            })
        }

        NodeKindId::WhileStmt => {
            let condition = Box::new(deserialize_node(reader).unwrap());
            let mut body = Vec::new();
            let count = read_number(reader).unwrap() as usize;
            
            while body.len() < count {
                body.push(Box::new(deserialize_node(reader).unwrap()));
            }
            Node::WhileStmt(WhileStmt { condition, body })
        }

        NodeKindId::StringLiteral => {
            Node::StringLiteral(StringLiteral { literal_value: read_string(reader).unwrap() })
        }

        NodeKindId::IfStmt => {
            let condition = Box::new(deserialize_node(reader).unwrap());
            let mut body = Vec::new();

            let count = read_number(reader).unwrap() as usize;
            while body.len() < count {
                body.push(Box::new(deserialize_node(reader).unwrap()));
            }

            Node::IfStmt(IfStmt {condition, body})
        }

        NodeKindId::Iterator => {
            let left = read_token(reader).unwrap();
            let right = Box::new(deserialize_node(reader).unwrap());
            let mut body = Vec::new(); 

            let count = read_number(reader).unwrap() as usize;
            while body.len() < count {
                body.push(Box::new(deserialize_node(reader).unwrap()));
            }

            Node::Iterator(Iterator {left, right, body})
        }

        NodeKindId::MatchExpr => {
            let target = Box::new(deserialize_node(reader).unwrap());
            let mut arms = Vec::new();
            let count = read_number(reader).unwrap() as usize;
            while arms.len() < count {
                let left = Box::new(deserialize_node(reader).unwrap());
                let right = Box::new(deserialize_node(reader).unwrap());

                arms.push((left, right));
            }
            
            Node::MatchExpr(MatchExpr {
                target,
                arms
            })
        }

        NodeKindId::BoolLiteral => {
            Node::BoolLiteral(BoolLiteral {literal_value: read_bool(reader).unwrap()})
        }

        NodeKindId::OptionalArg => {
            Node::OptionalArg(OptionalArg { arg: Box::new(deserialize_node(reader).unwrap()) })
        }

        NodeKindId::NullishCoalescing => {
            Node::NullishCoalescing(NullishCoalescing {
                left: Box::new(deserialize_node(reader).unwrap()),
                right: Box::new(deserialize_node(reader).unwrap())
            })
        }

        NodeKindId::Block => {
            let mut body = Vec::new();
            let count = read_number(reader).unwrap() as usize;
            while body.len() < count {
                body.push(Box::new(deserialize_node(reader).unwrap()));
            }
            Node::Block(Block {
                body
            })
        }

        NodeKindId::NullLiteral => {
            let mut null_byte = [0u8];
            reader.read_exact(&mut null_byte)?;
            if Signifier::from_byte(null_byte[0]) != Some(Signifier::Null) {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Expected Signifier::Null"));
            }

            Node::NullLiteral(NullLiteral {})
        }

        NodeKindId::Return => {
            let this_node = Box::new(deserialize_node(reader).unwrap());
            Node::Return(Return { return_statement: this_node })
        }

        NodeKindId::ObjectLiteral => {
            let mut props = HashMap::new();
            let prop_count = read_number(reader).unwrap() as usize;

            while props.len() < prop_count {
                let prop_name = read_string(reader).unwrap();
                let prop_value = Box::new(deserialize_node(reader).unwrap());
                props.insert(
                    prop_name,
                    prop_value
                );
            }

            Node::ObjectLiteral(ObjectLiteral {
                props
            })
        }

        NodeKindId::FunctionDefinition => {
            let mut params = Vec::new();
            let pcount = read_number(reader).unwrap() as usize;
            while params.len() < pcount {
                params.push(read_string(reader).unwrap());
            }
            
            let fn_name = read_string(reader).unwrap();

            let mut body = Vec::new();
            let bcount = read_number(reader).unwrap() as usize;
            while body.len() < bcount {
                body.push(Box::new(deserialize_node(reader).unwrap()));
            }

            let fn_ret_type = read_string(reader).unwrap();

            Node::FunctionDefinition(FunctionDefinition {
                params,
                name: fn_name,
                body: Rc::new(body),
                return_type: fn_ret_type
            })
        }

        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Unimplemented node kind: {:?}", kind))),
    };

    // 4. Expect END_NODE
    reader.read_exact(&mut tag)?;
    if tag[0] != Signifier::EndNode.to_byte() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Expected END_NODE for kind {:?}", kind)));
    }

    Ok(node)
}

