use super::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub position: usize,
    pub length: usize,
}

impl TokenInfo {
    pub fn new(token: Token, position: usize, length: usize) -> Self {
        Self {
            token,
            position,
            length,
        }
    }
}
