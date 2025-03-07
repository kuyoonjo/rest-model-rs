use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc<T> {
    pub _id: String,
    pub data: T,
    pub _created_at: i64,
    pub _updated_at: i64,
}
