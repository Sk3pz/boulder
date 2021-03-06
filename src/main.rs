#![feature(core_intrinsics)]

mod argument_parser;
pub mod input_reader;
pub mod lexer;
pub mod token;
pub mod operator;
pub mod error;
mod parser;
pub mod statement;
mod gen_c;
mod interpreter;

use std::{env, fs};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use better_term::{Color, flush_styles};
use cli_tree::TreeNode;
use crate::argument_parser::{Argument, parse_args};
use crate::error::{Error, print_error};
use crate::gen_c::generate_c_code;
use crate::input_reader::InputReader;
use crate::interpreter::interpret;
use crate::lexer::lex;
use crate::parser::parse;
use crate::token::TokenList;

fn round(value: f64, place: usize) -> f64 {
    let round_by = 10.0f64.powi(place as i32) as f64;
    (value * round_by).round() / round_by
}

/// Times a function and returns the function's result as well as the time it took
/// in milliseconds.
fn time_taken<T, F: FnMut() -> T>(mut f: F) -> (T, f64) {
    let start = std::time::Instant::now();
    let result = f();
    let end = start.elapsed();
    let millis = round(end.as_nanos() as f64 / 1000000.0, 3);
    (result, millis)
}

#[cfg(test)]
mod tests {
    use crate::{CodePos, InputReader, time_taken, TokenList};
    use crate::lexer::lex;
    use crate::token::{Token, TokenType};

    #[test]
    fn le_testing_zone() {

    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodePos {
    pub file: Option<String>,
    pub line: usize,
    pub ch: usize,
}

impl CodePos {
    pub fn new(file: String, line: usize, ch: usize) -> CodePos {
        CodePos {
            file: Some(file),
            line,
            ch,
        }
    }

    pub fn newline(&mut self) {
        self.line += 1;
        self.ch = 1;
    }

    pub fn next(&mut self) {
        self.ch += 1;
    }
}

impl Default for CodePos {
    fn default() -> Self {
        CodePos { file: None, line: 1, ch: 1 }
    }
}

impl Display for CodePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.file.is_some() {
            write!(f, "{}:{}:{}", self.file.as_ref().unwrap(), self.line, self.ch)
        } else {
            write!(f, "code:{}:{}", self.line, self.ch)
        }
    }
}

pub fn validate_file(file: &str) -> Result<(), String> {
    let path = Path::new(&file);
    if !path.exists() {
        return Err(format!("File {} does not exist", file));
    }
    if !path.is_file() {
        return Err(format!("File {} is not a file", file));
    }
    Ok(())
}

pub fn validate_boulder_file<S: Into<String>>(file: S) -> Result<(), String> {
    let f = file.into();
    let path = Path::new(&f);
    validate_file(&f)?;
    if !path.extension().unwrap().to_str().unwrap().eq("rock") {
        return Err(format!("File {} is not a boulder file", f));
    }
    Ok(())
}

fn print_help() {
    println!(
        "{}Boulder Help:\n\
        {}boulder {ob}[{o}mode{ob}] {ob}[{o}input_file{ob}] [{o}options{ob}]\n\
        {t}{}Modes:\n\
        {t}{c}int{ob}, {c}interpret    {c2}Interpretation mode\n\
        {t}{c}cmp{ob}, {c}compile      {c2}Compilation mode\n\
        {t}Defaults to interpret\n\
        {t}{}Options:\n\
        {t}{c}-h{ob}, {c}--help    {c2}Print this help message\n\
        {t}{c}-v{ob}, {c}--version {c2}Print the version of boulder\n\
        {t}{c}-o{ob}, {c}--output  {ob}[{o}output_file{ob}] {c2}Write the output to a file\n\
        {t}{c}-c{ob}, {c}--color   {ob}[{o}true{ob}|{o}false{ob}] {c2}Set the output to be colored or not\n\
        {t}{c}-d{ob}, {c}--debug   {c2}Compile in debug mode\n\
        {t}{c}-r{ob}, {c}--release {c2}Compile in release mode\n\
        {t}{c}-q{ob}, {c}--quiet   {c2}No output, just compile (this will still show errors)\n\
        {t}{c}-i{ob}, {c}--verbose {c2}All the output that is possible will appear.",
        Color::BrightWhite, Color::BrightWhite, Color::BrightWhite, c = Color::White, c2 = Color::BrightWhite,
        ob = Color::BrightBlack, o = Color::BrightWhite, t = "  "
    );
    flush_styles();
}

fn print_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

pub fn read_file<P: AsRef<Path>>(path: P) -> String {
    let input_file_result = fs::read_to_string(&path.as_ref());
    if input_file_result.is_err() {
        println!("{}Failed to read {}: {}", Color::Red, path.as_ref().to_str().unwrap(), input_file_result.unwrap_err());
        return String::new();
    }
    input_file_result.unwrap()
}

fn main() {
    // parse arguments
    let mut args = env::args().collect::<Vec<String>>();
    args.remove(0);

    let input_file: &Path;
    let mut color = true;
    let mut release = false;
    let mut quiet = false;
    let mut verbose = false;

    // if the arguments are empty, there is nothing to do
    if args.is_empty() {
        println!("{}No input file found!", Color::Red);
        flush_styles();
        print_help();
        return;
    }

    if args.len() < 2 {
        println!("{}No mode found!", Color::Red);
        flush_styles();
        print_help();
        return;
    }

    // get the mode defined
    let mode_in = &args.get(0).unwrap().clone();
    args.remove(0);

    // get the input file
    let input_file_in = &args.get(0).unwrap().clone();
    args.remove(0);

    // make sure its not a help command or a version command, and if it is, print the help or version
    // otherwise, validate the input file
    match input_file_in.as_str() {
        "-h" | "-H" | "--help" => {
            print_help();
            return;
        }
        "-v" | "-V" | "--version" => {
            print_version();
            return;
        }
        _ => {
            if let Err(e) = validate_boulder_file(input_file_in) {
                println!("{}{}", Color::Red, e);
                flush_styles();
                return;
            }
            input_file = Path::new(input_file_in);
        }
    }


    // get the compiler mode
    let mode = match mode_in.as_str() {
        "int" | "interpret" => 0,
        "cmp" | "compile" => 1,
        _ => {
            println!("{}Invalid mode! Can either be cmp or int!", Color::Red);
            flush_styles();
            print_help();
            return;
        }
    };

    // the output file
    let mut output_file_path = input_file.to_str().expect("failed in converting file path to string")
        .replace(".rock", "");

    if !args.is_empty() {
        let arguments = parse_args(&args);
        for a in arguments {
            match a {
                Argument::Output(o) => {
                    output_file_path = o;
                }
                Argument::Help => {
                    print_help();
                    return;
                }
                Argument::Version => {
                    print_version();
                    return;
                }
                Argument::Color(b) => {
                    color = b;
                }
                Argument::Quiet => {
                    quiet = true;
                }
                Argument::Verbose => {
                    if quiet {
                        println!("{}Cannot use verbose and quiet at the same time", Color::Red);
                        flush_styles();
                        return;
                    }
                    verbose = true;
                }
                Argument::Debug => {
                    release = false;
                }
                Argument::Release => {
                    release = true;
                }
            }
        }
    }

    let code = read_file(&input_file);

    if verbose {
        println!("Lexing the code...");
    }

    // handle reading the input
    let mut input_reader = InputReader::new(Some(input_file_in.clone()), &code);

    // tokenize the input through the lexer
    let (mut tokens, lex_time) = time_taken(|| lex(&mut input_reader));
    if tokens.is_err() {
        print_error(tokens.unwrap_err());
        return;
    }

    if verbose {
        let lex_display_time = if lex_time > 500.0 {
            format!("{}s", round(lex_time / 1000.0, 3))
        } else {
            format!("{}ms", lex_time)
        };
        println!("Lexing done. Took {}.", lex_display_time);
        //println!("Tokens:\n{}", tokens.unwrap());
        println!("Parsing tokens...");
    }

    // parse the tokens into an AST
    let (mut ast, parse_time) = time_taken(|| parse(&mut tokens.as_mut().unwrap()));
    if ast.is_err() {
        print_error(ast.unwrap_err());
        return;
    }

    if verbose {
        let parse_display_time = if parse_time > 500.0 {
            format!("{}s", round(parse_time / 1000.0, 3))
        } else {
            format!("{}ms", parse_time)
        };
        println!("Parsing done. Took {}.", parse_display_time);
        // println!("AST:\n{}", ast.unwrap());
        let tree_node: TreeNode = ast.as_mut().unwrap().as_treenode();
        println!("{}", tree_node);
    }

    if mode == 0 { // interpretation mode
        if verbose {
            println!("Interpreting code...");
        }
        // interpret the ast
        let (res, interpret_time) = time_taken(|| interpret(&mut ast.as_mut().unwrap()));

        if res.is_err() {
            println!("{}", res.unwrap_err());
            return;
        }

        if verbose {
            let parse_display_time = if interpret_time > 500.0 {
                format!("{}s", round(interpret_time / 1000.0, 3))
            } else {
                format!("{}ms", interpret_time)
            };
            println!("Ran the code. Took {}.", parse_display_time);
        }

    } else { // compilation mode
        if verbose {
            println!("Generating code...");
        }
        // compile to a string
        let (res, compile_time) = time_taken(|| generate_c_code(&mut ast.as_mut().unwrap()));

        // writing the string to a file

        if res.is_err() {
            println!("{}", res.unwrap_err());
            return;
        }

        let mut output_file = OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(output_file_path)
            .expect("Failed to open output file for writing.");

        let write_result = output_file.write_all(res.unwrap().as_bytes());

        if write_result.is_err() {
            println!("{}Failed to write to output file: {}", Color::Red, write_result.unwrap_err());
            return;
        }

        if verbose {
            let parse_display_time = if compile_time > 500.0 {
                format!("{}s", round(compile_time / 1000.0, 3))
            } else {
                format!("{}ms", compile_time)
            };
            println!("Generated code. Took {}.", parse_display_time);
        }
    }

    flush_styles();
}