use super::node::Node;
use super::tokenizer::Precedence;
use super::tokenizer::Token;
use super::tokenizer::Tokenizer;

//EXPR = <Number> | <CellRef> | (EXPR) | -EXPR | EXPR + EXPR | EXPR - EXPR | EXPR * EXPR | EXPR / EXPR | <Symbol>(EXPR,...)

const ERR_UNEXPECTED_TOKEN: &str = "Unexpected token";
const ERR_UNEXPECTED_END_OF_EXPRESSION: &str = "Unexpected end of expression";
const ERR_EXPECTED_OPENING_PARENTHESIS: &str = "Expected opening parenthesis";
const ERR_EXPECTED_CLOSING_PARENTHESIS: &str = "Expected closing parenthesis";

fn decode_cell_col(col: &str) -> u32 {
    const ASCIIA: u32 = 'A' as u32;
    const BASE: u32 = 'Z' as u32 - 'A' as u32 + 1;

    col.chars().fold(0, |acc, c| {
        acc * BASE + c.to_ascii_uppercase() as u32 - ASCIIA + 1
    }) - 1
}

fn decode_cell_row(row: &str) -> u32 {
    row.parse::<u32>().unwrap() - 1
}

fn parse_expression(tokenizer: &mut Tokenizer, greedy: bool) -> Result<Box<Node>, String> {
    let node = match tokenizer.peek() {
        None => return Err(tokenizer.error_message(ERR_UNEXPECTED_END_OF_EXPRESSION)),
        Some(Token::Comment(comment)) => {
            let comment = comment.clone();
            tokenizer.advance();
            Node::Comment(comment).boxed()
        }
        Some(Token::LPar) => {
            tokenizer.advance();
            let inner_expr = parse_expression(tokenizer, true)?;
            if let Some(Token::RPar) = tokenizer.peek() {
                tokenizer.advance();
                Node::Parentheses(inner_expr).boxed()
            } else {
                return Err(tokenizer.error_message(ERR_EXPECTED_CLOSING_PARENTHESIS));
            }
        }
        Some(Token::RPar) | Some(Token::Comma) | Some(Token::Plus) | Some(Token::Mul)
        | Some(Token::Div) => {
            return Err(tokenizer.error_message(ERR_UNEXPECTED_TOKEN));
        }
        Some(Token::Minus) => {
            tokenizer.advance();
            match tokenizer.peek() {
                Some(Token::Minus) => return Err(tokenizer.error_message(ERR_UNEXPECTED_TOKEN)),
                _ => Node::UnaryMinus(parse_expression(tokenizer, false)?).boxed(),
            }
        }
        Some(Token::Symbol(identifier)) => {
            let identifier = identifier.to_ascii_lowercase();
            tokenizer.advance();
            if tokenizer.peek() == Some(&Token::LPar) {
                tokenizer.advance();
                let mut args = Vec::new();
                if tokenizer.peek() != Some(&Token::RPar) {
                    loop {
                        args.push(parse_expression(tokenizer, true)?);
                        match tokenizer.peek() {
                            Some(Token::Comma) => {
                                tokenizer.advance();
                                continue;
                            }
                            Some(Token::RPar) => break,
                            _ => return Err(tokenizer.error_message(ERR_UNEXPECTED_TOKEN)),
                        }
                    }
                }
                tokenizer.advance();
                Node::Function(identifier.clone(), args).boxed()
            } else {
                return Err(tokenizer.error_message(ERR_EXPECTED_OPENING_PARENTHESIS));
            }
        }
        Some(Token::Number(number)) => {
            let number = *number;
            tokenizer.advance();
            Node::Number(number).boxed()
        }
        Some(Token::Cell(col, row)) => {
            let col = decode_cell_col(col);
            let row = decode_cell_row(row);
            tokenizer.advance();
            Node::Cell(col, row).boxed()
        }
    };
    if greedy {
        match tokenizer.peek() {
            Some(token) if matches!(token.precedence(), Precedence::Binary(_)) => {
                let token = token.clone();
                tokenizer.advance();
                let right = parse_expression(tokenizer, true)?;
                Ok(node.attach(token, right))
            }
            _ => Ok(node),
        }
    } else {
        Ok(node)
    }
}

pub fn parse(tokenizer: &mut Tokenizer) -> Result<Box<Node>, String> {
    let res = parse_expression(tokenizer, true)?;
    if tokenizer.peek().is_none() {
        Ok(res)
    } else {
        Err(tokenizer.error_message(ERR_UNEXPECTED_TOKEN))
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::decode_cell_col;
    use super::decode_cell_row;
    use super::parse;
    use super::Node;
    use super::Tokenizer;

    fn test_parse(e: &str) -> Result<Box<Node>, String> {
        parse(&mut Tokenizer::from(e)?)
    }

    fn cell(col: u32, row: u32) -> Box<Node> {
        Node::Cell(col, row).boxed()
    }

    fn number(n: i64, s: u32) -> Box<Node> {
        Node::Number(Decimal::new(n, s)).boxed()
    }

    #[test]
    fn decode_cell_col1() {
        let res = decode_cell_col("A");
        let expected = 0;
        assert_eq!(res, expected);
    }
    #[test]
    fn decode_cell_col2() {
        let res = decode_cell_col("Z");
        let expected = 25;
        assert_eq!(res, expected);
    }
    #[test]
    fn decode_cell_col3() {
        let res = decode_cell_col("AA");
        let expected = 26;
        assert_eq!(res, expected);
    }
    #[test]
    fn decode_cell_row1() {
        let res = decode_cell_row("1");
        let expected = 0;
        assert_eq!(res, expected);
    }
    #[test]
    fn decode_cell_row2() {
        let res = decode_cell_row("25");
        let expected = 24;
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_number() {
        let res = test_parse("2.1").unwrap();
        let expected = number(21, 1);
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_cell() {
        let res = test_parse("B2").unwrap();
        let expected = cell(1, 1);
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression1() {
        let res = test_parse("2.1 + a1 * 3").unwrap();
        let expected =
            Node::Add(number(21, 1), Node::Mul(cell(0, 0), number(3, 0)).boxed()).boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression2() {
        let res = test_parse("2.1 * a1 + 3").unwrap();
        let expected = Box::new(Node::Add(
            Node::Mul(number(21, 1), cell(0, 0)).boxed(),
            number(3, 0),
        ));
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression3() {
        let res = test_parse("(2.1 + a1) * 3").unwrap();
        let expected = Node::Mul(
            Node::Parentheses(Node::Add(number(21, 1), cell(0, 0)).boxed()).boxed(),
            number(3, 0),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression4() {
        let res = test_parse("-(2.1 + a1) * 3").unwrap();
        let expected = Node::Mul(
            Node::UnaryMinus(
                Node::Parentheses(Node::Add(number(21, 1), cell(0, 0)).boxed()).boxed(),
            )
            .boxed(),
            number(3, 0),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression5() {
        let res = test_parse("1+2*3-4").unwrap();
        let expected = Node::Add(
            number(1, 0),
            Node::Sub(Node::Mul(number(2, 0), number(3, 0)).boxed(), number(4, 0)).boxed(),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression6() {
        let res = test_parse("1+2-3+4").unwrap();
        let expected = Node::Add(
            number(1, 0),
            Node::Sub(number(2, 0), Node::Add(number(3, 0), number(4, 0)).boxed()).boxed(),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression7() {
        let res = test_parse("1*2+3/4").unwrap();
        let expected = Node::Add(
            Node::Mul(number(1, 0), number(2, 0)).boxed(),
            Node::Div(number(3, 0), number(4, 0)).boxed(),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_expression8() {
        let res = test_parse("-(1)*2+3").unwrap();
        let expected = Node::Add(
            Node::Mul(
                Node::UnaryMinus(Node::Parentheses(number(1, 0)).boxed()).boxed(),
                number(2, 0),
            )
            .boxed(),
            number(3, 0),
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_function1() {
        let res = test_parse("a(1+2,3)").unwrap();
        let expected = Node::Function(
            "a".to_string(),
            vec![Node::Add(number(1, 0), number(2, 0)).boxed(), number(3, 0)],
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_function2() {
        let res = test_parse("A(1+2,3)").unwrap();
        let expected = Node::Function(
            "a".to_string(),
            vec![Node::Add(number(1, 0), number(2, 0)).boxed(), number(3, 0)],
        )
        .boxed();
        assert_eq!(res, expected);
    }
    #[test]
    fn parse_error_expression1() {
        let res = test_parse("(a() + 1.0)) * 2");
        let expected = "(a() + 1.0)) * 2\n           ^    \nUnexpected token";
        assert_eq!(res.unwrap_err().to_string(), expected);
    }
    #[test]
    fn parse_error_expression2() {
        let res = test_parse("(a + 1.0)) * 2");
        let expected = "(a + 1.0)) * 2\n   ^          \nExpected opening parenthesis";
        assert_eq!(res.unwrap_err().to_string(), expected);
    }
    #[test]
    fn parse_unknown_character() {
        let res = test_parse("a ^ c");
        assert_eq!(
            res.unwrap_err().to_string(),
            "a ^ c\n  ^  \nUnknown character"
        );
    }
}
