use crate::error::Error;
use crate::input_reader::InputReader;
use crate::operator::Operator;
use crate::token::{Token, TokenList, TokenType};

// todo(eric): This will lex 10,000 lines of code in about 4 seconds, so it has a ton of room to
//  be optimized.

/// returns the next identifier to be processed by the lexer.
fn next_ident(input: &mut InputReader) -> String {
    let mut ident = String::new();
    while let Some(c) = input.peek() {
        if c.is_alphanumeric() || c == '_' {
            input.consume();
            ident.push(c);
        } else {
            break;
        }
    }
    ident
}

/// returns the next hex literal
fn lex_hex_lit(input: &mut InputReader) -> Result<Token, Error> {
    let start = input.pos();
    input.consume(); // remove the 0
    input.consume(); // remove the x
    let mut hex_lit = String::new();
    while let Some(c) = input.peek() {
        if !(c.is_numeric() || (c >= (97 as char) && c <= (102 as char)) || (c >= (65 as char) && c <= (70 as char))) {
            break;
        }
        input.consume();
        hex_lit.push(c);
    }
    Ok(Token::new_lit(TokenType::HexLit, hex_lit, start, input.pos()))
}

fn lex_bin_lit(input: &mut InputReader) -> Result<Token, Error> {
    let start = input.pos();
    input.consume(); // remove the 0
    input.consume(); // remove the b
    let mut bin_lit = String::new();
    while let Some(c) = input.peek() {
        if !(c == '1' || c == '0') {
            break;
        }
        input.consume();
        bin_lit.push(c);
    }
    Ok(Token::new_lit(TokenType::BinLit, bin_lit, start, input.pos()))
}

/// returns the next numeric literal
fn next_numeric(input: &mut InputReader) -> Result<String, Error> {
    let start = input.pos(); // used for error handling
    let mut num = String::new(); // the number that will be returned
    //let mut decimal = false; // if the number is a decimal
    while let Some(c) = input.peek() {
        if c.is_numeric() { // if the character is a number
            input.consume();
            num.push(c); // add the number to the string
        } else if c == '.' { // if the character is a decimal
            // if there is no next character,
            let next_peek = input.peek_at(1);
            if next_peek.is_none() {
                return Err(Error::new(
                    "Unexpected token at EOF", format!("Found {}", c), start));
            }
            // handle decimal numbers
            let next = next_peek.unwrap();
            if next.is_numeric() {
                return Err(Error::new(
                    "Unexpected Token", format!("Decimals/floating point numbers are not yet supported!"), input.pos()));
                // todo(eric): handle decimal numbers
                // if decimal {
                //     return Err(Error::new(
                //         "Extra Decimal Point", format!("The number literal is already a decimal!"), start));
                // }
                // decimal = true;
                // input.consume(); // consume the .
                // num.push(c);
                // continue; // continue until the number is complete
            } else {
                break; // the . is a property accessor, so break and return the number
            }
        } else { // todo(eric): Potentially handle type flags here for numeric literals (i.e. 10u8)
            break;
        }
    }
    Ok(num)
}

/// lex the current operator and determines if it is a multi-character operator or the default 1 character
fn lex_op_other(input: &mut InputReader, other: Operator, secondary: char) -> Option<Operator> {
    if let Some(next) = input.peek() {
        if next == secondary {
            input.consume();
            return Some(other);
        }
    }
    None
}
/// lex the current operator to determine if it is one or the other depending on the next char
fn lex_op_or(input: &mut InputReader, norm: Operator, other: Operator, secondary: char) -> Operator {
    lex_op_other(input, other, secondary).unwrap_or(norm)
}
/// determines if the next operator is an assignment or normal (i.e. += or +)
fn lex_op(input: &mut InputReader, norm: Operator, assign: Operator) -> Operator {
    lex_op_other(input, assign, '=').unwrap_or(norm)
}

pub fn next_token(input: &mut InputReader) -> Result<Token, Error> {
    let next = input.peek().unwrap();
    let start = input.pos();
    match next {
        // general tokens
        ' ' | '\n' | '\t' => {
            input.consume();
            Ok(Token::new(TokenType::Whitespace, start, input.pos()))
        }
        ';' => {
            input.consume();
            Ok(Token::new(TokenType::NOP, start, input.pos()))
        }
        '(' => {
            input.consume();
            Ok(Token::new(TokenType::OpenParen, start, input.pos()))
        }
        ')' => {
            input.consume();
            Ok(Token::new(TokenType::CloseParen, start, input.pos()))
        }
        '{' => {
            input.consume();
            Ok(Token::new(TokenType::OpenBracket, start, input.pos()))
        }
        '}' => {
            input.consume();
            Ok(Token::new(TokenType::CloseBracket, start, input.pos()))
        }
        '[' => {
            input.consume();
            Ok(Token::new(TokenType::OpenBrace, start, input.pos()))
        }
        ']' => {
            input.consume();
            Ok(Token::new(TokenType::CloseBrace, start, input.pos()))
        }
        ',' => {
            input.consume();
            Ok(Token::new(TokenType::Comma, start, input.pos()))
        }
        '@' => {
            input.consume();
            Ok(Token::new(TokenType::Interrupt, start, input.pos()))
        }
        '?' => { // ?
            input.consume();
            Ok(Token::new(TokenType::Panic, start, input.pos()))
        }
        ':' => {
            input.consume();
            if let Some(next) = input.peek() {
                if next == ':' {
                    input.consume();
                    return Ok(Token::new(TokenType::DoubleColon, start, input.pos()));
                }
            }
            Ok(Token::new(TokenType::Colon, start, input.pos()))
        }
        '.' => {
            input.consume();
            if let Some(next) = input.peek() {
                if next == '.' {
                    input.consume();
                    if let Some(next) = input.peek() {
                        if next == '=' {
                            input.consume();
                            return Ok(Token::new_op(Operator::IRange, start, input.pos()));
                        }
                    }
                    return Ok(Token::new_op(Operator::Range, start, input.pos()));
                }
            }
            Ok(Token::new(TokenType::Dot, start, input.pos()))
        }
        '"' => { // process string literals
            // todo(eric): handle escape sequences and string formatting
            input.consume();
            let mut strlit = String::new();
            while let Some(c) = input.peek() {
                if c == '"' {
                    input.consume();
                    break;
                } else {
                    input.consume();
                    strlit.push(c);
                    if input.peek().is_none() {
                        return Err(Error::new(
                            "String literal with no close",
                            format!("Reached EOF before finding closing '\"'"), start));
                    }
                }
            }
            Ok(Token::new_lit(TokenType::StringLit, strlit, start, input.pos()))
        }
        '\'' => {
            input.consume();
            if let None = input.peek() {
                return Err(Error::new(
                    "Character literal with no close",
                    format!("Reached EOF before finding closing '\''"), start));
            }
            let charlit = input.consume().unwrap();
            // todo(eric): handle escape characters
            if let None = input.peek() {
                return Err(Error::new(
                    "Character literal with no close",
                    format!("Reached EOF before finding closing '\''"), start));
            }
            let next = input.consume().unwrap();
            if next != '\'' {
                return Err(Error::new(
                    "Character literal with no close",
                    format!("Found '{}' instead of closing '\''", next), start));
            }
            return Ok(Token::new_lit(TokenType::CharLit, charlit.to_string(), start, input.pos()));
        }
        // Operators
        '+' => { // +, +=, ++
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::Inc, '+') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            Ok(Token::new_op(lex_op(input, Operator::Add, Operator::AddAssign),
                             start, input.pos()))
        }
        '-' => { // -, -=, --, ->
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::Dec, '-') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            if let Some(op) = lex_op_other(input, Operator::Move, '>') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            Ok(Token::new_op(lex_op(input, Operator::Sub, Operator::SubAssign),
                             start, input.pos()))
        }
        '*' => { // *, *=
            input.consume();
            Ok(Token::new_op(lex_op(input, Operator::Mul, Operator::MulAssign),
                             start, input.pos()))
        }
        '/' => { // /, /=
            input.consume();
            // disreguard comments
            if let Some(next) = input.peek() {
                if next == '/' {
                    input.consume();
                    while let Some(c) = input.peek() {
                        if c == '\n' {
                            input.consume();
                            break;
                        } else {
                            input.consume();
                        }
                    }
                    return Ok(Token::new(TokenType::Whitespace, start, input.pos()));
                }
                if next == '*' {
                    input.consume();
                    while let Some(c) = input.peek() {
                        if c == '*' {
                            input.consume();
                            if let Some(next) = input.peek() {
                                if next == '/' {
                                    input.consume();
                                    break;
                                }
                            }
                        } else {
                            input.consume();
                        }
                    }
                    return Ok(Token::new(TokenType::Whitespace, start, input.pos()));
                }
            }
            Ok(Token::new_op(lex_op(input, Operator::Div, Operator::DivAssign),
                             start, input.pos()))
        }
        '%' => { // %, %=
            input.consume();
            Ok(Token::new_op(lex_op(input, Operator::Mod, Operator::ModAssign),
                             start, input.pos()))
        }
        '^' => { // ^, ^=
            input.consume();
            Ok(Token::new_op(lex_op(input, Operator::Xor, Operator::XorAssign),
                             start, input.pos()))
        }
        '&' => { // &, &&, &=
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::BoolAnd, '&') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            Ok(Token::new_op(lex_op(input, Operator::And, Operator::AndAssign),
                             start, input.pos()))
        }
        '|' => { // |, ||, |=
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::BoolOr, '|') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            Ok(Token::new_op(lex_op(input, Operator::Or, Operator::OrAssign),
                             start, input.pos()))
        }
        '=' => { // =, ==, =>
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::Right, '>') {
                return Ok(Token::new_op(op, start, input.pos()));
            }
            Ok(Token::new_op(lex_op(input, Operator::Assign, Operator::Eq),
                             start, input.pos()))
        }
        '<' => { // <, <=, <<, <<<, <<=
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::Shl, '<') { // <<
                let slu = lex_op_or(input, op, Operator::Shlu, '<'); // << or <<<
                if slu == op { // if its <<, then it could be <<=
                    return Ok(Token::new_op(lex_op_or(input, op, Operator::ShlAssign, '='),
                                            start, input.pos()));
                }
                // otherwise it is just <<<
                return Ok(Token::new_op(slu, start, input.pos()));
            }
            // < or <=
            Ok(Token::new_op(lex_op(input, Operator::Gt, Operator::Gte),
                             start, input.pos()))
        }
        '>' => { // >, >=, >>, >>>, >>=
            input.consume();
            if let Some(op) = lex_op_other(input, Operator::Shr, '>') { // >>
                let sru = lex_op_or(input, op, Operator::Shru, '>'); // >> or >>>
                if sru == op { // if its <<, then it could be >>=
                    return Ok(Token::new_op(lex_op_or(input, op, Operator::ShrAssign, '='),
                                            start, input.pos()));
                }
                // otherwise it is just >>>
                return Ok(Token::new_op(sru, start, input.pos()));
            }
            // > or >=
            Ok(Token::new_op(lex_op(input, Operator::Lt, Operator::Lte),
                             start, input.pos()))
        }
        '!' => { // !, !=
            input.consume();
            Ok(Token::new_op(lex_op(input, Operator::Not, Operator::Neq),
                             start, input.pos()))
        }
        // binary and hex literals
        '0' => {
            if let Some(next) = input.peek_at(1) {
                if next == 'x' {
                    // get the number
                    return lex_hex_lit(input);
                }
                if next == 'b' {
                    // get the number
                    return lex_bin_lit(input);
                }
            }
            return Ok(Token::new_lit(TokenType::NumberLit, next_numeric(input)?, start, input.pos()));
        }
        // identifiers and numbers
        _ => {
            if next.is_alphabetic() || next == '_' { // identifiers
                let ident = next_ident(input);
                return match ident.as_str() {
                    // check for keywords
                    "fn" => Ok(Token::new(TokenType::Fn, start, input.pos())),
                    "let" => Ok(Token::new(TokenType::Let, start, input.pos())),
                    "if" => Ok(Token::new(TokenType::If, start, input.pos())),
                    "else" => Ok(Token::new(TokenType::Else, start, input.pos())),
                    "while" => Ok(Token::new(TokenType::While, start, input.pos())),
                    "for" => Ok(Token::new(TokenType::For, start, input.pos())),
                    "loop" => Ok(Token::new(TokenType::Loop, start, input.pos())),
                    "return" => Ok(Token::new(TokenType::Return, start, input.pos())),
                    "match" => Ok(Token::new(TokenType::Match, start, input.pos())),
                    "struct" => Ok(Token::new(TokenType::Struct, start, input.pos())),
                    "assert" => Ok(Token::new(TokenType::Assert, start, input.pos())),
                    "in" => Ok(Token::new(TokenType::In, start, input.pos())),
                    "use" => Ok(Token::new(TokenType::Use, start, input.pos())),
                    "true" => Ok(Token::new(TokenType::BoolTrue, start, input.pos())),
                    "false" => Ok(Token::new(TokenType::BoolFalse, start, input.pos())),
                    _ => Ok(Token::new_ident(ident, start, input.pos()))
                }
            }
            return if next.is_numeric() { // handle numeric literals
                Ok(Token::new_lit(TokenType::NumberLit, next_numeric(input)?, start, input.pos()))
            } else {
                Err(Error::new(
                    "Unexpected character",
                    format!("Found '{}'", next), start))
            }
        }
    }
}

pub fn lex(input: &mut InputReader) -> Result<TokenList, Error> {
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(_) = input.peek() {
        // Process the next token
        tokens.push(next_token(input)?); // push the next token into the list
    }

    // add an EOF token for the parser
    tokens.push(Token::new(TokenType::EOF, input.pos(), input.pos()));

    // return the list of tokens in a TokenList wrapper for use in the parser
    Ok(TokenList::new(tokens))
}