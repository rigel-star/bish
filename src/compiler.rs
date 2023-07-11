#![allow(dead_code)]
#![allow(unused)]
#![allow(non_camel_case_types)]

use std::collections::HashMap;
use std::fs;

use crate::chunk::OpCode;
use crate::scanner;
use crate::chunk;
use crate::scanner::TokenType;

#[derive(PartialEq, PartialOrd, Clone, Copy, Hash, Eq, Debug)]
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

pub struct Parser<'compiling>
{
    source_file_path: String,
    tokens: &'compiling Vec<scanner::Token>,
    chunk: &'compiling mut chunk::Chunk,
    current: &'compiling scanner::Token,
    previous: &'compiling scanner::Token,
    counter: usize,
    had_error: bool,
    panic_mode: bool,
    rules: HashMap<scanner::TokenType, &'compiling (Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)>
}

/* 'static-like' method definitions */
impl<'compiling> Parser<'compiling>
{
    pub fn new(source_file_path: String, tokens: &'compiling Vec<scanner::Token>, chunk: &'compiling mut chunk::Chunk) -> Parser<'compiling>
    {
        Parser {
            source_file_path,
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
                (scanner::TokenType::TOKEN_DEKHAU, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_SEMICOLON, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_RAKHA, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_MA, &(None, None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_IDENTIFIER, &(Some(Parser::parse_variable as fn(&mut Self)), None, Precedence::PREC_NONE)),
                (scanner::TokenType::TOKEN_NONE, &(None, None, Precedence::PREC_NONE)),
            ])
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum CompilationResult
{
    Ok,
    Error
}

impl<'compiling> Parser<'compiling>
{
    fn debugg(&self, fn_name: &str)
    {
        println!("DEBUG[{}]: current token = {:?}", fn_name, self.current.token_type);
        println!("DEBUG[{}]: previous token = {:?}", fn_name, self.previous.token_type);
    }

    #[inline]
    pub fn compile(&mut self) -> CompilationResult
    {
        // self.advance();
        while !self._match(&scanner::TokenType::TOKEN_NONE)
        {
            self._parse_decl_stmt();
            if self.panic_mode{ self._sync_err(); }
        }
        if self.had_error { CompilationResult::Error }
        else { CompilationResult::Ok }
    }

    #[inline]
    fn _parse_decl_stmt(&mut self)
    {
        self.advance();
        if self.previous.token_type == TokenType::TOKEN_RAKHA
        {
            self._parse_var_decl_stmt();
        }
        else 
        { 
            self._parse_stmt(); 
        }
    }

    fn _parse_var_decl_stmt(&mut self)
    {
        if !self._match(&scanner::TokenType::TOKEN_IDENTIFIER) {
            self.error_at_current(&format!("'rakha' lekhe pachhi tapaile variable ko naam dina parne hunchha. '{}' chai aasha gariyeko thiyena.", self.current.lexeme));
            return;
        }
        let var_name: &String = &self.previous.lexeme;
        if self._match(&TokenType::TOKEN_MA)
        {
            self.parse_expression();
            self.consume(TokenType::TOKEN_SEMICOLON, "Tapaile sayed 'rakha' statement lai antya garna ';' lekhna chhutaunu bhayo hola.");
        }
        else 
        {
            self.emit_bytecode(chunk::OpCode::OP_NIL as u8);
            if !self._match(&TokenType::TOKEN_SEMICOLON) {
                if self.current.token_type == scanner::TokenType::TOKEN_NONE {
                    self.consume(TokenType::TOKEN_SEMICOLON, &format!("Yadi '{}' ma kae value rakhnu chhaina bhane ';' lekhnus.", var_name));
                    return;
                }
                self.consume(TokenType::TOKEN_SEMICOLON, &format!("Yadi '{}' ma kae value rakhnu chhaina bhane ';' lekhnus. '{}' chai aasha gariyeko thiyena.", var_name, self.current.lexeme));
            }
        }
        self.chunk.write_const(chunk::PrimType::CString(var_name.len(), var_name.clone()));
        self.emit_bytecode(chunk::OpCode::OP_DEF_GLOBAL as u8);
    }

    #[inline]
    fn _parse_stmt(&mut self) {
        if self.previous.token_type == scanner::TokenType::TOKEN_DEKHAU {
            self._parse_print_stmt();  
        }
        else if self.previous.token_type == scanner::TokenType::TOKEN_YADI {
            self._parse_if_stmt();
        }
        else if self.previous.token_type == scanner::TokenType::TOKEN_NATRA {
            self._parse_natra_stmt();
        }
        else if self.previous.token_type == scanner::TokenType::TOKEN_LEFT_BRACE {
            self._parse_block_stmt();
        }
        else {
            self._parse_expr_stmt();
        }
    }

    fn _parse_if_stmt(&mut self) {
        self.parse_expression();
        self.emit_bytecode(chunk::OpCode::OP_JMP_IF_FALSE as u8);
        self.emit_bytecode(0xFF);
        self.emit_bytecode(0xFF);
        let jump_offset: usize = self.chunk.code.len() - 2;
        self.advance();
        self._parse_block_stmt();
        let jump_op_count: usize = self.chunk.code.len() - jump_offset - 2;
        self.chunk.code[jump_offset] = ((jump_op_count >> 8) & 0xFF) as u8;
        self.chunk.code[jump_offset + 1] = (jump_op_count & 0xFF) as u8;
    }

    fn _parse_natra_stmt(&mut self) {
        self.emit_bytecode(chunk::OpCode::OP_ELSE as u8);
        self.emit_bytecode(0xFF);
        self.emit_bytecode(0xFF);
        let jump_offset: usize = self.chunk.code.len() - 2;
        self.advance();
        self._parse_block_stmt();
        let jump_op_count: usize = self.chunk.code.len() - jump_offset - 2;
        self.chunk.code[jump_offset] = ((jump_op_count >> 8) & 0xFF) as u8;
        self.chunk.code[jump_offset + 1] = (jump_op_count & 0xFF) as u8;
    }
    
    fn emit_jump_bytecode(&mut self, code: chunk::OpCode) -> usize {
        self.emit_bytecode(code as u8);
        self.emit_bytecode(0xFF);
        self.emit_bytecode(0xFF);
        (self.chunk.code.len() - 2)
    }

    fn patch_jump_stmt(&mut self, jump_offset: usize) {
        let jump_op_count: usize = self.chunk.code.len() - jump_offset - 2;
        self.chunk.code[jump_offset] = ((jump_op_count >> 8) & 0xFF) as u8;
        self.chunk.code[jump_offset + 1] = (jump_op_count & 0xFF) as u8;
    }

    fn _parse_block_stmt(&mut self)
    {
        while !self._check(TokenType::TOKEN_RIGHT_BRACE) && !self._check(TokenType::TOKEN_NONE) {
            self._parse_decl_stmt();
        } 
        self.consume(TokenType::TOKEN_RIGHT_BRACE, "'{' lekhisake pachhi '}' pani lekhnus.");
    }

    #[inline]
    fn _parse_expr_stmt(&mut self)
    {
        self.parse_expression();
        self.consume(scanner::TokenType::TOKEN_SEMICOLON, format!("Tapaile sayed '{}' pachhi ';' lekhna chhutaunu bhayo hola.", self.previous.lexeme).as_str());
        // self.emit_bytecode(chunk::OpCode::OP_POP as u8);
    }

    fn _parse_print_stmt(&mut self)
    {
        self.parse_expression();
        self.consume(TokenType::TOKEN_SEMICOLON, "Tapaile sayed dekhau statement sakiye pachhi ';' lekhna chhutaunu bhayo hola.");
        self.emit_bytecode(chunk::OpCode::OP_PRINT as u8);
    }

    fn _sync_err(&mut self)
    {
        self.panic_mode = false;
        while self.current.token_type != scanner::TokenType::TOKEN_NONE
        {
            if self.current.token_type == scanner::TokenType::TOKEN_SEMICOLON { return; }
            match self.current.token_type
            {
                scanner::TokenType::TOKEN_DEKHAU | scanner::TokenType::TOKEN_GHUMAU => return,
                _ => ()
            }
            self.advance();
        }
        self.panic_mode = false;
    }

    #[inline]
    fn parse_expression(&mut self)
    {
        self.parse_precedence(Precedence::PREC_ASSIGNMENT);
    }

    fn parse_precedence(&mut self, prec: Precedence)
    {
        let now: &scanner::Token = self.previous;
        self.advance();
        let prefix = self.get_rule(self.previous.token_type);
        if let Some(func_tuple) = prefix 
        {
            if let Some(prefix_func) = func_tuple.0 
            {
                prefix_func(self);
            }
            // else
            // {
            //     self.error_at(self.counter - 1, &format!("'{}' pachhadi expression dinus.", now.lexeme));
            // }
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

    #[inline]
    fn parse_string(&mut self)
    {
        self.chunk.write_cstring(self.previous.literal.clone().unwrap());
    }

    fn parse_variable(&mut self)
    {
        self.chunk.write_const(chunk::PrimType::CString(self.previous.lexeme.len(), self.previous.lexeme.clone()));
        self.emit_bytecode(OpCode::OP_LOAD_GLOBAL as u8); 
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
        self.consume(scanner::TokenType::TOKEN_RIGHT_PAREN, "Tapaile sayed '(' lekhi sake pachhi, teslai antya garna ')' lekhna chhutaunu bhayo hola.");
    }

    fn get_rule(&self, token_type: scanner::TokenType) -> Option<&(Option<fn(&mut Self)>, Option<fn(&mut Self)>, Precedence)>
    {
        // println!("DEBUG[get_rule]: TokenType = {:?}", token_type);
        Some(self.rules[&token_type])
    }

    /*
    * This function advances the current pointer by one if given 
    * token type matches the current current token type.
    */
    fn _match(&mut self, token_type: &TokenType) -> bool 
    {
        if *token_type == self.current.token_type 
        { 
            self.advance();
            true 
        }
        else { false }
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
        self.counter += 1;
        if self.counter < self.tokens.len()
        {
            self.current = &self.tokens[self.counter];
        }
    }

    fn _check(&self, typ: TokenType) -> bool
    {
        self.current.token_type == typ
    }

    fn error_at_current(&mut self, message: &str)
    {
        self.error_at(self.counter, message);
    }

    fn error_at(&mut self, token_idx: usize, message: &str)
    {
        let token: &scanner::Token = &self.tokens[token_idx - 1];
        print!("\x1b[1;31mCompilation error\x1b[0;37m: {}", message);
        println!("\n  \x1b[1;34m-->\x1b[0;37m {}:{}:{}", self.source_file_path, token.line, token.column);
        println!("\n\x1b[0;37m");
        self._sync_err();
        self.had_error = true;
    }
}
