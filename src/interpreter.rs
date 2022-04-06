use std::collections::HashMap;
use std::fmt::Display;
use std::intrinsics::unreachable;
use better_term::{Color, Style};

use crate::statement::Statement;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub msg: String,
}

impl RuntimeError {
    pub fn new<S: Into<String>>(msg: S) -> RuntimeError {
        RuntimeError { msg: msg.into() }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}error{}: {}{}",
                   Style::default().fg(Color::BrightRed).bold(), Color::BrightWhite, self.msg, Style::reset())
    }
}

pub fn interpret(ast: &Statement) -> Result<(), RuntimeError> {

    let mut functions: HashMap<String, Statement> = HashMap::new();
    let mut variables: HashMap<String, Statement> = HashMap::new();

    let mut start_function: Option<Statement> = None;

    if let Statement::Program { exprs } = ast {
        for expr in exprs {
            match expr {
                Statement::Fn { ident, return_type, params, body } => {

                    let name = match *ident.clone() {
                        Statement::Identifier { ident } => ident,
                        _ => unreachable!()
                    };

                    if name.as_str() == "start" {
                        start_function = Some(expr.clone());
                    } else {
                        functions.insert(name, expr.clone());
                    }
                }
                _ => {
                    return Err(RuntimeError::new(format!("Unimplemented expression: {}", expr)));
                }
            }
        }
    } else {
        return Err(RuntimeError::new(format!("Unexpected AST root statement, expected Statement::Program, got Statement::{}", ast)));
    }

    if start_function.is_none() {
        return Err(RuntimeError::new("No start() function found! Could not interpret the code!"));
    }

    println!("RUNNING START FUNCTION");



    Ok(())
}