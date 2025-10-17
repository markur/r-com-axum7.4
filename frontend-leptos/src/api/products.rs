// Product API client

use crate::types::Product;
use super::{get, ApiError};

/// Fetch all products from the backend
pub async fn fetch_products() -> Result<Vec<Product>, ApiError> {
    get("/api/products").await
}

/// Fetch a single product by ID
pub async fn fetch_product(id: i32) -> Result<Product, ApiError> {
    get(&format!("/api/products/{}", id)).await
}
