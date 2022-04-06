use std::fmt::{Display, format, Formatter};
use cli_tree::TreeNode;
use crate::operator::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Number {
    pub value: String,
    pub negative: bool,
}

impl Number {
    pub fn new(value: String, negative: bool) -> Number {
        Number {
            value,
            negative,
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.negative {
            write!(f, "-{}", self.value)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuntedStackItem {
    operator: Option<Operator>,
    operand: Option<Statement>,
}

impl ShuntedStackItem {
    pub fn new_operand(statement: Statement) -> Self {
        Self {
            operator: None,
            operand: Some(statement),
        }
    }

    pub fn new_operator(operator: Operator) -> Self {
        Self {
            operator: Some(operator),
            operand: None,
        }
    }

    pub fn is_operator(&self) -> bool {
        self.operator.is_some()
    }

    pub fn is_operand(&self) -> bool {
        self.operand.is_some()
    }

    pub fn get_operator(&self) -> Option<&Operator> {
        self.operator.as_ref()
    }

    pub fn get_operand(&self) -> Option<&Statement> {
        self.operand.as_ref()
    }

    pub fn as_treenode(&self) -> TreeNode {
        if let Some(operator) = &self.operator {
            operator.as_treenode()
        } else if let Some(operand) = &self.operand {
            operand.as_treenode()
        } else {
            TreeNode::new("<error>")
        }
    }
}

impl Display for ShuntedStackItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_operator() {
            write!(f, "{}", self.get_operator().unwrap())
        } else {
            write!(f, "{}", self.get_operand().unwrap())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuntedStack {
    items: Vec<ShuntedStackItem>,
}

impl ShuntedStack {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, item: ShuntedStackItem) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<ShuntedStackItem> {
        self.items.pop()
    }

    pub fn peek(&self) -> Option<&ShuntedStackItem> {
        self.items.last()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn as_treenode(&self) -> TreeNode {
        let mut tree_nodes = TreeNode::new("Postfix");
        for item in &self.items {
            tree_nodes.add_child(item.as_treenode());
        }
        tree_nodes
    }
}

impl Display for ShuntedStack {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for item in self.items.iter() {
            result.push_str(&format!("{}", item));
        }
        write!(f, "{}", result)
    }
}

impl Iterator for ShuntedStack {
    type Item = ShuntedStackItem;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Program{ exprs: Vec<Statement> }, // program - contains the expressions of the program
    Use { exprs: Vec<Statement> }, // file - contains the expressions of the file
    Block { exprs: Vec<Statement> }, // block - holds a list of contained expressions
    Fn {ident: Box<Statement>, params: Vec<Statement>, return_type: Box<Statement>, body: Box<Statement> },
    // parameters are Declaration expressions, where if there is an assignment, its the default value
    FnCall { ident: Box<Statement>, params: Vec<Statement> },
    If {condition: Box<Statement>, body: Box<Statement>, else_statement: Option<Box<Statement>> }, // else is optional
    Return { value: Box<Statement> },
    Postfix { postfix: ShuntedStack },

    Panic { value: Box<Statement> },
    Assert { expr: Box<Statement> },

    Declaration { ident: Box<Statement>, type_ident: Option<Box<Statement>>, value: Option<Box<Statement>> },
    Assignment { ident: Box<Statement>, value: Box<Statement> },
    PropertyAccess { expr: Box<Statement>, property: Box<Statement> },
    ArrayAccess { ident: Box<Statement>, index: Box<Statement> },

    // Loops
    While { condition: Box<Statement>, body: Box<Statement> },
    For { ident: Box<Statement>, collection: Box<Statement>, body: Box<Statement> },
    Loop { body: Box<Statement> },
    Break, // break - holds nothing
    Continue, // continue - holds nothing

    // Literals / identifiers
    Type { modifiers: Vec<Statement>, type_ident: Box<Statement> },
    ArrayType { array_type: Box<Statement>, size: Box<Statement>, modifiers: Vec<Statement> },
    Identifier { ident: String }, // identifier - holds the name of the identifier
    StringLiteral { value: String }, // string literal - holds the string
    NumberLiteral { value: Number }, // number literal - holds the number todo(eric): expand this to different number types
    BinaryLiteral { value: String }, // binary literal - holds the binary literal
    HexLiteral { value: String }, // hexadecimal literal - holds the binary literal
    CharLiteral { value: char }, // char literal - holds the character
    BoolLiteral { value: bool }, // boolean literal - holds the boolean

    Reference,
    Pointer,
    NOP, // nop - holds nothing
    Void, // void - holds nothing, means nothing
}

fn output_params(params: &Vec<Statement>, depth: &usize, indent: &String) -> String {
    let mut param_out = format!("");
    if !params.is_empty() {
        param_out = format!("{}  - Parameters:\n", indent);
        for p in params {
            param_out += &p.display(depth + 3).as_str();
        }
    }
    param_out
}

impl Statement {
    fn display_modifiers(&self, mods: &Vec<Statement>, indent: String, depth: usize) -> String {
        let mut mods_str = format!("");
        if !mods.is_empty() {
            mods_str = format!("{}  - Modifiers:\n", indent);
            for m in mods {
                mods_str += &m.display(depth + 3).as_str();
            }
        }
        mods_str
    }

    pub fn display(&self, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        match self {
            Statement::Program { exprs } => {
                let mut output = format!("{indent}- Program:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Statement::Use { exprs } => {
               let mut output = format!("{indent}- Import:\n");
               for ex in exprs {
                   output += &ex.display(depth + 1).as_str();
               }
               output
            }
            Statement::Block { exprs } => {
                let mut output = format!("{indent}- Block:\n");
                for ex in exprs {
                    output += &ex.display(depth + 1).as_str();
                }
                output
            }
            Statement::Fn {
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
            Statement::FnCall { ident, params } => {
                let param_out = output_params(params, &depth, &indent);
                format!("{indent}- Function Call:\n{indent}  - Ident:\n{}{}",
                        ident.display(depth + 2),
                        param_out)
            }
            Statement::Panic { value } => {
                format!("{indent}- Panic:\n{}", value.display(depth + 1))
            }
            Statement::Assert { expr } => {
                format!("{indent}- Assert:\n{}", expr.display(depth + 1))
            }
            Statement::If {
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
            Statement::Return { value } => {
                format!("{indent}- Return:\n{indent}  - Value:\n{}", value.display(depth + 2))
            }
            Statement::Postfix { postfix: shunted } => {
                format!("{indent}- Postfix:\n{indent}  - Shunted:\n{}", shunted)
            }
            Statement::Declaration {
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
            Statement::Assignment { ident, value } => {
                format!("{indent}- Assignment:\n{}{indent}  - Value:\n{}",
                        ident.display(depth + 1),
                        value.display(depth + 2))
            }
            Statement::PropertyAccess { expr, property } => {
                format!("{indent}- Property Access:\n{indent}  - Expression:\n{}{indent}  - Property:\n{}",
                        expr.display(depth + 2),
                        property.display(depth + 2))
            }
            Statement::ArrayAccess { ident, index } => {
                format!("{indent}- Array Access:\n{}{indent}  - Index:\n{}",
                        ident.display(depth + 1),
                        index.display(depth + 2))
            }
            Statement::While { condition, body } => {
                format!("{indent}- While:\n{indent}  - Condition:\n{}{indent}  - Body:\n{}",
                        condition.display(depth + 2),
                        body.display(depth + 2))
            }
            Statement::For { ident, collection,
                body } => {
                format!("{indent}- For:\n{indent}  - Ident:\n{}{indent}  - Collection:\n{}{indent}  - Body:\n{}",
                        ident.display(depth + 2),
                        collection.display(depth + 2),
                        body.display(depth + 2))
            }
            Statement::Loop { body } => {
                format!("{indent}- Loop:\n{indent}  - Body:\n{}", body.display(depth + 2))
            }
            Statement::Break => return format!("{indent}- Break\n"),
            Statement::Continue => return format!("{indent}- Continue\n"),
            Statement::Type { modifiers, type_ident } => {
                let mut mods = self.display_modifiers(modifiers, indent.clone(), depth);
                format!("{indent}-  Type:\n{}{}",
                        type_ident.display(depth + 2),
                        mods)
            }
            Statement::ArrayType { array_type: type_ident, size, modifiers } => {
                let mut mods = self.display_modifiers(modifiers, indent.clone(), depth);
                format!("{indent}-  Array Type:\n{}{}{}",
                        type_ident.display(depth + 2),
                        format!("{indent}  - Size:\n{}", size.display(depth + 3)),
                        mods)
            }
            Statement::Identifier { ident } => return format!("{indent}- Identifier: {}\n", ident),
            Statement::StringLiteral { value } => return format!("{indent}- StringLiteral: \"{}\"\n", value),
            Statement::NumberLiteral { value } => return format!("{indent}- NumberLiteral: {}\n", value),
            Statement::BinaryLiteral { value } => return format!("{indent}- BinaryLiteral: {}\n", value),
            Statement::HexLiteral { value } => return format!("{indent}- HexLiteral: {}\n", value),
            Statement::CharLiteral { value } => return format!("{indent}- CharLiteral: {}\n", value),
            Statement::BoolLiteral { value } => return format!("{indent}- BoolLiteral: {}\n", value),
            Statement::Reference => return format!("{indent}- Reference\n"),
            Statement::Pointer => return format!("{indent}- Pointer\n"),
            Statement::Void => return format!("{indent}- Void\n"),
            Statement::NOP => return format!("{indent}- NOP\n"),
        }
    }

    fn mod_tree_child(&self, node: &mut TreeNode, modifiers: &Vec<Statement>) {
        if !modifiers.is_empty() {
            node.add_child(TreeNode::new_with_children("Modifiers:",
                                                       modifiers.iter().map(|m| m.as_treenode()).collect()));
        }
    }

    pub fn as_treenode(&self) -> TreeNode {
        match self {
            Statement::Program { exprs } => TreeNode::new_with_children("AST",
                                                                        exprs.iter().map(|e| e.as_treenode()).collect()),
            Statement::Block { exprs } => TreeNode::new_with_children("Block",
                                                                      exprs.iter().map(|e| e.as_treenode()).collect()),
            Statement::Use { exprs } => TreeNode::new_with_children("Use",
                                                                    exprs.iter().map(|e| e.as_treenode()).collect()),
            Statement::Fn { ident, params, return_type, body } => {
                let mut node = TreeNode::new("Function");
                node.add_child(ident.as_treenode());
                node.add_child(TreeNode::new_with_children("Parameters",
                                                           params.iter().map(|p| p.as_treenode()).collect()));
                node.add_child(TreeNode::new_with_children("Return Type",
                                                           vec![return_type.as_treenode()]));
                node.add_child(body.as_treenode());
                node
            }
            Statement::FnCall { ident, params } => {
                let mut node = TreeNode::new("Function Call");
                node.add_child(ident.as_treenode());
                node.add_child(TreeNode::new_with_children("Parameters",
                                                           params.iter().map(|p| p.as_treenode()).collect()));
                node
            }
            Statement::Declaration { ident, type_ident, value } => {
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
            Statement::Assignment { ident, value } => {
                let mut node = TreeNode::new("Assignment");
                node.add_child(ident.as_treenode());
                node.add_child(value.as_treenode());
                node
            }
            Statement::Panic { value } => {
                let mut node = TreeNode::new("Panic");
                node.add_child(TreeNode::new_with_children("Value:", vec![value.as_treenode()]));
                node
            }
            Statement::Assert { expr } => {
                let mut node = TreeNode::new("Assert");
                node.add_child(TreeNode::new_with_children("Value:", vec![expr.as_treenode()]));
                node
            }
            Statement::Postfix { postfix: shunted } => {
                shunted.as_treenode()
            }
            Statement::PropertyAccess { property, expr} => {
                let mut node = TreeNode::new("Property Access");
                node.add_child(property.as_treenode());
                node.add_child(expr.as_treenode());
                node
            }
            Statement::ArrayAccess { ident, index } => {
                let mut node = TreeNode::new("Array Access");
                node.add_child(ident.as_treenode());
                node.add_child(index.as_treenode());
                node
            }
            Statement::If { condition, body, else_statement } => {
                let mut node = TreeNode::new("If");
                node.add_child(condition.as_treenode());
                node.add_child(body.as_treenode());
                if let Some(else_statement) = else_statement {
                    node.add_child(else_statement.as_treenode());
                }
                node
            }
            Statement::Return { value } => {
                let mut node = TreeNode::new("Return");
                node.add_child(value.as_treenode());
                node
            }
            Statement::While { condition, body } => {
                let mut node = TreeNode::new("While");
                node.add_child(condition.as_treenode());
                node.add_child(body.as_treenode());
                node
            }
            Statement::Loop { body } => {
                let mut node = TreeNode::new("Loop");
                node.add_child(body.as_treenode());
                node
            }
            Statement::For { ident, collection, body } => {
                let mut node = TreeNode::new("For");
                node.add_child(ident.as_treenode());
                node.add_child(collection.as_treenode());
                node.add_child(body.as_treenode());
                node
            }
            Statement::Type { modifiers, type_ident } => {
                let mut node = TreeNode::new("Type");
                node.add_child(type_ident.as_treenode());
                self.mod_tree_child(&mut node, modifiers);
                node
            }
            Statement::ArrayType { array_type: type_ident, size, modifiers } => {
                let mut node = TreeNode::new("Array Type");
                node.add_child(type_ident.as_treenode());
                node.add_child(TreeNode::new(format!("Size: {}", size)));
                self.mod_tree_child(&mut node, modifiers);
                node
            }
            Statement::Break => TreeNode::new("Break"),
            Statement::Continue => TreeNode::new("Continue"),
            Statement::Identifier { ident } => TreeNode::new(format!("Identifier: {}", ident)),
            Statement::StringLiteral { value } => TreeNode::new(format!("String: {}", value)),
            Statement::NumberLiteral { value } => TreeNode::new(format!("Number: {}", value)),
            Statement::BoolLiteral { value } => TreeNode::new(format!("Boolean: {}", value)),
            Statement::CharLiteral { value } => TreeNode::new(format!("Char: {}", value)),
            Statement::HexLiteral { value } => TreeNode::new(format!("Hex: {}", value)),
            Statement::BinaryLiteral { value } => TreeNode::new(format!("Binary: {}", value)),
            Statement::Reference => TreeNode::new("Reference"),
            Statement::Pointer => TreeNode::new("Pointer"),
            Statement::Void => TreeNode::new("Void"),
            Statement::NOP => TreeNode::new("NOP"),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Program { .. } => write!(f, "Program"),
            Statement::Use { .. } => write!(f, "Use"),
            Statement::Block { .. } => write!(f, "Block"),
            Statement::Fn { .. } => write!(f, "Fn"),
            Statement::FnCall { .. } => write!(f, "FnCall"),
            Statement::If { .. } => write!(f, "If"),
            Statement::Return { .. } => write!(f, "Return"),
            Statement::Postfix { .. } => write!(f, "Postfix"),
            Statement::Panic { .. } => write!(f, "Panic"),
            Statement::Assert { .. } => write!(f, "Assert"),
            Statement::Declaration { .. } => write!(f, "Declaration"),
            Statement::Assignment { .. } => write!(f, "Assignment"),
            Statement::PropertyAccess { .. } => write!(f, "PropertyAccess"),
            Statement::ArrayAccess { .. } => write!(f, "ArrayAccess"),
            Statement::While { .. } => write!(f, "While"),
            Statement::For { .. } => write!(f, "For"),
            Statement::Loop { .. } => write!(f, "Loop"),
            Statement::Break => write!(f, "Break"),
            Statement::Continue => write!(f, "Continue"),
            Statement::Type { .. } => write!(f, "Type"),
            Statement::ArrayType { .. } => write!(f, "ArrayType"),
            Statement::Identifier { .. } => write!(f, "Identifier"),
            Statement::StringLiteral { .. } => write!(f, "StringLiteral"),
            Statement::NumberLiteral { .. } => write!(f, "NumberLiteral"),
            Statement::BinaryLiteral { .. } => write!(f, "BinaryLiteral"),
            Statement::HexLiteral { .. } => write!(f, "HexLiteral"),
            Statement::CharLiteral { .. } => write!(f, "CharLiteral"),
            Statement::BoolLiteral { .. } => write!(f, "BoolLiteral"),
            Statement::Reference => write!(f, "Reference"),
            Statement::Pointer => write!(f, "Pointer"),
            Statement::NOP => write!(f, "NOP"),
            Statement::Void => write!(f, "Void"),
        }
    }
}