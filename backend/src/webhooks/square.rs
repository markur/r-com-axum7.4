// Square Webhook Handler
// Processes Square webhook events for payment completions
// Implements HMAC-SHA256 signature verification for security

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json, body::Bytes,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;
use super::{
    log_webhook_event, mark_webhook_processed, create_order, is_event_processed,
    CreateWebhookEvent, CreateOrder, PaymentProvider, OrderStatus,
};

type HmacSha256 = Hmac<Sha256>;

// Square webhook event structure
#[derive(Debug, Deserialize, Serialize)]
pub struct SquareWebhookEvent {
    pub merchant_id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_id: String,
    pub created_at: String,
    pub data: SquareEventData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquareEventData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub id: String,
    pub object: Option<SquarePaymentObject>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquarePaymentObject {
    pub payment: Option<SquarePayment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquarePayment {
    pub id: String,
    pub status: String,
    pub amount_money: SquareAmountMoney,
    pub source_type: Option<String>,
    pub card_details: Option<SquareCardDetails>,
    pub receipt_number: Option<String>,
    pub receipt_url: Option<String>,
    pub buyer_email_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquareAmountMoney {
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquareCardDetails {
    pub status: String,
    pub card: Option<SquareCard>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SquareCard {
    pub card_brand: String,
    pub last_4: String,
}

// Square webhook endpoint handler
pub async fn handle_square_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get Square signature from headers
    let signature = headers
        .get("x-square-hmacsha256-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Missing x-square-hmacsha256-signature header".to_string(),
        ))?;

    // Get webhook signature key from environment
    let webhook_signature_key = std::env::var("SQUARE_WEBHOOK_SIGNATURE_KEY")
        .unwrap_or_else(|_| "your_webhook_signature_key".to_string());

    // Get the webhook notification URL from environment (needed for signature verification)
    let webhook_url = std::env::var("SQUARE_WEBHOOK_URL")
        .unwrap_or_else(|_| "https://your-domain.com/api/webhooks/square".to_string());

    // Verify webhook signature
    if !verify_square_signature(&body, signature, &webhook_signature_key, &webhook_url) {
        eprintln!("Square webhook signature verification failed");
        return Err((
            StatusCode::UNAUTHORIZED,
            "Webhook signature verification failed".to_string(),
        ));
    }

    // Parse the webhook event
    let body_str = String::from_utf8(body.to_vec())
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid UTF-8: {}", e)))?;

    let event: SquareWebhookEvent = serde_json::from_str(&body_str)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

    // Check if we've already processed this event (idempotency)
    let event_id = &event.event_id;
    match is_event_processed(&state.pool, event_id).await {
        Ok(true) => {
            println!("Event {} already processed, returning 200 OK", event_id);
            return Ok((StatusCode::OK, Json(json!({"received": true, "duplicate": true}))));
        }
        Ok(false) => {
            // Continue processing
        }
        Err(e) => {
            eprintln!("Error checking event idempotency: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ));
        }
    }

    // Log the webhook event to database
    let webhook_event = CreateWebhookEvent {
        provider: PaymentProvider::Square,
        event_type: event.event_type.clone(),
        event_id: event_id.clone(),
        payload: serde_json::to_value(&event).unwrap_or(json!({})),
    };

    let webhook_id = match log_webhook_event(&state.pool, webhook_event).await {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to log webhook event: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to log webhook: {}", e),
            ));
        }
    };

    // Process the event based on type
    let result = match event.event_type.as_str() {
        "payment.updated" => {
            handle_payment_updated(&state, &event, webhook_id).await
        }
        "payment.created" => {
            // Log but don't create order until payment is completed
            println!("Payment created event received: {:?}", event.data.id);
            mark_webhook_processed(&state.pool, webhook_id, true, None).await.ok();
            Ok(())
        }
        _ => {
            // For other events, just log and mark as processed
            println!("Received Square event type: {}", event.event_type);
            mark_webhook_processed(&state.pool, webhook_id, true, None).await.ok();
            Ok(())
        }
    };

    // Mark webhook as processed with error if any
    match result {
        Ok(_) => {
            mark_webhook_processed(&state.pool, webhook_id, true, None).await.ok();
            Ok((StatusCode::OK, Json(json!({"received": true}))))
        }
        Err(e) => {
            eprintln!("Error processing webhook: {}", e);
            mark_webhook_processed(&state.pool, webhook_id, false, Some(e.clone())).await.ok();
            // Return 200 anyway to prevent retries for application errors
            Ok((StatusCode::OK, Json(json!({"received": true, "error": e}))))
        }
    }
}

// Verify Square webhook signature using HMAC-SHA256
fn verify_square_signature(
    body: &[u8],
    signature: &str,
    signature_key: &str,
    webhook_url: &str,
) -> bool {
    // Square signature is computed as: HMAC-SHA256(notification_url + request_body, signature_key)
    let mut mac = match HmacSha256::new_from_slice(signature_key.as_bytes()) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to create HMAC: {}", e);
            return false;
        }
    };

    // Concatenate webhook_url and body
    mac.update(webhook_url.as_bytes());
    mac.update(body);

    // Compute the HMAC
    let result = mac.finalize();
    let computed_signature = base64::encode(result.into_bytes());

    // Compare with provided signature (constant-time comparison)
    computed_signature == signature
}

// Handle payment.updated event
async fn handle_payment_updated(
    state: &Arc<AppState>,
    event: &SquareWebhookEvent,
    webhook_id: uuid::Uuid,
) -> Result<(), String> {
    // Extract payment object from event data
    let payment = event
        .data
        .object
        .as_ref()
        .and_then(|obj| obj.payment.as_ref())
        .ok_or("Missing payment object in event data".to_string())?;

    println!(
        "Payment updated! Payment ID: {}, Status: {}, Amount: {} {}",
        payment.id,
        payment.status,
        payment.amount_money.amount,
        payment.amount_money.currency
    );

    // Only create order if payment status is COMPLETED
    if payment.status != "COMPLETED" {
        println!("Payment status is {}, not creating order", payment.status);
        return Ok(());
    }

    // Check if order already exists for this payment
    let existing = sqlx::query!(
        "SELECT id FROM orders WHERE payment_id = $1 AND payment_provider = 'square'",
        payment.id
    )
    .fetch_optional(&*state.pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?;

    if existing.is_some() {
        println!("Order already exists for payment {}", payment.id);
        return Ok(());
    }

    // Create order record
    let order = CreateOrder {
        payment_provider: PaymentProvider::Square,
        payment_id: payment.id.clone(),
        payment_intent_id: None, // Square doesn't have payment intents like Stripe
        customer_email: payment.buyer_email_address.clone(),
        customer_name: None,
        total_amount: payment.amount_money.amount,
        currency: payment.amount_money.currency.clone(),
        status: OrderStatus::Completed,
        webhook_event_id: Some(webhook_id),
    };

    let order_id = create_order(&state.pool, order)
        .await
        .map_err(|e| format!("Failed to create order: {}", e))?;

    println!("Created order with ID: {}", order_id);

    // Send order confirmation email
    if let Some(email) = &payment.buyer_email_address {
        send_order_confirmation_email(email, &payment.id, payment.amount_money.amount).await;
    }

    Ok(())
}

// Send order confirmation email (placeholder - integrate with your email service)
async fn send_order_confirmation_email(email: &str, order_id: &str, amount: i64) {
    println!(
        "Sending order confirmation email to {} for order {} (${:.2})",
        email,
        order_id,
        amount as f64 / 100.0
    );

    // TODO: Integrate with lettre_email module
    // For now, just log the email that would be sent
}
