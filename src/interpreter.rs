use std::fmt::Display;
use better_term::{Color, Style};

use crate::statement::Statement;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub msg: String,
}

impl RuntimeError {
    pub fn new(msg: String) -> RuntimeError {
        RuntimeError { msg }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}error{}: {}{}",
                   Style::default().fg(Color::BrightRed).bold(), Color::BrightWhite, self.msg, Style::reset())
    }
}

pub fn interpret(ast: &Statement) -> Result<(), RuntimeError> {

    if let Statement::Program { .. } = ast {
        
    } else {
        return Err(RuntimeError::new(format!("Unexpected AST root statement, expected Statement::Program, got Statement::{}", ast)));
    }

    Ok(())
}