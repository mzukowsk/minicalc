use lazy_static::lazy_static;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::error_message;
use super::token::Token;
use super::token_info::TokenInfo;

const ERR_UNKNOWN_CHARACTER: &str = "Unknown character";

pub fn tokenize(expression: &str) -> Result<Vec<TokenInfo>, String> {
    if expression.is_empty() {
        return Ok(vec![]);
    }

    let length = expression.chars().count();

    if let Some(comment) = expression.strip_prefix('\'') {
        return Ok(vec![TokenInfo::new(
            Token::Comment(comment.to_string()),
            1,
            length - 1,
        )]);
    }

    if length < expression.len() {
        for (position, c) in (expression.chars()).enumerate() {
            if c.len_utf8() > 1 {
                return Err(error_message::error_message(
                    expression,
                    position,
                    1,
                    ERR_UNKNOWN_CHARACTER,
                ));
            }
        }
    }

    let mut result = Vec::new();
    let mut expr = expression.trim_start();

    while !expr.is_empty() {
        let position = length - expr.len();
        lazy_static! {
            static ref RE_CELLREF: Regex =
                Regex::new(r"^(\$?([a-zA-Z]+)\$?([1-9][0-9]*))(?:\W|$)").unwrap();
        }
        if let Some(c) = RE_CELLREF.captures(expr) {
            result.push(TokenInfo::new(
                Token::Cell(c[2].to_string(), c[3].to_string()),
                position,
                c[1].len(),
            ));
            expr = &expr[c[1].len()..].trim_start();
            continue;
        }
        lazy_static! {
            static ref RE_NUMBER: Regex = Regex::new(r"^\d+(?:\.\d*)?|^\.\d+").unwrap();
        }
        if let Some(c) = RE_NUMBER.captures(expr) {
            result.push(TokenInfo::new(
                Token::Number(Decimal::from_str(&c[0]).unwrap()),
                position,
                c[0].len(),
            ));
            expr = &expr[c[0].len()..].trim_start();
            continue;
        }
        lazy_static! {
            static ref RE_SYMBOL: Regex = Regex::new(r"^\w+").unwrap();
        }
        if let Some(c) = RE_SYMBOL.captures(expr) {
            result.push(TokenInfo::new(
                Token::Symbol(c[0].to_string()),
                position,
                c[0].len(),
            ));
            expr = &expr[c[0].len()..].trim_start();
            continue;
        }
        if let Some(c) = expr.chars().next() {
            match c {
                '(' => result.push(TokenInfo::new(Token::LPar, position, 1)),
                ')' => result.push(TokenInfo::new(Token::RPar, position, 1)),
                '+' => result.push(TokenInfo::new(Token::Plus, position, 1)),
                '-' => result.push(TokenInfo::new(Token::Minus, position, 1)),
                '*' => result.push(TokenInfo::new(Token::Mul, position, 1)),
                '/' => result.push(TokenInfo::new(Token::Div, position, 1)),
                ',' => result.push(TokenInfo::new(Token::Comma, position, 1)),
                _ => {
                    return Err(error_message::error_message(
                        expression,
                        position,
                        1,
                        ERR_UNKNOWN_CHARACTER,
                    ))
                }
            }
            expr = &expr[1..].trim_start();
            continue;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::tokenize;
    use super::Decimal;
    use super::Token;
    use super::TokenInfo;

    fn assert_eq(actual: Vec<TokenInfo>, expected: Vec<TokenInfo>) {
        assert!(
            (actual.len() == expected.len())
                && !(0..actual.len()).any(|i| actual[i] != expected[i])
        );
    }

    #[test]
    fn tokenize_basic() {
        let res = tokenize("2.1+3 * 4").unwrap();
        let expected = vec![
            TokenInfo::new(Token::Number(Decimal::new(21, 1)), 0, 3),
            TokenInfo::new(Token::Plus, 3, 1),
            TokenInfo::new(Token::Number(Decimal::new(3, 0)), 4, 1),
            TokenInfo::new(Token::Mul, 6, 1),
            TokenInfo::new(Token::Number(Decimal::new(4, 0)), 8, 1),
        ];
        assert_eq(res, expected);
    }
    #[test]
    fn tokenize_cell_refs() {
        let res = tokenize("$aab1 / c$12 - a1").unwrap();
        let expected = vec![
            TokenInfo::new(Token::Cell("aab".to_string(), "1".to_string()), 0, 5),
            TokenInfo::new(Token::Div, 6, 1),
            TokenInfo::new(Token::Cell("c".to_string(), "12".to_string()), 8, 4),
            TokenInfo::new(Token::Minus, 13, 1),
            TokenInfo::new(Token::Cell("a".to_string(), "1".to_string()), 15, 2),
        ];
        assert_eq!(res, expected);
    }
    #[test]
    fn tokenize_symbols() {
        let res = tokenize("aab / c").unwrap();
        let expected = vec![
            TokenInfo::new(Token::Symbol("aab".to_string()), 0, 3),
            TokenInfo::new(Token::Div, 4, 1),
            TokenInfo::new(Token::Symbol("c".to_string()), 6, 1),
        ];
        assert_eq(res, expected);
    }
    #[test]
    fn tokenize_parentheses() {
        let res = tokenize("(2.1 + a) * 3").unwrap();
        let expected = vec![
            TokenInfo::new(Token::LPar, 0, 1),
            TokenInfo::new(Token::Number(Decimal::new(21, 1)), 1, 3),
            TokenInfo::new(Token::Plus, 5, 1),
            TokenInfo::new(Token::Symbol("a".to_string()), 7, 1),
            TokenInfo::new(Token::RPar, 8, 1),
            TokenInfo::new(Token::Mul, 10, 1),
            TokenInfo::new(Token::Number(Decimal::new(3, 0)), 12, 1),
        ];
        assert_eq(res, expected);
    }
    #[test]
    fn tokenize_function() {
        let res = tokenize("1+sqrt(a, b + 1, -1)").unwrap();
        let expected = vec![
            TokenInfo::new(Token::Number(Decimal::new(1, 0)), 0, 1),
            TokenInfo::new(Token::Plus, 1, 1),
            TokenInfo::new(Token::Symbol("sqrt".to_string()), 2, 4),
            TokenInfo::new(Token::LPar, 6, 1),
            TokenInfo::new(Token::Symbol("a".to_string()), 7, 1),
            TokenInfo::new(Token::Comma, 8, 1),
            TokenInfo::new(Token::Symbol("b".to_string()), 10, 1),
            TokenInfo::new(Token::Plus, 12, 1),
            TokenInfo::new(Token::Number(Decimal::new(1, 0)), 14, 1),
            TokenInfo::new(Token::Comma, 15, 1),
            TokenInfo::new(Token::Minus, 17, 1),
            TokenInfo::new(Token::Number(Decimal::new(1, 0)), 18, 1),
            TokenInfo::new(Token::RPar, 19, 1),
        ];
        assert_eq(res, expected);
    }
    #[test]
    fn tokenize_empty_string() {
        let res = tokenize("").unwrap();
        let expected = vec![];
        assert_eq!(res, expected);
    }
    #[test]
    fn tokenize_unknown_character() {
        let res = tokenize("a ^ c");
        assert_eq!(res.unwrap_err(), "a ^ c\n  ^  \nUnknown character");
    }
}
