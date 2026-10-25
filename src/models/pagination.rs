use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationRequest {
    #[validate(range(min = 1, message = "page must be greater than 0"))]
    pub page: Option<i64>,
    #[validate(range(min = 1, message = "limit must be greater than 0"))]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PaginationResponse {
    total_items: i64,
    total_pages: i64,
    current_page: i64,
    limit: i64,
    has_next_page: bool,
    has_previous_page: bool,
}

impl PaginationResponse {
    pub fn new(total: i64, limit: i64, page: i64) -> Self {
        let mut total_pages = (total + (limit - 1)) / limit;
        total_pages = total_pages.max(1);

        Self {
            total_items: total,
            total_pages,
            current_page: page,
            limit,
            has_next_page: page < total_pages,
            has_previous_page: page > 1,
        }
    }
}
