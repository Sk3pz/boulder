use std::fmt::Display;
use better_term::Color;
use crate::CodePos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub core_msg: Option<String>,
    pub msg: String,
    pub pos: CodePos
}

impl Error {
    pub fn new<S: Into<String>, S2: Into<String>>(core: Option<S>, msg: S2, pos: CodePos) -> Self {
        let core_msg = if core.is_some() {
            Some(core.unwrap().into())
        } else {
            None
        };
        Self {
            core_msg,
            msg: msg.into(),
            pos
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.core_msg.is_some() {
            write!(f, "{}error{}: {}: {}",
                   Color::BrightRed, Color::BrightWhite, self.core_msg.as_ref().unwrap().clone(), self.msg)
        } else {
            write!(f, "{}error{}: {}",
                   Color::BrightRed, Color::BrightWhite, self.msg)
        }
    }
}

pub fn print_error(err: Error, code: String) {
    let err_msg = format!("{}", err.clone());
    let mut spaces = " ".repeat(err.pos.line / 10);
    let full_line = code.split("\n").collect::<Vec<&str>>().get(err.pos.line - 1).unwrap().to_string();
    let line = full_line.split("//").collect::<Vec<&str>>().get(0).unwrap().to_string();
    let line_num = format!("{} | ", err.pos.line);
    let pipe = format!("{}|", " ".repeat(err.pos.line / 10 + 2));
    let carrot = " ".repeat(err.pos.ch - 1) + "^" + if err.core_msg.is_some() {
        format!(" {}", err.core_msg.unwrap())
    } else {
        format!("")
    }.as_str();
    let loc_spaces = " ".repeat(err.pos.line / 10 + 1);

    println!("{}\n\
    {pc}{}--> {}{}\n\
    {pc}{}\n\
    {pc}{}{ec}{}\n\
    {pc}{}{}{ec}{}",
             err_msg,
             loc_spaces, Color::White, err.pos,
             pipe.clone(),
             line_num, line,
             pipe, spaces, carrot,
             pc = Color::Blue, ec = Color::Red);
}