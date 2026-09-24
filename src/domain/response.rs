use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T> ApiResponse<T> {
    #[must_use]
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub total_items: u64,
    pub total_pages: u64,
}

impl<T> Paginated<T> {
    #[must_use]
    pub fn new(items: Vec<T>, page: u32, page_size: u32, total_items: u64) -> Self {
        let page = page.max(1);
        let page_size = page_size.max(1);
        let total_pages = total_items.div_ceil(page_size as u64);
        Self {
            items,
            pagination: Pagination {
                page,
                page_size,
                total_items,
                total_pages,
            },
        }
    }
}
