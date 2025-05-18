use async_trait::async_trait;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, transport::smtp::authentication::Credentials};
use std::sync::Arc;
use reqwest::Client;
use serde_json;

#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

pub struct SmtpEmailService {
    mailer: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    from: String,
}

impl SmtpEmailService {
    pub fn new(smtp_server: &str, smtp_port: u16, smtp_user: &str, smtp_pass: &str, from: &str) -> Self {
        let creds = Credentials::new(smtp_user.to_string(), smtp_pass.to_string());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_server)
            .unwrap()
            .port(smtp_port)
            .credentials(creds)
            .build();
        Self {
            mailer: Arc::new(mailer),
            from: from.to_string(),
        }
    }
}

#[async_trait]
impl EmailService for SmtpEmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let email = Message::builder()
            .from(self.from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
            .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;
        self.mailer
            .send(email)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;
        Ok(())
    }
}

// Stub for MailChimp integration
pub struct MailChimpEmailService;

#[async_trait]
impl EmailService for MailChimpEmailService {
    async fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), String> {
        // TODO: Implement MailChimp API integration
        Err("MailChimp integration not yet implemented".to_string())
    }
}

// Stub for SalesForce Marketing Cloud integration
pub struct SalesForceEmailService;

#[async_trait]
impl EmailService for SalesForceEmailService {
    async fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), String> {
        // TODO: Implement SalesForce Marketing Cloud API integration
        Err("SalesForce Marketing Cloud integration not yet implemented".to_string())
    }
}

// Stub for Acoustic integration
pub struct AcousticEmailService;

#[async_trait]
impl EmailService for AcousticEmailService {
    async fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), String> {
        // TODO: Implement Acoustic API integration
        Err("Acoustic integration not yet implemented".to_string())
    }
}

// Resend SMTP uses SmtpEmailService, but we add a ResendEmailService for API
pub struct ResendApiEmailService {
    api_key: String,
    from: String,
    client: Client,
}

impl ResendApiEmailService {
    pub fn new(api_key: &str, from: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            from: from.to_string(),
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EmailService for ResendApiEmailService {
    async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        let url = "https://api.resend.com/emails";
        let payload = serde_json::json!({
            "from": self.from,
            "to": to,
            "subject": subject,
            "html": body,
        });
        let resp = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Failed to send request to Resend API: {}", e))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(format!("Resend API error: {} - {}", resp.status(), text))
        }
    }
}

// Resend SMTP: just use SmtpEmailService with Resend's SMTP credentials
// See https://resend.com/docs/email-sending/sending-email-with-smtp for details 