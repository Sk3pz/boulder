use crate::expression::Expression;
use crate::{Error, TokenList};
use crate::token::{Token, TokenType};

/// Removes whitespace tokens until the next non-whitespace token or EOF
fn optional_whitespace(tokens: &mut TokenList) {
    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::Whitespace {
            tokens.consume();
        } else {
            break;
        }
    }
}

fn optional_expect(tokens: &mut TokenList, expected: TokenType) -> Result<Option<Token>, Error> {
    let next = tokens.peek();
    if next.is_none() {
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    let next_token = next.unwrap();
    if next_token.token_type == expected {
        tokens.consume();
        return Ok(Some(next_token));
    }
    Ok(None)
}

fn expect(tokens: &mut TokenList, expected: TokenType) -> Result<Token, Error> {
    let next = tokens.peek();
    if next.is_none() {
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    let next_token = next.unwrap();
    if next_token.token_type != expected {
        return Err(Error::new(format!("Expected {}", expected), format!("But found {}", next_token), next_token.start));
    }
    Ok(tokens.consume().unwrap())
}

fn parse_block(tokens: &mut TokenList) -> Result<Expression, Error> {
    todo!()
}

fn parse_fn(tokens: &mut TokenList) -> Result<Expression, Error> {
    todo!()
}



fn parse_expression(tokens: &mut TokenList) -> Result<Expression, Error> {
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    optional_whitespace(tokens); // remove any whitespace
    match tokens.peek().unwrap().token_type { // check the next token
        TokenType::OpenBlock => parse_block(tokens), // if the next token is an open block, parse the block
        TokenType::Fn => parse_fn(tokens), // if the next token is a function, parse the function
        _ => todo!() // otherwise, parse a statement
    }
}

pub fn parse(tokens: &mut TokenList) -> Result<Expression, Error> {
    let mut expressions = Vec::new();

    // parse the tokens and build the expression tree
    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Whitespace => { // remove whitespace outside of expressions
                tokens.consume();
                continue;
            },
            TokenType::EOF => { // if its an EOF token, break out of the loop
                tokens.consume(); // consume the EOF token
                break; // break out of the loop as there are no more tokens to parse
            },
            _ => { // parse other tokens into expressions
                expressions.push(parse_expression(tokens)?); // parse the next few tokens further into the AST
            }
        }
    }

    // return the abstract syntax tree
    Ok(Expression::Program(expressions))
}