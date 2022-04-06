use std::fmt::Display;
use cli_tree::TreeNode;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operator {
    Add,       // +   (binary)         0
    Sub,       // -   (binary & unary) 0
    Mul,       // *   (binary)         1
    Div,       // /   (binary)         1
    Mod,       // %   (binary)         1
    Xor,       // ^   (binary)         2
    And,       // &   (binary)         2
    Or,        // |   (binary)         2
    Not,       // !   (binary & unary) 2
    Assign,    // =   (binary)         -
    Eq,        // ==  (binary)         2
    Neq,       // !=  (binary)         2
    Lt,        // <   (binary)         0
    Lte,       // <=  (binary)         1
    Gt,        // >   (binary)         0
    Gte,       // >=  (binary)         1
    BoolAnd,   // &&  (binary)         1
    BoolOr,    // ||  (binary)         1
    MulAssign, // *=  (binary)         -
    DivAssign, // /=  (binary)         -
    ModAssign, // %=  (binary)         -
    AddAssign, // +=  (binary)         -
    SubAssign, // -=  (binary)         -
    XorAssign, // ^=  (binary)         -
    AndAssign, // &=  (binary)         -
    OrAssign,  // |=  (binary)         -
    Shl,       // <<  (binary)         2
    Shr,       // >>  (binary)         2
    ShlAssign, // <<= (binary)         -
    ShrAssign, // >>= (binary)         -
    Shlu,      // <<< (binary)         2
    Shru,      // >>> (binary)         2
    Move,      // ->  (binary)         -
    Inc,       // ++  (unary)          0
    Dec,       // --  (unary)          0
    Right,     // =>  (binary)         -
    Range,     // ..  (binary)         -
    IRange,    // ..= (binary)         -
}

impl Operator {
    pub fn as_treenode(&self) -> TreeNode {
        TreeNode::new(self.to_string())
    }

    // returns a number from 0 to 2, with higher numbers being higher precedence
    pub fn precedence(&self) -> Option<u8> {
        match self {
            Operator::Add | Operator::Sub | Operator::Gt | Operator::Lt => Some(0),
            Operator::Mul | Operator::Div | Operator::Mod |
            Operator::Gte | Operator::Lte | Operator::BoolAnd | Operator::BoolOr => Some(1),
            Operator::Xor | Operator::And | Operator::Or | Operator::Not |
            Operator::Shl | Operator::Shr | Operator::Shlu | Operator::Shru |
            Operator::Eq  | Operator::Neq => Some(2),
            _ => None
        }
    }

    pub fn is_boolean(&self) -> bool {
        match self {
            Operator::Eq | Operator::Neq | Operator::Lt | Operator::Gt | Operator::Lte |
            Operator::Gte | Operator::BoolAnd | Operator::BoolOr => true,
            _ => false
        }
    }

    pub fn as_raw(&self) -> String {
        match self {
            Operator::Add => String::from("+"),
            Operator::Sub => String::from("-"),
            Operator::Mul => String::from("*"),
            Operator::Div => String::from("+"),
            Operator::Mod => String::from("%"),
            Operator::Xor => String::from("^"),
            Operator::And => String::from("&"),
            Operator::Or => String::from("|"),
            Operator::Assign => String::from("="),
            Operator::Eq => String::from("=="),
            Operator::Neq => String::from("!="),
            Operator::Lt => String::from("<"),
            Operator::Lte => String::from("<="),
            Operator::Gt => String::from(">"),
            Operator::Gte => String::from(">="),
            Operator::BoolAnd => String::from("&&"),
            Operator::BoolOr => String::from("||"),
            Operator::Not => String::from("!"),
            Operator::MulAssign => String::from("*="),
            Operator::DivAssign => String::from("/="),
            Operator::ModAssign => String::from("%="),
            Operator::AddAssign => String::from("+="),
            Operator::SubAssign => String::from("-="),
            Operator::XorAssign => String::from("^="),
            Operator::AndAssign => String::from("&="),
            Operator::OrAssign => String::from("|="),
            Operator::Shl => String::from("<<"),
            Operator::Shr => String::from(">>"),
            Operator::ShlAssign => String::from("<<="),
            Operator::ShrAssign => String::from(">>="),
            Operator::Shlu => String::from("<<<"),
            Operator::Shru => String::from(">>>"),
            Operator::Move => String::from("->"),
            Operator::Inc => String::from("++"),
            Operator::Dec => String::from("--"),
            Operator::Right => String::from("=>"),
            Operator::Range => String::from(".."),
            Operator::IRange => String::from("..+"),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "Add `+`"),
            Operator::Sub => write!(f, "Sub `-`"),
            Operator::Mul => write!(f, "Mul `*`"),
            Operator::Div => write!(f, "Div `/`"),
            Operator::Mod => write!(f, "Mod `%`"),
            Operator::Xor => write!(f, "XOr `^`"),
            Operator::And => write!(f, "And `&`"),
            Operator::Or => write!(f, "Or `|`"),
            Operator::Assign => write!(f, "Assign `=`"),
            Operator::Eq => write!(f, "Eq `==`"),
            Operator::Neq => write!(f, "NEq`!=`"),
            Operator::Lt => write!(f, "Lt `<`"),
            Operator::Lte => write!(f, "Lte `<=`"),
            Operator::Gt => write!(f, "Gt `>`"),
            Operator::Gte => write!(f, "Gte `>=`"),
            Operator::BoolAnd => write!(f, "BoolAnd `&&`"),
            Operator::BoolOr => write!(f, "BoolOr `||`"),
            Operator::Not => write!(f, "Not `!`"),
            Operator::MulAssign => write!(f, "MulAssign `*=`"),
            Operator::DivAssign => write!(f, "DivAssign `/=`"),
            Operator::ModAssign => write!(f, "ModAssign `%=`"),
            Operator::AddAssign => write!(f, "AddAssign `+=`"),
            Operator::SubAssign => write!(f, "SubAssign `-=`"),
            Operator::XorAssign => write!(f, "XorAssign `^=`"),
            Operator::AndAssign => write!(f, "AndAssign `&=`"),
            Operator::OrAssign => write!(f, "OrAssign `|=`"),
            Operator::Shl => write!(f, "Shl `<<`"),
            Operator::Shr => write!(f, "Shr `>>`"),
            Operator::ShlAssign => write!(f, "ShlAssign `<<=`"),
            Operator::ShrAssign => write!(f, "ShrAssign `>>=`"),
            Operator::Shlu => write!(f, "Shlu `<<<`"),
            Operator::Shru => write!(f, "Shru `>>>`"),
            Operator::Move => write!(f, "Move `->`"),
            Operator::Inc => write!(f, "Inc `++`"),
            Operator::Dec => write!(f, "Dec `--`"),
            Operator::Right => write!(f, "Right `=>`"),
            Operator::Range => write!(f, "Range `..`"),
            Operator::IRange => write!(f, "InclusiveRange `..=`"),
        }
    }
}

impl Into<TreeNode> for Operator {
    fn into(self) -> TreeNode {
        TreeNode::new(self.to_string())
    }
}