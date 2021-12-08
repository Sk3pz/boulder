use crate::expression::Expression;
use crate::{Error, TokenList};
use crate::operator::Operator;
use crate::token::{Token, TokenType};

/// Removes whitespace tokens until the next non-whitespace token or EOF
fn optional_whitespace(tokens: &mut TokenList) {
    while let Some(token) = tokens.peek() { // Peek at the next token
        if token.token_type == TokenType::Whitespace { // If it's whitespace
            tokens.consume(); // Consume it
        } else {
            break; // Otherwise, break as the next token is not whitespace
        }
    }
}

/// Expects an operator token of a specific operator type and consumes it
fn expect_op(tokens: &mut TokenList, expected: Operator) -> Result<Operator, Error> {
    optional_whitespace(tokens); // Remove leading whitespace tokens
    // get the next token
    let next = tokens.peek();
    // make sure the token is not the eof
    if next.is_none() {
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    // check the next token is an operator
    let next_token = next.unwrap();
    if next_token.token_type != TokenType::Operator {
        // if not return an error
        return Err(Error::new(format!("Expected {}", expected), format!("But found {}", next_token), next_token.start));
    }
    // return the Operator type
    let op_type = next_token.op.unwrap();
    if op_type != expected {
        return Err(Error::new(format!("Expected {}", expected), format!("But found {}", next_token), next_token.start));
    }
    tokens.consume();
    Ok(op_type)
}

fn optional_op(tokens: &mut TokenList, expected: Operator) -> Result<Option<Operator>, Error> {
    optional_whitespace(tokens); // remove leading whitespace tokens
    // get the next token and make sure its not the eof
    let next = tokens.peek();
    if next.is_none() {
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    // check the next token is an operator
    let next_token = next.unwrap();
    if next_token.token_type == TokenType::Operator {
        tokens.consume();
        let op_type = next_token.op.unwrap();
        // if it is the expected operator return it
        if op_type == expected {
            return Ok(Some(op_type));
        }
    }
    // otherwise return None
    Ok(None)
}

fn optional_expect(tokens: &mut TokenList, expected: TokenType) -> Result<Option<Token>, Error> {
    optional_whitespace(tokens); // remove leading whitespace tokens
    let next = tokens.peek();
    if next.is_none() {
        // if the next token is the eof return an error
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    // if the next token is the expected token return it
    let next_token = next.unwrap();
    if next_token.token_type == expected {
        tokens.consume();
        return Ok(Some(next_token));
    }
    // otherwise return None
    Ok(None)
}

fn expect(tokens: &mut TokenList, expected: TokenType) -> Result<Token, Error> {
    optional_whitespace(tokens); // remove leading whitespace tokens
    let next = tokens.peek();
    // if the next token is the eof return an error
    if next.is_none() {
        return Err(Error::new("Unexpected End Of File", format!("Expected {}", expected), tokens.eof()));
    }
    // if the next token is the expected token return it, otherwise return an error
    let next_token = next.unwrap();
    if next_token.token_type != expected {
        return Err(Error::new(format!("Expected {}", expected), format!("But found {}", next_token), next_token.start));
    }
    Ok(tokens.consume().unwrap())
}

fn parse_block(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the '{' token
    let mut expressions = Vec::new();
    // while the next token is not the '}' token, parse the next expression
    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::CloseBrace {
            tokens.consume();
            break;
        }
        expressions.push(parse_statement(tokens)?);
    }
    Ok(Expression::Block(expressions))
}

fn define_params(tokens: &mut TokenList) -> Result<Vec<Expression>, Error> {
    // the vector to hold the parameters
    let mut params = Vec::new();
    expect(tokens, TokenType::OpenParen)?; // expect an open paren
    // while there is a next identifier, parse the next parameter
    while let Some(ident) = optional_expect(tokens, TokenType::Ident)? {
        // expect a type identifier
        expect(tokens, TokenType::Colon)?;
        // the type
        let param_type = expect(tokens, TokenType::Ident)?;
        // if there is a default value, push the full declaration as a parameter
        if optional_op(tokens, Operator::Eq)?.is_some() {
            let default_value = parse_statement(tokens)?;
            params.push(Expression::Declaration(Box::new(Expression::Identifier(ident.value.unwrap())),
                                                Some(Box::new(Expression::Identifier(param_type.value.unwrap()))),
                                                Some(Box::new(default_value))));
        } else {
            // otherwise push the ident and type as the param
            params.push(Expression::Declaration(Box::new(Expression::Identifier(ident.value.unwrap())),
                                                Some(Box::new(Expression::Identifier(param_type.value.unwrap()))),
                                                None));
        }
        // if the next token is not a comma, break out of the loop
        if optional_expect(tokens, TokenType::Comma)?.is_none() {
            break;
        }
    }
    // expect the closing paren
    expect(tokens, TokenType::CloseParen)?;
    // return the closing parameters
    Ok(params)
}

fn parse_fn(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove fn
    expect(tokens, TokenType::Whitespace)?; // separator between fn and name
    let name = expect(tokens, TokenType::Ident)?; // the identifier of the function
    let params = define_params(tokens)?;
    // optional return type
    let mut rt = Expression::Void;
    if optional_op(tokens, Operator::Move)?.is_some() {
        let return_type = expect(tokens, TokenType::Ident)?;
        rt = Expression::Identifier(return_type.value.unwrap());
    }
    // parse the body of the expression
    expect(tokens, TokenType::OpenBracket)?;
    let body = parse_block(tokens)?;
    Ok(Expression::Fn(Box::new(Expression::Identifier(name.value.unwrap())),
                            params,
                            Box::new(rt),
                            Box::new(body)))
}

fn parse_declaration(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the let token
    expect(tokens, TokenType::Whitespace)?; // separator between let and name
    let ident = expect(tokens, TokenType::Ident)?; // the identifier of the declaration
    // if there is a type, parse it
    let mut def_type: Option<Box<Expression>> = None;
    if optional_expect(tokens, TokenType::Colon)?.is_some() {
        def_type = Some(Box::new(
            Expression::Identifier(expect(tokens, TokenType::Ident)?.value.unwrap())));
    }
    // if there is a default value, parse it
    let mut def_value: Option<Box<Expression>> = None;
    if optional_op(tokens, Operator::Eq)?.is_some() {
        def_value = Some(Box::new(parse_statement(tokens)?));
    }
    // return the declaration
    Ok(Expression::Declaration(Box::new(Expression::Identifier(ident.value.unwrap())),
                               def_type,
                               def_value))
}

// statements inside of blocks
fn parse_statement(tokens: &mut TokenList) -> Result<Expression, Error> {
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    optional_whitespace(tokens); // remove any whitespace
    match tokens.peek().unwrap().token_type {
        TokenType::Let => parse_declaration(tokens), // if the next token is a let, parse the declaration
        _ => {
            Err(Error::new("Expected an expression",
                           format!("Found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
    }
}

/// expressions outside of blocks
fn parse_expression(tokens: &mut TokenList) -> Result<Expression, Error> {
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    optional_whitespace(tokens); // remove any whitespace
    match tokens.peek().unwrap().token_type { // check the next token
        TokenType::OpenBracket => parse_block(tokens), // if the next token is an open block, parse the block
        TokenType::Fn => parse_fn(tokens), // if the next token is a function, parse the function
        _ => { // other tokens
            Err(Error::new("Expected an expression",
                           format!("Found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
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