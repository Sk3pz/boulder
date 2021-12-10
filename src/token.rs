use std::fmt::{Display, write};
use crate::{CodePos, Error};
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    NOP, EOF,
    Whitespace,
    Comma, // ,
    Dot, // .
    Range, // ..
    Colon, // :
    DoubleColon, // ::
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Interrupt, // @
    Panic, // ?

    Ident,

    StringLit,
    CharLit,
    NumberLit,
    HexLit,
    BinLit,

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
    Use,       // "use"
    Macro,     // "macro" (currently unused)
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
            TokenType::Range => write!(f, "Range"),
            TokenType::Colon => write!(f, "Colon"),
            TokenType::OpenParen => write!(f, "OpenParen"),
            TokenType::CloseParen => write!(f, "CloseParen"),
            TokenType::OpenBrace => write!(f, "OpenBrace"),
            TokenType::CloseBrace => write!(f, "CloseBrace"),
            TokenType::OpenBracket => write!(f, "OpenBracket"),
            TokenType::CloseBracket => write!(f, "CloseBracket"),
            TokenType::DoubleColon => write!(f, "DoubleColon"),
            TokenType::Interrupt => write!(f, "Interrupt"),
            TokenType::Panic => write!(f, "Panic"),
            TokenType::Use => write!(f, "Use"),
            TokenType::Macro => write!(f, "Macro"),
            TokenType::HexLit => write!(f, "HexLit"),
            TokenType::BinLit => write!(f, "BinLit"),
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

    /// removes whitespace until the next non-whitespace token is found
    pub fn optional_whitespace(&mut self) {
        while let Some(t) = self.peek() {
            if t.token_type == TokenType::Whitespace {
                self.consume();
            } else {
                break;
            }
        }
    }

    pub fn expect_whitespace(&mut self) -> Result<Token, Error> {
        let token = self.consume()
            .ok_or_else(|| Error::new("Unexpected EOF", "expected Whitespace", self.eof()))?;
        if token.token_type != TokenType::Whitespace {
            return Err(Error::new("Unexpected token", format!("expected Whitespace"), token.start));
        }
        self.optional_whitespace(); // remove following whitespace
        Ok(token)
    }

    /// expects a specific token type and consumes it.
    pub fn expect(&mut self, token_type: TokenType) -> Result<Token, Error> {
        if token_type == TokenType::Whitespace {
            return self.expect_whitespace(); // if its expecting whitespace, return the proper funciton
        } else {
            self.optional_whitespace(); // remove leading whitespace
        }
        let token = self.consume()
            .ok_or_else(|| Error::new("Unexpected EOF", format!("expected {}", token_type), self.eof()))?;
        if token.token_type != token_type {
            return Err(Error::new("Unexpected token", format!("expected {}", token_type), token.start));
        }
        Ok(token)
    }

    pub fn optional_expect(&mut self, token_type: TokenType) -> Result<Option<Token>, Error> {
        if token_type != TokenType::Whitespace {
            self.optional_whitespace(); // remove leading whitespace
        } else { // optionally expecting whitespace (this is 100% over-engineered, but its cool)
            return if let Some(t) = self.peek() {
                if t.token_type == TokenType::Whitespace {
                    self.consume();
                    self.optional_whitespace(); // remove following whitespace
                    Ok(Some(t))
                } else {
                    Ok(None) // no whitespace found
                }
            } else {
                Ok(None) // EOF
            }
        }

        // if there is no next token, return none as it can't be the expected token
        if self.peek().is_none() {
            return Ok(None);
        }

        // if it is the expected token type, return the token, otherwise return none
        if self.peek().unwrap().token_type == token_type {
            Ok(Some(self.consume().unwrap())) // return it
        } else {
            Ok(None) // return none as the token was not the expected type
        }
    }

    pub fn expect_op(&mut self, op: Operator) -> Result<Operator, Error> {
        let token = self.expect(TokenType::Operator)?;
        Ok(token.op.unwrap())
    }

    pub fn optional_op(&mut self, op: Operator) -> Result<Option<Operator>, Error> {
        if let Some(t) = self.optional_expect(TokenType::Operator)? {
            Ok(Some(t.op.unwrap()))
        } else {
            Ok(None)
        }
    }

    pub fn next_is(&self, tt: TokenType) -> bool {
        self.peek().map(|t| t.token_type == tt).unwrap_or(false)
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