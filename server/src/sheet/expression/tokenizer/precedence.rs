#[derive(Debug, PartialEq)]
pub enum Precedence {
    Unary,
    Binary(u32),
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Precedence::Binary(sp), Precedence::Binary(op)) => Some(sp.cmp(op)),
            _ => None,
        }
    }
}
