// Webhooks Module - Shared types and utilities for webhook processing
// Handles common webhook operations like logging events and managing orders

pub mod stripe;
pub mod square;

use axum::{Router, routing::post};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::types::chrono::{DateTime, Utc};
use std::sync::Arc;
use crate::AppState;

// Enum for payment providers
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum PaymentProvider {
    #[sqlx(rename = "stripe")]
    Stripe,
    #[sqlx(rename = "square")]
    Square,
}

impl std::fmt::Display for PaymentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentProvider::Stripe => write!(f, "stripe"),
            PaymentProvider::Square => write!(f, "square"),
        }
    }
}

// Enum for order status
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum OrderStatus {
    #[sqlx(rename = "pending")]
    Pending,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "refunded")]
    Refunded,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Completed => write!(f, "completed"),
            OrderStatus::Failed => write!(f, "failed"),
            OrderStatus::Refunded => write!(f, "refunded"),
        }
    }
}

// Database model for webhook events
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub provider: String,
    pub event_type: String,
    pub event_id: String,
    pub payload: serde_json::Value,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Database model for orders
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub payment_provider: String,
    pub payment_id: String,
    pub payment_intent_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub total_amount: i64,
    pub currency: String,
    pub status: String,
    pub webhook_event_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Database model for order items
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Option<i32>,
    pub product_name: String,
    pub product_description: Option<String>,
    pub quantity: i32,
    pub unit_price: i64,
    pub total_price: i64,
    pub created_at: DateTime<Utc>,
}

// Struct for creating new webhook events
pub struct CreateWebhookEvent {
    pub provider: PaymentProvider,
    pub event_type: String,
    pub event_id: String,
    pub payload: serde_json::Value,
}

// Struct for creating new orders
pub struct CreateOrder {
    pub payment_provider: PaymentProvider,
    pub payment_id: String,
    pub payment_intent_id: Option<String>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub total_amount: i64,
    pub currency: String,
    pub status: OrderStatus,
    pub webhook_event_id: Option<Uuid>,
}

// Utility function to log webhook events to database
pub async fn log_webhook_event(
    pool: &sqlx::PgPool,
    event: CreateWebhookEvent,
) -> Result<Uuid, sqlx::Error> {
    let provider_str = event.provider.to_string();

    let result = sqlx::query!(
        r#"
        INSERT INTO webhook_events (provider, event_type, event_id, payload, processed)
        VALUES ($1, $2, $3, $4, FALSE)
        RETURNING id
        "#,
        provider_str,
        event.event_type,
        event.event_id,
        event.payload,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

// Utility function to mark webhook event as processed
pub async fn mark_webhook_processed(
    pool: &sqlx::PgPool,
    webhook_id: Uuid,
    success: bool,
    error_message: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE webhook_events
        SET processed = $1, processed_at = NOW(), error_message = $2
        WHERE id = $3
        "#,
        success,
        error_message,
        webhook_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Utility function to create orders
pub async fn create_order(
    pool: &sqlx::PgPool,
    order: CreateOrder,
) -> Result<Uuid, sqlx::Error> {
    let provider_str = order.payment_provider.to_string();
    let status_str = order.status.to_string();

    let result = sqlx::query!(
        r#"
        INSERT INTO orders (
            payment_provider, payment_id, payment_intent_id,
            customer_email, customer_name, total_amount, currency,
            status, webhook_event_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id
        "#,
        provider_str,
        order.payment_id,
        order.payment_intent_id,
        order.customer_email,
        order.customer_name,
        order.total_amount,
        order.currency,
        status_str,
        order.webhook_event_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

// Utility function to check if webhook event already processed (idempotency)
pub async fn is_event_processed(
    pool: &sqlx::PgPool,
    event_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM webhook_events WHERE event_id = $1) as "exists!"
        "#,
        event_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

// Export webhook routes for main.rs
pub fn webhook_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/webhooks/stripe", post(stripe::handle_stripe_webhook))
        .route("/api/webhooks/square", post(square::handle_square_webhook))
        .with_state(app_state)
}
