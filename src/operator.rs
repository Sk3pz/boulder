use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Operator {
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /
    Mod,       // %
    Xor,       // ^
    And,       // &
    Or,        // |
    Assign,    // =
    Eq,        // ==
    Neq,       // !=
    Lt,        // <
    Lte,       // <=
    Gt,        // >
    Gte,       // >=
    BoolAnd,   // &&
    BoolOr,    // ||
    Not,       // !
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
    AddAssign, // +=
    SubAssign, // -=
    XorAssign, // ^=
    AndAssign, // &=
    OrAssign,  // |=
    Shl,       // <<
    Shr,       // >>
    ShlAssign, // <<=
    ShrAssign, // >>=
    Shlu,      // <<<
    Shru,      // >>>
    Move,      // ->
    Inc,       // ++
    Dec,       // --
    Right,     // =>
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mul => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Mod => write!(f, "%"),
            Operator::Xor => write!(f, "^"),
            Operator::And => write!(f, "&"),
            Operator::Or => write!(f, "|"),
            Operator::Assign => write!(f, "="),
            Operator::Eq => write!(f, "=="),
            Operator::Neq => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::Lte => write!(f, "<="),
            Operator::Gt => write!(f, ">"),
            Operator::Gte => write!(f, ">="),
            Operator::BoolAnd => write!(f, "&&"),
            Operator::BoolOr => write!(f, "||"),
            Operator::Not => write!(f, "!"),
            Operator::MulAssign => write!(f, "*="),
            Operator::DivAssign => write!(f, "/="),
            Operator::ModAssign => write!(f, "%="),
            Operator::AddAssign => write!(f, "+="),
            Operator::SubAssign => write!(f, "-="),
            Operator::XorAssign => write!(f, "^="),
            Operator::AndAssign => write!(f, "&="),
            Operator::OrAssign => write!(f, "|="),
            Operator::Shl => write!(f, "<<"),
            Operator::Shr => write!(f, ">>"),
            Operator::ShlAssign => write!(f, "<<="),
            Operator::ShrAssign => write!(f, ">>="),
            Operator::Shlu => write!(f, "<<<"),
            Operator::Shru => write!(f, ">>>"),
            Operator::Move => write!(f, "->"),
            Operator::Inc => write!(f, "++"),
            Operator::Dec => write!(f, "--"),
            Operator::Right => write!(f, "=>"),
        }
    }
}