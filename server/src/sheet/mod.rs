mod cell;
mod cell_update_request;
mod cell_update_response;
mod expression;

pub use self::cell_update_request::CellUpdateRequest;
pub use self::cell_update_response::CellUpdateResponse;
pub use self::expression::FuncDef;

use std::collections::HashMap;
use std::collections::HashSet;

use self::cell::Cell;
use self::cell::CellValue;
use self::expression::CellCallback;
use self::expression::Expression;

type CellReference = (u32, u32);

const ERR_CIRCULAR_REFERENCES_DETECTED: &str = "Circular references detected";
const ERR_CELL_EMPTY: &str = "No value";

pub struct Sheet {
    cells: HashMap<CellReference, Cell>,
    functions: HashMap<String, FuncDef>,
    dependencies: HashMap<CellReference, HashSet<CellReference>>,
}

impl Sheet {
    pub fn new(functions: HashMap<String, FuncDef>) -> Self {
        Sheet {
            cells: HashMap::new(),
            functions,
            dependencies: HashMap::new(),
        }
    }
    pub fn set_cell_expression(&mut self, request: CellUpdateRequest) -> Vec<CellUpdateResponse> {
        let CellUpdateRequest {
            col,
            row,
            expression: expression_param,
        } = request;
        let cell_addr: CellReference = (col, row);
        let mut old_dependencies = None;
        let mut new_dependencies = None;

        match expression_param {
            Some(expression_string) if !expression_string.trim().is_empty() => {
                let expression_result =
                    self.get_expression_from_str(cell_addr, &expression_string, true);
                let old_value = match self.cells.get(&cell_addr) {
                    Some(cell) => cell.value.clone(),
                    None => CellValue::CalcPending,
                };
                let value = match &expression_result {
                    Ok(_) => old_value,
                    Err(error) => CellValue::Error(error.clone()),
                };
                let new_cell = Cell {
                    expression: expression_result.ok(),
                    value,
                };

                new_dependencies = new_cell.get_dependencies();

                if let Some(old_cell) = self.cells.insert(cell_addr, new_cell) {
                    old_dependencies = old_cell.get_dependencies();
                }
            }
            _ => {
                if let Some(old_cell) = self.cells.remove(&cell_addr) {
                    old_dependencies = old_cell.get_dependencies();
                };
            }
        }

        let old_dependencies = old_dependencies.unwrap_or_default();
        let new_dependencies = new_dependencies.unwrap_or_default();

        self.remove_cell_dependencies(cell_addr, old_dependencies.difference(&new_dependencies));
        self.add_cell_dependencies(cell_addr, new_dependencies.difference(&old_dependencies));

        self.propagate_changes(cell_addr)
    }
    fn get_expression_from_str(
        &self,
        cell_addr: CellReference,
        expression: &str,
        optimize: bool,
    ) -> Result<Expression, String> {
        match Expression::from(expression, optimize) {
            Ok(expression) => {
                match self.check_for_cycles(cell_addr, expression.get_cell_dependencies()) {
                    Ok(_) => Ok(expression),
                    Err(_) => Err(ERR_CIRCULAR_REFERENCES_DETECTED.to_string()),
                }
            }
            Err(error) => Err(error),
        }
    }
    fn check_for_cycles(
        &self,
        cell_addr: CellReference,
        new_cell_dependencies: &HashSet<CellReference>,
    ) -> Result<(), ()> {
        let mut visited = HashSet::new();
        let mut pending;
        let mut new = new_cell_dependencies.clone();
        loop {
            (pending, new) = (new, HashSet::new());
            for cell_ref in pending {
                if cell_ref == cell_addr {
                    return Err(());
                }
                if visited.insert(cell_ref) {
                    if let Some(dependencies) = self
                        .cells
                        .get(&cell_ref)
                        .map(|cell| cell.get_dependencies())
                        .unwrap_or_default()
                    {
                        new.extend(dependencies);
                    }
                }
            }
            if new.is_empty() {
                return Ok(());
            }
        }
    }
    fn remove_cell_dependencies<'a, I: IntoIterator<Item = &'a CellReference>>(
        &mut self,
        referencing_cell: CellReference,
        dependencies: I,
    ) {
        for referenced_cell in dependencies {
            if let Some(dependent_cells) = self.dependencies.get_mut(referenced_cell) {
                dependent_cells.remove(&referencing_cell);
                if dependent_cells.is_empty() {
                    self.dependencies.remove(referenced_cell);
                }
            }
        }
    }
    fn add_cell_dependencies<'a, I: IntoIterator<Item = &'a CellReference>>(
        &mut self,
        referencing_cell: CellReference,
        dependencies: I,
    ) {
        for referenced_cell in dependencies {
            self.dependencies
                .entry(*referenced_cell)
                .or_default()
                .insert(referencing_cell);
        }
    }
    fn propagate_changes(&mut self, updated_cell: CellReference) -> Vec<CellUpdateResponse> {
        let mut result = vec![];
        let mut values = HashMap::new();

        let get_cell_callback: for<'a> fn(
            &'a HashMap<CellReference, CellValue>,
            &'a HashMap<CellReference, Cell>,
        ) -> CellCallback<'a> = |values, cells| {
            Box::new(|col, row| {
                let cell_addr = (col, row);
                match values
                    .get(&cell_addr)
                    .or(cells.get(&cell_addr).map(|c| &c.value))
                {
                    Some(value) => match value {
                        CellValue::Decimal(decimal) => Ok(*decimal),
                        CellValue::Comment(comment) => Err(comment.clone()),
                        CellValue::Error(error) => Err(error.clone()),
                        CellValue::CalcPending => Err(ERR_CELL_EMPTY.to_string()),
                    },
                    None => Err(ERR_CELL_EMPTY.to_string()),
                }
            })
        };

        if let Some(Cell {
            expression: None,
            value: cell_value,
        }) = self.cells.get(&updated_cell)
        {
            result.push(cell_update_response(updated_cell, cell_value));
        }

        for dependent_cell in self.prepare_update_plan(updated_cell) {
            if let Some(Cell {
                expression: Some(expression),
                value: old_value,
            }) = self.cells.get(&dependent_cell)
            {
                let value = get_cell_value(
                    expression,
                    &get_cell_callback(&values, &self.cells),
                    &self.functions,
                );
                if value != *old_value {
                    result.push(cell_update_response(dependent_cell, &value));
                    values.insert(dependent_cell, value);
                }
            }
        }

        for (cell_addr, cell_value) in values.into_iter() {
            self.cells
                .entry(cell_addr)
                .and_modify(|cell| cell.value = cell_value);
        }

        result
    }
    fn prepare_update_plan(&self, cell_addr: CellReference) -> Vec<CellReference> {
        let mut updates = HashMap::new();
        updates.insert(cell_addr, 0);
        let mut level = 1u32;
        let mut pending;
        let mut new = self
            .dependencies
            .get(&cell_addr)
            .cloned()
            .unwrap_or_else(HashSet::new);
        while !new.is_empty() {
            (pending, new) = (new, HashSet::new());
            for cell_addr in pending.into_iter() {
                updates.insert(cell_addr, level);
                if let Some(new_dependencies) = self.dependencies.get(&cell_addr) {
                    new.extend(new_dependencies);
                }
            }
            level += 1;
        }
        let mut updates = updates.into_iter().collect::<Vec<_>>();
        updates.sort_unstable_by_key(|u| (u.1, u.0 .0, u.0 .1));
        updates.into_iter().map(|u| u.0).collect()
    }
}

fn cell_update_response(cell_addr: CellReference, cell_value: &CellValue) -> CellUpdateResponse {
    CellUpdateResponse {
        col: cell_addr.0,
        row: cell_addr.1,
        value: cell_value.to_value(),
        error: cell_value.to_error(),
    }
}

fn get_cell_value(
    expression: &Expression,
    cell_callback: &CellCallback,
    functions: &HashMap<String, FuncDef>,
) -> CellValue {
    match expression.comment() {
        Some(comment) => CellValue::Comment(comment),
        None => match expression.solve(cell_callback, functions) {
            Ok(value) => CellValue::Decimal(value),
            Err(error) => CellValue::Error(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rust_decimal::Decimal;
    use rust_decimal::MathematicalOps;

    use super::cell_update_response;
    use super::CellUpdateRequest;
    use super::CellUpdateResponse;
    use super::CellValue;
    use super::FuncDef;
    use super::Sheet;

    #[derive(Debug, PartialEq)]
    struct TestCellUpdateResponse {
        pub col: u32,
        pub row: u32,
        pub value: Option<String>,
        pub error: Option<String>,
    }

    impl TestCellUpdateResponse {
        pub fn from(source: CellUpdateResponse) -> Self {
            TestCellUpdateResponse {
                col: source.col,
                row: source.row,
                value: source.value,
                error: source.error,
            }
        }
    }

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

    fn decode_cell_addr(cell_addr: &str) -> (u32, u32) {
        let cut = cell_addr.chars().position(|c| c.is_ascii_digit()).unwrap();
        (
            decode_cell_col(&cell_addr[..cut]),
            decode_cell_row(&cell_addr[cut..]),
        )
    }

    fn request(cell_addr: &str, expression: &str) -> CellUpdateRequest {
        let (col, row) = decode_cell_addr(cell_addr);
        CellUpdateRequest {
            col,
            row,
            expression: if !expression.is_empty() {
                Some(expression.to_string())
            } else {
                None
            },
        }
    }

    fn number(n: i64, s: u32) -> CellValue {
        CellValue::Decimal(Decimal::new(n, s))
    }

    fn error(message: &str) -> CellValue {
        CellValue::Error(message.to_string())
    }

    fn comment(comment: &str) -> CellValue {
        CellValue::Comment(comment.to_string())
    }

    fn response(cell_addr: &str, cell_value: CellValue) -> TestCellUpdateResponse {
        TestCellUpdateResponse::from(cell_update_response(
            decode_cell_addr(cell_addr),
            &cell_value,
        ))
    }

    fn get_functions() -> HashMap<String, FuncDef> {
        HashMap::new()
    }

    macro_rules! sheet_response {
        ($f:expr; $($a:literal: $e:literal),* ; $la:literal: $le:literal) => {{
            let mut sheet = Sheet::new($f);
            $(
                sheet.set_cell_expression(request($a, $e));
            )*
            sheet.set_cell_expression(request($la, $le))
                .into_iter()
                .map(|r| TestCellUpdateResponse::from(r))
                .collect::<Vec<_>>()
        }}
    }

    #[test]
    fn sheet_update_cell1() {
        let res = sheet_response!(get_functions(); ; "A1":"1");
        let expected = vec![response("A1", number(1, 0))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_update_cell2() {
        let res = sheet_response!(get_functions(); "A1":"1"; "A1":"1");
        let expected = vec![];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_update_cell3() {
        let res = sheet_response!(get_functions(); ; "A1":"");
        let expected = vec![];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_update_cell4() {
        let res = sheet_response!(get_functions(); "A1":"1"; "A1":"");
        let expected = vec![];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_update_cell5() {
        let res = sheet_response!(get_functions(); "A1":"1"; "A1":"2");
        let expected = vec![response("A1", number(2, 0))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_circular_references1() {
        let res = sheet_response!(get_functions(); ; "A1":"1+A1");
        let expected = vec![response("A1", error("Circular references detected"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_circular_references2() {
        let res = sheet_response!(get_functions(); "E5":"E6", "F4":"G4", "G4":"E4"; "E4":"F4");
        let expected = vec![response("E4", error("Circular references detected"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_comment1() {
        let res = sheet_response!(get_functions(); ; "A1":"'Comment");
        let expected = vec![response("A1", comment("Comment"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_comment2() {
        let res = sheet_response!(get_functions(); "A1":"'Comment"; "A2":"A1");
        let expected = vec![response("A2", error("A1: Value error"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_function1() {
        let res = sheet_response!(get_functions(); ; "A1":"sqrt(9)");
        let expected = vec![response("A1", error("Function not found: sqrt"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_function2() {
        let mut functions = get_functions();
        functions.insert("sqrt".to_string(), |params| {
            if params.len() == 1 {
                let param = params[0];
                match param.sqrt() {
                    Some(value) => Ok(value),
                    None => Err(format!("Error applying sqrt to {}", param)),
                }
            } else {
                Err(format!("sqrt expected 1 parameter, got {}", params.len()))
            }
        });
        let res = sheet_response!(functions; ; "A1":"sqrt(8+1)");
        let expected = vec![response("A1", number(3, 0))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_function3() {
        let mut functions = get_functions();
        functions.insert("sqrt".to_string(), |params| {
            if params.len() == 1 {
                let param = params[0];
                match param.sqrt() {
                    Some(value) => Ok(value),
                    None => Err(format!("Error applying sqrt to {}", param)),
                }
            } else {
                Err(format!("sqrt expected 1 parameter, got {}", params.len()))
            }
        });
        let res = sheet_response!(functions; ; "A1":"sqrt(-1)");
        let expected = vec![response("A1", error("Error applying sqrt to -1"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_function4() {
        let mut functions = get_functions();
        functions.insert("sqrt".to_string(), |params| {
            if params.len() == 1 {
                let param = params[0];
                match param.sqrt() {
                    Some(value) => Ok(value),
                    None => Err(format!("Error applying sqrt to {}", param)),
                }
            } else {
                Err(format!("sqrt expected 1 parameter, got {}", params.len()))
            }
        });
        let res = sheet_response!(functions; ; "A1":"sqrt(9, 1)");
        let expected = vec![response("A1", error("sqrt expected 1 parameter, got 2"))];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_propagate_changes1() {
        let res = sheet_response!(get_functions(); "A1":"1", "A2":"A1+1", "A3":"A1*3"; "A1":"2");
        let expected = vec![
            response("A1", number(2, 0)),
            response("A2", number(3, 0)),
            response("A3", number(6, 0)),
        ];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_propagate_changes2() {
        let res = sheet_response!(get_functions(); "A1":"1", "A2":"A1+1", "A3":"A1*3"; "A1":"");
        let expected = vec![
            response("A2", error("A1: Value error")),
            response("A3", error("A1: Value error")),
        ];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_propagate_changes3() {
        let res = sheet_response!(get_functions(); "A1":"1", "A2":"A1+1", "A3":"A1*3"; "A1":"1+");
        let expected = vec![
            response("A1", error("1+\n  ^\nUnexpected end of expression")),
            response("A2", error("A1: Value error")),
            response("A3", error("A1: Value error")),
        ];
        assert_eq!(res, expected);
    }
    #[test]
    fn sheet_propagate_changes4() {
        let res = sheet_response!(get_functions(); "A1":"1", "A2":"A1+1", "A3":"A1*3", "A5":"A3+1", "A4":"A5+1"; "A1":"2");
        let expected = vec![
            response("A1", number(2, 0)),
            response("A2", number(3, 0)),
            response("A3", number(6, 0)),
            response("A5", number(7, 0)),
            response("A4", number(8, 0)),
        ];
        assert_eq!(res, expected);
    }
}
