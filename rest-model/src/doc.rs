use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{DbClient, RestModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Doc<T> {
    pub _id: String,
    pub data: T,
    pub _created_at: i64,
    pub _updated_at: i64,
}

impl<T> Doc<T>
where
    T: RestModel,
{
    pub fn new(db: &impl DbClient<T>, data: T) -> Self {
        Self {
            _id: db.generate_id(),
            data,
            _created_at: Utc::now().timestamp_millis(),
            _updated_at: Utc::now().timestamp_millis(),
        }
    }
}
