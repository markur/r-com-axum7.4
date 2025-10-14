// Stripe Webhook Handler
// Processes Stripe webhook events for payment intents and charges
// Implements signature verification for security

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use stripe::{Event, EventObject, EventType, Webhook};

use crate::AppState;
use super::{
    log_webhook_event, mark_webhook_processed, create_order, is_event_processed,
    CreateWebhookEvent, CreateOrder, PaymentProvider, OrderStatus,
};

// Stripe webhook endpoint handler
pub async fn handle_stripe_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get Stripe signature from headers
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "Missing stripe-signature header".to_string(),
        ))?;

    // Get webhook secret from environment
    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .unwrap_or_else(|_| "whsec_test_secret".to_string());

    // Verify webhook signature and construct event
    let event = Webhook::construct_event(&body, signature, &webhook_secret)
        .map_err(|e| {
            eprintln!("Stripe webhook signature verification failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                format!("Webhook signature verification failed: {}", e),
            )
        })?;

    // Check if we've already processed this event (idempotency)
    let event_id = event.id.as_str();
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
        provider: PaymentProvider::Stripe,
        event_type: event.type_.to_string(),
        event_id: event_id.to_string(),
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
    let result = match event.type_ {
        EventType::PaymentIntentSucceeded => {
            handle_payment_intent_succeeded(&state, &event, webhook_id).await
        }
        EventType::ChargeSucceeded => {
            handle_charge_succeeded(&state, &event, webhook_id).await
        }
        EventType::CheckoutSessionCompleted => {
            handle_checkout_session_completed(&state, &event, webhook_id).await
        }
        _ => {
            // For other events, just log and mark as processed
            println!("Received Stripe event type: {:?}", event.type_);
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

// Handle payment_intent.succeeded event
async fn handle_payment_intent_succeeded(
    state: &Arc<AppState>,
    event: &Event,
    webhook_id: uuid::Uuid,
) -> Result<(), String> {
    let payment_intent = match &event.data.object {
        EventObject::PaymentIntent(pi) => pi,
        _ => return Err("Expected PaymentIntent object".to_string()),
    };

    println!(
        "Payment succeeded! PaymentIntent ID: {}, Amount: {} {}",
        payment_intent.id,
        payment_intent.amount,
        payment_intent.currency
    );

    // Extract customer information
    let customer_email = payment_intent
        .receipt_email
        .as_ref()
        .map(|e| e.to_string());

    // Create order record
    let order = CreateOrder {
        payment_provider: PaymentProvider::Stripe,
        payment_id: payment_intent.id.to_string(),
        payment_intent_id: Some(payment_intent.id.to_string()),
        customer_email: customer_email.clone(),
        customer_name: None, // Could extract from billing details if available
        total_amount: payment_intent.amount,
        currency: payment_intent.currency.to_string().to_uppercase(),
        status: OrderStatus::Completed,
        webhook_event_id: Some(webhook_id),
    };

    let order_id = create_order(&state.pool, order)
        .await
        .map_err(|e| format!("Failed to create order: {}", e))?;

    println!("Created order with ID: {}", order_id);

    // Send order confirmation email
    if let Some(email) = customer_email {
        send_order_confirmation_email(&email, &payment_intent.id.to_string(), payment_intent.amount)
            .await;
    }

    Ok(())
}

// Handle charge.succeeded event
async fn handle_charge_succeeded(
    state: &Arc<AppState>,
    event: &Event,
    webhook_id: uuid::Uuid,
) -> Result<(), String> {
    let charge = match &event.data.object {
        EventObject::Charge(c) => c,
        _ => return Err("Expected Charge object".to_string()),
    };

    println!(
        "Charge succeeded! Charge ID: {}, Amount: {}",
        charge.id, charge.amount
    );

    // Check if order already exists for this payment intent
    let payment_intent_str = charge.payment_intent
        .as_ref()
        .map(|pi| pi.id().to_string());

    if let Some(pi_str) = &payment_intent_str {
        // Check if we already created an order for this payment intent
        let existing = sqlx::query!(
            "SELECT id FROM orders WHERE payment_intent_id = $1",
            pi_str
        )
        .fetch_optional(&*state.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        if existing.is_some() {
            println!("Order already exists for payment intent {}", pi_str);
            return Ok(());
        }
    }

    // Create order record
    let customer_email = charge.billing_details.email.clone();
    let customer_name = charge.billing_details.name.clone();

    let order = CreateOrder {
        payment_provider: PaymentProvider::Stripe,
        payment_id: charge.id.to_string(),
        payment_intent_id: payment_intent_str,
        customer_email: customer_email.clone(),
        customer_name: customer_name.clone(),
        total_amount: charge.amount,
        currency: charge.currency.to_string().to_uppercase(),
        status: OrderStatus::Completed,
        webhook_event_id: Some(webhook_id),
    };

    let order_id = create_order(&state.pool, order)
        .await
        .map_err(|e| format!("Failed to create order: {}", e))?;

    println!("Created order with ID: {}", order_id);

    Ok(())
}

// Handle checkout.session.completed event
async fn handle_checkout_session_completed(
    state: &Arc<AppState>,
    event: &Event,
    webhook_id: uuid::Uuid,
) -> Result<(), String> {
    let session = match &event.data.object {
        EventObject::CheckoutSession(s) => s,
        _ => return Err("Expected CheckoutSession object".to_string()),
    };

    println!(
        "Checkout session completed! Session ID: {}, Amount: {:?}",
        session.id, session.amount_total
    );

    // Extract customer information
    let customer_email = session.customer_email.as_ref().map(|e| e.to_string());

    // Extract payment intent ID from Expandable if present
    let payment_intent_str = session.payment_intent
        .as_ref()
        .map(|pi| pi.id().to_string());

    // Create order record
    let order = CreateOrder {
        payment_provider: PaymentProvider::Stripe,
        payment_id: session.id.to_string(),
        payment_intent_id: payment_intent_str,
        customer_email: customer_email.clone(),
        customer_name: None,
        total_amount: session.amount_total.unwrap_or(0),
        currency: session.currency.as_ref().map(|c| c.to_string().to_uppercase()).unwrap_or_else(|| "USD".to_string()),
        status: OrderStatus::Completed,
        webhook_event_id: Some(webhook_id),
    };

    let order_id = create_order(&state.pool, order)
        .await
        .map_err(|e| format!("Failed to create order: {}", e))?;

    println!("Created order with ID: {}", order_id);

    // Send order confirmation email
    if let Some(email) = customer_email {
        send_order_confirmation_email(&email, &session.id.to_string(), session.amount_total.unwrap_or(0))
            .await;
    }

    Ok(())
}

// Send order confirmation email using lettre
async fn send_order_confirmation_email(email: &str, order_id: &str, amount: i64) {
    use crate::lettre_email::{EmailConfig, OrderConfirmationRequest, OrderItem};
    use lettre::{Message, SmtpTransport, Transport, message::header::ContentType, transport::smtp::authentication::Credentials};

    println!(
        "Sending order confirmation email to {} for order {} (${:.2})",
        email,
        order_id,
        amount as f64 / 100.0
    );

    // Try to get email config
    let config = match EmailConfig::from_env() {
        Some(c) => c,
        None => {
            eprintln!("Email not configured. Set SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD, FROM_EMAIL");
            return;
        }
    };

    // Build HTML email
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
        .total {{ font-size: 18px; font-weight: bold; margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🎉 Payment Successful!</h1>
        </div>
        <div class="content">
            <p>Hi there,</p>
            <p>Thank you for your payment! Your transaction has been completed successfully.</p>
            <p><strong>Order ID:</strong> {}</p>
            <p class="total">Amount Paid: ${:.2}</p>
            <p>We've received your payment and will process your order shortly. You'll receive a shipping confirmation email once your order ships.</p>
            <p>If you have any questions, please don't hesitate to contact us.</p>
        </div>
        <div class="footer">
            <p>© 2025 R-Com Store. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
        "#,
        order_id,
        amount as f64 / 100.0
    );

    // Send email
    match send_html_email(&config, email, &format!("Payment Confirmation - {}", order_id), &html_body).await {
        Ok(_) => println!("✓ Order confirmation email sent to {}", email),
        Err(e) => eprintln!("✗ Failed to send email: {}", e),
    }
}

// Helper function to send HTML email
async fn send_html_email(config: &crate::lettre_email::EmailConfig, to: &str, subject: &str, html_body: &str) -> Result<(), String> {
    use lettre::{Message, SmtpTransport, Transport, message::header::ContentType, transport::smtp::authentication::Credentials};

    let from_mailbox = format!("{} <{}>", config.from_name, config.from_email)
        .parse()
        .map_err(|e| format!("Invalid from address: {}", e))?;

    let to_mailbox = to.parse()
        .map_err(|e| format!("Invalid to address: {}", e))?;

    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html_body.to_string())
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let creds = Credentials::new(
        config.smtp_username.clone(),
        config.smtp_password.clone(),
    );

    let mailer = SmtpTransport::starttls_relay(&config.smtp_host)
        .map_err(|e| format!("Failed to create SMTP relay: {}", e))?
        .port(config.smtp_port)
        .credentials(creds)
        .build();

    mailer.send(&email)
        .map_err(|e| format!("Failed to send email: {}", e))?;

    Ok(())
}
