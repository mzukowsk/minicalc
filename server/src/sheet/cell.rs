use std::collections::HashSet;
use rust_decimal::Decimal;

use super::expression::Expression;

#[derive(Clone, PartialEq)]
pub enum CellValue {
    CalcPending,
    Decimal(Decimal),
    Comment(String),
    Error(String),
}

impl CellValue {
    pub fn to_value(&self) -> Option<String> {
        match self {
            CellValue::Decimal(decimal) => Some(decimal.normalize().to_string()),
            CellValue::Comment(comment) => Some(comment.clone()),
            _ => None,
        }
    }
    pub fn to_error(&self) -> Option<String> {
        match self {
            CellValue::Error(error) => Some(error.clone()),
            _ => None,
        }
    }
}

pub struct Cell {
    pub expression: Option<Expression>,
    pub value: CellValue,
}

impl Cell {
    pub fn get_dependencies(&self) -> Option<HashSet<(u32, u32)>> {
        self.expression
            .as_ref()
            .map(|expression| expression.get_cell_dependencies().clone())
    }
}
