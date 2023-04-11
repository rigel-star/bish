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
    panic_mode: bool,
    rules: HashMap<scanner::TokenType, &'compiling (Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)>
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
            panic_mode: false,
            rules: HashMap::from_iter(vec![
                (scanner::TokenType::TOKEN_LEFT_PAREN, &(Some(Parser::parse_grouping as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_RIGHT_PAREN, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_LEFT_BRACE, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_RIGHT_BRACE, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_FLOAT_NUM, &(Some(Parser::parse_number as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_INT_NUM, &(Some(Parser::parse_number as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_PLUS, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_TERM)),
                (scanner::TokenType::TOKEN_MINUS, &(Some(Parser::parse_unary as fn(&mut Self)), Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_TERM)),
                (scanner::TokenType::TOKEN_SLASH, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_FACTOR)),
                (scanner::TokenType::TOKEN_STAR, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_FACTOR)),
                (scanner::TokenType::TOKEN_THULO, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_COMPARISON)),
                (scanner::TokenType::TOKEN_SANO, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_COMPARISON)),
                (scanner::TokenType::TOKEN_BARABAR, &(None, Some(Parser::parse_binary as fn(&mut Self)), Precedence::PREC_EQUALITY)),
                (scanner::TokenType::TOKEN_SAHI, &(Some(Parser::parse_literal as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_GALAT, &(Some(Parser::parse_literal as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_NIL, &(Some(Parser::parse_literal as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_STRING, &(Some(Parser::parse_string as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_CHHAINA, &(Some(Parser::parse_unary as fn(&mut Self)), None, Precedence::PREC_UNARY)),
                (scanner::TokenType::TOKEN_NONE, &(None, None, Precedence::PREC_NONE)),
            ])
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

    fn parse_expression(&mut self)
    {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }

    fn parse_precedence(&mut self, prec: Precedence)
    {
        self.advance();
        let prefix = self.get_rule(self.previous.token_type);
        if let Some(func_tuple) = prefix 
        {
            if let Some(prefix_func) = func_tuple.0 
            {
                prefix_func(self);
            }
            else
            {
                panic!("Expected expression!")
            }
        }

        // maybe prec has to be reassigned?
        while prec < self.get_rule(self.current.token_type).unwrap().2
        {
            self.advance();
            let infix = self.get_rule(self.previous.token_type);
            if let Some(func_tuple) = infix 
            {
                if let Some(infix_func) = func_tuple.1 
                {
                    infix_func(self);
                }
            }
        }
    }

    fn parse_literal(&mut self)
    {
        match self.previous.token_type
        {
            scanner::TokenType::TOKEN_SAHI => self.chunk.write_bool(true),
            scanner::TokenType::TOKEN_GALAT => self.chunk.write_bool(false),
            scanner::TokenType::TOKEN_NIL => self.chunk.write_nil(),
            _ => ()
        }
    }

    fn parse_string(&mut self)
    {
        self.chunk.write_cstring(self.previous.literal.clone().unwrap());
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
        let prec: Precedence = self.get_rule(ttype).unwrap().2;
        self.parse_precedence(prec);
        match ttype 
        {
            scanner::TokenType::TOKEN_PLUS => {
                self.emit_bytecode(chunk::OpCode::OP_ADD as u8);
            },
            scanner::TokenType::TOKEN_MINUS => {
                self.emit_bytecode(chunk::OpCode::OP_SUBTRACT as u8);
            },
            scanner::TokenType::TOKEN_STAR => {
                self.emit_bytecode(chunk::OpCode::OP_MULTIPLY as u8);
            },
            scanner::TokenType::TOKEN_SLASH => {
                self.emit_bytecode(chunk::OpCode::OP_DIVIDE as u8);
            },
            scanner::TokenType::TOKEN_THULO => self.emit_bytecode(chunk::OpCode::OP_GT as u8),
            scanner::TokenType::TOKEN_SANO => self.emit_bytecode(chunk::OpCode::OP_LT as u8),
            scanner::TokenType::TOKEN_BARABAR => self.emit_bytecode(chunk::OpCode::OP_EQ_EQ as u8),
            _ => ()
        }
    }

    #[allow(clippy::single_match)]
    fn parse_unary(&mut self)
    {
        let token: &scanner::Token = self.previous;
        self.parse_precedence(Precedence::PREC_UNARY);
        match token.token_type
        {
            scanner::TokenType::TOKEN_MINUS => self.emit_bytecode(chunk::OpCode::OP_NEGATE as u8),
            scanner::TokenType::TOKEN_CHHAINA => self.emit_bytecode(chunk::OpCode::OP_NOT as u8),
            _ => ()
        }
    }

    fn parse_grouping(&mut self)
    {
        self.parse_expression();
        self.consume(scanner::TokenType::TOKEN_RIGHT_PAREN, "Expected ')' after expression.");
    }

    fn get_rule(&mut self, token_type: scanner::TokenType) -> Option<&(Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)>
    {
        println!("DEBUG[get_rule]: TokenType = {:?}", token_type);
        Some(self.rules[&token_type])
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

        println!(": {}", message);
        self.had_error = true;
    }
}
