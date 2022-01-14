use std::path::Path;
use crate::expression::Expression;
use crate::{Error, InputReader, lex, read_file, TokenList, validate_boulder_file};
use crate::operator::Operator;
use crate::token::TokenType;

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
    Ok(Expression::Block { exprs: expressions })
}

fn parse_array_dec(tokens: &mut TokenList, mods: Vec<Expression>) -> Result<Expression, Error> {
    tokens.expect(TokenType::OpenBracket)?; // expect the '[' token
    let array_type = Box::new(get_type(tokens, false)?); // get the type of the array with its modifiers
    tokens.expect(TokenType::NOP)?; // consume the ';' token
    let size = Box::new(parse_expression(tokens)?); // parse the size of the array
    tokens.expect(TokenType::CloseBracket)?; // consume the ']' token
    return Ok(Expression::ArrayType {
        array_type,
        size,
        modifiers: mods,
    });
}

fn get_type(tokens: &mut TokenList, expect_colon: bool) -> Result<Expression, Error> {
    if expect_colon {
        tokens.expect(TokenType::Colon)?;
    }
    let mut modifiers: Vec<Expression> = Vec::new();
    while let Some(op) = tokens.optional_expect(TokenType::Operator)? {
        match op.op.unwrap() {
            Operator::And => modifiers.push(Expression::Reference),
            Operator::Mul => modifiers.push(Expression::Pointer),
            _ => {
                return Err(Error::new("Expected &, *, [...], or nothing",
                                      format!("found {}", op.op.unwrap()), op.start));
            }
        }
    }
    if let Some(_) = tokens.optional_expect(TokenType::OpenBracket)? {
        return parse_array_dec(tokens, modifiers);
    }
    // the type
    let type_ident = Box::new(Expression::Identifier{
        ident: tokens.expect(TokenType::Ident)?.value.unwrap()
    });
    Ok(Expression::Type {
        type_ident, modifiers
    })
}

fn define_params(tokens: &mut TokenList) -> Result<Vec<Expression>, Error> {
    // the vector to hold the parameters
    let mut params = Vec::new();
    tokens.expect(TokenType::OpenParen)?; // expect an open paren
    // while there is a next identifier, parse the next parameter
    while let Some(ident) = tokens.optional_expect(TokenType::Ident)? {
        // expect a type identifier
        let param_type = get_type(tokens, true)?;
        // if there is a default value, push the full declaration as a parameter
        if tokens.optional_op(Operator::Assign)?.is_some() {
            let default_value = parse_statement(tokens)?;
            params.push(Expression::Declaration {
                ident: Box::new(Expression::Identifier { ident: ident.value.unwrap() }),
                type_ident: Some(Box::new(param_type)),
                value: Some(Box::new(default_value))
            });
        } else {
            // otherwise push the ident and type as the param
            params.push(Expression::Declaration {
                ident: Box::new(Expression::Identifier { ident: ident.value.unwrap() }),
                type_ident: Some(Box::new(param_type)),
                value: None
            });
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
        rt = get_type(tokens, false)?;
    }
    // parse the body of the expression
    let body = parse_statement(tokens)?;
    Ok(Expression::Fn {
        ident: Box::new(Expression::Identifier { ident: name.value.unwrap() }),
        params,
        return_type: Box::new(rt),
        body: Box::new(body)
    })
}

fn parse_declaration(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the let token
    tokens.expect_whitespace()?; // separator between let and name
    let ident = tokens.expect(TokenType::Ident)?; // the identifier of the declaration
    // if there is a type, parse it
    let mut def_type: Option<Box<Expression>> = None;
    if tokens.optional_expect(TokenType::Colon)?.is_some() {
        def_type = Some(Box::new(get_type(tokens, false)?));
    } else {
        // TODO: inferred types!
        return Err(Error::new("Inferred types are not yet implemented!", "Variable types must be defined!",
                       tokens.peek().unwrap().start))?;
    }
    // if there is a default value, parse it
    let mut def_value: Option<Box<Expression>> = None;
    if tokens.optional_op(Operator::Assign)?.is_some() {
        def_value = Some(Box::new(parse_statement(tokens)?));
    }
    // return the declaration
    Ok(Expression::Declaration {
        ident: Box::new(Expression::Identifier { ident: ident.value.unwrap() }),
        type_ident: def_type,
        value: def_value
    })
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
    Ok(Expression::Use { exprs: parse_file(&mut file_tokens)? })
}

fn parse_if(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // consume the if token
    let condition = parse_statement(tokens)?;
    let body = parse_statement(tokens)?;
    let mut else_body = None;
    if tokens.next_is(TokenType::Else) {
        tokens.consume();
        else_body = Some(Box::new(parse_statement(tokens)?));
    }
    Ok(Expression::If {
        condition: Box::new(condition),
        body: Box::new(body),
        else_statement: else_body
    })
}

fn parse_while(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // consume the while token
    let condition = parse_statement(tokens)?;
    let body = parse_statement(tokens)?;
    Ok(Expression::While {
        condition: Box::new(condition),
        body: Box::new(body)
    })
}

fn parse_loop(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // consume the loop token
    let body = parse_statement(tokens)?;
    Ok(Expression::Loop {
        body: Box::new(body)
    })
}

fn parse_for(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the 'for'

    let ident = Box::new(parse_statement(tokens)?);
    tokens.expect(TokenType::In)?;
    let collection = Box::new(parse_statement(tokens)?);

    let body = Box::new(parse_statement(tokens)?);

    Ok(Expression::For { ident, collection, body })
}

fn parse_return(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // consume the return token

    // if the next token is a closing block, there is no return type
    // this is kind of a hack? could probably be done better...
    return if tokens.next_after_ws(TokenType::CloseBracket) {
        tokens.optional_whitespace();
        Ok(Expression::Return {
            value: Box::new(Expression::Void)
        })
    } else {
        tokens.expect_whitespace()?;
        Ok(Expression::Return {
            value: Box::new(parse_statement(tokens)?)
        })
    }
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
    return Ok(Expression::FnCall {
        ident: Box::new(ident),
        params
    });
}

fn parse_property(tokens: &mut TokenList, accessed: Expression) -> Result<Expression, Error> {
    tokens.consume(); // consume the '.'
    let property = parse_identifier(tokens)?; // the property to access
    // the property access expression
    let expr = Expression::PropertyAccess {
        expr: Box::new(accessed),
        property: Box::from(property)
    };
    if tokens.next_is(TokenType::Dot) {
        // recursively parse the property access expressions
        // this will handle things like a.b.c().d
        return parse_property(tokens, expr);
    }
    return Ok(expr);
}

/// array indexes
fn parse_index(tokens: &mut TokenList, accessed: Expression) -> Result<Expression, Error> {
    tokens.consume(); // remove the open '['
    // ensure it is not the end of file
    if tokens.peek().is_none() {
        return Err(Error::new("Expected closing ']'", "found end of file", tokens.eof()));
    }
    // get the indexer
    let index = parse_statement(tokens)?;
    // expect a closing ']'
    tokens.expect(TokenType::CloseBrace)?;

    Ok(Expression::ArrayAccess {
        ident: Box::new(accessed),
        index: Box::new(index),
    })
}

fn parse_assert(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // remove the "assert"
    let assertion = parse_statement(tokens)?;
    Ok(Expression::Assert { expr: Box::new(assertion) })
}

fn parse_bin_op(tokens: &mut TokenList, left: Expression, op: Operator) -> Result<Expression, Error> {
    let right = parse_statement(tokens)?;
    if op == Operator::Assign {
        return Ok(Expression::Assignment {
            ident: Box::new(left),
            value: Box::new(right)
        });
    }
    Ok(Expression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right)
    })
}

/// parse operators following an identifier or number
fn parse_op(tokens: &mut TokenList, left: Expression) -> Result<Expression, Error> {
    let op = tokens.consume().unwrap().op.unwrap(); // the operator token
    return if op != Operator::Inc && op != Operator::Dec {
        parse_bin_op(tokens, left, op)
    } else {
        Ok(Expression::Unary {
            op,
            expr: Box::new(left),
            leading: false
        })
    }
}

fn parse_unary_op(tokens: &mut TokenList, op: Operator) -> Result<Expression, Error> {
    let right = parse_statement(tokens)?;
    Ok(Expression::Unary {
        op,
        expr: Box::new(right),
        leading: true
    })
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
    let number = Expression::NumberLiteral {
        value: tokens.consume().unwrap().value.unwrap()
    };
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

fn parse_ident_statement(tokens: &mut TokenList, left: Expression) -> Result<Option<Expression>, Error> {
    tokens.optional_whitespace();
    if tokens.peek().is_none() {
        return Ok(None);
    }
    let next = tokens.peek().unwrap();
    match next.token_type {
        TokenType::OpenParen =>      Ok(Some(parse_fn_call(tokens, left)?)),
        TokenType::OpenBrace =>      Ok(Some(parse_index(tokens, left)?)),
        TokenType::Dot =>            Ok(Some(parse_property(tokens, left)?)),
        TokenType::Operator =>       Ok(Some(parse_op(tokens, left)?)),
        _ => Ok(None)
    }
}

fn parse_identifier(tokens: &mut TokenList) -> Result<Expression, Error> {
    let ident = Expression::Identifier { ident: tokens.consume().unwrap().value.unwrap() };
    tokens.optional_whitespace();
    let mut expr = ident;

    while let Some(stmnt) = parse_ident_statement(tokens, expr.clone())? {
        expr = stmnt;
    }

    Ok(expr)
}

fn parse_panic(tokens: &mut TokenList) -> Result<Expression, Error> {
    tokens.consume(); // consume the '?'
    return if tokens.next_after_ws(TokenType::CloseParen) {
        tokens.optional_whitespace();
        Ok(Expression::Panic { value: Box::new(Expression::Void) })
    } else {
        Ok(Expression::Panic { value: Box::new(parse_statement(tokens)?) })
    }
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
        TokenType::OpenBracket => parse_block(tokens),
        TokenType::Let => parse_declaration(tokens), // if the next token is a let, parse the declaration
        TokenType::If => parse_if(tokens),
        TokenType::While => parse_while(tokens),
        TokenType::Loop => parse_loop(tokens),
        TokenType::For => parse_for(tokens),
        TokenType::Assert => parse_assert(tokens),
        TokenType::Return => return parse_return(tokens),
        TokenType::NumberLit => parse_number_lit(tokens),
        TokenType::Ident => parse_identifier(tokens),
        TokenType::Operator => parse_leading_op(tokens),
        TokenType::Panic => parse_panic(tokens),
        TokenType::BoolTrue => ret(tokens, Expression::BoolLiteral { value: true }),
        TokenType::BoolFalse => ret(tokens, Expression::BoolLiteral { value: false }),
        TokenType::BinLit => Ok(Expression::BinaryLiteral { value: tokens.consume().unwrap().value.unwrap() } ),
        TokenType::HexLit => Ok(Expression::HexLiteral { value: tokens.consume().unwrap().value.unwrap() } ),
        TokenType::NOP => ret(tokens, Expression::NOP), // remove semicolons
        TokenType::StringLit => {
            let string = tokens.consume().unwrap().value.unwrap();
            Ok(Expression::StringLiteral { value: string })
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
        // blocks have been removed from global space
        //TokenType::OpenBracket => parse_block(tokens), // if the next token is an open block, parse the block
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
    Ok(Expression::Program { exprs: parse_file(tokens)? } )
}