use serde::{Deserialize, Serialize};
use crate::Condition;

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort: Option<String>,
    pub filter: Option<Condition>,
    pub custom: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub total_count: u32,
    pub total_pages: u32,
    pub current_page: u32,
    pub items_per_page: u32,
}

pub const HEADER_EXPOSE: &str = "Access-Control-Expose-Headers";
pub const DEFAULT_PAGE: u32 = 1;
pub const DEFAULT_LIMIT: u32 = 10;
pub const HEADER_TOTAL_COUNT: &str = "X-Total-Count";
pub const HEADER_TOTAL_PAGES: &str = "X-Total-Pages";
pub const HEADER_CURRENT_PAGE: &str = "X-Current-Page";
pub const HEADER_ITEMS_PER_PAGE: &str = "X-Items-Per-Page";
