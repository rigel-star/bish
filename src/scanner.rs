#[allow(unused)]

use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub enum TokenType
{
    TOKEN_FLOAT_NUM,
    TOKEN_INT_NUM,
    TOKEN_PLUS,
    TOKEN_MINUS,
    TOKEN_AND,
    TOKEN_OR,
    TOKEN_STRING,
    TOKEN_LEFT_BRACE,
    TOKEN_RIGHT_BRACE,
    TOKEN_LEFT_PAREN,
    TOKEN_RIGHT_PAREN,
    TOKEN_DOT,
    TOKEN_COMMA,
    TOKEN_STAR,
    TOKEN_SLASH,
    TOKEN_LEFT_BRACEKT,
    TOKEN_RIGHT_BRACKET,
    TOKEN_GHUMAU,
    TOKEN_PATAK,
    TOKEN_MA,
    TOKEN_RAKHA,
    TOKEN_DEKHAU,
    TOKEN_SEMICOLON,
    TOKEN_IDENTIFIER,
    TOKEN_SAHI,
    TOKEN_GALAT,
    TOKEN_NIL,
    TOKEN_THULO,
    TOKEN_SANO,
    TOKEN_BARABAR,
    TOKEN_CHHAINA,
    TOKEN_YADI,
    TOKEN_NATRA,
    TOKEN_NONE
}

#[derive(Debug)]
pub struct Token
{
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<String>,
    pub line: usize,
    pub column: usize
}

impl Token
{
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<String>, line: usize, column: usize) -> Token
    {
        Token {
            token_type,
            lexeme,
            literal,
            line,
            column
        }
    }

    pub fn none() -> Token
    {
        Token {
            token_type: TokenType::TOKEN_NONE,
            lexeme: String::from(""),
            literal: Option::Some(String::from("")),
            line: 1,
            column: 1
        }
    }
}

pub struct Scanner
{
    current: usize,
    start: usize,
    line: usize,
    source: String,
    keywords: HashMap<String, TokenType>
}

// static-like methods
impl Scanner
{
    pub fn new(source: String) -> Scanner
    {
        let mut keywords: HashMap<String, TokenType> = HashMap::new();
        keywords.insert(String::from("ghumau"), TokenType::TOKEN_GHUMAU);
        keywords.insert(String::from("patak"), TokenType::TOKEN_PATAK);
        keywords.insert(String::from("rakha"), TokenType::TOKEN_RAKHA);
        keywords.insert(String::from("dekhau"), TokenType::TOKEN_DEKHAU);
        keywords.insert(String::from("ma"), TokenType::TOKEN_MA);
        keywords.insert(String::from("sahi"), TokenType::TOKEN_SAHI);
        keywords.insert(String::from("galat"), TokenType::TOKEN_GALAT);
        keywords.insert(String::from("nil"), TokenType::TOKEN_NIL);
        keywords.insert(String::from("thulo"), TokenType::TOKEN_THULO);
        keywords.insert(String::from("sano"), TokenType::TOKEN_SANO);
        keywords.insert(String::from("barabar"), TokenType::TOKEN_BARABAR);
        keywords.insert(String::from("chhaina"), TokenType::TOKEN_CHHAINA);
        keywords.insert(String::from("yadi"), TokenType::TOKEN_YADI);
        keywords.insert(String::from("natra"), TokenType::TOKEN_NATRA);

        Scanner {
            current: 0,
            start: 0,
            line: 1,
            source,
            keywords
        }
    }
}

impl Scanner
{
    pub fn start_scan(&mut self) -> Vec<Token>
    {
        let mut result: Vec<Token> = Vec::new();
        while !self.is_at_end()
        {
            self.start = self.current;
            let token: Token = self.scan_token();
            match token.token_type
            {
                TokenType::TOKEN_NONE => (),
                _ => result.push(token)
            }
        }
        result.push(Token::none());
        result
    }

    fn _non_literal_token(&self, token_type: TokenType, lexeme: String) -> Token
    {
        self._literal_token(token_type, lexeme, None)
    }
    
    fn _literal_token(&self, token_type: TokenType, lexeme: String, literal: Option<String>) -> Token
    {
        Token {
            token_type,
            lexeme,
            literal,
            line: self.line,
            column: self.current
        }
    }

    fn scan_token(&mut self) -> Token
    {
        let chr: char = *self.advance() as char;
        if chr == '+'
        {
            self._non_literal_token(TokenType::TOKEN_PLUS, String::from("+"))
        }
        else if chr == '-'
        {
            self._non_literal_token(TokenType::TOKEN_MINUS, String::from("-"))
        }
        else if chr == ';'
        {
            self._non_literal_token(TokenType::TOKEN_SEMICOLON, String::from(";"))
        }
        else if chr == '.'
        {
            self._non_literal_token(TokenType::TOKEN_DOT, String::from("."))
        }
        else if chr == '*'
        {
            self._non_literal_token(TokenType::TOKEN_STAR, String::from("*"))
        }
        else if chr == '/'
        {
            Token::new(TokenType::TOKEN_SLASH, String::from("/"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '&'
        {
            Token::new(TokenType::TOKEN_AND, String::from("&"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '|'
        {
            Token::new(TokenType::TOKEN_OR, String::from("|"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '{'
        {
            Token::new(TokenType::TOKEN_LEFT_BRACE, String::from("{"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '}'
        {
            Token::new(TokenType::TOKEN_RIGHT_BRACE, String::from("}"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '['
        {
            Token::new(TokenType::TOKEN_LEFT_BRACEKT, String::from("["), Option::<_>::None, self.line, self.current)
        }
        else if chr == ']'
        {
            Token::new(TokenType::TOKEN_RIGHT_BRACKET, String::from("]"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '('
        {
            Token::new(TokenType::TOKEN_LEFT_PAREN, String::from("("), Option::<_>::None, self.line, self.current)
        }
        else if chr == ')'
        {
            Token::new(TokenType::TOKEN_RIGHT_PAREN, String::from(")"), Option::<_>::None, self.line, self.current)
        }
        else if chr == '_' || chr.is_alphabetic()
        {
            while (*self.peek() as char).is_alphanumeric() || (*self.peek() as char) == '_'
            {
                let _ = self.advance();
            }
            let ident: &str = &self.source[self.start..self.current];
            let token_type: Option<&TokenType> = self.keywords.get(ident);
            match token_type
            {
                Option::Some(value) => { return self._non_literal_token(*value, String::from(ident)); },
                Option::None => { return self._non_literal_token(TokenType::TOKEN_IDENTIFIER, String::from(ident)); }
            };
            Token::none()
        }
        else if chr == '"'
        {
            let _ = self.advance();
            let output: &str = self._parse_string();
            Token::new(TokenType::TOKEN_STRING, String::from(output), Option::Some(String::from(output)), self.line, self.current)
        }
        else if chr.is_ascii_digit()
        {
            let (number, is_double): (&str, bool) = self._parse_number();
            match is_double {
                true => Token::new(TokenType::TOKEN_FLOAT_NUM, String::from(number), Option::Some(String::from(number)), self.line, self.current),
                false => Token::new(TokenType::TOKEN_INT_NUM, String::from(number), Option::Some(String::from(number)), self.line, self.current)
            }
        }
        else if chr == '\n'
        {
            self.line += 1;
            Token::none()
        }
        else 
        {
            Token::none()
        }
    }

    fn _parse_identifier(&mut self) -> &str
    {
        &self.source[self.start + 1..self.current]
    }

    fn _parse_string(&mut self) -> &str 
    {
        while !self.is_at_end() && (*self.peek() as char) != '"'
        {
            let _ = self.advance();
        }

        if self.is_at_end()
        {
            println!("Unterminated string!");
            std::process::exit(4);
        }
        let current: usize = self.current;
        let _ = self.advance();
        &self.source[self.start + 1..current]
    }

    fn _parse_number(&mut self) -> (&str, bool)
    {
        let source: &str = &self.source[self.current..];
        let mut is_double: bool = false;
        while self.peek().is_ascii_digit() {
            let _ = self.advance();
        }

        if *self.peek() as char == '.'
        {
            is_double = true;
            self.advance();
            while self.peek().is_ascii_digit() {
                let _ = self.advance();
            }
        }
        (&self.source[self.start..self.current], is_double)
    }

    fn advance(&mut self) -> &u8
    {
        if !self.is_at_end()
        {
            let result: &u8 = &self.source.as_bytes()[self.current];
            self.current += 1;
            result
        }
        else {
            &0
        }
    }

    fn peek(&self) -> &u8
    {
        if self.is_at_end() { return &0; }
        &self.source.as_bytes()[self.current]
    }

    fn is_at_end(&self) -> bool
    {
        self.current >= self.source.len()
    }
}