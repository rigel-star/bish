#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;

use crate::scanner;
use crate::chunk;

#[derive(PartialEq, PartialOrd, Clone, Copy, Hash, Eq)]
enum Precedence 
{
    PREC_NONE,
    PREC_ASSIGNMENT,  // =
    PREC_OR,          // or
    PREC_AND,         // and
    PREC_EQUALITY,    // == !=
    PREC_COMPARISON,  // < > <= >=
    PREC_TERM,        // + -
    PREC_FACTOR,      // * /
    PREC_UNARY,       // ! -
    PREC_CALL,        // . ()
    PREC_PRIMARY
}

pub struct Parser<'compiling: 'pointer, 'pointer>
{
    tokens: &'compiling Vec<scanner::Token>,
    chunk: &'compiling mut chunk::Chunk,
    current: &'pointer scanner::Token,
    previous: &'pointer scanner::Token,
    counter: usize,
    had_error: bool,
    panic_mode: bool
}

/* 'static-like' method definitions */
impl<'compiling, 'pointer> Parser<'compiling, 'pointer>
{
    pub fn new(tokens: &'compiling Vec<scanner::Token>, chunk: &'compiling mut chunk::Chunk) -> Parser<'compiling, 'pointer>
    {
        Parser {
            tokens,
            chunk,
            current: &tokens[0],
            previous: &tokens[0],
            counter: 0,
            had_error: false,
            panic_mode: false
        }
    }
}

impl<'compiling: 'pointer, 'pointer> Parser<'compiling, 'pointer>
{
    pub fn compile(&mut self)
    {
        self.advance();
        self.parse_expression();
    }

    fn parse_expression(&self)
    {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }

    fn parse_precedence(&self, prec: Precedence)
    {
    }

    fn parse_number(&mut self)
    {
        let token: &scanner::Token = self.previous;
        match token.token_type
        {
            scanner::TokenType::TOKEN_FLOAT_NUM => 
            {
                self.chunk.write_const_double(token.lexeme.parse().unwrap());
            },
            scanner::TokenType::TOKEN_INT_NUM => 
            {
                self.chunk.write_const_int(token.lexeme.parse().unwrap());
            }
            _ => ()
        }
    }

    fn parse_binary(&mut self)
    {
        let ttype: scanner::TokenType = self.previous.token_type;
        match ttype 
        {
            scanner::TokenType::TOKEN_PLUS => {
                self.chunk.write(chunk::OpCode::OP_ADD);
            },
            scanner::TokenType::TOKEN_MINUS => {
                self.chunk.write(chunk::OpCode::OP_SUBTRACT);
            },
            scanner::TokenType::TOKEN_STAR => {
                self.chunk.write(chunk::OpCode::OP_MULTIPLY);
            },
            scanner::TokenType::TOKEN_SLASH => {
                self.chunk.write(chunk::OpCode::OP_DIVIDE);
            },
            _ => ()
        }
    }

    fn get_rule(&self, token_type: scanner::TokenType) -> (Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)
    {
        let mut rules: HashMap<scanner::TokenType, (Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)> = HashMap::new();
        rules.insert(scanner::TokenType::TOKEN_LEFT_PAREN, (Some(Parser::parse_grouping as fn(&mut Self)), None, Precedence::PREC_NONE));
        rules.insert(scanner::TokenType::TOKEN_MINUS, (Some(Parser::parse_unary as fn(&mut Self)), Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_TERM));
        rules.insert(scanner::TokenType::TOKEN_PLUS, (None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_TERM));
        rules[&token_type]
    }

    fn parse_unary(&mut self)
    {
        let token: &scanner::Token = self.previous;
        self.parse_precedence(Precedence::PREC_UNARY);
        match token.token_type
        {
            scanner::TokenType::TOKEN_MINUS => 
            {
                self.chunk.write(chunk::OpCode::OP_NEGATE);
            },
            _ => ()
        }
    }

    fn parse_grouping(&mut self)
    {
        self.parse_expression();
        self.consume(scanner::TokenType::TOKEN_RIGHT_PAREN, "Expected ')' after expression.");
    }

    fn consume(&mut self, token_type: scanner::TokenType, msg: &str)
    {
        if token_type == self.current.token_type
        {
            self.advance();
            return;
        }
        self.error_at_current(msg);
    }

    fn emit_bytecode(&mut self, byte: u8)
    {
        self.chunk.write(chunk::OpCode::from_u8(byte));
    }

    fn advance(&mut self)
    {
        self.previous = self.current;
        if !self.is_at_end()
        {
            self.current = &self.tokens[self.counter];
            self.counter += 1;
        }
    }

    fn is_at_end(&self) -> bool
    {
        self.current_token().token_type == scanner::TokenType::TOKEN_NONE
    }

    fn current_token(&self) -> &scanner::Token
    {
        &self.tokens[self.counter]
    }

    fn error_at_current(&mut self, message: &str)
    {
        self.error_at(self.counter, message);
    }

    fn error_at(&mut self, token_idx: usize, message: &str)
    {
        let token: &scanner::Token = &self.tokens[token_idx];
        print!("[{}] Error", token.line);
        match token.token_type
        {
            scanner::TokenType::TOKEN_NONE =>
            {
                print!(" at end");
            },
            _ => 
            {
                print!(" at {}", token.lexeme);
            }
        }

        print!(": {}\n", message);
        self.had_error = true;
    }
}
