// Letre Email Marketing Integration Module
// Handles email marketing campaigns, subscriber management, and automated emails

use axum::{Json, Router, routing::{post, get}, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;

// Letre API client configuration
pub struct LetreClient {
    pub api_key: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

impl LetreClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

// Email subscriber structures
#[derive(Deserialize, Serialize)]
pub struct EmailSubscriber {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_fields: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub source: Option<String>, // "checkout", "newsletter", "account_creation"
}

#[derive(Serialize)]
pub struct LetreSubscribeRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub tags: Vec<String>,
    pub double_optin: bool,
    pub send_welcome: bool,
}

// Email campaign structures
#[derive(Deserialize)]
pub struct EmailCampaignRequest {
    pub subject: String,
    pub content: String,
    pub recipient_tags: Vec<String>,
    pub send_immediately: Option<bool>,
    pub scheduled_at: Option<String>, // ISO 8601 datetime
}

#[derive(Serialize)]
pub struct LetreCampaignRequest {
    pub name: String,
    pub subject: String,
    pub content: String,
    pub recipient_filter: LetreRecipientFilter,
    pub send_immediately: bool,
    pub scheduled_at: Option<String>,
}

#[derive(Serialize)]
pub struct LetreRecipientFilter {
    pub tags: Vec<String>,
    pub include_all_tags: bool,
}

// Automated email triggers
#[derive(Deserialize)]
pub struct TriggerEmailRequest {
    pub email: String,
    pub template_id: String,
    pub variables: Option<serde_json::Value>,
}

#[derive(Serialize)]
pub struct LetreTriggerEmailRequest {
    pub recipient: String,
    pub template_id: String,
    pub variables: serde_json::Value,
}

// Response structures
#[derive(Serialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
    pub id: Option<String>,
}

// Add Letre client to AppState
impl AppState {
    pub fn letre_client(&self) -> Option<LetreClient> {
        let api_key = std::env::var("LETRE_API_KEY").ok()?;
        let base_url = std::env::var("LETRE_API_URL").unwrap_or_else(|_| "https://api.letre.io".to_string());
        
        Some(LetreClient::new(api_key, base_url))
    }
}

// Letre email marketing routes
pub fn letre_email_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/email/subscribe", post(subscribe_email))
        .route("/api/email/unsubscribe", post(unsubscribe_email))
        .route("/api/email/campaign", post(send_campaign))
        .route("/api/email/trigger", post(trigger_email))
        .route("/api/email/subscribers", get(list_subscribers))
        .with_state(app_state)
}

// Subscribe email handler
async fn subscribe_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubscribeRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let letre_client = state.letre_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Letre client not configured".to_string()))?;

    // Determine tags based on source
    let mut tags = vec!["customer".to_string()];
    if let Some(source) = &payload.source {
        tags.push(source.clone());
    }

    let letre_request = LetreSubscribeRequest {
        email: payload.email.clone(),
        first_name: payload.first_name,
        last_name: payload.last_name,
        tags,
        double_optin: true, // Require email confirmation
        send_welcome: true, // Send welcome email
    };

    // Make request to Letre API
    let response = letre_client
        .client
        .post(&format!("{}/v1/subscribers", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .header("Content-Type", "application/json")
        .json(&letre_request)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Letre API request failed: {}", e)))?;

    if response.status().is_success() {
        Ok(Json(EmailResponse {
            success: true,
            message: format!("Successfully subscribed {}", payload.email),
            id: None,
        }))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((StatusCode::BAD_REQUEST, format!("Letre API error: {}", error_text)))
    }
}

// Unsubscribe email handler
async fn unsubscribe_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let letre_client = state.letre_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Letre client not configured".to_string()))?;

    let email = payload.get("email")
        .and_then(|e| e.as_str())
        .ok_or((StatusCode::BAD_REQUEST, "Email is required".to_string()))?;

    // Make request to Letre API
    let response = letre_client
        .client
        .delete(&format!("{}/v1/subscribers/{}", letre_client.base_url, email))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Letre API request failed: {}", e)))?;

    if response.status().is_success() {
        Ok(Json(EmailResponse {
            success: true,
            message: format!("Successfully unsubscribed {}", email),
            id: None,
        }))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((StatusCode::BAD_REQUEST, format!("Letre API error: {}", error_text)))
    }
}

// Send email campaign handler
async fn send_campaign(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EmailCampaignRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let letre_client = state.letre_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Letre client not configured".to_string()))?;

    let letre_request = LetreCampaignRequest {
        name: format!("Campaign: {}", payload.subject),
        subject: payload.subject,
        content: payload.content,
        recipient_filter: LetreRecipientFilter {
            tags: payload.recipient_tags,
            include_all_tags: false, // Match any of the tags
        },
        send_immediately: payload.send_immediately.unwrap_or(true),
        scheduled_at: payload.scheduled_at,
    };

    // Make request to Letre API
    let response = letre_client
        .client
        .post(&format!("{}/v1/campaigns", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .header("Content-Type", "application/json")
        .json(&letre_request)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Letre API request failed: {}", e)))?;

    if response.status().is_success() {
        let response_data: serde_json::Value = response.json().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)))?;
        
        let campaign_id = response_data.get("id")
            .and_then(|id| id.as_str())
            .map(|s| s.to_string());

        Ok(Json(EmailResponse {
            success: true,
            message: "Campaign sent successfully".to_string(),
            id: campaign_id,
        }))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((StatusCode::BAD_REQUEST, format!("Letre API error: {}", error_text)))
    }
}

// Trigger automated email handler
async fn trigger_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<TriggerEmailRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let letre_client = state.letre_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Letre client not configured".to_string()))?;

    let letre_request = LetreTriggerEmailRequest {
        recipient: payload.email.clone(),
        template_id: payload.template_id,
        variables: payload.variables.unwrap_or_else(|| serde_json::json!({})),
    };

    // Make request to Letre API
    let response = letre_client
        .client
        .post(&format!("{}/v1/emails/trigger", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .header("Content-Type", "application/json")
        .json(&letre_request)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Letre API request failed: {}", e)))?;

    if response.status().is_success() {
        Ok(Json(EmailResponse {
            success: true,
            message: format!("Triggered email sent to {}", payload.email),
            id: None,
        }))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((StatusCode::BAD_REQUEST, format!("Letre API error: {}", error_text)))
    }
}

// List subscribers handler (admin only)
async fn list_subscribers(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let letre_client = state.letre_client()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Letre client not configured".to_string()))?;

    // Make request to Letre API
    let response = letre_client
        .client
        .get(&format!("{}/v1/subscribers", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Letre API request failed: {}", e)))?;

    if response.status().is_success() {
        let subscribers: serde_json::Value = response.json().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)))?;
        Ok(Json(subscribers))
    } else {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        Err((StatusCode::BAD_REQUEST, format!("Letre API error: {}", error_text)))
    }
}

// Helper functions for common email marketing tasks

// Subscribe user after successful purchase
pub async fn subscribe_after_purchase(
    letre_client: &LetreClient,
    email: String,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<(), String> {
    let request = LetreSubscribeRequest {
        email: email.clone(),
        first_name,
        last_name,
        tags: vec!["customer".to_string(), "purchased".to_string()],
        double_optin: false, // Skip double opt-in for customers
        send_welcome: false, // Don't send welcome email, send purchase confirmation instead
    };

    let response = letre_client
        .client
        .post(&format!("{}/v1/subscribers", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to subscribe customer: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Letre API error: {}", response.status()));
    }

    Ok(())
}

// Send order confirmation email
pub async fn send_order_confirmation(
    letre_client: &LetreClient,
    email: String,
    order_details: serde_json::Value,
) -> Result<(), String> {
    let request = LetreTriggerEmailRequest {
        recipient: email,
        template_id: "order_confirmation".to_string(), // Template ID in Letre
        variables: order_details,
    };

    let response = letre_client
        .client
        .post(&format!("{}/v1/emails/trigger", letre_client.base_url))
        .header("Authorization", format!("Bearer {}", letre_client.api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send order confirmation: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Letre API error: {}", response.status()));
    }

    Ok(())
}