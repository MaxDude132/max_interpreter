use std::fmt::Display;

use crate::value::Value;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Dot,
    Minus,
    Plus,
    Colon,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Integer,
    Float,

    // Type annotations
    TypeFloat,
    TypeInt,
    TypeString,
    TypeBool,
    TypeFunction,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    In,
    Break,
    Continue,
    If,
    Or,
    None,
    Print,
    Return,
    Super,
    Me,
    Cls,
    True,
    While,

    Error,
    Eof,
    Newline,
    Empty,
}

impl TokenType {
    pub fn is_type(&self) -> bool {
        match self {
            TokenType::TypeFloat
            | TokenType::TypeInt
            | TokenType::TypeString
            | TokenType::TypeBool => true,
            _ => false,
        }
    }

    pub fn is_correct_type(&self, value: &Value) -> bool {
        match self {
            TokenType::TypeFloat => match value {
                Value::Float(_) => true,
                Value::FloatNone => true,
                _ => false,
            },
            TokenType::TypeInt => match value {
                Value::Integer(_) => true,
                Value::IntegerNone => true,
                _ => false,
            },
            TokenType::TypeString => match value {
                Value::String(_) => true,
                Value::StringNone => true,
                _ => false,
            },
            TokenType::TypeBool => match value {
                Value::True | Value::False => true,
                Value::BoolNone => true,
                _ => false,
            },
            TokenType::TypeFunction => match value {
                Value::ObjFunction(_) => true,
                _ => false,
            },
            TokenType::None => true,
            _ => false,
        }
    }

    pub fn get_none_type(&self) -> Value {
        match self {
            TokenType::TypeFloat => Value::FloatNone,
            TokenType::TypeInt => Value::IntegerNone,
            TokenType::TypeString => Value::StringNone,
            TokenType::TypeBool => Value::BoolNone,
            _ => Value::None,
        }
    }
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::LeftSquareBracket => "[",
            TokenType::RightSquareBracket => "]",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Minus => "-",
            TokenType::Plus => "+",
            TokenType::Colon => ":",
            TokenType::Semicolon => ";",
            TokenType::Slash => "/",
            TokenType::Star => "*",
            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            TokenType::Identifier => "identifier",
            TokenType::String => "string",
            TokenType::Integer => "integer",
            TokenType::Float => "float",
            TokenType::TypeFloat => "float",
            TokenType::TypeInt => "int",
            TokenType::TypeString => "string",
            TokenType::TypeBool => "bool",
            TokenType::And => "and",
            TokenType::Class => "class",
            TokenType::Else => "else",
            TokenType::False => "false",
            TokenType::For => "for",
            TokenType::In => "in",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::If => "if",
            TokenType::Or => "or",
            TokenType::None => "none",
            TokenType::Print => "print",
            TokenType::Return => "return",
            TokenType::Super => "super",
            TokenType::Me => "me",
            TokenType::Cls => "cls",
            TokenType::True => "true",
            TokenType::While => "while",
            TokenType::Error => "error",
            TokenType::Eof => "eof",
            TokenType::Newline => "newline",
            TokenType::Empty => "empty",
            TokenType::TypeFunction => "function",
        };
        write!(f, "{}", token)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(r#type: TokenType, line: usize) -> Token {
        Token {
            r#type,
            lexeme: String::new(),
            line,
        }
    }
}

#[derive(Clone)]
pub struct Scanner {
    start: usize,
    current: usize,
    line: usize,
    source: String,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            start: 0,
            current: 0,
            line: 1,
            source,
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        if c.is_alphabetic() {
            return self.identifier();
        }
        if c.is_digit(10) {
            return self.number();
        }
        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            '[' => return self.make_token(TokenType::LeftSquareBracket),
            ']' => return self.make_token(TokenType::RightSquareBracket),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            ':' => return self.make_token(TokenType::Colon),
            ';' => return self.make_token(TokenType::Semicolon),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '\n' => {
                self.start = self.current;
                let token = self.make_token(TokenType::Newline);
                self.line += 1;
                return token;
            }
            '!' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::BangEqual);
                } else {
                    return self.make_token(TokenType::Bang);
                }
            }
            '=' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::EqualEqual);
                } else {
                    return self.make_token(TokenType::Equal);
                }
            }
            '<' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::LessEqual);
                } else {
                    return self.make_token(TokenType::Less);
                }
            }
            '>' => {
                if self.match_char('=') {
                    return self.make_token(TokenType::GreaterEqual);
                } else {
                    return self.make_token(TokenType::Greater);
                }
            }
            '"' | '\'' => {
                return self.string();
            }
            _ => {}
        }

        return self.error_token("Unexpected character.");
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.current += 1;
                }
                '-' => {
                    if self.peek_next() == '-' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.current += 1;
                        }
                    } else if self.peek_next() == '*' {
                        self.current += 2;
                        while self.peek() != '*' && self.peek_next() != '-' && !self.is_at_end() {
                            if self.peek() == '\n' {
                                self.line += 1;
                            }
                            self.current += 1;
                        }
                        self.current += 2;
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source.chars().nth(self.current - 1).unwrap()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).unwrap()
    }

    fn string(&mut self) -> Token {
        let quote = self.source.chars().nth(self.start).unwrap();
        self.start += 1;
        let start_line = self.line;
        while self.peek() != quote && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            if self.peek() == '\\' && self.peek_next() == quote {
                self.current += 1;
            }
            self.current += 1;
        }

        if self.is_at_end() {
            return self.error_token_with_line("Unterminated string.", start_line);
        }

        let token = self.make_token(TokenType::String);
        self.current += 1;
        return token;
    }

    fn number(&mut self) -> Token {
        while self.peek().is_digit(10) {
            self.current += 1;
        }

        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.current += 1;
            while self.peek().is_digit(10) {
                self.current += 1;
            }
        } else {
            return self.make_token(TokenType::Integer);
        }

        self.make_token(TokenType::Float)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            self.current += 1;
        }

        let token = self.make_token(self.identifier_type());
        return token;
    }

    fn identifier_type(&self) -> TokenType {
        match self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect::<String>()
            .as_str()
        {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "if" => TokenType::If,
            "or" => TokenType::Or,
            "print" => TokenType::Print, // TODO: Remove eventually
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "me" => TokenType::Me,
            "cls" => TokenType::Cls,
            "true" => TokenType::True,
            "while" => TokenType::While,
            "none" => TokenType::None,
            "int" => TokenType::TypeInt,
            "float" => TokenType::TypeFloat,
            "bool" => TokenType::TypeBool,
            "string" => TokenType::TypeString,
            _ => TokenType::Identifier,
        }
    }

    fn make_token(&self, r#type: TokenType) -> Token {
        Token {
            r#type,
            lexeme: self
                .source
                .chars()
                .skip(self.start)
                .take(self.current - self.start)
                .collect(),
            line: self.line,
        }
    }

    fn error_token(&self, message: &str) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: message.to_string(),
            line: self.line,
        }
    }

    fn error_token_with_line(&self, message: &str, line: usize) -> Token {
        Token {
            r#type: TokenType::Error,
            lexeme: message.to_string(),
            line: line,
        }
    }
}
