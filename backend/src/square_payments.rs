// Square Payments Integration Module
// Handles Square payment processing as an alternative to Stripe

use axum::{Json, Router, routing::post, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::AppState;

// Square API client configuration
pub struct SquareClient {
    pub access_token: String,
    pub application_id: String,
    pub environment: String, // "sandbox" or "production"
    pub base_url: String,
    pub client: reqwest::Client,
}

impl SquareClient {
    pub fn new(access_token: String, application_id: String, environment: String) -> Self {
        let base_url = match environment.as_str() {
            "production" => "https://connect.squareup.com".to_string(),
            _ => "https://connect.squareupsandbox.com".to_string(), // Default to sandbox
        };

        Self {
            access_token,
            application_id,
            environment,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

// Request/Response structures for Square API
#[derive(Deserialize)]
pub struct SquarePaymentRequest {
    pub amount_money: AmountMoney,
    pub source_id: String, // Card nonce from Square Web Payments SDK
    pub idempotency_key: Option<String>,
    pub location_id: Option<String>, // Optional - will use default if not provided
}

#[derive(Deserialize, Serialize)]
pub struct AmountMoney {
    pub amount: i64, // Amount in smallest currency unit (cents for USD)
    pub currency: String, // "USD", "EUR", etc.
}

#[derive(Serialize)]
pub struct SquareCreatePaymentRequest {
    pub source_id: String,
    pub idempotency_key: String,
    pub amount_money: AmountMoney,
    pub location_id: String,
    pub app_fee_money: Option<AmountMoney>,
    pub autocomplete: Option<bool>,
    pub order_id: Option<String>,
    pub buyer_email_address: Option<String>,
    pub billing_address: Option<Address>,
    pub shipping_address: Option<Address>,
    pub note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Address {
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub locality: Option<String>, // City
    pub administrative_district_level_1: Option<String>, // State/Province
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Deserialize)]
pub struct SquarePaymentResponse {
    pub payment: Option<Payment>,
    pub errors: Option<Vec<SquareError>>,
}

#[derive(Deserialize)]
pub struct Payment {
    pub id: String,
    pub status: String,
    pub amount_money: AmountMoney,
    pub source_type: String,
    pub card_details: Option<CardDetails>,
    pub receipt_number: Option<String>,
    pub receipt_url: Option<String>,
}

#[derive(Deserialize)]
pub struct CardDetails {
    pub status: String,
    pub card: Option<Card>,
    pub entry_method: String,
}

#[derive(Deserialize)]
pub struct Card {
    pub card_brand: String,
    pub last_4: String,
    pub exp_month: Option<i32>,
    pub exp_year: Option<i32>,
}

#[derive(Deserialize)]
pub struct SquareError {
    pub category: String,
    pub code: String,
    pub detail: String,
    pub field: Option<String>,
}

// Response structure for our API
#[derive(Serialize)]
pub struct SquarePaymentIntentResponse {
    pub payment_id: String,
    pub status: String,
    pub receipt_url: Option<String>,
}

// Add Square client to AppState
impl AppState {
    pub fn square_client(&self) -> Option<SquareClient> {
        let access_token = std::env::var("SQUARE_ACCESS_TOKEN").ok()?;
        let application_id = std::env::var("SQUARE_APPLICATION_ID").ok()?;
        let environment = std::env::var("SQUARE_ENVIRONMENT").unwrap_or_else(|_| "sandbox".to_string());
        
        Some(SquareClient::new(access_token, application_id, environment))
    }

    pub fn square_location_id(&self) -> String {
        std::env::var("SQUARE_LOCATION_ID").unwrap_or_else(|_| "LP7V5561FPK0B".to_string())
    }
}

// Square payment routes
pub fn square_payment_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/square/create-payment", post(create_square_payment))
        .with_state(app_state)
}

// Create Square payment handler
async fn create_square_payment(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SquarePaymentRequest>,
) -> Result<Json<SquarePaymentIntentResponse>, (StatusCode, String)> {
    let square_client = state.square_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Square client not configured".to_string()))?;

    // Generate idempotency key if not provided
    let idempotency_key = payload.idempotency_key
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Use provided location_id or default from environment
    let location_id = payload.location_id.unwrap_or_else(|| state.square_location_id());

    // Prepare Square API request
    let square_request = SquareCreatePaymentRequest {
        source_id: payload.source_id,
        idempotency_key,
        amount_money: payload.amount_money,
        location_id,
        app_fee_money: None,
        autocomplete: Some(true), // Auto-complete the payment
        order_id: None,
        buyer_email_address: None,
        billing_address: None,
        shipping_address: None,
        note: Some("E-commerce platform payment".to_string()),
    };

    // Make request to Square API
    let response = square_client
        .client
        .post(&format!("{}/v2/payments", square_client.base_url))
        .header("Authorization", format!("Bearer {}", square_client.access_token))
        .header("Content-Type", "application/json")
        .header("Square-Version", "2025-05-21") // Use the API version from your test
        .json(&square_request)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Square API request failed: {}", e)))?;

    let square_response: SquarePaymentResponse = response
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse Square response: {}", e)))?;

    // Handle Square API response
    if let Some(errors) = square_response.errors {
        let error_details = errors.iter()
            .map(|e| format!("{}: {}", e.code, e.detail))
            .collect::<Vec<_>>()
            .join(", ");
        return Err((StatusCode::BAD_REQUEST, format!("Square API errors: {}", error_details)));
    }

    if let Some(payment) = square_response.payment {
        Ok(Json(SquarePaymentIntentResponse {
            payment_id: payment.id,
            status: payment.status,
            receipt_url: payment.receipt_url,
        }))
    } else {
        Err((StatusCode::INTERNAL_SERVER_ERROR, "No payment data returned from Square".to_string()))
    }
}