use serde::{Deserialize, Serialize};

use crate::{pagination::Pagination, Doc};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationResult<T: Serialize> {
    pub items: Vec<Doc<T>>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResult {
    pub updated_count: u32,
}

#[derive(Debug, Serialize)]
pub struct UpsertResult {
    pub created_count: u32,
    pub updated_count: u32,
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    pub deleted_count: u32,
}
