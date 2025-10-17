// Checkout and payment API

use crate::types::{CheckoutRequest, Order};
use super::{post, ApiError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntentRequest {
    pub amount: i64,  // Amount in cents
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntentResponse {
    pub client_secret: String,
}

/// Create Stripe payment intent
pub async fn create_payment_intent(amount: f64) -> Result<PaymentIntentResponse, ApiError> {
    let amount_cents = (amount * 100.0) as i64;

    let request = PaymentIntentRequest {
        amount: amount_cents,
        currency: "usd".to_string(),
    };

    post("/api/create-payment-intent", &request).await
}

/// Submit checkout order (placeholder - will integrate with backend)
pub async fn submit_order(checkout: &CheckoutRequest) -> Result<Order, ApiError> {
    // TODO: Implement actual backend endpoint
    // For now, return a mock order
    Err(ApiError {
        message: "Order submission not yet implemented".to_string(),
        status: 501,
    })
}
