use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct Data {
    pub id: String,
    pub name: String,
    pub date: String,
    pub biography: String,
    pub sources: String,
    pub rows: HashMap<String, String>,
}
