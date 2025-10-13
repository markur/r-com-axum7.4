// SMS Module - Supports Twilio and Textbelt
// Twilio: https://www.twilio.com/docs/sms/api
// Textbelt: https://textbelt.com/

use axum::{Json, Router, routing::post, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use reqwest;

// SMS Provider enum
#[derive(Debug, Clone)]
pub enum SmsProvider {
    Twilio,
    Textbelt,
}

// SMS configuration supporting multiple providers
pub struct SmsConfig {
    pub provider: SmsProvider,
    // Twilio config
    pub twilio_account_sid: Option<String>,
    pub twilio_auth_token: Option<String>,
    pub twilio_from_phone: Option<String>,
    // Textbelt config
    pub textbelt_api_key: Option<String>,
    pub textbelt_api_url: String,
}

impl SmsConfig {
    pub fn from_env() -> Option<Self> {
        let provider_str = std::env::var("SMS_PROVIDER")
            .unwrap_or_else(|_| "textbelt".to_string())
            .to_lowercase();

        let provider = match provider_str.as_str() {
            "twilio" => SmsProvider::Twilio,
            _ => SmsProvider::Textbelt,
        };

        Some(Self {
            provider,
            twilio_account_sid: std::env::var("TWILIO_ACCOUNT_SID").ok(),
            twilio_auth_token: std::env::var("TWILIO_AUTH_TOKEN").ok(),
            twilio_from_phone: std::env::var("TWILIO_FROM_PHONE").ok(),
            textbelt_api_key: std::env::var("TEXTBELT_API_KEY").ok(),
            textbelt_api_url: std::env::var("TEXTBELT_API_URL")
                .unwrap_or_else(|_| "https://textbelt.com/text".to_string()),
        })
    }
}

// Add SMS config to AppState
impl AppState {
    pub fn sms_config(&self) -> Option<SmsConfig> {
        SmsConfig::from_env()
    }
}

// Request structures
#[derive(Deserialize)]
pub struct SendSmsRequest {
    pub phone: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct OrderConfirmationSmsRequest {
    pub phone: String,
    pub order_id: String,
    pub order_total: f64,
}

#[derive(Deserialize)]
pub struct ShippingUpdateSmsRequest {
    pub phone: String,
    pub order_id: String,
    pub tracking_number: String,
    pub carrier: String,
}

#[derive(Deserialize)]
pub struct DeliveryNotificationSmsRequest {
    pub phone: String,
    pub order_id: String,
}

// Response structure
#[derive(Serialize)]
pub struct SmsResponse {
    pub success: bool,
    pub message: String,
    pub quota_remaining: Option<i32>,
}

// Textbelt API response structure
#[derive(Deserialize)]
struct TextbeltResponse {
    pub success: bool,
    #[serde(rename = "quotaRemaining")]
    pub quota_remaining: Option<i32>,
    pub text_id: Option<String>,
    pub error: Option<String>,
}

// Twilio API response structure
#[derive(Deserialize)]
struct TwilioResponse {
    pub sid: Option<String>,
    pub status: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

// Textbelt SMS routes
pub fn textbelt_sms_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/sms/send", post(send_sms))
        .route("/api/sms/order-confirmation", post(send_order_confirmation))
        .route("/api/sms/shipping-update", post(send_shipping_update))
        .route("/api/sms/delivery-notification", post(send_delivery_notification))
        .with_state(app_state)
}

// Helper function to send SMS via Textbelt
async fn send_textbelt_sms(
    config: &SmsConfig,
    phone: &str,
    message: &str,
) -> Result<TextbeltResponse, String> {
    let client = reqwest::Client::new();

    let api_key = config.textbelt_api_key.as_ref()
        .ok_or_else(|| "TEXTBELT_API_KEY not configured".to_string())?;

    let params = [
        ("phone", phone),
        ("message", message),
        ("key", api_key.as_str()),
    ];

    let response = client
        .post(&config.textbelt_api_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send SMS: {}", e))?;

    let textbelt_response: TextbeltResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Textbelt response: {}", e))?;

    if !textbelt_response.success {
        return Err(textbelt_response.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    Ok(textbelt_response)
}

// Helper function to send SMS via Twilio
async fn send_twilio_sms(
    config: &SmsConfig,
    phone: &str,
    message: &str,
) -> Result<TwilioResponse, String> {
    let account_sid = config.twilio_account_sid.as_ref()
        .ok_or_else(|| "TWILIO_ACCOUNT_SID not configured".to_string())?;
    let auth_token = config.twilio_auth_token.as_ref()
        .ok_or_else(|| "TWILIO_AUTH_TOKEN not configured".to_string())?;
    let from_phone = config.twilio_from_phone.as_ref()
        .ok_or_else(|| "TWILIO_FROM_PHONE not configured".to_string())?;

    // Ensure from_phone has + prefix for Twilio
    let from_phone_formatted = if from_phone.starts_with('+') {
        from_phone.to_string()
    } else {
        format!("+{}", from_phone)
    };

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
        account_sid
    );

    let params = [
        ("To", phone),
        ("From", from_phone_formatted.as_str()),
        ("Body", message),
    ];

    let response = client
        .post(&url)
        .basic_auth(account_sid.as_str(), Some(auth_token.as_str()))
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to send SMS via Twilio: {}", e))?;

    let status = response.status();

    // If error, get text body for better error message
    if !status.is_success() {
        let error_text = response.text().await
            .unwrap_or_else(|_| format!("HTTP {}", status));
        return Err(format!("Twilio API error ({}): {}", status, error_text));
    }

    let twilio_response: TwilioResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Twilio response: {}", e))?;

    Ok(twilio_response)
}

// Unified SMS sending function that routes to the correct provider
async fn send_sms_via_provider(
    config: &SmsConfig,
    phone: &str,
    message: &str,
) -> Result<(bool, Option<i32>), String> {
    match config.provider {
        SmsProvider::Twilio => {
            send_twilio_sms(config, phone, message).await?;
            Ok((true, None)) // Twilio doesn't return quota info
        }
        SmsProvider::Textbelt => {
            let response = send_textbelt_sms(config, phone, message).await?;
            Ok((response.success, response.quota_remaining))
        }
    }
}

// Helper function to validate and format phone number
fn format_phone_number(phone: &str) -> Result<String, String> {
    // Remove all non-digit characters
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

    // Check length and format for E.164 (Twilio requires +1XXXXXXXXXX)
    if digits.len() == 10 {
        // US number without country code - add +1
        Ok(format!("+1{}", digits))
    } else if digits.len() == 11 && digits.starts_with('1') {
        // US number with country code - add +
        Ok(format!("+{}", digits))
    } else if digits.len() >= 10 {
        // International number - add + prefix
        Ok(format!("+{}", digits))
    } else {
        Err("Invalid phone number format. Must be at least 10 digits.".to_string())
    }
}

// Send generic SMS
async fn send_sms(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendSmsRequest>,
) -> Result<Json<SmsResponse>, (StatusCode, String)> {
    let config = state.sms_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "SMS not configured".to_string()))?;

    let formatted_phone = format_phone_number(&payload.phone)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let (success, quota_remaining) = send_sms_via_provider(&config, &formatted_phone, &payload.message)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("SMS error: {}", e)))?;

    Ok(Json(SmsResponse {
        success,
        message: format!("SMS sent to {}", payload.phone),
        quota_remaining,
    }))
}

// Send order confirmation SMS
async fn send_order_confirmation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrderConfirmationSmsRequest>,
) -> Result<Json<SmsResponse>, (StatusCode, String)> {
    let config = state.sms_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "SMS not configured".to_string()))?;

    let formatted_phone = format_phone_number(&payload.phone)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let message = format!(
        "R-Com Order Confirmed! Order #{} - Total: ${:.2}. Thank you for your purchase! Track your order at rcom.store/orders/{}",
        payload.order_id,
        payload.order_total,
        payload.order_id
    );

    let (success, quota_remaining) = send_sms_via_provider(&config, &formatted_phone, &message)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("SMS error: {}", e)))?;

    Ok(Json(SmsResponse {
        success,
        message: format!("Order confirmation SMS sent to {}", payload.phone),
        quota_remaining,
    }))
}

// Send shipping update SMS
async fn send_shipping_update(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ShippingUpdateSmsRequest>,
) -> Result<Json<SmsResponse>, (StatusCode, String)> {
    let config = state.sms_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "SMS not configured".to_string()))?;

    let formatted_phone = format_phone_number(&payload.phone)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let message = format!(
        "R-Com Shipping Update! Order #{} has shipped via {}. Tracking: {}. Estimated delivery 3-5 business days.",
        payload.order_id,
        payload.carrier,
        payload.tracking_number
    );

    let (success, quota_remaining) = send_sms_via_provider(&config, &formatted_phone, &message)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("SMS error: {}", e)))?;

    Ok(Json(SmsResponse {
        success,
        message: format!("Shipping update SMS sent to {}", payload.phone),
        quota_remaining,
    }))
}

// Send delivery notification SMS
async fn send_delivery_notification(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeliveryNotificationSmsRequest>,
) -> Result<Json<SmsResponse>, (StatusCode, String)> {
    let config = state.sms_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "SMS not configured".to_string()))?;

    let formatted_phone = format_phone_number(&payload.phone)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let message = format!(
        "R-Com Delivery Complete! Your order #{} has been delivered. Enjoy your purchase! Questions? Contact support@rcom.store",
        payload.order_id
    );

    let (success, quota_remaining) = send_sms_via_provider(&config, &formatted_phone, &message)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("SMS error: {}", e)))?;

    Ok(Json(SmsResponse {
        success,
        message: format!("Delivery notification SMS sent to {}", payload.phone),
        quota_remaining,
    }))
}
