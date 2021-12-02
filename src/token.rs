use std::fmt::{Display, write};
use crate::CodePos;
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    NOP, EOF,
    Whitespace,
    Comma,
    Dot,
    Colon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Ident,

    StringLit,
    CharLit,
    NumberLit,

    Operator,

    // keywords
    Fn,        // "fn"
    Let,       // "let"
    If,        // "if"
    Else,      // "else"
    While,     // "while"
    For,       // "for"
    Loop,      // "loop"
    Return,    // "return"
    Match,     // "match"
    Struct,    // "struct"
    BoolTrue,  // "true"
    BoolFalse, // "false"
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TokenType::NOP => write!(f, "NOP"),
            TokenType::EOF => write!(f, "EOF"),
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Ident => write!(f, "Ident"),
            TokenType::StringLit => write!(f, "StringLit"),
            TokenType::CharLit => write!(f, "CharLit"),
            TokenType::NumberLit => write!(f, "NumberLit"),
            TokenType::Operator => write!(f, "Operator"),
            TokenType::Fn => write!(f, "Fn"),
            TokenType::Let => write!(f, "Let"),
            TokenType::If => write!(f, "If"),
            TokenType::Else => write!(f, "Else"),
            TokenType::While => write!(f, "While"),
            TokenType::For => write!(f, "For"),
            TokenType::Loop => write!(f, "Loop"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Match => write!(f, "Match"),
            TokenType::Struct => write!(f, "Struct"),
            TokenType::BoolTrue => write!(f, "BoolTrue"),
            TokenType::BoolFalse => write!(f, "BoolFalse"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Colon => write!(f, "Colon"),
            TokenType::OpenParen => write!(f, "OpenParen"),
            TokenType::CloseParen => write!(f, "CloseParen"),
            TokenType::OpenBrace => write!(f, "OpenBrace"),
            TokenType::CloseBrace => write!(f, "CloseBrace"),
            TokenType::OpenBracket => write!(f, "OpenBracket"),
            TokenType::CloseBracket => write!(f, "CloseBracket"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub value: Option<String>,
    pub op: Option<Operator>,
    pub start: CodePos,
    pub end: CodePos,
}

impl Token {
    pub fn new(token_type: TokenType, start: CodePos, end: CodePos) -> Token {
        Token {
            token_type,
            value: None,
            op: None,
            start,
            end,
        }
    }

    pub fn new_lit(token_type: TokenType, value: String, start: CodePos, end: CodePos) -> Token {
        Token {
            token_type,
            value: Some(value),
            op: None,
            start,
            end,
        }
    }

    pub fn new_ident(value: String, start: CodePos, end: CodePos) -> Token {
        Token {
            token_type: TokenType::Ident,
            value: Some(value),
            op: None,
            start,
            end,
        }
    }

    pub fn new_op(value: Operator, start: CodePos, end: CodePos) -> Token {
        Token {
            token_type: TokenType::Operator,
            value: None,
            op: Some(value),
            start,
            end,
        }
    }

}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = format!("{}", self.token_type);
        if let Some(ref value) = self.value {
            output += format!(": {}", value).as_str();
        } else if let Some(ref value) = self.op {
            output += format!(": {}", value).as_str();
        }
        write!(f, "{}", output)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenList {
    pub tokens: Vec<Token>,
    pub iter_place: usize,
    eof_loc: CodePos,
}

impl TokenList {

    pub fn new(tokens: Vec<Token>) -> Self {
        let eof_loc = tokens.last().unwrap().clone().start;
        Self {
            tokens,
            iter_place: 0,
            eof_loc,
        }
    }

    /// Used for functions in the parser that reach an unexpected end of file and need access to the location for errors.
    pub fn eof(&self) -> CodePos {
        self.eof_loc.clone()
    }

    /// Peek at the nth token in the list without removing it
    /// Returns None if the index is out of bounds
    pub fn peek_nth(&self, n: usize) -> Option<Token> {
        return if let Some(t) = self.tokens.get(n) {
            Some(t.clone())
        } else {
            None
        }
    }

    /// Peek at the next token in the list without removing it.
    /// Returns None if there is no next token
    pub fn peek(&self) -> Option<Token> {
        self.peek_nth(0)
    }

    /// Removes the nth token in the list and returns it.
    /// Returns None if the index is out of bounds
    pub fn consume_nth(&mut self, n: usize) -> Option<Token> {
        return if n >= self.tokens.len() {
            None
        } else {
            Some(self.tokens.remove(n))
        }
    }

    /// Removes the next token in the list and returns it.
    /// Returns None if there is no next token.
    pub fn consume(&mut self) -> Option<Token> {
        self.consume_nth(0)
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns the length of the token list.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
}


impl Iterator for TokenList {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        return if self.iter_place < self.len() {
            let place = self.iter_place;
            self.iter_place += 1;
            self.peek_nth(place)
        } else {
            None
        }
    }
}

impl Display for TokenList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut total = String::new();
        for x in 0..self.tokens.len() {
            if x % 5 == 0 && x != 0 {
                total += format!("\n").as_str();
            }
            total += format!("{}", self.tokens.get(x).unwrap()).as_str();
            if x != self.tokens.len() - 1 {
                total += format!(", ").as_str();
            }
        }
        write!(f, "[{}]", total)
    }
}