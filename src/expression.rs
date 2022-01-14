use std::fmt::{Display, format, Formatter};
use cli_tree::TreeNode;
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Program{ exprs: Vec<Expression> }, // program - contains the expressions of the program
    Use { exprs: Vec<Expression> }, // file - contains the expressions of the file
    Block { exprs: Vec<Expression> }, // block - holds a list of contained expressions
    Fn {ident: Box<Expression>, params: Vec<Expression>, return_type: Box<Expression>, body: Box<Expression> },
    // parameters are Declaration expressions, where if there is an assignment, its the default value
    FnCall { ident: Box<Expression>, params: Vec<Expression> },
    If {condition: Box<Expression>, body: Box<Expression>, else_statement: Option<Box<Expression>> }, // else is optional
    Return { value: Box<Expression> },
    Unary { op: Operator, expr: Box<Expression>, leading: bool },
    Binary { left: Box<Expression>, op: Operator, right: Box<Expression> },

    Panic { value: Box<Expression> },
    Assert { expr: Box<Expression> },

    Declaration { ident: Box<Expression>, type_ident: Option<Box<Expression>>, value: Option<Box<Expression>> },
    Assignment { ident: Box<Expression>, value: Box<Expression> },
    PropertyAccess { expr: Box<Expression>, property: Box<Expression> },
    ArrayAccess { ident: Box<Expression>, index: Box<Expression> },

    // Loops
    While { condition: Box<Expression>, body: Box<Expression> },
    For { ident: Box<Expression>, collection: Box<Expression>, body: Box<Expression> },
    Loop { body: Box<Expression> },
    Break, // break - holds nothing
    Continue, // continue - holds nothing

    // Literals / identifiers
    Type { modifiers: Vec<Expression>, type_ident: Box<Expression> },
    ArrayType { array_type: Box<Expression>, size: Box<Expression>, modifiers: Vec<Expression> },
    Identifier { ident: String }, // identifier - holds the name of the identifier
    StringLiteral { value: String }, // string literal - holds the string
    NumberLiteral { value: String }, // number literal - holds the number todo(eric): expand this to different number types
    BinaryLiteral { value: String }, // binary literal - holds the binary literal
    HexLiteral { value: String }, // hexadecimal literal - holds the binary literal
    CharLiteral { value: char }, // char literal - holds the character
    BoolLiteral { value: bool }, // boolean literal - holds the boolean

    Reference,
    Pointer,
    NOP, // nop - holds nothing
    Void, // void - holds nothing, means nothing
}

fn output_params(params: &Vec<Expression>, depth: &usize, indent: &String) -> String {
    let mut param_out = format!("");
    if !params.is_empty() {
        param_out = format!("{}  - Parameters:\n", indent);
        for p in params {
            param_out += &p.display(depth + 3).as_str();
        }
    }
    param_out
}

impl Expression {
    pub fn display(&self, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        match self {
            Expression::Program { exprs } => {
                let mut output = format!("{indent}- Program:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Expression::Use { exprs } => {
               let mut output = format!("{indent}- Import:\n");
               for ex in exprs {
                   output += &ex.display(depth + 1).as_str();
               }
               output
            }
            Expression::Block { exprs } => {
                let mut output = format!("{indent}- Block:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Expression::Fn {
                ident, params,
                return_type: ret, body
            } => {
                let param_out = output_params(params, &depth, &indent);
                format!("{indent}- Function:\n{indent}  - Ident:\n{}{}{indent}  - Returns:\n{}{indent}  - Body:\n{}",
                        ident.display(depth + 2),
                        param_out,
                        ret.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::FnCall { ident, params } => {
                let param_out = output_params(params, &depth, &indent);
                format!("{indent}- Function Call:\n{indent}  - Ident:\n{}{}",
                        ident.display(depth + 2),
                        param_out)
            }
            Expression::Panic { value } => {
                format!("{indent}- Panic:\n{}", value.display(depth + 1))
            }
            Expression::Assert { expr } => {
                format!("{indent}- Assert:\n{}", expr.display(depth + 1))
            }
            Expression::If {
                condition, body,
                else_statement } => {
                format!("{indent}- If:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}{}",
                        condition.display(depth + 2),
                        body.display(depth + 2),
                        if else_statement.is_some() {
                            format!("{indent}  - Else:\n{}", else_statement.as_ref().unwrap().display(depth + 2))
                        } else {
                            format!("")
                        })
            }
            Expression::Return { value } => {
                format!("{indent}- Return:\n{indent}  - Value:\n{}", value.display(depth + 2))
            }
            Expression::Unary { op, expr: expression, leading } => {
                format!("{indent}- Unary:\n{indent}  - Placement: {}\n{indent}  - Operator: {}\n{indent}  - Expression:\n{}",
                        if *leading { "prefix" } else { "postfix" },
                        op,
                        expression.display(depth + 2))
            }
            Expression::Binary { left, op, right } => {
                format!("{indent}- Binary:\n{indent}  - Left:\n{}{indent}  - Operator: {}\n{indent}  - Right:\n{}",
                        left.display(depth + 2),
                        op,
                        right.display(depth + 2))
            }
            Expression::Declaration {
                ident, type_ident, value} => {
                format!("{indent}- Declaration:\n{}{}{}",
                        ident.display(depth + 1),
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
            Expression::Assignment { ident, value } => {
                format!("{indent}- Assignment:\n{}{indent}  - Value:\n{}",
                        ident.display(depth + 1),
                        value.display(depth + 2))
            }
            Expression::PropertyAccess { expr, property } => {
                format!("{indent}- Property Access:\n{indent}  - Expression:\n{}{indent}  - Property:\n{}",
                        expr.display(depth + 2),
                        property.display(depth + 2))
            }
            Expression::ArrayAccess { ident, index } => {
                format!("{indent}- Array Access:\n{}{indent}  - Index:\n{}",
                        ident.display(depth + 1),
                        index.display(depth + 2))
            }
            Expression::While { condition, body } => {
                format!("{indent}- While:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}",
                        condition.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::For { ident, collection,
                body } => {
                format!("{indent}- For:\n{indent}  - Ident:\n{}{indent}  - Collection:\n{}{indent}  - Body:\n{}",
                        ident.display(depth + 2),
                        collection.display(depth + 2),
                        body.display(depth + 2))
            }
            Expression::Loop { body } => {
                format!("{indent}- Loop:\n{indent}  - Body:\n{}", body.display(depth + 2))
            }
            Expression::Break => return format!("{indent}- Break\n"),
            Expression::Continue => return format!("{indent}- Continue\n"),
            Expression::Type { modifiers, type_ident } => {
                let mut mods = format!("");
                if !modifiers.is_empty() {
                    mods = format!("{}  - Modifiers:\n", indent);
                    for m in modifiers {
                        mods += &m.display(depth + 3).as_str();
                    }
                }
                format!("{indent}-  Type:\n{}{}",
                        type_ident.display(depth + 2),
                        mods)
            }
            Expression::ArrayType { array_type: type_ident, size, modifiers } => {
                let mut mods = format!("");
                if !modifiers.is_empty() {
                    mods = format!("{}  - Modifiers:\n", indent);
                    for m in modifiers {
                        mods += &m.display(depth + 3).as_str();
                    }
                }
                format!("{indent}-  Array Type:\n{}{}{}",
                        type_ident.display(depth + 2),
                        format!("{indent}  - Size:\n{}", size.display(depth + 3)),
                        mods)
            }
            Expression::Identifier { ident } => return format!("{indent}- Identifier: {}\n", ident),
            Expression::StringLiteral { value } => return format!("{indent}- StringLiteral: \"{}\"\n", value),
            Expression::NumberLiteral { value } => return format!("{indent}- NumberLiteral: {}\n", value),
            Expression::BinaryLiteral { value } => return format!("{indent}- BinaryLiteral: {}\n", value),
            Expression::HexLiteral { value } => return format!("{indent}- HexLiteral: {}\n", value),
            Expression::CharLiteral { value } => return format!("{indent}- CharLiteral: {}\n", value),
            Expression::BoolLiteral { value } => return format!("{indent}- BoolLiteral: {}\n", value),
            Expression::Reference => return format!("{indent}- Reference\n"),
            Expression::Pointer => return format!("{indent}- Pointer\n"),
            Expression::Void => return format!("{indent}- Void\n"),
            Expression::NOP => return format!("{indent}- NOP\n"),
        }
    }

    pub fn as_treenode(&self) -> TreeNode {
        match self {
            Expression::Program { exprs } => TreeNode::new_with_children("AST",
                exprs.iter().map(|e| e.as_treenode()).collect()),
            Expression::Block { exprs } => TreeNode::new_with_children("Block",
                                                                         exprs.iter().map(|e| e.as_treenode()).collect()),
            Expression::Use { exprs } => TreeNode::new_with_children("Use",
                                                                         exprs.iter().map(|e| e.as_treenode()).collect()),
            Expression::Fn { ident, params, return_type, body } => {
                let mut node = TreeNode::new("Function");
                node.add_child(ident.as_treenode());
                node.add_child(TreeNode::new_with_children("Parameters",
                                                           params.iter().map(|p| p.as_treenode()).collect()));
                node.add_child(TreeNode::new_with_children("Return Type",
                                                           vec![return_type.as_treenode()]));
                node.add_child(body.as_treenode());
                node
            }
            Expression::FnCall { ident, params } => {
                let mut node = TreeNode::new("Function Call");
                node.add_child(ident.as_treenode());
                node.add_child(TreeNode::new_with_children("Parameters",
                                                           params.iter().map(|p| p.as_treenode()).collect()));
                node
            }
            Expression::Declaration { ident, type_ident, value } => {
                let mut node = TreeNode::new("Declaration");
                node.add_child(ident.as_treenode());
                if let Some(t) = type_ident {
                    node.add_child(t.as_treenode());
                }
                if let Some(v) = value {
                    node.add_child(v.as_treenode());
                }
                node
            }
            Expression::Assignment { ident, value } => {
                let mut node = TreeNode::new("Assignment");
                node.add_child(ident.as_treenode());
                node.add_child(value.as_treenode());
                node
            }
            Expression::Panic { value } => {
                let mut node = TreeNode::new("Panic");
                node.add_child(TreeNode::new_with_children("Value:", vec![value.as_treenode()]));
                node
            }
            Expression::Assert { expr } => {
                let mut node = TreeNode::new("Assert");
                node.add_child(TreeNode::new_with_children("Value:", vec![expr.as_treenode()]));
                node
            }
            Expression::Binary { left, op, right} => {
                let mut node = TreeNode::new("Binary");
                node.add_child(left.as_treenode());
                node.add_child((*op).into());
                node.add_child(right.as_treenode());
                node
            }
            Expression::Unary { expr, op, leading } => {
                let mut node = TreeNode::new("Unary");
                node.add_child((*op).into());
                node.add_child(expr.as_treenode());
                node.add_child(TreeNode::new(format!("Leading: {}", leading)));
                node
            }
            Expression::PropertyAccess { property, expr} => {
                let mut node = TreeNode::new("Property Access");
                node.add_child(property.as_treenode());
                node.add_child(expr.as_treenode());
                node
            }
            Expression::ArrayAccess { ident, index } => {
                let mut node = TreeNode::new("Array Access");
                node.add_child(ident.as_treenode());
                node.add_child(index.as_treenode());
                node
            }
            Expression::If { condition, body, else_statement } => {
                let mut node = TreeNode::new("If");
                node.add_child(condition.as_treenode());
                node.add_child(body.as_treenode());
                if let Some(else_statement) = else_statement {
                    node.add_child(else_statement.as_treenode());
                }
                node
            }
            Expression::Return { value } => {
                let mut node = TreeNode::new("Return");
                node.add_child(value.as_treenode());
                node
            }
            Expression::While { condition, body } => {
                let mut node = TreeNode::new("While");
                node.add_child(condition.as_treenode());
                node.add_child(body.as_treenode());
                node
            }
            Expression::Loop { body } => {
                let mut node = TreeNode::new("Loop");
                node.add_child(body.as_treenode());
                node
            }
            Expression::For { ident, collection, body } => {
                let mut node = TreeNode::new("For");
                node.add_child(ident.as_treenode());
                node.add_child(collection.as_treenode());
                node.add_child(body.as_treenode());
                node
            }
            Expression::Type { modifiers, type_ident } => {
                let mut node = TreeNode::new("Type");
                node.add_child(type_ident.as_treenode());
                if !modifiers.is_empty() {
                    node.add_child(TreeNode::new_with_children("Modifiers:",
                                                               modifiers.iter().map(|m| m.as_treenode()).collect()));
                }
                node
            }
            Expression::ArrayType { array_type: type_ident, size, modifiers } => {
                let mut node = TreeNode::new("Array Type");
                node.add_child(type_ident.as_treenode());
                node.add_child(TreeNode::new(format!("Size: {}", size)));
                if !modifiers.is_empty() {
                    node.add_child(TreeNode::new_with_children("Modifiers:",
                                                               modifiers.iter().map(|m| m.as_treenode()).collect()));
                }
                node
            }
            Expression::Break => TreeNode::new("Break"),
            Expression::Continue => TreeNode::new("Continue"),
            Expression::Identifier { ident } => TreeNode::new(format!("Identifier: {}", ident)),
            Expression::StringLiteral { value } => TreeNode::new(format!("String: {}", value)),
            Expression::NumberLiteral { value } => TreeNode::new(format!("Number: {}", value)),
            Expression::BoolLiteral { value } => TreeNode::new(format!("Boolean: {}", value)),
            Expression::CharLiteral { value } => TreeNode::new(format!("Char: {}", value)),
            Expression::HexLiteral { value } => TreeNode::new(format!("Hex: {}", value)),
            Expression::BinaryLiteral { value } => TreeNode::new(format!("Binary: {}", value)),
            Expression::Reference => TreeNode::new("Reference"),
            Expression::Pointer => TreeNode::new("Pointer"),
            Expression::Void => TreeNode::new("Void"),
            Expression::NOP => TreeNode::new("NOP"),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display(0))
    }
}