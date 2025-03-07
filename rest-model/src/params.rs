use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchParams {
    pub filter: Condition,
    pub patch: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteParams {
    pub filter: Condition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    And(Vec<Box<Condition>>),
    Or(Vec<Box<Condition>>),
    Not(Box<Condition>),
    Regex(String, Value),
    Regexi(String, Value),
    Eq(String, Value),
    Ne(String, Value),
    Gt(String, Value),
    Lt(String, Value),
    Gte(String, Value),
    Lte(String, Value),
    In(String, Value),
    Nin(String, Value),
}