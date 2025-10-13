// Lettre Transactional Email Module
// Sends transactional emails via SMTP using the lettre.rs library
// https://github.com/lettre/lettre

use axum::{Json, Router, routing::post, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
};

// Email configuration
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            smtp_host: std::env::var("SMTP_HOST").ok()?,
            smtp_port: std::env::var("SMTP_PORT").ok()?.parse().ok()?,
            smtp_username: std::env::var("SMTP_USERNAME").ok()?,
            smtp_password: std::env::var("SMTP_PASSWORD").ok()?,
            from_email: std::env::var("FROM_EMAIL").ok()?,
            from_name: std::env::var("FROM_NAME").unwrap_or_else(|_| "R-Com Store".to_string()),
        })
    }
}

// Request structures
#[derive(Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body: String,
    pub html: Option<bool>,
}

#[derive(Deserialize)]
pub struct OrderConfirmationRequest {
    pub to: String,
    pub to_name: Option<String>,
    pub order_id: String,
    pub order_total: f64,
    pub items: Vec<OrderItem>,
}

#[derive(Deserialize, Serialize)]
pub struct OrderItem {
    pub name: String,
    pub quantity: i32,
    pub price: f64,
}

#[derive(Deserialize)]
pub struct PasswordResetRequest {
    pub to: String,
    pub to_name: Option<String>,
    pub reset_token: String,
    pub reset_url: String,
}

#[derive(Deserialize)]
pub struct WelcomeEmailRequest {
    pub to: String,
    pub to_name: Option<String>,
}

// Response structure
#[derive(Serialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
}

// Add email config to AppState
impl AppState {
    pub fn email_config(&self) -> Option<EmailConfig> {
        EmailConfig::from_env()
    }
}

// Lettre email routes
pub fn lettre_email_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/email/send", post(send_email))
        .route("/api/email/order-confirmation", post(send_order_confirmation))
        .route("/api/email/password-reset", post(send_password_reset))
        .route("/api/email/welcome", post(send_welcome))
        .with_state(app_state)
}

// Helper function to create SMTP transport
fn create_mailer(config: &EmailConfig) -> Result<SmtpTransport, String> {
    let creds = Credentials::new(
        config.smtp_username.clone(),
        config.smtp_password.clone(),
    );

    // Gmail requires STARTTLS on port 587
    let mailer = SmtpTransport::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    Ok(mailer)
}

// Helper function to parse email address
fn parse_mailbox(email: &str, name: Option<String>) -> Result<Mailbox, String> {
    if let Some(n) = name {
        format!("{} <{}>", n, email)
            .parse()
            .map_err(|e| format!("Invalid email address: {}", e))
    } else {
        email.parse().map_err(|e| format!("Invalid email address: {}", e))
    }
}

// Send generic email
async fn send_email(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SendEmailRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let config = state.email_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Email not configured".to_string()))?;

    let from = parse_mailbox(&config.from_email, Some(config.from_name.clone()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let to = parse_mailbox(&payload.to, payload.to_name)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let email = if payload.html.unwrap_or(false) {
        Message::builder()
            .from(from)
            .to(to)
            .subject(&payload.subject)
            .header(ContentType::TEXT_HTML)
            .body(payload.body.clone())
    } else {
        Message::builder()
            .from(from)
            .to(to)
            .subject(&payload.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(payload.body.clone())
    }.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build email: {}", e)))?;

    let mailer = create_mailer(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    mailer.send(&email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send email: {}", e)))?;

    Ok(Json(EmailResponse {
        success: true,
        message: format!("Email sent to {}", payload.to),
    }))
}

// Send order confirmation email
async fn send_order_confirmation(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OrderConfirmationRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let config = state.email_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Email not configured".to_string()))?;

    let from = parse_mailbox(&config.from_email, Some(config.from_name.clone()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let to = parse_mailbox(&payload.to, payload.to_name.clone())
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Build HTML email body
    let mut items_html = String::new();
    for item in &payload.items {
        items_html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>${:.2}</td></tr>",
            item.name, item.quantity, item.price
        ));
    }

    let html_body = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #1976d2; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9f9f9; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
        table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        th, td {{ padding: 10px; text-align: left; border-bottom: 1px solid #ddd; }}
        th {{ background: #f0f0f0; }}
        .total {{ font-size: 18px; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Order Confirmation</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>Thank you for your order! Your order has been confirmed.</p>
            <p><strong>Order ID:</strong> {}</p>

            <table>
                <thead>
                    <tr>
                        <th>Item</th>
                        <th>Quantity</th>
                        <th>Price</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>

            <p class="total">Total: ${:.2}</p>

            <p>We'll send you a shipping confirmation email as soon as your order ships.</p>
        </div>
        <div class="footer">
            <p>© 2025 R-Com Store. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
        "#,
        payload.to_name.as_deref().unwrap_or("Customer"),
        payload.order_id,
        items_html,
        payload.order_total
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(format!("Order Confirmation - #{}", payload.order_id))
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build email: {}", e)))?;

    let mailer = create_mailer(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    mailer.send(&email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send email: {}", e)))?;

    Ok(Json(EmailResponse {
        success: true,
        message: format!("Order confirmation sent to {}", payload.to),
    }))
}

// Send password reset email
async fn send_password_reset(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PasswordResetRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let config = state.email_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Email not configured".to_string()))?;

    let from = parse_mailbox(&config.from_email, Some(config.from_name.clone()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let to = parse_mailbox(&payload.to, payload.to_name.clone())
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let html_body = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #1976d2; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9f9f9; }}
        .button {{ display: inline-block; padding: 12px 24px; background: #1976d2; color: white; text-decoration: none; border-radius: 4px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Password Reset Request</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>We received a request to reset your password. Click the button below to create a new password:</p>
            <p style="text-align: center;">
                <a href="{}" class="button">Reset Password</a>
            </p>
            <p><strong>This link will expire in 24 hours.</strong></p>
            <p>If you didn't request a password reset, please ignore this email.</p>
        </div>
        <div class="footer">
            <p>© 2025 R-Com Store. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
        "#,
        payload.to_name.as_deref().unwrap_or("there"),
        payload.reset_url
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Password Reset Request")
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build email: {}", e)))?;

    let mailer = create_mailer(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    mailer.send(&email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send email: {}", e)))?;

    Ok(Json(EmailResponse {
        success: true,
        message: format!("Password reset email sent to {}", payload.to),
    }))
}

// Send welcome email
async fn send_welcome(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WelcomeEmailRequest>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let config = state.email_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Email not configured".to_string()))?;

    let from = parse_mailbox(&config.from_email, Some(config.from_name.clone()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let to = parse_mailbox(&payload.to, payload.to_name.clone())
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let html_body = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #1976d2; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9f9f9; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to R-Com!</h1>
        </div>
        <div class="content">
            <p>Hi {},</p>
            <p>Welcome to R-Com Store! We're excited to have you as part of our community.</p>
            <p>Start exploring our products and enjoy shopping with us!</p>
            <p>If you have any questions, feel free to reach out to our support team.</p>
        </div>
        <div class="footer">
            <p>© 2025 R-Com Store. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
        "#,
        payload.to_name.as_deref().unwrap_or("there")
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject("Welcome to R-Com Store!")
        .header(ContentType::TEXT_HTML)
        .body(html_body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to build email: {}", e)))?;

    let mailer = create_mailer(&config)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    mailer.send(&email)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to send email: {}", e)))?;

    Ok(Json(EmailResponse {
        success: true,
        message: format!("Welcome email sent to {}", payload.to),
    }))
}
