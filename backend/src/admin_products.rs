// AXUM 0.7.4 UPDATE: Only needed routing imports
use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
// PgPool accessed through AppState
// use sqlx::PgPool;
use std::sync::Arc;
use crate::admin_auth::AuthenticatedAdmin;
use crate::AppState;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub inventory: i32,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
}

#[derive(Deserialize)]
pub struct ProductInput {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub inventory: i32,
}

pub fn admin_product_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/admin/products", get(list_products).post(create_product))
        .route("/api/admin/products/:id", put(update_product).delete(delete_product))
        .with_state(app_state)
}

async fn list_products(
    _admin: AuthenticatedAdmin,
    State(app_state): State<Arc<AppState>>,
) -> Json<Vec<Product>> {
    let products = sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY id")
        .fetch_all(&*app_state.pool)
        .await
        .unwrap_or_default();
    Json(products)
}

async fn create_product(
    _admin: AuthenticatedAdmin,
    State(app_state): State<Arc<AppState>>,
    Json(input): Json<ProductInput>,
) -> Json<Product> {
    let rec = sqlx::query_as::<_, Product>(
        "INSERT INTO products (name, description, price, inventory) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(input.price)
    .bind(input.inventory)
    .fetch_one(&*app_state.pool)
    .await
    .unwrap();
    Json(rec)
}

async fn update_product(
    _admin: AuthenticatedAdmin,
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(input): Json<ProductInput>,
) -> Json<Product> {
    let rec = sqlx::query_as::<_, Product>(
        "UPDATE products SET name = $1, description = $2, price = $3, inventory = $4 WHERE id = $5 RETURNING *"
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(input.price)
    .bind(input.inventory)
    .bind(id)
    .fetch_one(&*app_state.pool)
    .await
    .unwrap();
    Json(rec)
}

async fn delete_product(
    _admin: AuthenticatedAdmin,
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Json<bool> {
    let res = sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(id)
        .execute(&*app_state.pool)
        .await
        .unwrap();
    Json(res.rows_affected() > 0)
}
