use rust_decimal::Decimal;
use std::fmt::Display;

use super::tokenizer::Precedence;
use super::tokenizer::Token;

//EXPR = <Number> | <CellRef> | (EXPR) | -EXPR | EXPR + EXPR | EXPR - EXPR | EXPR * EXPR | EXPR / EXPR | <Symbol>(EXPR,...)

#[derive(Debug, PartialEq)]
pub enum Node {
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Parentheses(Box<Node>),
    UnaryMinus(Box<Node>),
    Number(Decimal),
    Cell(u32, u32),
    Function(String, Vec<Box<Node>>),
    Comment(String),
}

fn write_nodes(
    left: &Node,
    right: &Node,
    operator: &str,
    operator_precedence: Precedence,
) -> String {
    let left_precedence = left.precedence();
    let right_precedence = right.precedence();
    let mut strings = Vec::with_capacity(2);
    if left_precedence < operator_precedence {
        strings.push(format!("({})", *left));
    } else {
        strings.push(format!("{}", *left));
    }
    if right_precedence < operator_precedence {
        strings.push(format!("({})", *right));
    } else {
        strings.push(format!("{}", *right));
    }
    strings.join(operator)
}

fn write_cell(col: u32, row: u32) -> String {
    const ASCIIA: u32 = 'A' as u32;
    const BASE: u32 = 'Z' as u32 - 'A' as u32 + 1;

    let mut result = vec![];
    let mut num = col;
    loop {
        result.push(char::from_u32(num % BASE + ASCIIA).unwrap());
        num /= BASE;
        if num == 0 {
            break;
        }
        num -= 1;
    }
    format!("{}{}", result.iter().rev().collect::<String>(), row + 1)
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Add(ref left, ref right) => {
                write!(f, "{}", write_nodes(left, right, "+", self.precedence()))
            }
            Node::Sub(ref left, ref right) => {
                write!(f, "{}", write_nodes(left, right, "-", self.precedence()))
            }
            Node::Mul(ref left, ref right) => {
                write!(f, "{}", write_nodes(left, right, "*", self.precedence()))
            }
            Node::Div(ref left, ref right) => {
                write!(f, "{}", write_nodes(left, right, "/", self.precedence()))
            }
            Node::Parentheses(ref node) => write!(f, "({})", *node),
            Node::UnaryMinus(ref node) => match (**node).precedence() {
                Precedence::Binary(_) => write!(f, "-({})", *node),
                Precedence::Unary => write!(f, "-{}", *node),
            },
            Node::Number(n) => write!(f, "{}", n),
            Node::Cell(col, row) => write!(f, "{}", write_cell(*col, *row)),
            Node::Function(ref name, ref args) => write!(
                f,
                "{}({})",
                name,
                args.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Node::Comment(ref comment) => write!(f, "'{}", comment),
        }
    }
}

impl Node {
    fn join_with_token(left: Box<Node>, token: Token, right: Box<Node>) -> Box<Self> {
        match token {
            Token::Plus => Node::Add(left, right).boxed(),
            Token::Minus => Node::Sub(left, right).boxed(),
            Token::Mul => Node::Mul(left, right).boxed(),
            Token::Div => Node::Div(left, right).boxed(),
            _ => panic!("Should never hapen"),
        }
    }
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    pub fn attach(self: Box<Self>, token: Token, node: Box<Node>) -> Box<Self> {
        if token.precedence() > node.precedence() {
            match *node {
                Node::Add(left, right) => {
                    Node::Add(Node::join_with_token(self, token, left), right).boxed()
                }
                Node::Sub(left, right) => {
                    Node::Sub(Node::join_with_token(self, token, left), right).boxed()
                }
                Node::Mul(left, right) => {
                    Node::Mul(Node::join_with_token(self, token, left), right).boxed()
                }
                Node::Div(left, right) => {
                    Node::Div(Node::join_with_token(self, token, left), right).boxed()
                }
                _ => panic!("Should never hapen"),
            }
        } else {
            Node::join_with_token(self, token, node)
        }
    }
    fn precedence(&self) -> Precedence {
        match self {
            Node::Add(_, _) => Token::Plus.precedence(),
            Node::Sub(_, _) => Token::Minus.precedence(),
            Node::Mul(_, _) => Token::Mul.precedence(),
            Node::Div(_, _) => Token::Div.precedence(),
            _ => Precedence::Unary,
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::write_cell;
    use super::Node;
    use super::Token;

    fn cell(col: u32, row: u32) -> Box<Node> {
        Node::Cell(col, row).boxed()
    }

    fn number(n: i64) -> Box<Node> {
        Node::Number(Decimal::new(n, 0)).boxed()
    }

    #[test]
    fn write_cell1() {
        let res = write_cell(0, 0);
        let expected = "A1".to_string();
        assert_eq!(res, expected);
    }
    #[test]
    fn write_cell2() {
        let res = write_cell(25, 5);
        let expected = "Z6".to_string();
        assert_eq!(res, expected);
    }
    #[test]
    fn write_cell3() {
        let res = write_cell(26, 9);
        let expected = "AA10".to_string();
        assert_eq!(res, expected);
    }
    #[test]
    fn node_to_string1() {
        let node = Node::Add(
            Node::Sub(number(1), number(2)).boxed(),
            Node::Mul(number(3), number(4)).boxed(),
        )
        .boxed();
        let expected = "1-2+3*4".to_string();
        let res = node.to_string();
        assert_eq!(res, expected);
    }
    #[test]
    fn node_to_string2() {
        let node = Node::Div(
            Node::Sub(number(1), number(2)).boxed(),
            Node::Mul(number(3), number(4)).boxed(),
        )
        .boxed();
        let expected = "(1-2)/3*4".to_string();
        let res = node.to_string();
        assert_eq!(res, expected);
    }
    #[test]
    fn node_attach_plus_cell() {
        let left_node = cell(0, 0);
        let right_node = cell(1, 0);
        let expected = Node::Add(cell(0, 0), cell(1, 0)).boxed();
        let res = left_node.attach(Token::Plus, right_node);
        assert_eq!(res, expected);
    }
    #[test]
    fn node_attach_plus_add() {
        let left_node = cell(0, 0);
        let right_node = Node::Add(number(1), number(2)).boxed();
        let expected = Node::Add(cell(0, 0), Node::Add(number(1), number(2)).boxed()).boxed();
        let res = left_node.attach(Token::Plus, right_node);
        assert_eq!(res, expected);
    }
    #[test]
    fn node_attach_plus_mul() {
        let left_node = cell(0, 0);
        let right_node = Node::Mul(number(1), number(2)).boxed();
        let expected = Node::Add(cell(0, 0), Node::Mul(number(1), number(2)).boxed()).boxed();
        let res = left_node.attach(Token::Plus, right_node);
        assert_eq!(res, expected);
    }
    #[test]
    fn node_attach_mul_add() {
        let left_node = cell(0, 0);
        let right_node = Node::Add(number(1), number(2)).boxed();
        let expected = Node::Add(Node::Mul(cell(0, 0), number(1)).boxed(), number(2)).boxed();
        let res = left_node.attach(Token::Mul, right_node);
        assert_eq!(res, expected);
    }
    #[test]
    fn node_attach_mul_mul() {
        let left_node = cell(0, 0);
        let right_node = Node::Mul(number(1), number(2)).boxed();
        let expected = Node::Mul(cell(0, 0), Node::Mul(number(1), number(2)).boxed()).boxed();
        let res = left_node.attach(Token::Mul, right_node);
        assert_eq!(res, expected);
    }
}
