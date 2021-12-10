use std::path::Path;
use crate::expression::Expression;
use crate::{Error, InputReader, lex, read_file, time_taken, TokenList, validate_boulder_file};
use crate::operator::Operator;
use crate::token::{Token, TokenType};

fn parse_block(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the '{' token
    let mut expressions = Vec::new();
    // while the next token is not the '}' token, parse the next expression
    while let Some(token) = tokens.peek() {
        // if its a whitespace token, continue on to the next token
        if token.token_type == TokenType::Whitespace {
            tokens.consume();
            continue;
        }
        if token.token_type == TokenType::CloseBracket { // if its a '}' token
            tokens.consume(); // remove it
            break; // break out of the loop
        }
        expressions.push(parse_statement(tokens)?);
    }
    Ok(Expression::Block(expressions))
}

fn define_params(tokens: &mut TokenList) -> Result<Vec<Expression>, Error> {
    // the vector to hold the parameters
    let mut params = Vec::new();
    tokens.expect(TokenType::OpenParen)?; // expect an open paren
    // while there is a next identifier, parse the next parameter
    while let Some(ident) = tokens.optional_expect(TokenType::Ident)? {
        // expect a type identifier
        tokens.expect(TokenType::Colon)?;
        // the type
        let param_type = tokens.expect(TokenType::Ident)?;
        // if there is a default value, push the full declaration as a parameter
        if tokens.optional_op(Operator::Assign)?.is_some() {
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
        if tokens.optional_expect(TokenType::Comma)?.is_none() {
            break;
        }
    }
    // expect the closing paren
    tokens.expect(TokenType::CloseParen)?;
    // return the closing parameters
    Ok(params)
}

fn parse_fn(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove fn
    tokens.expect_whitespace()?; // separator between fn and name
    let name = tokens.expect(TokenType::Ident)?; // the identifier of the function
    let params = define_params(tokens)?;
    // optional return type
    let mut rt = Expression::Void;
    if tokens.optional_op(Operator::Move)?.is_some() {
        let return_type = tokens.expect(TokenType::Ident)?;
        rt = Expression::Identifier(return_type.value.unwrap());
    }
    // parse the body of the expression
    tokens.expect(TokenType::OpenBracket)?;
    let body = parse_block(tokens)?;
    Ok(Expression::Fn(Box::new(Expression::Identifier(name.value.unwrap())),
                            params,
                            Box::new(rt),
                            Box::new(body)))
}

fn parse_declaration(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the let token
    tokens.expect_whitespace()?; // separator between let and name
    let ident = tokens.expect(TokenType::Ident)?; // the identifier of the declaration
    // if there is a type, parse it
    let mut def_type: Option<Box<Expression>> = None;
    if tokens.optional_expect(TokenType::Colon)?.is_some() {
        def_type = Some(Box::new(
            Expression::Identifier(tokens.expect(TokenType::Ident)?.value.unwrap())));
    }
    // if there is a default value, parse it
    let mut def_value: Option<Box<Expression>> = None;
    if tokens.optional_op(Operator::Eq)?.is_some() {
        def_value = Some(Box::new(parse_statement(tokens)?));
    }
    // return the declaration
    Ok(Expression::Declaration(Box::new(Expression::Identifier(ident.value.unwrap())),
                               def_type,
                               def_value))
}

fn parse_use(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the use token
    tokens.expect_whitespace()?; // separator between use and file
    let file = tokens.expect(TokenType::StringLit)?; // the file to import
    // ensure the file is valid and is a .rock file
    let mut file_path = file.value.as_ref().unwrap().clone();
    if file.start.file.is_some() {
        let dir = file.start.file.as_ref().unwrap().clone();
        let path = Path::new(&dir);
        let parent = path.parent().unwrap();
        file_path = format!("{}/{}", parent.to_str().unwrap(), file_path);
    }
    let valid = validate_boulder_file(file_path.clone());
    if valid.is_err() {
        return Err(Error::new("Invalid boulder file import", valid.unwrap_err(), file.start));
    }
    // lex the file
    let mut ir = InputReader::new(Some(file_path.clone()), read_file(file_path));
    let mut file_tokens = lex(&mut ir)?;
    // return the file's AST in an expression
    Ok(Expression::Use(parse_file(&mut file_tokens)?))
}

fn parse_fn_call(tokens: &mut TokenList, ident: Expression) -> Result<Expression, Error> {
    tokens.consume(); // remove the open paren
    let mut params = Vec::new();
    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::CloseParen {
            tokens.consume(); // remove the close paren
            break;
        }
        params.push(parse_statement(tokens)?);
        // if the next token is not a comma, break out of the loop
        if tokens.optional_expect(TokenType::Comma)?.is_none() {
            tokens.expect(TokenType::CloseParen)?;
            break;
        }
    }
    return Ok(Expression::FnCall(Box::new(ident), params));
}

fn parse_property(tokens: &mut TokenList, accessed: Expression) -> Result<Expression, Error> {
    tokens.consume(); // consume the '.'
    let property = parse_identifier(tokens)?; // the property to access
    // the property access expression
    let expr = Expression::PropertyAccess(Box::new(accessed), Box::from(property));
    if tokens.next_is(TokenType::Dot) {
        // recursively parse the property access expressions
        // this will handle things like a.b.c().d
        return parse_property(tokens, expr);
    }
    return Ok(expr);
}

/// array indexes
fn parse_index(tokens: &mut TokenList, accessed: Expression) -> Result<Expression, Error> {
    todo!()
}

fn parse_bin_op(tokens: &mut TokenList, left: Expression, op: Operator) -> Result<Expression, Error> {
    let right = parse_statement(tokens)?;
    Ok(Expression::Binary(Box::new(left), op, Box::new(right)))
}

/// parse operators following an identifier or number
fn parse_op(tokens: &mut TokenList, ident: Expression) -> Result<Expression, Error> {
    let op = tokens.consume().unwrap().op.unwrap(); // the operator token
    if op != Operator::Inc && op != Operator::Dec {
        return parse_bin_op(tokens, ident, op);
    }
    todo!()
}

fn parse_unary_op(tokens: &mut TokenList, op: Operator) -> Result<Expression, Error> {
    let right = parse_statement(tokens)?;
    Ok(Expression::Unary(op, Box::new(right)))
}

fn parse_leading_op(tokens: &mut TokenList) -> Result<Expression, Error> {
    let op = tokens.consume().unwrap(); // the operator token
    match op.op.unwrap() {
        Operator::Inc | Operator::Dec |
        Operator::Not | Operator::Sub => parse_unary_op(tokens, op.op.unwrap()),
        _ => {
            Err(Error::new("Unexpected operator",
                           format!("{} requires a left and right statement, but no left was found", op.op.unwrap()),
                           op.start))
        }
    }
}

fn parse_number_lit(tokens: &mut TokenList) -> Result<Expression, Error> {
    let number = Expression::NumberLiteral(tokens.consume().unwrap().value.unwrap());
    tokens.optional_whitespace();
    let next = tokens.peek();
    if next.is_some() {
        let next_token = next.unwrap();
        match next_token.token_type {
            TokenType::Dot => return parse_property(tokens, number),
            TokenType::Operator => return parse_op(tokens, number),
            _ => {}
        }
    }

    Ok(number)
}

fn parse_identifier(tokens: &mut TokenList) -> Result<Expression, Error> {
    let ident = Expression::Identifier(tokens.consume().unwrap().value.unwrap());
    tokens.optional_whitespace();
    let next = tokens.peek();
    if next.is_some() {
        let next_token = next.unwrap();
        match next_token.token_type {
            TokenType::OpenParen => return parse_fn_call(tokens, ident),
            TokenType::OpenBracket => return parse_index(tokens, ident),
            TokenType::Dot => return parse_property(tokens, ident),
            TokenType::Operator => return parse_op(tokens, ident),
            _ => {}
        };
    }

    Ok(ident)
}

fn ret(tokens: &mut TokenList, expr: Expression) -> Result<Expression, Error> {
    tokens.consume();
    Ok(expr)
}

// statements inside of blocks
fn parse_statement(tokens: &mut TokenList) -> Result<Expression, Error> {
    // todo(eric): add support for string {} things
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    tokens.optional_whitespace(); // remove any whitespace
    match tokens.peek().unwrap().token_type {
        TokenType::Let => parse_declaration(tokens), // if the next token is a let, parse the declaration
        TokenType::NumberLit => parse_number_lit(tokens),
        TokenType::Ident => parse_identifier(tokens),
        TokenType::Operator => parse_leading_op(tokens),
        // todo(eric): if, while, for, etc
        //  this will require a lot of work to make sure that the statements are parsed correctly :(
        //  for example, conditionals and making sure they result in either true or false
        TokenType::BoolTrue => ret(tokens, Expression::BoolLiteral(true)),
        TokenType::BoolFalse => ret(tokens, Expression::BoolLiteral(false)),
        TokenType::BinLit => Ok(Expression::BinaryLiteral(tokens.consume().unwrap().value.unwrap())),
        TokenType::HexLit => Ok(Expression::HexLiteral(tokens.consume().unwrap().value.unwrap())),
        TokenType::NOP => ret(tokens, Expression::NOP), // remove semicolons
        TokenType::StringLit => {
            let string = tokens.consume().unwrap().value.unwrap();
            Ok(Expression::StringLiteral(string))
        },
        _ => {
            Err(Error::new("Expected an expression",
                           format!("found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
    }
}

/// expressions outside of blocks
fn parse_expression(tokens: &mut TokenList) -> Result<Expression, Error> {
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    tokens.optional_whitespace(); // remove any whitespace
    match tokens.peek().unwrap().token_type { // check the next token
        TokenType::OpenBracket => parse_block(tokens), // if the next token is an open block, parse the block
        TokenType::Fn => parse_fn(tokens), // if the next token is a function, parse the function
        TokenType::NOP => ret(tokens, Expression::NOP), // if the next token is a no-op, return a no-op (basically removes semicolons)
        TokenType::Use => parse_use(tokens),
        _ => { // other tokens
            Err(Error::new("Expected an expression",
                           format!("found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
    }
}

fn parse_file(tokens: &mut TokenList) -> Result<Vec<Expression>, Error> {
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
    Ok(expressions)
}

pub fn parse(tokens: &mut TokenList) -> Result<Expression, Error> {
    Ok(Expression::Program(parse_file(tokens)?))
}