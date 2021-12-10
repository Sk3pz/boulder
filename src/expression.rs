use std::fmt::{Display, Formatter};
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Program(Vec<Expression>), // program - contains the expressions of the program
    Use(Vec<Expression>), // file - contains the expressions of the file
    Block(Vec<Expression>), // block - holds a list of contained expressions
    Fn(Box<Expression>, Vec<Expression>, Box<Expression>, Box<Expression>), // fn - holds the identifier, the parameters, the return type, and the body
    // parameters are Declaration expressions, where if there is an assignment, its the default value
    FnCall(Box<Expression>, Vec<Expression>), // FnCall - holds the identifier and the parameters
    If(Box<Expression>, Box<Expression>, Option<Box<Expression>>), // if - holds the condition, the body, and the potential else
    ElseIf(Box<Expression>, Box<Expression>, Option<Box<Expression>>), // elseif - holds the condition, the body, and the potential else
    Else(Box<Expression>), // else - holds the body
    Return(Box<Expression>), // return - holds the expression
    Unary(Operator, Box<Expression>), // unary - holds the operator and the expression
    Binary(Box<Expression>, Operator, Box<Expression>), // binary - holds the left and right expressions and the operator

    Declaration(Box<Expression>, Option<Box<Expression>>, Option<Box<Expression>>), // declaration - holds the identifier, the type and the expression
    Assignment(Box<Expression>, Box<Expression>), // assignment - holds the identifier and the expression
    PropertyAccess(Box<Expression>, Box<Expression>), // property access - holds the expression and the property

    // Loops
    While(Box<Expression>, Box<Expression>), // while - holds the condition and the body
    For(Box<Expression>, Box<Expression>, Box<Expression>), // for - holds the identifier, the collection / range, and the body
    Loop(Box<Expression>), // loop - holds the body
    Break, // break - holds nothing
    Continue, // continue - holds nothing

    // Literals / identifiers
    Identifier(String), // identifier - holds the name of the identifier
    StringLiteral(String), // string literal - holds the string
    NumberLiteral(String), // number literal - holds the number todo(eric): expand this to different number types
    BinaryLiteral(String), // binary literal - holds the binary literal
    HexLiteral(String), // hexadecimal literal - holds the binary literal
    CharLiteral(char), // char literal - holds the character
    BoolLiteral(bool), // boolean literal - holds the boolean
    NOP, // nop - holds nothing
    Void, // void - holds nothing, means nothing
}

impl Expression {
    pub fn display(&self, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        match self {
            Expression::Program(exprs) => {
                let mut output = format!("{indent}- Program:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Expression::Use(exprs) => {
               let mut output = format!("{indent}- Import:\n");
               for ex in exprs {
                   output += &ex.display(depth + 1).as_str();
               }
               output
            }
            Expression::Block(exprs) => {
                let mut output = format!("{indent}- Block:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Expression::Fn(ident, params,
                           ret, body) => {
                let mut param_out = format!("");
                if !params.is_empty() {
                    param_out = format!("{indent}  - Parameters:\n");
                    for p in params {
                        param_out += &p.display(depth + 3).as_str();
                    }
                }
                format!("{indent}- Function:\n{indent}  - Ident:\n{}{}{indent}  - Returns:\n{}{indent}  - Body:\n{}",
                        ident.display(depth + 2),
                        param_out,
                        ret.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::FnCall(ident, params) => {
                let mut param_out = format!("");
                if !params.is_empty() {
                    param_out = format!("{indent}  - Parameters:\n");
                    for p in params {
                        param_out += &p.display(depth + 3).as_str();
                    }
                }
                format!("{indent}- Function Call:\n{indent}  - Ident:\n{}{}",
                        ident.display(depth + 2),
                        param_out)
            }
            Expression::If(condition, body,
                           else_statement) => {
                format!("{indent}- If:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}{}",
                        condition.display(depth + 2),
                        body.display(depth + 2),
                        if else_statement.is_some() {
                            format!("{indent}  - Else:\n{}", else_statement.as_ref().unwrap().display(depth + 2))
                        } else {
                            format!("")
                        })
            }
            Expression::ElseIf(condition, body,
                               else_statement) => {
                format!("{indent}- ElseIf:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}{}",
                        condition.display(depth + 2),
                        body.display(depth + 2),
                        if else_statement.is_some() {
                            format!("{indent}  - Else:\n{}", else_statement.as_ref().unwrap().display(depth + 2))
                        } else {
                            format!("")
                        })
            }
            Expression::Else(body) => {
                format!("{indent}- Else:\n{indent}  - Body:\n{}", body.display(depth + 2))
            }
            Expression::Return(value) => {
                format!("{indent}- Return:\n{indent}  - Value:\n{}", value.display(depth + 2))
            }
            Expression::Unary(op, expression) => {
                format!("{indent}- Unary:\n{indent}  - Operator: {}\n{indent}  - Expression:\n{}",
                        op,
                        expression.display(depth + 2))
            }
            Expression::Binary(left, op,
                               right) => {
                format!("{indent}- Binary:\n{indent}  - Left:\n{}{indent}  - Operator: {}\n{indent}  - Right:\n{}",
                        left.display(depth + 2),
                        op,
                        right.display(depth + 2))
            }
            Expression::Declaration(ident, type_ident,
                                    value) => {
                format!("{indent}- Declaration:\n{indent}  - Ident:\n{}{}{}",
                        ident.display(depth + 2),
                        if type_ident.is_some() {
                            format!("{indent}  - Type:\n{}", type_ident.as_ref().unwrap().display(depth + 2))
                        } else {
                            format!("{indent}  - Type: Unspecified\n")
                        },
                        if value.is_some() {
                            format!("{indent}  - Value:\n{}", value.as_ref().unwrap().display(depth + 2))
                        } else {
                            format!("{indent}  - Value: None\n")
                        })
            }
            Expression::Assignment(ident, value) => {
                format!("{indent}- Assignment:\n{indent}  - Ident:\n{}{indent}  - Value:\n{}",
                        ident.display(depth + 2),
                        value.display(depth + 2))
            }
            Expression::PropertyAccess(expr, property) => {
                format!("{indent}- Property Access:\n{indent}  - Expression:\n{}{indent}  - Property:\n{}",
                        expr.display(depth + 2),
                        property.display(depth + 2))
            }
            Expression::While(condition, body) => {
                format!("{indent}- While:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}",
                        condition.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::For(ident, collection,
                            body) => {
                format!("{indent}- For:\n{indent}  - Ident:\n{}{indent}  - Collection:\n{}{indent}  - Body:\n{}",
                        ident.display(depth + 2),
                        collection.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::Loop(body) => {
                format!("{indent}- Loop:\n{indent}  - Body:\n{}", body.display(depth + 2))
            }
            Expression::Break => return format!("{indent}- Break\n"),
            Expression::Continue => return format!("{indent}- Continue\n"),
            Expression::Identifier(s) => return format!("{indent}- Identifier: {}\n", s),
            Expression::StringLiteral(s) => return format!("{indent}- StringLiteral: \"{}\"\n", s),
            Expression::NumberLiteral(s) => return format!("{indent}- NumberLiteral: {}\n", s),
            Expression::BinaryLiteral(s) => return format!("{indent}- BinaryLiteral: {}\n", s),
            Expression::HexLiteral(s) => return format!("{indent}- HexLiteral: {}\n", s),
            Expression::CharLiteral(c) => return format!("{indent}- CharLiteral: {}\n", c),
            Expression::BoolLiteral(b) => return format!("{indent}- BoolLiteral: {}\n", b),
            Expression::Void => return format!("{indent}- Void\n"),
            Expression::NOP => return format!("{indent}- NOP\n"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(0))
    }
}