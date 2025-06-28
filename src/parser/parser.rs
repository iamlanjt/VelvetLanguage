use crate::{parser::nodetypes::{BinaryExpr, Node, NumericLiteral, VarDeclaration}, tokenizer::{token::{VelvetToken, VelvetTokenType}, tokenizer::tokenize}};

pub struct Parser {
    tokens: Vec<VelvetToken>,
    token_pointer: usize
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokenized_result = tokenize(input);
        Self {
            tokens: tokenized_result,
            token_pointer: 0
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

    fn eat(&mut self) -> &VelvetToken {
        let current_token = &self.tokens[self.token_pointer];
        self.token_pointer += 1;
        current_token
    }

    fn expect_token(&mut self, expected_type: VelvetTokenType, message: &str) -> &VelvetToken {
        let tkn = self.eat();
        if tkn.kind != expected_type {
            panic!("Token expectation failed: expected token type {}, got {}:\n{}", expected_type, tkn.kind, message);
        } else {
            tkn
        }
    }

    // pub fn produce_ast(&mut self)
    // todo: make program type

    pub fn produce_ast(&mut self) -> Vec<Box<Node>> {
        let mut program: Vec<Box<Node>> = Vec::new();
            program.push(self.parse_stmt());

        program
    }

    fn parse_stmt(&mut self) -> Box<Node> {
        match self.current().kind {
            VelvetTokenType::Keywrd_Bindmutable => self.parse_var_declaration(),
            _ => self.parse_expr()
        }
    }

    // bindm my_counter as i32 = 0
    // bind my_counter as i32 = 0
    fn parse_var_declaration(&mut self) -> Box<Node> {
        let is_mutable = self.eat().kind == VelvetTokenType::Keywrd_Bindmutable;
        let identifier = self.expect_token(VelvetTokenType::Identifier, "Variable name required").literal_value.clone();

        self.expect_token(VelvetTokenType::Keywrd_As, "Explicit typing required");

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

    fn parse_expr(&mut self) -> Box<Node> {
        self.parse_additive_expr()
    }

    fn parse_additive_expr(&mut self) -> Box<Node> {
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

    fn parse_multiplicative_expr(&mut self) -> Box<Node> {
        let mut left = self.parse_primary_expr();

        loop {
            if self.at_end() {
                break;
            }
            let next_kind = self.current().kind.clone();

            if next_kind != VelvetTokenType::Asterisk && next_kind != VelvetTokenType::Slash {
                break;
            }

            let operator = self.eat().literal_value.clone();
            let right = self.parse_primary_expr();
            left = Box::new(Node::BinaryExpr(BinaryExpr {
                left,
                right: right,
                op: operator.to_string()
            }))
        }

        left
    }

    fn parse_primary_expr(&mut self) -> Box<Node> {
        // lowest level
        let tk = self.eat();

        match tk.kind {
            VelvetTokenType::Number => Box::new(Node::NumericLiteral(NumericLiteral {
                literal_value: tk.literal_value.clone()
            })),
            _ => panic!("Parsing for token {} not yet implemented!", tk.kind)
        }
    }
}