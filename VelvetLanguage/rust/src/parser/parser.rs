use std::{collections::HashMap, rc::Rc};

use crate::{
    parser::nodetypes::{
        AssignmentExpr, AstSnippet, BinaryExpr, Block, BoolLiteral, CallExpr, Comparator,
        FunctionDefinition, Identifier, IfStmt, InterpreterBlock, Iterator, ListLiteral, MatchExpr,
        MemberExpr, NoOpNode, Node, NullLiteral, NullishCoalescing, NumericLiteral, ObjectLiteral,
        OptionalArg, Return, SnippetParam, StringLiteral, TypeCast, VarDeclaration, WhileStmt,
    },
    tokenizer::{
        token::{VelvetToken, VelvetTokenType},
        tokenizer::{join_tokenizer_output, tokenize},
    },
    typecheck::typecheck::T,
};

pub struct Parser {
    tokens: Vec<VelvetToken>,
    token_pointer: usize,
    tkn_chain: Vec<VelvetToken>,
    ast_snippets: Vec<AstSnippet>,
    etech: ExecutionTechnique,
    cur_node: usize,
}

#[derive(Clone, PartialEq)]
pub enum ExecutionTechnique {
    Interpretation,
    Compilation,
}

impl Parser {
    pub fn new(input: &str, inject_stdlib_snippets: bool, etech: ExecutionTechnique) -> Self {
        let tokenized_result = tokenize(input, inject_stdlib_snippets, etech.clone());
        Self {
            tokens: join_tokenizer_output(tokenized_result),
            token_pointer: 0,
            tkn_chain: Vec::new(),
            ast_snippets: Vec::new(),
            etech,
            cur_node: 0,
        }
    }

    fn current(&mut self) -> &VelvetToken {
        (&self.tokens[self.token_pointer]) as _
    }

    fn at_end(&self) -> bool {
        self.token_pointer >= self.tokens.len()
    }

    fn eat(&mut self) -> VelvetToken {
        let current_token = self.tokens[self.token_pointer].clone();
        self.tkn_chain.push(self.tokens[self.token_pointer].clone());
        self.token_pointer += 1;

        current_token
    }

    fn alloc_node_id(&mut self) -> usize {
        let id = self.cur_node;
        self.cur_node += 1;
        id
    }

    fn identifier_to_type(&self, ident: &str) -> T {
        match ident.to_lowercase().as_str() {
            "i8" => T::Integer8,
            "i16" => T::Integer16,
            "i32" | "number" => T::Integer32,
            "i64" => T::Integer64,
            "i128" => T::Integer128,
            "bool" => T::Boolean,
            "string" => T::String,
            "any" => T::Any,
            _ => panic!("`{}` is not a valid type", ident),
        }
    }

    fn error(&mut self, faulty_token: &VelvetToken, msg: &str) {
        panic!(
            "\nParser error:  {}\nat src:{}:{}",
            msg, faulty_token.line, faulty_token.column
        );
    }

    pub fn expect_token(&mut self, expected_type: VelvetTokenType, message: &str) -> VelvetToken {
        let tkn = self.eat();
        if tkn.kind != expected_type {
            self.error(&tkn, &format!("\nParser expected token type \"{}\", got \"{}\"\nExpectation fault message: \n{}\n", expected_type, tkn.kind, message));
            // panic!();
        }
        tkn
    }

    pub fn produce_ast(&mut self) -> Vec<Node> {
        let mut program: Vec<Node> = Vec::new();
        loop {
            if self.at_end() {
                break;
            }

            self.tkn_chain.clear();

            program.push(*self.parse_stmt());
        }

        program.retain(|x| match x {
            Node::NoOpNode(_) => false,
            _ => true,
        });

        program
    }

    pub fn parse_stmt(&mut self) -> Box<Node> {
        match self.current().kind {
            VelvetTokenType::Keywrd_Bind => self.parse_var_declaration(),
            VelvetTokenType::Keywrd_Bindmutable => self.parse_var_declaration(),
            VelvetTokenType::Arrow => self.parse_fn_declaration(),
            VelvetTokenType::Semicolon => self.parse_return_statement(),
            VelvetTokenType::DollarSign => self.parse_interpreter_block(),
            VelvetTokenType::NoOp => {
                self.eat();
                Box::new(Node::NoOpNode(NoOpNode {
                    id: Some(self.alloc_node_id()),
                }))
            }
            _ => self.parse_expr(),
        }
    }

    pub fn parse_type_cast(&mut self, left: Box<Node>, right: T) -> Box<Node> {
        if self.etech == ExecutionTechnique::Interpretation {
            let current_token = self.current().clone();
            self.error(&current_token, "You can only typecast in compilation mode.");
        }

        Box::new(Node::TypeCast(TypeCast {
            id: Some(self.alloc_node_id()),
            left,
            target_type: right,
        }))
    }

    pub fn parse_interpreter_block(&mut self) -> Box<Node> {
        self.eat(); // eat `$` token
        let feature = self
            .expect_token(VelvetTokenType::Identifier, "Expected interpreter feature")
            .literal_value;

        self.expect_token(
            VelvetTokenType::LBrace,
            "Expected body of interpreter block",
        );

        let mut body_nodes: Vec<Box<Node>> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body_nodes.push(self.parse_stmt());
        }

        self.expect_token(
            VelvetTokenType::RBrace,
            "Expected end of body of interpreter block",
        );

        Box::new(Node::InterpreterBlock(InterpreterBlock {
            id: Some(self.alloc_node_id()),
            feature,
            body: body_nodes,
        }))
    }

    pub fn parse_return_statement(&mut self) -> Box<Node> {
        self.eat(); // eat `;` token
        if self.at_end() {
            panic!("Unexpected EOF: expected return statement, got EOF.");
        };

        let this_statmenet = self.parse_expr();
        Box::new(Node::Return(Return {
            id: Some(self.alloc_node_id()),
            return_statement: this_statmenet,
        }))
    }

    pub fn parse_snippet_definition(&mut self) {
        let snippet_name = self.expect_token(VelvetTokenType::Identifier, "Expected snippet name");
        let args = self.parse_args();
        let mut new_args: Vec<SnippetParam> = Vec::new();

        for arg in args {
            match arg {
                Node::Identifier(ident) => {
                    new_args.push(SnippetParam {
                        id: Some(self.alloc_node_id()),
                        name: ident.identifier_name,
                        is_optional: false,
                    });
                }
                Node::OptionalArg(opt_arg) => match *opt_arg.arg {
                    Node::Identifier(inner_ident) => new_args.push(SnippetParam {
                        id: Some(self.alloc_node_id()),
                        name: inner_ident.identifier_name,
                        is_optional: true,
                    }),
                    _ => panic!("Attempt to declare optional Snippet Arg as non-identifier"),
                },
                _ => panic!("Attempt to declare Snippet Arg as non-identifier"),
            }
        }

        self.expect_token(VelvetTokenType::LBrace, "Expected Snippet body");
        let mut body: Vec<Node> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(*self.parse_stmt())
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected end of Snippet body");

        let alloc = self.alloc_node_id();
        self.ast_snippets.push(AstSnippet {
            id: Some(alloc),
            name: snippet_name.literal_value.to_string(),
            args: new_args,
            body,
        });
    }

    pub fn parse_fn_declaration(&mut self) -> Box<Node> {
        self.eat(); // eat `->` token

        let function_name = self
            .expect_token(VelvetTokenType::Identifier, "Function name expected")
            .literal_value
            .clone();
        let args = self.parse_fdef_args(); // self.parse_args();
        let mut parameter_names: Vec<(String, T)> = Vec::new();
        for arg in args {
            match arg.0 {
                Node::Identifier(ref name) => {
                    parameter_names.push((name.identifier_name.to_string(), arg.1));
                }
                _ => {
                    panic!("Expected identifier for function parameter");
                }
            }
        }

        self.expect_token(VelvetTokenType::EqArrow, "Expected function return type using =>. Did you mean to define a Snippet? Define it with |-> instead of ->.");

        let return_type = self.expect_token(VelvetTokenType::Identifier, "Expected type after eqarrow. Did you mean to define a Snippet? Define it with |-> instead of ->.").literal_value.clone();

        // parse fn body
        self.expect_token(VelvetTokenType::LBrace, "Expected function body start");

        let mut body: Vec<Node> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(*self.parse_stmt());
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected end of body for function");

        let rc_body = Rc::new(body);

        return Box::new(Node::FunctionDefinition(FunctionDefinition {
            id: Some(self.alloc_node_id()),
            params: parameter_names,
            name: function_name,
            body: rc_body,
            return_type: self.identifier_to_type(&return_type),
        }));
    }

    // Same thing as `parse_args`, but requires explicit type declaration using `as` keyword
    pub fn parse_fdef_args(&mut self) -> Vec<(Node, T)> {
        self.expect_token(VelvetTokenType::LParen, "Expected function params");
        let args: Vec<(Node, T)> = if self.current().kind == VelvetTokenType::RParen {
            Vec::new()
        } else {
            self.parse_fdef_argument_list()
        };

        self.expect_token(VelvetTokenType::RParen, "Expected function params end");

        args
    }

    pub fn parse_fdef_argument_list(&mut self) -> Vec<(Node, T)> {
        let mut args: Vec<(Node, T)> = Vec::new();
        let first_arg = *self.parse_assignment_expr();
        self.expect_token(
            VelvetTokenType::Keywrd_As,
            "Function parameters must be explicitly typed",
        );
        let arg_type = self.expect_token(VelvetTokenType::Identifier, "Expected type identifier");
        args.push((first_arg, self.identifier_to_type(&arg_type.literal_value)));

        while !self.at_end() && self.current().kind == VelvetTokenType::Comma {
            self.eat();
            let next_arg = *self.parse_assignment_expr();
            self.expect_token(
                VelvetTokenType::Keywrd_As,
                "Function parameters must be explicitly typed",
            );

            let arg_type =
                self.expect_token(VelvetTokenType::Identifier, "Expected type identifier");
            args.push((next_arg, self.identifier_to_type(&arg_type.literal_value)));
        }

        args
    }

    pub fn parse_args(&mut self) -> Vec<Node> {
        self.expect_token(VelvetTokenType::LParen, "Expected args");
        let args: Vec<Node> = if self.current().kind == VelvetTokenType::RParen {
            Vec::new()
        } else {
            self.parse_argument_list()
        };

        self.expect_token(VelvetTokenType::RParen, "Expected args end");

        args
    }

    pub fn parse_argument_list(&mut self) -> Vec<Node> {
        let mut args: Vec<Node> = Vec::new();
        args.push(*self.parse_assignment_expr());

        while !self.at_end() && self.current().kind == VelvetTokenType::Comma {
            self.eat();
            let to_push = self.parse_assignment_expr();
            if !self.at_end() && self.current().kind == VelvetTokenType::QuestionMark {
                self.eat();
                args.push(Node::OptionalArg(OptionalArg {
                    id: Some(self.alloc_node_id()),
                    arg: to_push,
                }));
            } else {
                args.push(*to_push);
            }
        }

        args
    }

    pub fn parse_assignment_expr(&mut self) -> Box<Node> {
        let left = self.parse_nullish_expr();
        if !self.at_end() && self.current().kind == VelvetTokenType::Eq {
            self.eat();
            let value = self.parse_assignment_expr();
            return Box::new(Node::AssignmentExpr(AssignmentExpr {
                id: Some(self.alloc_node_id()),
                left,
                value,
            }));
        }
        left
    }

    pub fn parse_nullish_expr(&mut self) -> Box<Node> {
        let mut left = self.parse_comparator_expr();

        while !self.at_end() && self.current().kind == VelvetTokenType::Exclaimation {
            self.eat();

            let right = self.parse_comparator_expr();
            left = Box::new(Node::NullishCoalescing(NullishCoalescing {
                id: Some(self.alloc_node_id()),
                left,
                right,
            }));
        }

        left
    }

    pub fn parse_comparator_expr(&mut self) -> Box<Node> {
        let lhs = self.parse_list_expr();

        if !self.at_end()
            && matches!(
                self.current().kind,
                VelvetTokenType::Gt | VelvetTokenType::Lt | VelvetTokenType::DoubleEq
            )
        {
            let operator = self.eat().literal_value.clone();

            let rhs = self.parse_list_expr();

            return Box::new(Node::Comparator(Comparator {
                id: Some(self.alloc_node_id()),
                lhs,
                rhs,
                op: operator,
            }));
        }

        lhs
    }

    pub fn parse_list_expr(&mut self) -> Box<Node> {
        if self.current().kind != VelvetTokenType::LBracket {
            return self.parse_object_expr();
        }

        self.eat();
        let mut props: Vec<Node> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBracket {
            let value = self.parse_expr();
            if self.current().kind == VelvetTokenType::Comma {
                self.eat();
                props.push(*value);
                continue;
            } else if self.current().kind == VelvetTokenType::RBracket {
                props.push(*value);
                continue;
            }
        }
        self.eat();

        Box::new(Node::ListLiteral(ListLiteral {
            id: Some(self.alloc_node_id()),
            props,
        }))
    }

    pub fn parse_object_expr(&mut self) -> Box<Node> {
        if self.current().kind != VelvetTokenType::LBrace {
            return self.parse_additive_expr();
        }

        self.eat();
        // self.expect_token(VelvetTokenType::RBrace, "temp debug: expected {} empty object");

        let mut props: HashMap<String, Node> = HashMap::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            let field_name = self.expect_token(
                VelvetTokenType::Identifier,
                "Expected identifier for field name",
            );
            self.expect_token(VelvetTokenType::Colon, "Expected :");
            let right = self.parse_expr();
            match field_name.kind {
                VelvetTokenType::Identifier => {
                    props.insert(field_name.literal_value, *right);
                }
                _ => {
                    panic!("");
                }
            }
            if !self.at_end() && self.current().kind == VelvetTokenType::Comma {
                self.eat();
            }
        }
        self.expect_token(VelvetTokenType::RBrace, "Expected end of object");

        Box::new(Node::ObjectLiteral(ObjectLiteral {
            id: Some(self.alloc_node_id()),
            props,
        }))
    }

    // bindm my_counter as i32 = 0
    // bind my_counter as i32 = 0
    pub fn parse_var_declaration(&mut self) -> Box<Node> {
        let is_mutable = self.eat().kind == VelvetTokenType::Keywrd_Bindmutable;
        let identifier = self
            .expect_token(VelvetTokenType::Identifier, "Variable name required")
            .literal_value
            .clone();

        self.expect_token(
            VelvetTokenType::Keywrd_As,
            "Explicit typing when defining a var is required. [bind example as bool = true]",
        );

        let var_type_str = &self
            .expect_token(VelvetTokenType::Identifier, "Expected type")
            .literal_value;
        let var_type = self.identifier_to_type(var_type_str);

        self.expect_token(VelvetTokenType::Eq, "");

        let var_value = self.parse_expr();

        Box::new(Node::VarDeclaration(VarDeclaration {
            id: Some(self.alloc_node_id()),
            is_mutable,
            var_identifier: identifier,
            var_type,
            var_value,
        }))
    }

    pub fn parse_expr(&mut self) -> Box<Node> {
        self.parse_assignment_expr()
    }

    pub fn parse_additive_expr(&mut self) -> Box<Node> {
        let mut left = self.parse_multiplicative_expr();

        loop {
            if self.at_end() {
                break;
            }
            let next_kind = self.current().kind.clone();

            if next_kind != VelvetTokenType::Plus && next_kind != VelvetTokenType::Minus {
                break;
            }

            let operator = self.eat().literal_value.clone();
            let right = self.parse_multiplicative_expr();
            left = Box::new(Node::BinaryExpr(BinaryExpr {
                id: Some(self.alloc_node_id()),
                left,
                right,
                op: operator.to_string(),
            }))
        }

        left
    }

    pub fn parse_multiplicative_expr(&mut self) -> Box<Node> {
        let mut left = self.parse_call_member_expr();

        loop {
            if self.at_end() {
                break;
            }
            let next_kind = self.current().kind.clone();

            if next_kind == VelvetTokenType::At {
                self.eat();
                let t = self
                    .expect_token(VelvetTokenType::Identifier, "Expected type after typecast")
                    .literal_value
                    .clone();
                let t_parsed = self.identifier_to_type(&t);

                left = self.parse_type_cast(left, t_parsed);
            } else {
                if next_kind != VelvetTokenType::Asterisk && next_kind != VelvetTokenType::Slash {
                    break;
                }

                let operator = self.eat().literal_value.clone();

                // MARKER: Bugx001 fix, `self.parse_primary_expr()` -> `self.parse_additive_expr()`
                // Putting a marker here because this is such a volatile change that future possible bugs relating to this change may be hard to deduce.

                // Jul 7 change, `self.parse_additive_expr()` -> `self.parse_call_member_expr()` due to precedence issues. This seems to have fixed them and not caused
                // any other bugs.
                let right = self.parse_call_member_expr();
                left = Box::new(Node::BinaryExpr(BinaryExpr {
                    id: Some(self.alloc_node_id()),
                    left,
                    right,
                    op: operator.to_string(),
                }))
            }
        }

        left
    }

    pub fn parse_call_member_expr(&mut self) -> Box<Node> {
        let member = self.parse_member_expr();

        if !self.at_end() && self.current().kind == VelvetTokenType::LParen {
            return self.parse_call_expr(member);
        }

        return member;
    }

    pub fn parse_member_expr(&mut self) -> Box<Node> {
        let mut object = self.parse_primary_expr();

        while !self.at_end()
            && (self.current().kind == VelvetTokenType::Dot
                || self.current().kind == VelvetTokenType::LBracket)
        {
            let op = self.eat();

            if op.kind == VelvetTokenType::Dot {
                let property = self.parse_primary_expr();

                match property.as_ref() {
                    Node::Identifier(_) => {
                        object = Box::new(Node::MemberExpr(MemberExpr {
                            id: Some(self.alloc_node_id()),
                            object,
                            property,
                            is_computed: false,
                        }));
                    }
                    _ => {
                        panic!(
                            "Right-hand of '.' must be an Identifier, found '{:#?}'",
                            property
                        );
                    }
                }
            } else {
                // Handle computed property access: object[expr]
                let property = self.parse_expr();
                self.expect_token(VelvetTokenType::RBracket, "Expected closing bracket");

                object = Box::new(Node::MemberExpr(MemberExpr {
                    id: Some(self.alloc_node_id()),
                    object,
                    property,
                    is_computed: true,
                }));
            }
        }

        object
    }

    fn substitute_snippet_vars(
        node: &Box<Node>,
        bindings: &HashMap<String, Box<Node>>,
    ) -> Box<Node> {
        match node.as_ref() {
            Node::Identifier(Identifier {
                id: None,
                identifier_name,
            }) => {
                if let Some(replacement) = bindings.get(identifier_name) {
                    return replacement.clone();
                }
                Box::new(Node::Identifier(Identifier {
                    id: None,
                    identifier_name: identifier_name.clone(),
                }))
            }

            Node::CallExpr(CallExpr {
                id: None,
                caller,
                args,
            }) => Box::new(Node::CallExpr(CallExpr {
                id: None,
                caller: Self::substitute_snippet_vars(caller, bindings),
                args: args
                    .iter()
                    .map(|arg| *Self::substitute_snippet_vars(&Box::new(arg.clone()), bindings))
                    .collect(),
            })),

            Node::BinaryExpr(BinaryExpr {
                id: None,
                left,
                right,
                op,
            }) => Box::new(Node::BinaryExpr(BinaryExpr {
                id: None,
                left: Self::substitute_snippet_vars(left, bindings),
                right: Self::substitute_snippet_vars(right, bindings),
                op: op.clone(),
            })),

            Node::NullishCoalescing(NullishCoalescing {
                id: None,
                left,
                right,
            }) => Box::new(Node::NullishCoalescing(NullishCoalescing {
                id: None,
                left: Self::substitute_snippet_vars(left, bindings),
                right: Self::substitute_snippet_vars(right, bindings),
            })),

            Node::Return(Return {
                id: None,
                return_statement,
            }) => Box::new(Node::Return(Return {
                id: None,
                return_statement: Self::substitute_snippet_vars(return_statement, bindings),
            })),

            Node::VarDeclaration(VarDeclaration {
                id: None,
                is_mutable,
                var_identifier,
                var_type,
                var_value,
            }) => Box::new(Node::VarDeclaration(VarDeclaration {
                id: None,
                is_mutable: *is_mutable,
                var_identifier: var_identifier.clone(),
                var_type: var_type.clone(),
                var_value: Self::substitute_snippet_vars(var_value, bindings),
            })),

            Node::Block(Block { id: None, body }) => Box::new(Node::Block(Block {
                id: None,
                body: body
                    .iter()
                    .map(|stmt| *Self::substitute_snippet_vars(&Box::new(stmt.clone()), bindings))
                    .collect(),
            })),

            Node::IfStmt(IfStmt {
                id: None,
                condition,
                body,
            }) => Box::new(Node::IfStmt(IfStmt {
                id: None,
                condition: Self::substitute_snippet_vars(condition, bindings),
                body: body
                    .iter()
                    .map(|stmt| *Self::substitute_snippet_vars(&Box::new(stmt.clone()), bindings))
                    .collect(),
            })),

            Node::WhileStmt(WhileStmt {
                id: None,
                condition,
                body,
            }) => Box::new(Node::WhileStmt(WhileStmt {
                id: None,
                condition: Self::substitute_snippet_vars(condition, bindings),
                body: body
                    .iter()
                    .map(|stmt| *Self::substitute_snippet_vars(&Box::new(stmt.clone()), bindings))
                    .collect(),
            })),

            Node::Iterator(Iterator {
                id: None,
                left,
                right,
                body,
            }) => {
                let new_right = Self::substitute_snippet_vars(right, bindings);
                let new_body = body
                    .iter()
                    .map(|stmt| *Self::substitute_snippet_vars(&Box::new(stmt.clone()), bindings))
                    .collect();
                Box::new(Node::Iterator(Iterator {
                    id: None,
                    left: left.clone(), // assume left is a plain identifier token
                    right: new_right,
                    body: new_body,
                }))
            }

            Node::MatchExpr(MatchExpr {
                id: None,
                target,
                arms,
            }) => Box::new(Node::MatchExpr(MatchExpr {
                id: None,
                target: Self::substitute_snippet_vars(target, bindings),
                arms: arms
                    .iter()
                    .map(|(l, r)| {
                        (
                            *Self::substitute_snippet_vars(&Box::new(l.clone()), bindings),
                            *Self::substitute_snippet_vars(&Box::new(r.clone()), bindings),
                        )
                    })
                    .collect(),
            })),

            Node::MemberExpr(MemberExpr {
                id: None,
                object,
                property,
                is_computed,
            }) => Box::new(Node::MemberExpr(MemberExpr {
                id: None,
                object: Self::substitute_snippet_vars(object, bindings),
                property: Self::substitute_snippet_vars(property, bindings),
                is_computed: *is_computed,
            })),

            Node::ListLiteral(ListLiteral { id: None, props }) => {
                Box::new(Node::ListLiteral(ListLiteral {
                    id: None,
                    props: props
                        .iter()
                        .map(|p| *Self::substitute_snippet_vars(&Box::new(p.clone()), bindings))
                        .collect(),
                }))
            }

            Node::ObjectLiteral(ObjectLiteral { id: None, props }) => {
                Box::new(Node::ObjectLiteral(ObjectLiteral {
                    id: None,
                    props: props
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                *Self::substitute_snippet_vars(&Box::new(v.clone()), bindings),
                            )
                        })
                        .collect(),
                }))
            }
            Node::Comparator(Comparator {
                id: None,
                lhs,
                rhs,
                op,
            }) => Box::new(Node::Comparator(Comparator {
                id: None,
                lhs: Self::substitute_snippet_vars(lhs, bindings),
                rhs: Self::substitute_snippet_vars(rhs, bindings),
                op: op.to_string(),
            })),

            // If it's a literal or something that doesn't contain other nodes, just clone it
            other => Box::new(other.clone()),
        }
    }

    fn expand_snippet_invocation(&mut self, snippet_name: String) -> Box<Node> {
        let snippet_name_trimmed = snippet_name.trim_end_matches('#');

        let snippet_opt = self
            .ast_snippets
            .iter()
            .find(|s| s.name == snippet_name_trimmed)
            .cloned();

        let snippet = snippet_opt.expect(&format!("Snippet '{}' not found", snippet_name_trimmed));

        let call_args = self.parse_args();

        if call_args.len() > snippet.args.len() {
            panic!(
                "Too many arguments provided to snippet '{}'",
                snippet_name_trimmed
            );
        }

        let mut arg_bindings: HashMap<String, Box<Node>> = HashMap::new();

        for (i, param) in snippet.args.iter().enumerate() {
            let arg_val = call_args.get(i).cloned().unwrap_or_else(|| {
                if param.is_optional {
                    Node::NullLiteral(NullLiteral {
                        id: Some(self.alloc_node_id()),
                    })
                } else {
                    panic!(
                        "Missing required argument '{}' in snippet '{}'",
                        param.name, snippet_name_trimmed
                    )
                }
            });

            arg_bindings.insert(param.name.clone(), Box::new(arg_val));
        }

        let mut new_body = Vec::new();
        for stmt in &snippet.body {
            let substituted =
                *Self::substitute_snippet_vars(&Box::new(stmt.clone()), &arg_bindings);
            new_body.push(substituted);
        }

        Box::new(Node::Block(Block {
            id: Some(self.alloc_node_id()),
            body: new_body,
        }))
    }

    pub fn parse_call_expr(&mut self, caller: Box<Node>) -> Box<Node> {
        if let Node::Identifier(Identifier {
            id: None,
            identifier_name,
        }) = caller.as_ref()
        {
            if identifier_name.ends_with('#') {
                return self.expand_snippet_invocation(identifier_name.clone());
            }
        }

        let mut call_expr = Box::new(Node::CallExpr(CallExpr {
            id: Some(self.alloc_node_id()),
            args: self.parse_args(),
            caller,
        }));

        if !self.at_end() && self.current().kind == VelvetTokenType::LParen {
            call_expr = self.parse_call_expr(call_expr);
        }

        call_expr
    }

    pub fn parse_if_statement(&mut self) -> Box<Node> {
        let condition = self.parse_stmt();

        self.expect_token(VelvetTokenType::LBrace, "Expected body of loop");

        let mut body: Vec<Node> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(*self.parse_stmt());
        }

        self.expect_token(
            VelvetTokenType::RBrace,
            "Expected closing brace for body of if statement",
        );
        Box::new(Node::IfStmt(IfStmt {
            id: Some(self.alloc_node_id()),
            condition,
            body,
        }))
    }

    pub fn parse_while_stmt(&mut self) -> Box<Node> {
        let loop_condition = self.parse_comparator_expr();

        self.expect_token(VelvetTokenType::Keywrd_Do, "Expected 'do' for while loop");
        self.expect_token(VelvetTokenType::LBrace, "Expected body of loop");

        let mut body: Vec<Node> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(*self.parse_stmt());
        }

        self.expect_token(
            VelvetTokenType::RBrace,
            "Expected closing brace for body of while loop",
        );
        Box::new(Node::WhileStmt(WhileStmt {
            id: Some(self.alloc_node_id()),
            condition: loop_condition,
            body,
        }))
    }

    pub fn parse_for_loop(&mut self) -> Box<Node> {
        let left = self.expect_token(VelvetTokenType::Identifier, "Expected identifier for loop");
        self.expect_token(VelvetTokenType::Keywrd_Of, "Expected of");
        let right = self.parse_expr();
        self.expect_token(VelvetTokenType::Keywrd_Do, "Expected do");
        self.expect_token(VelvetTokenType::LBrace, "Expected for loop body");

        let mut body: Vec<Node> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(*self.parse_stmt());
        }

        self.expect_token(
            VelvetTokenType::RBrace,
            "Expected closing brace for body of for loop",
        );
        Box::new(Node::Iterator(Iterator {
            id: Some(self.alloc_node_id()),
            left,
            right,
            body,
        }))
    }

    pub fn parse_block_expr(&mut self) -> Box<Node> {
        if self.current().kind == VelvetTokenType::LBrace {
            self.eat();
            let mut block_body: Vec<Node> = Vec::new();
            while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
                block_body.push(*self.parse_stmt());
            }
            self.eat();

            Box::new(Node::Block(Block {
                id: Some(self.alloc_node_id()),
                body: block_body,
            }))
        } else {
            self.parse_expr()
        }
    }

    pub fn parse_match_expr(&mut self) -> Box<Node> {
        // let target = self.parse_expr();
        let target = self.parse_expr();

        self.expect_token(VelvetTokenType::LBrace, "Expected start of match body");
        let mut arms: Vec<(Node, Node)> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            let left = *self.parse_expr();
            self.expect_token(VelvetTokenType::EqArrow, "Expected director");
            // let right = self.parse_expr();
            let right = *self.parse_block_expr();

            // if !is_node_literal(&left.as_ref()) {
            //     panic!("LHS of Director must be a literal value.");
            // }

            arms.push((left, right));

            if !self.at_end() && self.current().kind == VelvetTokenType::Comma {
                self.eat();
            }
        }
        self.expect_token(
            VelvetTokenType::RBrace,
            "Expected end of match statement body",
        );
        Box::new(Node::MatchExpr(MatchExpr {
            id: Some(self.alloc_node_id()),
            target,
            arms,
        }))
    }

    pub fn parse_primary_expr(&mut self) -> Box<Node> {
        // lowest level
        let tk = self.eat();

        match tk.kind {
            VelvetTokenType::Number => Box::new(Node::NumericLiteral(NumericLiteral {
                id: Some(self.alloc_node_id()),
                literal_value: tk.literal_value.clone(),
            })),
            VelvetTokenType::Identifier => {
                if tk.literal_value == "true" {
                    Box::new(Node::BoolLiteral(BoolLiteral {
                        id: Some(self.alloc_node_id()),
                        literal_value: true,
                    }))
                } else if tk.literal_value == "false" {
                    Box::new(Node::BoolLiteral(BoolLiteral {
                        id: Some(self.alloc_node_id()),
                        literal_value: false,
                    }))
                } else {
                    Box::new(Node::Identifier(Identifier {
                        id: Some(self.alloc_node_id()),
                        identifier_name: tk.literal_value.clone(),
                    }))
                }
            }
            VelvetTokenType::Str => Box::new(Node::StringLiteral(StringLiteral {
                id: Some(self.alloc_node_id()),
                literal_value: tk.literal_value.clone(),
            })),
            VelvetTokenType::Keywrd_While => self.parse_while_stmt(),
            VelvetTokenType::LParen => {
                // self.eat();
                self.parse_expr()
            }
            VelvetTokenType::Keywrd_If => self.parse_if_statement(),
            VelvetTokenType::Keywrd_For => self.parse_for_loop(),
            VelvetTokenType::WallArrow => {
                self.parse_snippet_definition();
                return Box::new(Node::NoOpNode(NoOpNode { id: None }));
            }
            VelvetTokenType::Keywrd_Match => self.parse_match_expr(),
            _ => {
                self.error(
                    &tk,
                    &format!(
                        "This token sequence has no applicable parsing path yet: {}",
                        tk.kind
                    ),
                );
                panic!()
            }
        }
    }
}
