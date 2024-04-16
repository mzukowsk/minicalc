mod node;
mod optimize;
mod parse;
mod solve;
mod tokenizer;

pub use self::solve::CellCallback;
pub use self::solve::FuncDef;

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;

use self::node::Node;
use self::tokenizer::Tokenizer;

pub struct Expression {
    node: Box<Node>,
    cell_dependencies: HashSet<(u32, u32)>,
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node)
    }
}

impl Expression {
    fn from_node(node: Box<Node>) -> Self {
        let cell_dependencies = get_cell_dependencies(&node);
        Expression {
            node,
            cell_dependencies,
        }
    }
    pub fn from(expression: &str, optimize: bool) -> Result<Self, String> {
        let mut tokenizer = Tokenizer::from(expression)?;
        let parsed = parse::parse(&mut tokenizer)?;
        if optimize {
            let optimized = optimize::optimize(parsed)?;
            Ok(Expression::from_node(optimized))
        } else {
            Ok(Expression::from_node(parsed))
        }
    }
    pub fn get_cell_dependencies(&self) -> &HashSet<(u32, u32)> {
        &self.cell_dependencies
    }
    pub fn solve(
        &self,
        cell_callback: &CellCallback,
        functions: &HashMap<String, FuncDef>,
    ) -> Result<Decimal, String> {
        solve::solve(&self.node, cell_callback, functions)
    }
    pub fn comment(&self) -> Option<String> {
        match *self.node {
            Node::Comment(ref comment) => Some(comment.clone()),
            _ => None,
        }
    }
}

fn get_cell_dependencies(node: &Node) -> HashSet<(u32, u32)> {
    let mut res = HashSet::new();
    get_subtree_dependencies(&mut res, node);
    res
}

fn get_subtree_dependencies(dependencies: &mut HashSet<(u32, u32)>, node: &Node) {
    match *node {
        Node::Add(ref left, ref right)
        | Node::Sub(ref left, ref right)
        | Node::Mul(ref left, ref right)
        | Node::Div(ref left, ref right) => {
            get_subtree_dependencies(dependencies, left);
            get_subtree_dependencies(dependencies, right);
        }
        Node::Cell(col, row) => {
            dependencies.insert((col, row));
        }
        Node::Parentheses(ref inner) | Node::UnaryMinus(ref inner) => {
            get_subtree_dependencies(dependencies, inner);
        }
        Node::Function(_, ref params) => {
            for param in params {
                get_subtree_dependencies(dependencies, param);
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use std::collections::HashSet;

    use super::get_cell_dependencies;
    use super::Node;

    fn cell(col: u32, row: u32) -> Box<Node> {
        Node::Cell(col, row).boxed()
    }

    fn number(n: i64, s: u32) -> Box<Node> {
        Node::Number(Decimal::new(n, s)).boxed()
    }

    #[test]
    fn get_cell_dependencies1() {
        let node = Node::Add(
            cell(0, 0),
            Node::UnaryMinus(
                Node::Mul(
                    cell(0, 1),
                    Node::Parentheses(Node::Add(cell(0, 2), cell(0, 3)).boxed()).boxed(),
                )
                .boxed(),
            )
            .boxed(),
        )
        .boxed();
        let expected = HashSet::<(u32, u32)>::from_iter(vec![(0, 0), (0, 1), (0, 2), (0, 3)]);
        let res = get_cell_dependencies(&node);
        assert_eq!(res, expected);
    }
    #[test]
    fn get_cell_dependencies2() {
        let node = Node::Add(
            cell(0, 0),
            Node::Add(number(1, 0), Node::Add(cell(0, 0), cell(0, 1)).boxed()).boxed(),
        )
        .boxed();
        let expected = HashSet::<(u32, u32)>::from_iter(vec![(0, 0), (0, 1)]);
        let res = get_cell_dependencies(&node);
        assert_eq!(res, expected);
    }
    #[test]
    fn get_cell_dependencies3() {
        let node = Node::Add(
            cell(0, 0),
            Node::Function("foka".to_string(), vec![number(1, 0), cell(0, 1), cell(0, 2)]).boxed(),
        )
        .boxed();
        let expected = HashSet::<(u32, u32)>::from_iter(vec![(0, 0), (0, 1), (0, 2)]);
        let res = get_cell_dependencies(&node);
        assert_eq!(res, expected);
    }
}
