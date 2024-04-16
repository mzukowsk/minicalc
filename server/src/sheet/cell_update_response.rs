use serde::Serialize;

#[derive(Serialize)]
pub struct CellUpdateResponse {
    pub col: u32,
    pub row: u32,
    pub value: Option<String>,
    pub error: Option<String>,
}
