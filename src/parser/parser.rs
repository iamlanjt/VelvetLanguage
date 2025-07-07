use std::{collections::HashMap, rc::Rc};

use crate::{parser::nodetypes::{AssignmentExpr, BinaryExpr, BoolLiteral, CallExpr, Comparator, FunctionDefinition, Identifier, IfStmt, Iterator, ListLiteral, MatchExpr, MemberExpr, Node, NumericLiteral, ObjectLiteral, Return, StringLiteral, VarDeclaration, WhileStmt}, tokenizer::{token::{VelvetToken, VelvetTokenType}, tokenizer::tokenize}};

pub struct Parser {
    tokens: Vec<VelvetToken>,
    token_pointer: usize,
    tkn_chain: Vec<VelvetToken>
}

fn is_node_literal(node: &Node) -> bool {
    match node {
        Node::NumericLiteral(l) => true,
        Node::StringLiteral(s) => true,
        _ => false
    }
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokenized_result = tokenize(input);
        Self {
            tokens: tokenized_result,
            token_pointer: 0,
            tkn_chain: Vec::new()
        }
    }

    fn current(&mut self) -> &VelvetToken {
        let current_token = &self.tokens[self.token_pointer];
        return current_token;
    }

    fn at_end(&self) -> bool {
        self.token_pointer >= self.tokens.len()
    }

    fn peek(&mut self) -> Option<&VelvetToken> {
        if self.token_pointer + 1 >= self.tokens.len() {
            None
        } else {
            Some(&self.tokens[self.token_pointer + 1])
        }
    }

    fn eat(&mut self) -> VelvetToken {
        let current_token = self.tokens[self.token_pointer].clone();
        self.tkn_chain.push(self.tokens[self.token_pointer].clone());
        self.token_pointer += 1;

        current_token
    }

    fn error(&mut self, faulty_token: &VelvetToken, msg: &str) {
        let mut reconstructed_consumed_tokens = "".to_owned();
        for consumed_token in &self.tkn_chain {
            reconstructed_consumed_tokens = reconstructed_consumed_tokens + " " + &consumed_token.literal_value
        }
        let length = &reconstructed_consumed_tokens.len() - 1;
        let indicator = " ".repeat(
            ("Token chain reconstruction:   ").len()
        ) + &"-".repeat(length-2) + &format!(" ^ FAULT {} @ idx{}", faulty_token.kind, faulty_token.start_index);
        panic!("\nParser error:  {}\nToken chain reconstruction:  {}\n{indicator}", msg, reconstructed_consumed_tokens);
    }

    pub fn expect_token(&mut self, expected_type: VelvetTokenType, message: &str) -> VelvetToken {
        let tkn = self.eat();
        if tkn.kind != expected_type {
            self.error(&tkn, &format!("\nParser expected token type \"{}\", got \"{}\"\nExpectation fault message: \n{}\n", expected_type, tkn.kind, message));
            // panic!();
        }
        tkn
    }

    pub fn produce_ast(&mut self) -> Vec<Box<Node>> {
        let mut program: Vec<Box<Node>> = Vec::new();
        loop {
            if self.at_end() { break }

            self.tkn_chain.clear();
            
            program.push(self.parse_stmt());
        }

        program
    }

    pub fn parse_stmt(&mut self) -> Box<Node> {
        match self.current().kind {
            VelvetTokenType::Keywrd_Bind => self.parse_var_declaration(),
            VelvetTokenType::Keywrd_Bindmutable => self.parse_var_declaration(),
            VelvetTokenType::Arrow => self.parse_fn_declaration(),
            VelvetTokenType::Semicolon => self.parse_return_statement(),
            _ => self.parse_expr()
        }
    }

    pub fn parse_return_statement(&mut self) -> Box<Node> {
        self.eat(); // eat `;` token
        if self.at_end() { panic!("Unexpected EOF: expected return statement, got EOF."); };

        let this_statmenet = self.parse_expr();
        Box::new(Node::Return(Return {
            return_statement: this_statmenet
        }))
    }

    pub fn parse_fn_declaration(&mut self) -> Box<Node> {
        self.eat(); // eat `->` token

        let function_name = self.expect_token(VelvetTokenType::Identifier, "Function name expected").literal_value.clone();
        let args = self.parse_args();
        let mut parameter_names: Vec<String> = Vec::new();
        for arg in args {
            match *arg {
                Node::Identifier(ref name) => {
                    parameter_names.push(name.identifier_name.to_string());
                },
                _ => {
                    panic!("Expected identifier for function parameter");
                }
            }
        }

        self.expect_token(VelvetTokenType::EqArrow, "Expected function return type using =>. Did you mean to define a Snippet? Define it with |-> instead of ->.");

        let return_type = self.expect_token(VelvetTokenType::Identifier, "Expected type after eqarrow. Did you mean to define a Snippet? Define it with |-> instead of ->.").literal_value.clone();

        // parse fn body
        self.expect_token(VelvetTokenType::LBrace, "Expected function body start");

        let mut body: Vec<Box<Node>> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(self.parse_stmt());
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected end of body for function");

        let rc_body = Rc::new(body);
        
        return Box::new(Node::FunctionDefinition(FunctionDefinition {
            params: parameter_names,
            name: function_name,
            body: rc_body,
            return_type: return_type
        }))
    }

    pub fn parse_args(&mut self) -> Vec<Box<Node>> {
        self.expect_token(VelvetTokenType::LParen, "Expected args");
        let args: Vec<Box<Node>> = if self.current().kind == VelvetTokenType::RParen { Vec::new() } else { self.parse_argument_list() };

        self.expect_token(VelvetTokenType::RParen, "Expected args end");

        args
    }

    pub fn parse_argument_list(&mut self) -> Vec<Box<Node>> {
        let mut args: Vec<Box<Node>> = Vec::new();
        args.push(self.parse_assignment_expr());
        
        while !self.at_end() && self.current().kind == VelvetTokenType::Comma {
            self.eat();
            args.push(self.parse_assignment_expr());
        }

        args
    }

    pub fn parse_assignment_expr(&mut self) -> Box<Node> {
        let left = self.parse_comparator_expr();
        if !self.at_end() && self.current().kind == VelvetTokenType::Eq {
            self.eat();
            let value = self.parse_assignment_expr();
            return Box::new(Node::AssignmentExpr(AssignmentExpr {
                left,
                value
            }))
        }
        left
    }

    pub fn parse_comparator_expr(&mut self) -> Box<Node> {
        if self.at_end() {
            panic!("Unexpected EOF: comparator operator expected");
        }

        let is_comparator = match self.peek() {
            Some(x) => matches!(x.kind, VelvetTokenType::Gt | VelvetTokenType::Lt | VelvetTokenType::DoubleEq),
            None => false,
        };

        if !is_comparator {
            return self.parse_list_expr();
        }

        let lhs = self.parse_list_expr();
        let operator = self.eat().literal_value.clone();
        
        let rhs = self.parse_list_expr();

        return Box::new(Node::Comparator(Comparator {
            lhs,
            rhs,
            op: operator
        }))
    }

    pub fn parse_list_expr(&mut self) -> Box<Node> {
        if self.current().kind != VelvetTokenType::LBracket {
            return self.parse_object_expr();
        }

        self.eat();
        let mut props: Vec<Box<Node>> = Vec::new();

        while !self.at_end() && self.current().kind != VelvetTokenType::RBracket {
            let value = self.parse_expr();
            if self.current().kind == VelvetTokenType::Comma {
                self.eat();
                props.push(value);
                continue;
            } else if self.current().kind == VelvetTokenType::RBracket {
                props.push(value);
                continue;
            }
        }
        self.eat();

        Box::new(Node::ListLiteral(ListLiteral {
            props
        }))
    }

    pub fn parse_object_expr(&mut self) -> Box<Node> {
        if self.current().kind != VelvetTokenType::LBrace {
            return self.parse_additive_expr();
        }

        self.eat();
        // self.expect_token(VelvetTokenType::RBrace, "temp debug: expected {} empty object");

        let mut props: HashMap<String, Box<Node>> = HashMap::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            let field_name = self.expect_token(VelvetTokenType::Identifier, "Expected identifier for field name");
            self.expect_token(VelvetTokenType::Colon, "Expected :");
            let right = self.parse_expr();
            match field_name.kind {
                VelvetTokenType::Identifier => {
                    props.insert(field_name.literal_value, right);
                }
                _ => { panic!(""); }
            }
            if !self.at_end() && self.current().kind == VelvetTokenType::Comma {
                self.eat();
            }
        }
        self.expect_token(VelvetTokenType::RBrace, "Expected end of object");

        Box::new(Node::ObjectLiteral(ObjectLiteral {
            props
        }))
    }

    // bindm my_counter as i32 = 0
    // bind my_counter as i32 = 0
    pub fn parse_var_declaration(&mut self) -> Box<Node> {
        let is_mutable = self.eat().kind == VelvetTokenType::Keywrd_Bindmutable;
        let identifier = self.expect_token(VelvetTokenType::Identifier, "Variable name required").literal_value.clone();

        self.expect_token(VelvetTokenType::Keywrd_As, "Explicit typing when defining a var is required. [bind example as bool = true]");

        let var_type = self.expect_token(VelvetTokenType::Identifier, "Expected type").literal_value.clone();

        self.expect_token(VelvetTokenType::Eq, "");

        let var_value = self.parse_expr();

        Box::new(Node::VarDeclaration(VarDeclaration {
            is_mutable,
            var_identifier: identifier,
            var_type,
            var_value
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
                left,
                right: right,
                op: operator.to_string()
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

            if next_kind != VelvetTokenType::Asterisk && next_kind != VelvetTokenType::Slash {
                break;
            }

            let operator = self.eat().literal_value.clone();
            
            // MARKER: Bugx001 fix, `self.parse_primary_expr()` -> `self.parse_additive_expr()`
            // Putting a marker here because this is such a volatile change that future possible bugs relating to this change may be hard to deduce.
            let right = self.parse_additive_expr();
            left = Box::new(Node::BinaryExpr(BinaryExpr {
                left,
                right: right,
                op: operator.to_string()
            }))
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

        while !self.at_end() && 
            (self.current().kind == VelvetTokenType::Dot || self.current().kind == VelvetTokenType::LBracket) {

            let op = self.eat();

            if op.kind == VelvetTokenType::Dot {
                let property = self.parse_primary_expr();

                match property.as_ref() {
                    Node::Identifier(_) => {
                        object = Box::new(Node::MemberExpr(MemberExpr {
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
                    object,
                    property,
                    is_computed: true,
                }));
            }
        }

        object
    }

    pub fn parse_call_expr(&mut self, caller: Box<Node>) -> Box<Node> {
        let mut call_expr = Box::new(Node::CallExpr(CallExpr {
            args: self.parse_args(),
            caller
        }));

        if !self.at_end() && self.current().kind == VelvetTokenType::LParen {
            call_expr = self.parse_call_expr(call_expr);
        }

        call_expr
    }

    pub fn parse_if_statement(&mut self) -> Box<Node> {
        let condition = self.parse_comparator_expr();

        self.expect_token(VelvetTokenType::LBrace, "Expected body of loop");
        
        let mut body: Vec<Box<Node>> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(self.parse_stmt());
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected closing brace for body of if statement");
        Box::new(Node::IfStmt(IfStmt {
            condition,
            body
        }))
    }

    pub fn parse_while_stmt(&mut self) -> Box<Node> {
        let loop_condition = self.parse_comparator_expr();

        self.expect_token(VelvetTokenType::Keywrd_Do, "Expected 'do' for while loop");
        self.expect_token(VelvetTokenType::LBrace, "Expected body of loop");

        let mut body: Vec<Box<Node>> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(self.parse_stmt());
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected closing brace for body of while loop");
        Box::new(Node::WhileStmt(WhileStmt {
            condition: loop_condition,
            body
        }))
    }

    pub fn parse_for_loop(&mut self) -> Box<Node> {
        let left = self.expect_token(VelvetTokenType::Identifier, "Expected identifier for loop");
        self.expect_token(VelvetTokenType::Keywrd_Of, "Expected of");
        let right = self.parse_expr();
        self.expect_token(VelvetTokenType::Keywrd_Do, "Expected do");
        self.expect_token(VelvetTokenType::LBrace, "Expected for loop body");

        let mut body: Vec<Box<Node>> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            body.push(self.parse_stmt());
        }

        self.expect_token(VelvetTokenType::RBrace, "Expected closing brace for body of for loop");
        Box::new(Node::Iterator(Iterator {
            left,
            right,
            body
        }))
    }

    pub fn parse_snippet_definition(&mut self) -> Box<Node> {
        return Box::new(Node::StringLiteral(StringLiteral { literal_value: String::from("This is debug; make sure to remove.") }));
    }

    pub fn parse_match_expr(&mut self) -> Box<Node> {
        let target = self.parse_expr();

        self.expect_token(VelvetTokenType::LBrace, "Expected start of match body");
        let mut arms: Vec<(Box<Node>, Box<Node>)> = Vec::new();
        while !self.at_end() && self.current().kind != VelvetTokenType::RBrace {
            let left = self.parse_expr();
            self.expect_token(VelvetTokenType::EqArrow, "Expected director");
            let right = self.parse_expr();

            /*
            if !is_node_literal(&left.as_ref()) {
                panic!("LHS of Director must be a literal value.");
            }
            */

            arms.push((
                left,
                right
            ));

            if !self.at_end() && self.current().kind == VelvetTokenType::Comma {
                self.eat();
            }
        }
        self.expect_token(VelvetTokenType::RBrace, "Expected end of match statement body");
        Box::new(Node::MatchExpr(MatchExpr {
            target,
            arms
        }))
    }

    pub fn parse_primary_expr(&mut self) -> Box<Node> {
        // lowest level
        let tk = self.eat();

        match tk.kind {
            VelvetTokenType::Number => Box::new(Node::NumericLiteral(NumericLiteral {
                literal_value: tk.literal_value.clone()
            })),
            VelvetTokenType::Identifier => {
                if tk.literal_value == "true" {
                    Box::new(Node::BoolLiteral(BoolLiteral {
                        literal_value: true
                    }))
                } else if tk.literal_value == "false" {
                    Box::new(Node::BoolLiteral(BoolLiteral { literal_value: false }))
                } else {
                    Box::new(Node::Identifier(Identifier {
                        identifier_name: tk.literal_value.clone()
                    }))
                }
            },
            VelvetTokenType::Str => Box::new(Node::StringLiteral(StringLiteral {
                literal_value: tk.literal_value.clone()
            })),
            VelvetTokenType::Keywrd_While => {
                self.parse_while_stmt()
            }
            VelvetTokenType::LParen => {
                // self.eat();
                self.parse_expr()
            }
            VelvetTokenType::Keywrd_If => {
                self.parse_if_statement()
            }
            VelvetTokenType::Keywrd_For => {
                self.parse_for_loop()
            }
            VelvetTokenType::WallArrow => {
                self.parse_snippet_definition()
            }
            VelvetTokenType::Keywrd_Match => {
                self.parse_match_expr()
            }
            _ => panic!("This token sequence has no applicable parsing path yet: {}", tk.kind)
        }
    }
}