#[derive(Debug, Clone)]
pub enum Argument {
    Help,
    Version,
    Output(String),
    Verbose,
    Quiet,
    Debug,
    Release,
    Color(bool),
}

pub fn parse_args(args: &[String]) -> Vec<Argument> {
    let mut arguments = Vec::new();
    let mut expecting_output = false;
    let mut expecting_color = false;
    for arg in args {
        match arg.as_str() {
            "-h" | "-H" | "--help" | "?" => {
                return vec![Argument::Help];
            }
            "-v" | "-V" | "--version" => {
                return vec![Argument::Version];
            }
            "-o" | "-O" | "--output" => {
                expecting_output = true;
            }
            "-c" | "-C" | "--color" => {
                expecting_color = true;
            }
            "-i" | "-I" | "--verbose" => {
                arguments.push(Argument::Verbose);
            }
            "-q" | "-Q" | "--quiet" => {
                arguments.push(Argument::Quiet);
            }
            "-d" | "-D" | "--debug" => {
                arguments.push(Argument::Debug);
            }
            "-r" | "-R" | "--release" => {
                arguments.push(Argument::Release);
            }
            _ => {
                if expecting_output {
                    arguments.push(Argument::Output(arg.clone()));
                    expecting_output = false;
                    continue;
                }
                if expecting_color {
                    arguments.push(Argument::Color(arg.as_str() == "true"));
                    expecting_color = false;
                    continue;
                }
                println!("Unknown argument: {}", arg);
            }
        }
    }
    arguments
}