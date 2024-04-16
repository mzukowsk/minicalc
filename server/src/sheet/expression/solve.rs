use std::collections::HashMap;
use rust_decimal::Decimal;

use super::node::Node;

pub type CellCallback<'a> = Box<dyn Fn(u32, u32) -> Result<Decimal, String> + 'a>;
pub type FuncDef = fn(Vec<Decimal>) -> Result<Decimal, String>;

pub fn solve(
    node: &Node,
    cell_callback: &CellCallback,
    functions: &HashMap<String, FuncDef>,
) -> Result<Decimal, String> {
    match *node {
        Node::Comment(ref comment) => Err(format!("Comment: '{}'", comment)),
        Node::Add(ref left, ref right) => {
            Ok(solve(left, cell_callback, functions)? + solve(right, cell_callback, functions)?)
        }
        Node::Sub(ref left, ref right) => {
            Ok(solve(left, cell_callback, functions)? - solve(right, cell_callback, functions)?)
        }
        Node::Mul(ref left, ref right) => {
            Ok(solve(left, cell_callback, functions)? * solve(right, cell_callback, functions)?)
        }
        Node::Div(ref left, ref right) => {
            let left_value = solve(left, cell_callback, functions)?;
            let right_value = solve(right, cell_callback, functions)?;
            if right_value != Decimal::ZERO {
                Ok(left_value / right_value)
            } else {
                Err(format!("Trying to divide {} by 0", left_value))
            }
        }
        Node::Parentheses(ref node) => solve(node, cell_callback, functions),
        Node::UnaryMinus(ref node) => Ok(-solve(node, cell_callback, functions)?),
        Node::Number(number) => Ok(number),
        Node::Cell(col, row) => cell_callback(col, row).map_err(|_| format!("{}: Value error", *node)),
        Node::Function(ref name, ref args) => match functions.get(name) {
            Some(function) => {
                let mut function_params = Vec::new();
                for node in args.iter() {
                    function_params.push(solve(node, cell_callback, functions)?.normalize());
                }
                function(function_params)
            }
            None => Err(format!("Function not found: {}", name)),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rust_decimal::Decimal;

    use super::solve;
    use super::CellCallback;
    use super::FuncDef;
    use super::Node;

    fn cell_callback<'a>() -> CellCallback<'a> {
        Box::new(|col, row| Ok(Decimal::new(col as i64, row)))
    }

    fn get_functions() -> HashMap<String, FuncDef> {
        HashMap::new()
    }

    fn cell(col: u32, row: u32) -> Box<Node> {
        Node::Cell(col, row).boxed()
    }

    fn number(n: i64, s: u32) -> Box<Node> {
        Node::Number(Decimal::new(n, s)).boxed()
    }

    #[test]
    fn solve_number() {
        let node = number(1, 0);
        let expected = Decimal::new(1, 0);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_add() {
        let node = Node::Add(number(3, 0), number(15, 1)).boxed();
        let expected = Decimal::new(45, 1);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_sub() {
        let node = Node::Sub(number(3, 0), number(15, 1)).boxed();
        let expected = Decimal::new(15, 1);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_mul() {
        let node = Node::Mul(number(3, 0), number(15, 1)).boxed();
        let expected = Decimal::new(45, 1);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_div() {
        let node = Node::Div(number(3, 0), number(15, 1)).boxed();
        let expected = Decimal::new(2, 0);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_div_by_zero() {
        let node = Node::Div(number(3, 0), number(0, 0)).boxed();
        let res = solve(&node, &cell_callback(), &get_functions());
        assert_eq!(&res.unwrap_err(), "Trying to divide 3 by 0");
    }
    #[test]
    fn solve_nested() {
        let node = Node::Sub(
            Node::Add(number(3, 0), number(15, 1)).boxed(),
            number(2, 0),
        )
        .boxed();
        let expected = Decimal::new(25, 1);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_parentheses() {
        let node = Node::Parentheses(number(1, 0)).boxed();
        let expected = Decimal::new(1, 0);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_unary_minus() {
        let node = Node::UnaryMinus(number(1, 0)).boxed();
        let expected = Decimal::new(-1, 0);
        let res = solve(&node, &cell_callback(), &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_cell_reference() {
        let cell_callback: CellCallback = Box::new(|_c, _r| Ok(Decimal::new(1, 0)));
        let node = cell(0, 0);
        let expected = Decimal::new(1, 0);
        let res = solve(&node, &cell_callback, &get_functions()).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_error_cell_reference() {
        let cell_callback: CellCallback = Box::new(|_c, _r| Err("Foka".to_string()));
        let node = cell(0, 0);
        let expected = "A1: Value error".to_string();
        let res = solve(&node, &cell_callback, &get_functions()).unwrap_err();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_function() {
        let mut functions = get_functions();
        functions.insert("a".to_string(), |params| Ok(params.iter().sum()));
        let node = Node::Function("a".to_string(), vec![number(1, 0), number(2, 0)]).boxed();
        let expected = Decimal::new(3, 0);
        let res = solve(&node, &cell_callback(), &functions).unwrap();
        assert_eq!(res, expected);
    }
    #[test]
    fn solve_unknown_function() {
        let mut functions = get_functions();
        functions.insert("b".to_string(), |params| Ok(params.iter().sum()));
        let node = Node::Function("a".to_string(), vec![number(1, 0), number(2, 0)]).boxed();
        let res = solve(&node, &cell_callback(), &functions);
        assert_eq!(res.unwrap_err(), "Function not found: a");
    }
    #[test]
    fn solve_function_error() {
        let mut functions = get_functions();
        functions.insert("a".to_string(), |_params| Err("Foka".to_string()));
        let node = Node::Function("a".to_string(), vec![number(1, 0), number(2, 0)]).boxed();
        let res = solve(&node, &cell_callback(), &functions);
        assert_eq!(res.unwrap_err(), "Foka");
    }
}
