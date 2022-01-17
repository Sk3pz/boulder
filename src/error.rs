use std::fmt::{Display, Formatter};
use better_term::{Color, Style};
use crate::{CodePos, read_file};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub core_msg: Option<String>,
    pub msg: String,
    pub pos: CodePos
}

impl Error {
    pub fn new<S: Into<String>, S2: Into<String>>(core: S, msg: S2, pos: CodePos) -> Self {
        Self {
            core_msg: Some(core.into()),
            msg: msg.into(),
            pos
        }
    }
    pub fn new_singular<S: Into<String>>(msg: S, pos: CodePos) -> Self {
        Self {
            core_msg: None,
            msg: msg.into(),
            pos
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.core_msg.is_some() {
            write!(f, "{}error{}: {}: {}{}",
                   Style::default().fg(Color::BrightRed).bold(),
                   Color::BrightWhite, self.core_msg.as_ref().unwrap().clone(), self.msg, Style::reset())
        } else {
            write!(f, "{}error{}: {}{}",
                   Style::default().fg(Color::BrightRed).bold(), Color::BrightWhite, self.msg, Style::reset())
        }
    }
}

pub fn print_error(err: Error) {
    let mut code = String::new();
    if err.pos.file.is_some() {
        code = read_file(err.pos.file.as_ref().unwrap().clone());
    }
    let err_msg = format!("{}", err.clone()); // the message to display
    // the line the error is on
    let mut full_line = code.split("\n").collect::<Vec<&str>>().get(err.pos.line - 1).unwrap().to_string();
    // remove leading spaces
    let mut removed = 0;
    while full_line.len() > 0 && full_line.chars().collect::<Vec<char>>().first().unwrap().clone() == ' ' {
        full_line.remove(0);
        removed += 1;
    }
    // the line with the error with comments removed
    let line = full_line.split("//").collect::<Vec<&str>>().get(0).unwrap().to_string();
    // the number to display for the error
    let line_num = format!("{} | ", err.pos.line);
    // the pipes with spaces before them before and after the error line
    let pipe = format!("{} | ", " ".repeat(err.pos.line.to_string().len()));
    // the carrot to display for the error
    let carrot_spaces = " ".repeat(err.pos.ch - 1 - removed);
    let carrot = carrot_spaces + "^" + if err.core_msg.is_some() {
        format!(" {}", err.core_msg.unwrap())
    } else {
        format!("")
    }.as_str();
    // how many spaces before the location
    let loc_spaces = " ".repeat(err.pos.line / 10);

    let error_style = Style::default().fg(Color::BrightRed).bold();

    if line.is_empty() {
        println!("{}{}\n\
            {}  --> {}{}", err_msg,
                 loc_spaces, Color::Blue, Color::White, err.pos);
    } else {
        println!("{}\n\
            {pc}{}--> {}{}\n\
            {pc}{}\n\
            {pc}{}{}{}\n\
            {pc}{}{ec}{}{ecr}",
                 err_msg,
                 loc_spaces, Color::White, err.pos,
                 pipe.clone(),
                 line_num, Color::White, line,
                 pipe, carrot,
                 pc = Color::Blue, ec = error_style, ecr = Style::reset());
    }
}

pub struct CompilerError {
    pub msg: String,
}

impl CompilerError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self {
            msg: msg.into()
        }
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}error{}: {}{}",
               Style::default().fg(Color::BrightRed).bold(), Color::BrightWhite, self.msg, Style::reset())
    }
}