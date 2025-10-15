// Brevo Email Marketing Integration
// Provides transactional email and marketing campaign functionality via Brevo API
// API Documentation: https://developers.brevo.com/docs/send-a-transactional-email

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

// ============================================================================
// Configuration
// ============================================================================

#[derive(Clone, Debug)]
pub struct BrevoConfig {
    pub api_key: String,
    pub api_base_url: String,
    pub from_email: String,
    pub from_name: String,
}

impl BrevoConfig {
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("BREVO_API_KEY").ok()?;
        let from_email = std::env::var("BREVO_FROM_EMAIL")
            .unwrap_or_else(|_| "noreply@yourdomain.com".to_string());
        let from_name = std::env::var("BREVO_FROM_NAME")
            .unwrap_or_else(|_| "R-Com Store".to_string());
        let api_base_url = std::env::var("BREVO_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.brevo.com/v3".to_string());

        Some(Self {
            api_key,
            api_base_url,
            from_email,
            from_name,
        })
    }
}

// ============================================================================
// Request/Response Structures
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailAddress {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionalEmailRequest {
    pub sender: EmailAddress,
    pub to: Vec<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "htmlContent")]
    pub html_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "textContent")]
    pub text_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrevoApiResponse {
    #[serde(rename = "messageId")]
    pub message_id: Option<String>,
    pub message: Option<String>,
}

// API request structures for our endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub html_content: String,
    pub text_content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMarketingCampaignRequest {
    pub recipients: Vec<EmailAddress>,
    pub subject: String,
    pub html_content: String,
    pub campaign_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddContactRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub list_ids: Option<Vec<i64>>,
}

// ============================================================================
// Core Brevo Client
// ============================================================================

pub struct BrevoClient {
    config: BrevoConfig,
    client: Client,
}

impl BrevoClient {
    pub fn new(config: BrevoConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    /// Send a transactional email via Brevo API
    pub async fn send_transactional_email(
        &self,
        request: SendTransactionalEmailRequest,
    ) -> Result<BrevoApiResponse, String> {
        let url = format!("{}/smtp/email", self.config.api_base_url);

        let response = self
            .client
            .post(&url)
            .header("accept", "application/json")
            .header("api-key", &self.config.api_key)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Brevo: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if status.is_success() {
            serde_json::from_str::<BrevoApiResponse>(&body)
                .map_err(|e| format!("Failed to parse Brevo response: {}. Body: {}", e, body))
        } else {
            Err(format!(
                "Brevo API returned error status {}: {}",
                status, body
            ))
        }
    }

    /// Add or update a contact in Brevo
    pub async fn add_contact(
        &self,
        email: &str,
        attributes: Option<serde_json::Value>,
        list_ids: Option<Vec<i64>>,
    ) -> Result<serde_json::Value, String> {
        let url = format!("{}/contacts", self.config.api_base_url);

        let mut body = json!({
            "email": email,
            "updateEnabled": true,
        });

        if let Some(attrs) = attributes {
            body["attributes"] = attrs;
        }

        if let Some(lists) = list_ids {
            body["listIds"] = json!(lists);
        }

        let response = self
            .client
            .post(&url)
            .header("accept", "application/json")
            .header("api-key", &self.config.api_key)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to add contact to Brevo: {}", e))?;

        let status = response.status();
        let response_body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if status.is_success() || status.as_u16() == 204 {
            Ok(json!({"success": true, "email": email}))
        } else {
            Err(format!(
                "Brevo API returned error status {}: {}",
                status, response_body
            ))
        }
    }

    /// Get contact lists from Brevo
    pub async fn get_contact_lists(&self) -> Result<serde_json::Value, String> {
        let url = format!("{}/contacts/lists", self.config.api_base_url);

        let response = self
            .client
            .get(&url)
            .header("accept", "application/json")
            .header("api-key", &self.config.api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to get contact lists from Brevo: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        if status.is_success() {
            serde_json::from_str(&body)
                .map_err(|e| format!("Failed to parse Brevo response: {}", e))
        } else {
            Err(format!(
                "Brevo API returned error status {}: {}",
                status, body
            ))
        }
    }
}

// ============================================================================
// Axum Route Handlers
// ============================================================================

/// Send a transactional email via Brevo
pub async fn send_email_handler(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<SendEmailRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config = BrevoConfig::from_env().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Brevo not configured. Set BREVO_API_KEY in environment.".to_string(),
    ))?;

    let client = BrevoClient::new(config.clone());

    let brevo_request = SendTransactionalEmailRequest {
        sender: EmailAddress {
            email: config.from_email.clone(),
            name: Some(config.from_name.clone()),
        },
        to: vec![EmailAddress {
            email: request.to_email.clone(),
            name: request.to_name.clone(),
        }],
        reply_to: None,
        subject: Some(request.subject),
        html_content: Some(request.html_content),
        text_content: request.text_content,
        tags: request.tags,
        params: None,
    };

    match client.send_transactional_email(brevo_request).await {
        Ok(response) => {
            println!("✓ Email sent successfully via Brevo: {:?}", response.message_id);
            Ok((
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message_id": response.message_id,
                    "provider": "brevo"
                })),
            ))
        }
        Err(e) => {
            eprintln!("✗ Failed to send email via Brevo: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": e
                })),
            ))
        }
    }
}

/// Add a contact to Brevo mailing list
pub async fn add_contact_handler(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<AddContactRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config = BrevoConfig::from_env().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Brevo not configured. Set BREVO_API_KEY in environment.".to_string(),
    ))?;

    let client = BrevoClient::new(config);

    // Build attributes JSON
    let mut attributes = json!({});
    if let Some(first) = request.first_name {
        attributes["FIRSTNAME"] = json!(first);
    }
    if let Some(last) = request.last_name {
        attributes["LASTNAME"] = json!(last);
    }
    if let Some(custom_attrs) = request.attributes {
        if let Some(obj) = custom_attrs.as_object() {
            for (key, value) in obj {
                attributes[key] = value.clone();
            }
        }
    }

    match client
        .add_contact(&request.email, Some(attributes), request.list_ids)
        .await
    {
        Ok(response) => {
            println!("✓ Contact added to Brevo: {}", request.email);
            Ok((StatusCode::OK, Json(response)))
        }
        Err(e) => {
            eprintln!("✗ Failed to add contact to Brevo: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": e
                })),
            ))
        }
    }
}

/// Get all contact lists from Brevo
pub async fn get_lists_handler(
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let config = BrevoConfig::from_env().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "Brevo not configured. Set BREVO_API_KEY in environment.".to_string(),
    ))?;

    let client = BrevoClient::new(config);

    match client.get_contact_lists().await {
        Ok(lists) => Ok((StatusCode::OK, Json(lists))),
        Err(e) => {
            eprintln!("✗ Failed to get contact lists from Brevo: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": e
                })),
            ))
        }
    }
}

/// Send a welcome email using Brevo template
pub async fn send_welcome_email(
    email: &str,
    name: Option<&str>,
) -> Result<BrevoApiResponse, String> {
    let config = BrevoConfig::from_env()
        .ok_or_else(|| "Brevo not configured".to_string())?;

    let client = BrevoClient::new(config.clone());

    let html_content = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #4CAF50; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9f9f9; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
        .button {{ background: #4CAF50; color: white; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block; margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to R-Com!</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>Thank you for joining R-Com! We're excited to have you as part of our community.</p>
            <p>As a welcome gift, here's a special discount code for your first purchase:</p>
            <p style="text-align: center; font-size: 24px; font-weight: bold; color: #4CAF50;">WELCOME10</p>
            <p>Use this code at checkout to get 10% off your first order!</p>
            <a href="https://your-domain.com/shop" class="button">Start Shopping</a>
            <p>If you have any questions, feel free to reach out to our support team.</p>
        </div>
        <div class="footer">
            <p>© 2025 R-Com Store. All rights reserved.</p>
            <p>You're receiving this email because you signed up for R-Com.</p>
        </div>
    </div>
</body>
</html>
        "#,
        name.unwrap_or("there")
    );

    let request = SendTransactionalEmailRequest {
        sender: EmailAddress {
            email: config.from_email.clone(),
            name: Some(config.from_name.clone()),
        },
        to: vec![EmailAddress {
            email: email.to_string(),
            name: name.map(String::from),
        }],
        reply_to: None,
        subject: Some("Welcome to R-Com!".to_string()),
        html_content: Some(html_content),
        text_content: Some(format!(
            "Hi {},\n\nThank you for joining R-Com! Use code WELCOME10 for 10% off your first order.\n\nHappy shopping!",
            name.unwrap_or("there")
        )),
        tags: Some(vec!["welcome".to_string(), "onboarding".to_string()]),
        params: None,
    };

    client.send_transactional_email(request).await
}

// ============================================================================
// Router Function for Axum Integration
// ============================================================================

use axum::{routing::{post, get}, Router};

/// Create Brevo email marketing routes
pub fn brevo_email_routes(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/brevo/send-email", post(send_email_handler))
        .route("/api/brevo/add-contact", post(add_contact_handler))
        .route("/api/brevo/lists", get(get_lists_handler))
}
