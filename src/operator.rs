use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operator {
    Add,       // +   (binary)
    Sub,       // -   (binary & unary)
    Mul,       // *   (binary)
    Div,       // /   (binary)
    Mod,       // %   (binary)
    Xor,       // ^   (binary)
    And,       // &   (binary)
    Or,        // |   (binary)
    Assign,    // =   (binary)
    Eq,        // ==  (binary)
    Neq,       // !=  (binary)
    Lt,        // <   (binary)
    Lte,       // <=  (binary)
    Gt,        // >   (binary)
    Gte,       // >=  (binary)
    BoolAnd,   // &&  (binary)
    BoolOr,    // ||  (binary)
    Not,       // !   (unary)
    MulAssign, // *=  (binary)
    DivAssign, // /=  (binary)
    ModAssign, // %=  (binary)
    AddAssign, // +=  (binary)
    SubAssign, // -=  (binary)
    XorAssign, // ^=  (binary)
    AndAssign, // &=  (binary)
    OrAssign,  // |=  (binary)
    Shl,       // <<  (binary)
    Shr,       // >>  (binary)
    ShlAssign, // <<= (binary)
    ShrAssign, // >>= (binary)
    Shlu,      // <<< (binary)
    Shru,      // >>> (binary)
    Move,      // ->  (binary)
    Inc,       // ++  (unary)
    Dec,       // --  (unary)
    Right,     // =>  (binary)
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
        }
    }
}