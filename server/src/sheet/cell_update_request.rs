use serde::Deserialize;

#[derive(Deserialize)]
pub struct CellUpdateRequest {
    pub col: u32,
    pub row: u32,
    pub expression: Option<String>,
}
