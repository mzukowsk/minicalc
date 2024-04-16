mod error_message;
mod precedence;
mod token;
mod token_info;
mod tokenize;

pub use self::precedence::Precedence;
pub use self::token::Token;

use self::token_info::TokenInfo;

pub struct Tokenizer {
    expression: String,
    tokens: Vec<TokenInfo>,
    position: usize,
}

impl Tokenizer {
    pub fn from(expression: &str) -> Result<Self, String> {
        let tokens = tokenize::tokenize(expression)?;
        Ok(Tokenizer {
            expression: expression.to_string(),
            tokens,
            position: 0,
        })
    }
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position).map(|ti| &ti.token)
    }
    pub fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
    pub fn error_message(&self, message: &str) -> String {
        let token_info = self.tokens.get(self.position);
        let (position, length) = match token_info {
            None => (self.expression.chars().count(), 1usize),
            Some(TokenInfo {
                position, length, ..
            }) => (*position, *length),
        };
        error_message::error_message(&self.expression, position, length, message)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::Token;
    use super::TokenInfo;
    use super::Tokenizer;

    fn token(number: usize) -> TokenInfo {
        TokenInfo::new(Token::Number(Decimal::new(number as i64, 0)), number, 1)
    }

    #[test]
    fn tokenizer_peek() {
        let tokens = vec![token(1), token(2)];
        let mut tokenizer = Tokenizer {
            expression: "".to_string(),
            tokens,
            position: 0,
        };

        let peek1 = tokenizer.peek();
        assert_eq!(*peek1.unwrap(), token(1).token);
        let peek2 = tokenizer.peek();
        assert_eq!(*peek2.unwrap(), token(1).token);
        tokenizer.advance();

        let peek3 = tokenizer.peek();
        assert_eq!(*peek3.unwrap(), token(2).token);
        let peek4 = tokenizer.peek();
        assert_eq!(*peek4.unwrap(), token(2).token);
        tokenizer.advance();

        let peek5 = tokenizer.peek();
        assert_eq!(peek5, None);
        tokenizer.advance();
    }
    #[test]
    fn tokenizer_from_empty_string() {
        let tokenizer = Tokenizer::from("").unwrap();
        assert_eq!(tokenizer.peek(), None);
    }
}
