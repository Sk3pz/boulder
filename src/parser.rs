use std::path::Path;
use crate::statement::{Number, ShuntedStack, ShuntedStackItem, Statement};
use crate::{Error, InputReader, lex, read_file, TokenList, validate_boulder_file};
use crate::operator::Operator;
use crate::token::{Token, TokenType};

fn parse_block(tokens: &mut TokenList) -> Result<Statement, Error> {
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
    Ok(Statement::Block { exprs: expressions })
}

fn parse_array_dec(tokens: &mut TokenList, mods: Vec<Statement>) -> Result<Statement, Error> {
    tokens.expect(TokenType::OpenBracket)?; // expect the '[' token
    let array_type = Box::new(get_type(tokens, false)?); // get the type of the array with its modifiers
    tokens.expect(TokenType::NOP)?; // consume the ';' token
    let size = Box::new(parse_global(tokens)?); // parse the size of the array
    tokens.expect(TokenType::CloseBracket)?; // consume the ']' token
    return Ok(Statement::ArrayType {
        array_type,
        size,
        modifiers: mods,
    });
}

fn get_type(tokens: &mut TokenList, expect_colon: bool) -> Result<Statement, Error> {
    if expect_colon {
        tokens.expect(TokenType::Colon)?;
    }
    let mut modifiers: Vec<Statement> = Vec::new();
    while let Some(op) = tokens.optional_expect(TokenType::Operator)? {
        match op.op.unwrap() {
            Operator::And => modifiers.push(Statement::Reference),
            Operator::Mul => modifiers.push(Statement::Pointer),
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
    let type_ident = Box::new(Statement::Identifier{
        ident: tokens.expect(TokenType::Ident)?.value.unwrap()
    });
    Ok(Statement::Type {
        type_ident, modifiers
    })
}

fn define_params(tokens: &mut TokenList) -> Result<Vec<Statement>, Error> {
    // the vector to hold the parameters
    let mut params = Vec::new();
    tokens.expect(TokenType::OpenParen)?; // expect an open paren
    // while there is a next identifier, parse the next parameter
    while let Some(ident) = tokens.optional_expect(TokenType::Ident)? {
        // expect a type identifier
        let param_type = get_type(tokens, true)?;
        // if there is a default value, push the full declaration as a parameter
        // todo(eric): when this is implemented, there is no need to check if tokens is empty and store next_loc
        //  so it can be removed
        tokens.optional_whitespace();
        if tokens.is_empty() {
            return Err(Error::new("Expected parameter or closing ')'", "Found end of file", tokens.eof()));
        }
        let next_loc = tokens.peek_loc().unwrap();
        if tokens.optional_op(Operator::Assign)?.is_some() {
            // todo(eric): handle default values (removed because the compiler doesn't support them, this code works)
            return Err(Error::new("Invalid Assignment", "Default parameter values are not yet supported!", next_loc));
            // let default_value = parse_statement(tokens)?;
            // params.push(Statement::Declaration {
            //     ident: Box::new(Statement::Identifier { ident: ident.value.unwrap() }),
            //     type_ident: Some(Box::new(param_type)),
            //     value: Some(Box::new(default_value))
            // });
        } else {
            // otherwise push the ident and type as the param
            params.push(Statement::Declaration {
                ident: Box::new(Statement::Identifier { ident: ident.value.unwrap() }),
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

fn parse_fn(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // remove fn
    tokens.expect_whitespace()?; // separator between fn and name
    let name = tokens.expect(TokenType::Ident)?; // the identifier of the function
    let params = define_params(tokens)?;
    // optional return type
    let mut rt = Statement::Void;
    if tokens.optional_op(Operator::Move)?.is_some() {
        rt = get_type(tokens, false)?;
    }
    // parse the body of the expression
    let body = parse_statement(tokens)?;
    Ok(Statement::Fn {
        ident: Box::new(Statement::Identifier { ident: name.value.unwrap() }),
        params,
        return_type: Box::new(rt),
        body: Box::new(body)
    })
}

fn parse_declaration(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // remove the let token
    tokens.expect_whitespace()?; // separator between let and name
    let ident = tokens.expect(TokenType::Ident)?; // the identifier of the declaration
    // if there is a type, parse it
    let mut def_type: Option<Box<Statement>> = None;
    if tokens.optional_expect(TokenType::Colon)?.is_some() {
        def_type = Some(Box::new(get_type(tokens, false)?));
    } else {
        // TODO: inferred types!
        return Err(Error::new("Inferred types are not yet implemented!", "Variable types must be defined!",
                       tokens.peek().unwrap().start))?;
    }
    // if there is a default value, parse it
    let mut def_value: Option<Box<Statement>> = None;
    if tokens.optional_op(Operator::Assign)?.is_some() {
        def_value = Some(Box::new(parse_statement(tokens)?));
    }
    // return the declaration
    Ok(Statement::Declaration {
        ident: Box::new(Statement::Identifier { ident: ident.value.unwrap() }),
        type_ident: def_type,
        value: def_value
    })
}

fn parse_use(tokens: &mut TokenList) -> Result<Statement, Error> {
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
    Ok(Statement::Use { exprs: parse_file(&mut file_tokens)? })
}

fn parse_if(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // consume the if token
    let condition = parse_statement(tokens)?;
    let body = parse_statement(tokens)?;
    let mut else_body = None;
    if tokens.next_is(TokenType::Else) {
        tokens.consume();
        else_body = Some(Box::new(parse_statement(tokens)?));
    }
    Ok(Statement::If {
        condition: Box::new(condition),
        body: Box::new(body),
        else_statement: else_body
    })
}

fn parse_while(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // consume the while token
    let condition = parse_statement(tokens)?;
    let body = parse_statement(tokens)?;
    Ok(Statement::While {
        condition: Box::new(condition),
        body: Box::new(body)
    })
}

fn parse_loop(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // consume the loop token
    let body = parse_statement(tokens)?;
    Ok(Statement::Loop {
        body: Box::new(body)
    })
}

fn parse_for(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // remove the 'for'

    let ident = Box::new(parse_statement(tokens)?);
    tokens.expect(TokenType::In)?;
    let collection = Box::new(parse_statement(tokens)?);

    let body = Box::new(parse_statement(tokens)?);

    Ok(Statement::For { ident, collection, body })
}

fn parse_return(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // consume the return token

    // if the next token is a closing block, there is no return type
    // this is kind of a hack? could probably be done better...
    return if tokens.next_after_ws(TokenType::CloseBracket) {
        tokens.optional_whitespace();
        Ok(Statement::Return {
            value: Box::new(Statement::Void)
        })
    } else {
        tokens.expect_whitespace()?;
        Ok(Statement::Return {
            value: Box::new(parse_statement(tokens)?)
        })
    }
}

fn parse_fn_call(tokens: &mut TokenList, ident: Statement) -> Result<Statement, Error> {
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
    return Ok(Statement::FnCall {
        ident: Box::new(ident),
        params
    });
}

fn parse_property(tokens: &mut TokenList, accessed: Statement) -> Result<Statement, Error> {
    tokens.consume(); // consume the '.'
    let property = parse_identifier(tokens, true)?; // the property to access
    // the property access expression
    let expr = Statement::PropertyAccess {
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
fn parse_index(tokens: &mut TokenList, accessed: Statement) -> Result<Statement, Error> {
    tokens.consume(); // remove the open '['
    // ensure it is not the end of file
    if tokens.peek().is_none() {
        return Err(Error::new("Expected closing ']'", "found end of file", tokens.eof()));
    }
    // get the indexer
    let index = parse_statement(tokens)?;
    // expect a closing ']'
    tokens.expect(TokenType::CloseBrace)?;

    Ok(Statement::ArrayAccess {
        ident: Box::new(accessed),
        index: Box::new(index),
    })
}

fn parse_assert(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // remove the "assert"
    let assertion = parse_statement(tokens)?;
    Ok(Statement::Assert { expr: Box::new(assertion) })
}

/// returns true if negative
fn parse_shunting_yard_leading_op(op: Token, postfix: &mut ShuntedStack) -> Result<bool, Error> {
    match op.op.unwrap() {
        Operator::Sub => {
            return Ok(true);
        }
        Operator::Inc => {
            // immediately push the add because ++x = (1 + x)
            postfix.push(ShuntedStackItem::new_operator(Operator::Add));
            postfix.push(ShuntedStackItem::new_operand(
                Statement::NumberLiteral {
                    value: Number::new("1".to_string(), false)
                }
            ));
        }
        Operator::Dec => {
            // immediately push the subtract because --x = (1 - x)
            postfix.push(ShuntedStackItem::new_operator(Operator::Sub));
            postfix.push(ShuntedStackItem::new_operand(
                Statement::NumberLiteral {
                    value: Number::new("1".to_string(), false)
                }
            ));
        }
        _ => {
            return Err(Error::new(
                "Unexpected operator",
                "found binary operator after another binary operator",
                op.start
            ));
        }
    }
   Ok(false)
}

fn shunting_yard(tokens: &mut TokenList, unary_start: bool, leading: Option<Statement>) -> Result<Statement, Error> {
    // create a postfix stack using the shunting yard algorithm
    // references:
    // PEGwiki: https://wcipeg.com/wiki/Shunting_yard_algorithm
    // wikipedia: https://en.wikipedia.org/wiki/Shunting-yard_algorithm
    let mut postfix = ShuntedStack::new();
    let mut op_stack: Vec<Token> = Vec::new();

    if leading.is_some() {
        // if there is a leading operator, push it onto the stack
        postfix.push(ShuntedStackItem::new_operand(leading.unwrap()));
    }

    // last_op is used to handle unary operations mixed in with binary operations
    let mut last_op: Option<Operator> = None;
    let mut negative = false;

    if unary_start {
        let op = tokens.expect(TokenType::Operator)?;
        negative = parse_shunting_yard_leading_op(op, &mut postfix)?;
    }

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::NumberLit => {
                tokens.consume();
                // numbers get pushed to the stack
                postfix.push(ShuntedStackItem::new_operand(
                    Statement::NumberLiteral {
                        value: Number::new(token.value.unwrap(), negative)
                    }
                ));
                last_op = None;
                negative = false;
            }
            TokenType::Ident => {
                let stmt = parse_identifier(tokens, false)?;
                postfix.push(ShuntedStackItem::new_operand(stmt));
                last_op = None;
                negative = false;
            }
            TokenType::Operator => {
                tokens.consume();
                let operator = token.op.unwrap();

                // handle leading ops (unary)
                if last_op.is_some() {
                    negative = parse_shunting_yard_leading_op(token, &mut postfix)?;
                    last_op = Some(operator.clone());
                    continue;
                }

                // if the operator is a unary operator, push it to the stack
                if operator == Operator::Inc || operator == Operator::Dec {
                    // immediately push it to the stack as it directly applies to the number it follows
                    postfix.push(ShuntedStackItem::new_operator(operator));
                    continue;
                }

                // handle normal operators
                while let Some(o2_token) = op_stack.last() {
                    if o2_token.token_type == TokenType::OpenParen {
                        // this means a '(' so the program should not continue,
                        break;
                    }
                    if o2_token.token_type != TokenType::Operator {
                        return Err(Error::new_singular("unexpected error found in op_stack during shunting yard algorithm",
                        o2_token.clone().start));
                    }
                    let o2 = o2_token.op.unwrap();

                    if o2.precedence() <= operator.precedence() {
                        // or that o2 has a lower precedence than operator
                        break;
                    }
                    // pop the operator off the stack and push it to the postfix stack
                    let op = op_stack.pop().unwrap();
                    postfix.push(ShuntedStackItem::new_operator(op.op.unwrap()));
                }
                op_stack.push(token);

                last_op = Some(operator.clone());
            }
            TokenType::OpenParen => {
                tokens.consume();
                op_stack.push(token);
                // set to an operator not used by this algorithm, because '(' is not an operator
                last_op = Some(Operator::AddAssign);
            }
            TokenType::CloseParen => {
                let mut found = false;
                while let Some(op) = op_stack.pop() {
                    if op.token_type == TokenType::OpenParen {
                        // this means a '(' so the program should not continue
                        found = true;
                        break;
                    }
                    if op.token_type != TokenType::Operator {
                        return Err(Error::new_singular("unexpected error found in op_stack during shunting yard algorithm",
                                                       tokens.peek().unwrap().start));
                    }
                    postfix.push(ShuntedStackItem::new_operator(op.op.unwrap()));
                }

                if !found {
                    // probably not a part of the algorithm, most likely a closing function call
                    // or something similar. If not, the compiler will probably catch it.
                    break;
                    // return Err(Error::new(
                    //     "Mismatched Parentheses",
                    //     "Close ')' found with no open '('",
                    //     token.start
                    // ));
                }
                tokens.consume();

                last_op = None;
                negative = false;
            }
            TokenType::Whitespace => {
                tokens.consume();
                continue;
            }
            _ => {
                // another operator found, break and continue to the next expression
                break;
            }
        }
    }

    // loop through remaining operators and push them to the postfix stack
    while let Some(op) = op_stack.pop() {
        if op.token_type == TokenType::OpenParen {
            // this means there's a '(' with no closing brace
            return Err(Error::new(
                "Open found with no close",
                "mismatched parentheses",
                op.start
            ));
        }
        if op.token_type != TokenType::Operator {
            return Err(Error::new_singular("unexpected error found in op_stack during shunting yard algorithm",
            tokens.peek().unwrap().start));
        }
        postfix.push(ShuntedStackItem::new_operator(op.op.unwrap()));
    }

    Ok(Statement::Postfix {
        postfix,
    })
}

fn parse_bin_op(tokens: &mut TokenList, left: Statement, op: Operator) -> Result<Statement, Error> {
    let right = parse_statement(tokens)?;
    if op == Operator::Assign {
        return Ok(Statement::Assignment {
            ident: Box::new(left),
            value: Box::new(right)
        });
    }
    Ok(Statement::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right)
    })
}

/// parse operators following an identifier or number
fn parse_op(tokens: &mut TokenList, left: Statement) -> Result<Statement, Error> {
    let op = tokens.consume().unwrap().op.unwrap(); // the operator token
    return if op != Operator::Inc && op != Operator::Dec {
        parse_bin_op(tokens, left, op)
    } else {
        Ok(Statement::Unary {
            op,
            expr: Box::new(left),
            leading: false
        })
    }
}

fn parse_unary_op(tokens: &mut TokenList, op: Operator) -> Result<Statement, Error> {
    let right = parse_statement(tokens)?;
    Ok(Statement::Unary {
        op,
        expr: Box::new(right),
        leading: true
    })
}

fn parse_leading_op(tokens: &mut TokenList) -> Result<Statement, Error> {
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

fn parse_number_lit(tokens: &mut TokenList) -> Result<Statement, Error> {
    let mut num = tokens.consume().unwrap().value.unwrap();
    let mut number = Statement::NumberLiteral {
        value: Number::new(num, false)
    };
    tokens.optional_whitespace();
    let next = tokens.peek();
    if let Some(next) = tokens.peek() {
        if next.token_type == TokenType::Dot {
            number = parse_property(tokens, number)?
        }
    }
    if next.is_some() {
        let next_token = next.unwrap();
        match next_token.token_type {
            TokenType::Dot => return parse_property(tokens, number),
            TokenType::Operator => return shunting_yard(tokens, false, Some(number)),
            _ => {}
        }
    }

    shunting_yard(tokens, false, Some(number))
}

fn parse_ident_statement(tokens: &mut TokenList, left: Statement, shunt: bool) -> Result<Option<Statement>, Error> {
    tokens.optional_whitespace();
    if tokens.peek().is_none() {
        return Ok(None);
    }
    let next = tokens.peek().unwrap();
    if shunt {
        match next.token_type {
            TokenType::OpenParen =>      Ok(Some(parse_fn_call(tokens, left)?)),
            TokenType::OpenBrace =>      Ok(Some(parse_index(tokens, left)?)),
            TokenType::Dot =>            Ok(Some(parse_property(tokens, left)?)),
            TokenType::Operator =>       Ok(Some(shunting_yard(tokens, false, Some(left))?)),
            _ => Ok(None)
        }
    } else {
        match next.token_type {
            TokenType::OpenParen =>      Ok(Some(parse_fn_call(tokens, left)?)),
            TokenType::OpenBrace =>      Ok(Some(parse_index(tokens, left)?)),
            TokenType::Dot =>            Ok(Some(parse_property(tokens, left)?)),
            _ => Ok(None)
        }
    }
}

fn parse_identifier(tokens: &mut TokenList, shunt: bool) -> Result<Statement, Error> {
    let ident = Statement::Identifier { ident: tokens.consume().unwrap().value.unwrap() };
    tokens.optional_whitespace();
    let mut expr = ident;

    while let Some(stmnt) = parse_ident_statement(tokens, expr.clone(), shunt)? {
        expr = stmnt;
    }

    Ok(expr)
}

fn parse_panic(tokens: &mut TokenList) -> Result<Statement, Error> {
    tokens.consume(); // consume the '?'
    return if tokens.next_after_ws(TokenType::CloseParen) {
        tokens.optional_whitespace();
        Ok(Statement::Panic { value: Box::new(Statement::Void) })
    } else {
        Ok(Statement::Panic { value: Box::new(parse_statement(tokens)?) })
    }
}

fn ret(tokens: &mut TokenList, expr: Statement) -> Result<Statement, Error> {
    tokens.consume();
    Ok(expr)
}

// statements inside of blocks
fn parse_statement(tokens: &mut TokenList) -> Result<Statement, Error> {
    // todo(eric): add support for string {} things
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    tokens.optional_whitespace(); // remove any whitespace
    match tokens.peek().unwrap().token_type {
        TokenType::OpenBracket => parse_block(tokens),
        TokenType::OpenParen => shunting_yard(tokens, false, None),
        TokenType::Let => parse_declaration(tokens), // if the next token is a let, parse the declaration
        TokenType::If => parse_if(tokens),
        TokenType::While => parse_while(tokens),
        TokenType::Loop => parse_loop(tokens),
        TokenType::For => parse_for(tokens),
        TokenType::Assert => parse_assert(tokens),
        TokenType::Return => return parse_return(tokens),
        TokenType::NumberLit => parse_number_lit(tokens),
        TokenType::Ident => parse_identifier(tokens, true),
        TokenType::Operator => shunting_yard(tokens, true, None),
        //TokenType::Operator => parse_leading_op(tokens),
        TokenType::Panic => parse_panic(tokens),
        TokenType::BoolTrue => ret(tokens, Statement::BoolLiteral { value: true }),
        TokenType::BoolFalse => ret(tokens, Statement::BoolLiteral { value: false }),
        TokenType::BinLit => Ok(Statement::BinaryLiteral { value: tokens.consume().unwrap().value.unwrap() } ),
        TokenType::HexLit => Ok(Statement::HexLiteral { value: tokens.consume().unwrap().value.unwrap() } ),
        TokenType::NOP => ret(tokens, Statement::NOP), // remove semicolons
        TokenType::StringLit => {
            let string = tokens.consume().unwrap().value.unwrap();
            Ok(Statement::StringLiteral { value: string })
        },
        _ => {
            Err(Error::new("Expected an expression",
                           format!("found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
    }
}

/// expressions outside of blocks
fn parse_global(tokens: &mut TokenList) -> Result<Statement, Error> {
    if tokens.peek().is_none() { // if there are no more tokens
        return Err(Error::new_singular("Reached end of file without finding an expression!", tokens.eof())); // return an error
    }
    tokens.optional_whitespace(); // remove any whitespace
    match tokens.peek().unwrap().token_type { // check the next token
        // blocks have been removed from global space
        //TokenType::OpenBracket => parse_block(tokens), // if the next token is an open block, parse the block
        TokenType::Fn => parse_fn(tokens), // if the next token is a function, parse the function
        TokenType::NOP => ret(tokens, Statement::NOP), // if the next token is a no-op, return a no-op (basically removes semicolons)
        TokenType::Use => parse_use(tokens),
        _ => { // other tokens
            Err(Error::new("Expected an expression",
                           format!("found: {}", tokens.peek().unwrap().token_type), tokens.peek().unwrap().start))
        }
    }
}

fn parse_file(tokens: &mut TokenList) -> Result<Vec<Statement>, Error> {
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
                expressions.push(parse_global(tokens)?); // parse the next few tokens further into the AST
            }
        }
    }

    // return the abstract syntax tree
    Ok(expressions)
}

pub fn parse(tokens: &mut TokenList) -> Result<Statement, Error> {
    Ok(Statement::Program { exprs: parse_file(tokens)? } )
}