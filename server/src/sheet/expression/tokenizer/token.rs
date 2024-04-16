use rust_decimal::Decimal;
use std::fmt::Display;

use super::Precedence;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Comment(String),
    Cell(String, String),
    Number(Decimal),
    Symbol(String),
    LPar,
    RPar,
    Plus,
    Minus,
    Mul,
    Div,
    Comma,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Token::Comment(ref comment) => write!(f, "{}", comment),
            Token::Cell(ref col, ref row) => write!(f, "{}{}", col, row),
            Token::Number(ref number) => write!(f, "{}", number),
            Token::Symbol(ref identifier) => write!(f, "{}", identifier),
            Token::LPar => write!(f, "("),
            Token::RPar => write!(f, ")"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Comma => write!(f, ","),
        }
    }
}

impl Token {
    pub fn precedence(&self) -> Precedence {
        match *self {
            Token::Plus | Token::Minus => Precedence::Binary(1),
            Token::Div | Token::Mul => Precedence::Binary(2),
            _ => Precedence::Unary,
        }
    }
}
