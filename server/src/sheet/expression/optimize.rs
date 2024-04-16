use rust_decimal::Decimal;

use super::node::Node;

const ERR_DIVISION_BY_0: &str = "Division by 0";

pub fn optimize(node: Box<Node>) -> Result<Box<Node>, String> {
    match *node {
        Node::Parentheses(inner) => {
            let inner = optimize(inner)?;
            Ok(inner)
        }
        Node::UnaryMinus(inner) => {
            let inner = optimize(inner)?;
            if let Node::Number(number) = *inner {
                Ok(Node::Number(-number).boxed())
            } else {
                Ok(Node::UnaryMinus(inner).boxed())
            }
        }
        Node::Add(left, right) => {
            let left = optimize(left)?;
            let right = optimize(right)?;
            if let Node::Number(right_number) = &*right {
                if *right_number == Decimal::ZERO {
                    Ok(left)
                } else if let Node::Number(left_number) = &*left {
                    Ok(Node::Number(left_number + right_number).boxed())
                } else {
                    match *left {
                        Node::Add(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Add(
                                    Node::Number(right_number + subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        Node::Sub(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Sub(
                                    Node::Number(right_number + subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        _ => Ok(Node::Add(right, left).boxed()),
                    }
                }
            } else if let Node::Number(left_number) = &*left {
                match *right {
                    _ if *left_number == Decimal::ZERO => Ok(right),
                    Node::Add(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Add(
                                Node::Number(left_number + subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    Node::Sub(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Sub(
                                Node::Number(left_number + subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    _ => Ok(Node::Add(left, right).boxed()),
                }
            } else {
                Ok(Node::Add(left, right).boxed())
            }
        }
        Node::Sub(left, right) => {
            let left = optimize(left)?;
            let right = optimize(right)?;
            if let Node::Number(right_number) = &*right {
                if *right_number == Decimal::ZERO {
                    Ok(left)
                } else if let Node::Number(left_number) = &*left {
                    Ok(Node::Number(left_number - right_number).boxed())
                } else {
                    match *left {
                        Node::Add(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Add(
                                    Node::Number(subleft_number - right_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        Node::Sub(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Sub(
                                    Node::Number(subleft_number - right_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        _ => Ok(Node::Add(Node::Number(-right_number).boxed(), left).boxed()),
                    }
                }
            } else if let Node::Number(left_number) = &*left {
                match *right {
                    Node::Add(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Sub(
                                Node::Number(left_number - subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    Node::Sub(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Add(
                                Node::Number(left_number - subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    _ if *left_number == Decimal::ZERO => Ok(Node::UnaryMinus(right).boxed()),
                    _ => Ok(Node::Sub(left, right).boxed()),
                }
            } else {
                Ok(Node::Sub(left, right).boxed())
            }
        }
        Node::Mul(left, right) => {
            let left = optimize(left)?;
            let right = optimize(right)?;
            if let Node::Number(right_number) = &*right {
                if *right_number == Decimal::ONE {
                    Ok(left)
                } else if let Node::Number(left_number) = &*left {
                    Ok(Node::Number(left_number * right_number).boxed())
                } else {
                    match *left {
                        Node::Mul(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Mul(
                                    Node::Number(right_number * subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        Node::Div(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Div(
                                    Node::Number(right_number * subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        _ => Ok(Node::Mul(right, left).boxed()),
                    }
                }
            } else if let Node::Number(left_number) = &*left {
                match *right {
                    _ if *left_number == Decimal::ONE => Ok(right),
                    Node::Mul(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Mul(
                                Node::Number(left_number * subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    Node::Div(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            Ok(Node::Div(
                                Node::Number(left_number * subleft_number).boxed(),
                                subright,
                            )
                            .boxed())
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    _ => Ok(Node::Mul(left, right).boxed()),
                }
            } else {
                Ok(Node::Mul(left, right).boxed())
            }
        }
        Node::Div(left, right) => {
            let left = optimize(left)?;
            let right = optimize(right)?;
            if let Node::Number(right_number) = &*right {
                if *right_number == Decimal::ZERO {
                    Err(ERR_DIVISION_BY_0.to_string())
                } else if *right_number == Decimal::ONE {
                    Ok(left)
                } else if let Node::Number(left_number) = &*left {
                    Ok(Node::Number(left_number / right_number).boxed())
                } else {
                    match *left {
                        Node::Mul(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Mul(
                                    Node::Number(subleft_number / right_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        Node::Div(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                            if let Node::Number(subleft_number) = *subleft {
                                Ok(Node::Div(
                                    Node::Number(subleft_number / right_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                panic!("Should never happen");
                            }
                        }
                        _ => Ok(Node::Mul(
                            Node::Number(Decimal::ONE / right_number).boxed(),
                            left,
                        )
                        .boxed()),
                    }
                }
            } else if let Node::Number(left_number) = &*left {
                match *right {
                    Node::Mul(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            if subleft_number != Decimal::ZERO {
                                Ok(Node::Div(
                                    Node::Number(left_number / subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                Err(ERR_DIVISION_BY_0.to_string())
                            }
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    Node::Div(subleft, subright) if matches!(*subleft, Node::Number(_)) => {
                        if let Node::Number(subleft_number) = *subleft {
                            if subleft_number != Decimal::ZERO {
                                Ok(Node::Mul(
                                    Node::Number(left_number / subleft_number).boxed(),
                                    subright,
                                )
                                .boxed())
                            } else {
                                Err(ERR_DIVISION_BY_0.to_string())
                            }
                        } else {
                            panic!("Should never happen");
                        }
                    }
                    _ => Ok(Node::Div(left, right).boxed()),
                }
            } else {
                Ok(Node::Div(left, right).boxed())
            }
        }
        Node::Function(name, params) => {
            let mut optimized_params = Vec::with_capacity(params.len());
            for param in params.into_iter() {
                let param = optimize(param)?;
                optimized_params.push(param);
            }
            Ok(Node::Function(name, optimized_params).boxed())
        }
        Node::Comment(_) | Node::Cell(_, _) | Node::Number(_) => Ok(node),
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::optimize;
    use super::Node;

    fn cell(col: u32, row: u32) -> Box<Node> {
        Node::Cell(col, row).boxed()
    }

    fn number(n: i64, s: u32) -> Box<Node> {
        Node::Number(Decimal::new(n, s)).boxed()
    }

    #[test]
    fn optimize_number() {
        let node = number(1, 0);
        let expected = number(1, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_cell() {
        let node = cell(0, 0);
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_parentheses() {
        let node = Node::Parentheses(number(1, 0)).boxed();
        let expected = number(1, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_unary_minus() {
        let node = Node::UnaryMinus(number(1, 0)).boxed();
        let expected = number(-1, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add() {
        let node = Node::Add(number(35, 1), number(15, 1)).boxed();
        let expected = number(5, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_move_number_to_left1() {
        let node = Node::Add(number(35, 1), cell(0, 0)).boxed();
        let expected = Node::Add(number(35, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_move_number_to_left2() {
        let node = Node::Add(cell(0, 0), number(35, 1)).boxed();
        let expected = Node::Add(number(35, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_nested_add1() {
        let node = Node::Add(Node::Add(number(35, 1), cell(0, 0)).boxed(), number(15, 1)).boxed();
        let expected = Node::Add(number(5, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_nested_add2() {
        let node = Node::Add(number(35, 1), Node::Add(number(15, 1), cell(0, 0)).boxed()).boxed();
        let expected = Node::Add(number(5, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_nested_sub1() {
        let node = Node::Add(Node::Sub(number(35, 1), cell(0, 0)).boxed(), number(15, 1)).boxed();
        let expected = Node::Sub(number(5, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_nested_sub2() {
        let node = Node::Add(number(35, 1), Node::Sub(number(15, 1), cell(0, 0)).boxed()).boxed();
        let expected = Node::Sub(number(5, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_left_zero() {
        let node = Node::Add(number(0, 0), cell(0, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_add_right_zero() {
        let node = Node::Add(cell(0, 0), number(0, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub() {
        let node = Node::Sub(number(35, 1), number(15, 1)).boxed();
        let expected = number(20, 1);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_move_number_to_left1() {
        let node = Node::Sub(number(35, 1), cell(0, 0)).boxed();
        let expected = Node::Sub(number(35, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_nested_sub1() {
        let node = Node::Sub(Node::Sub(number(35, 1), cell(0, 0)).boxed(), number(15, 1)).boxed();
        let expected = Node::Sub(number(20, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_nested_sub2() {
        let node = Node::Sub(number(35, 1), Node::Sub(number(15, 1), cell(0, 0)).boxed()).boxed();
        let expected = Node::Add(number(20, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_nested_add1() {
        let node = Node::Sub(Node::Add(number(35, 1), cell(0, 0)).boxed(), number(15, 1)).boxed();
        let expected = Node::Add(number(20, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_nested_add2() {
        let node = Node::Add(number(35, 1), Node::Sub(number(15, 1), cell(0, 0)).boxed()).boxed();
        let expected = Node::Sub(number(5, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_move_number_to_left2() {
        let node = Node::Sub(cell(0, 0), number(35, 1)).boxed();
        let expected = Node::Add(number(-35, 1).boxed(), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_left_zero() {
        let node = Node::Sub(number(0, 0), cell(0, 0)).boxed();
        let expected = Node::UnaryMinus(cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_sub_right_zero() {
        let node = Node::Sub(cell(0, 0), number(0, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul() {
        let node = Node::Mul(number(35, 1), number(15, 1)).boxed();
        let expected = number(525, 2).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_move_number_to_left1() {
        let node = Node::Mul(number(35, 1), cell(0, 0)).boxed();
        let expected = Node::Mul(number(35, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_move_number_to_left2() {
        let node = Node::Mul(cell(0, 0), number(35, 1)).boxed();
        let expected = Node::Mul(number(35, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_nested_mul1() {
        let node = Node::Mul(Node::Mul(number(3, 0), cell(0, 0)).boxed(), number(2, 0)).boxed();
        let expected = Node::Mul(number(6, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_nested_mul2() {
        let node = Node::Mul(number(3, 0), Node::Mul(number(2, 0), cell(0, 0)).boxed()).boxed();
        let expected = Node::Mul(number(6, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_nested_div1() {
        let node = Node::Mul(Node::Div(number(3, 0), cell(0, 0)).boxed(), number(2, 0)).boxed();
        let expected = Node::Div(number(6, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_nested_div2() {
        let node = Node::Mul(number(3, 0), Node::Div(number(2, 0), cell(0, 0)).boxed()).boxed();
        let expected = Node::Div(number(6, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_left_one() {
        let node = Node::Mul(number(1, 0), cell(0, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_mul_right_one() {
        let node = Node::Mul(cell(0, 0), number(1, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div() {
        let node = Node::Div(number(6, 0), number(15, 1)).boxed();
        let expected = number(4, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_move_number_to_left1() {
        let node = Node::Div(number(2, 0), cell(0, 0)).boxed();
        let expected = Node::Div(number(2, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_move_number_to_left2() {
        let node = Node::Div(cell(0, 0), number(2, 0)).boxed();
        let expected = Node::Mul(number(5, 1), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_nested_div1() {
        let node = Node::Div(Node::Div(number(6, 0), cell(0, 0)).boxed(), number(2, 0)).boxed();
        let expected = Node::Div(number(3, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_nested_div2() {
        let node = Node::Div(number(6, 0), Node::Div(number(2, 0), cell(0, 0)).boxed()).boxed();
        let expected = Node::Mul(number(3, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_nested_mul1() {
        let node = Node::Div(Node::Mul(number(6, 0), cell(0, 0)).boxed(), number(2, 0)).boxed();
        let expected = Node::Mul(number(3, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_nested_mul2() {
        let node = Node::Div(number(6, 0), Node::Mul(number(2, 0), cell(0, 0)).boxed()).boxed();
        let expected = Node::Div(number(3, 0), cell(0, 0)).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_right_one() {
        let node = Node::Div(cell(0, 0), number(1, 0)).boxed();
        let expected = cell(0, 0);
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn optimize_div_by_zero() {
        let node = Node::Div(number(3, 0), number(0, 0)).boxed();
        let res = optimize(node);
        assert_eq!(res.unwrap_err(), "Division by 0");
    }
    #[test]
    fn optimize_div_nested_div_by_zero1() {
        let node = Node::Div(number(6, 0), Node::Mul(number(0, 0), cell(0, 0)).boxed()).boxed();
        let res = optimize(node);
        assert_eq!(res.unwrap_err(), "Division by 0");
    }
    #[test]
    fn optimize_div_nested_div_by_zero2() {
        let node = Node::Div(number(6, 0), Node::Div(number(0, 0), cell(0, 0)).boxed()).boxed();
        let res = optimize(node);
        assert_eq!(res.unwrap_err(), "Division by 0");
    }
    #[test]
    fn optimize_function() {
        let node = Node::Function(
            "a".to_string(),
            vec![
                Node::Add(number(6, 0), number(15, 1)).boxed(),
                Node::Div(number(4, 0), number(2, 0)).boxed(),
            ],
        )
        .boxed();
        let expected = Node::Function("a".to_string(), vec![number(75, 1), number(2, 0)]).boxed();
        let res = optimize(node).unwrap();
        assert_eq!(res, expected);
    }
}
