// ============================================================================
// AXUM VERSION MIGRATION NOTES (0.6 → 0.7.4)
// ============================================================================
// This code was originally written for Axum 0.6 but has been updated for 0.7.4
// 
// Key Breaking Changes in Axum 0.7+:
// 1. axum::Server removed → Use axum::serve() with TcpListener
// 2. .into_make_service() removed → Pass Router directly to axum::serve()
// 3. Server binding pattern changed → Manual TcpListener creation required
//
// Migration Summary:
// OLD: axum::Server::bind(&addr).serve(app.into_make_service()).await
// NEW: axum::serve(TcpListener::bind(&addr).await?, app).await
// ============================================================================

// --- Imports ---
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use dotenv::dotenv;
// AXUM 0.7.4 UPDATE: Added TcpListener import
// In Axum 0.7+, axum::Server was removed and replaced with axum::serve()
// which requires a tokio::net::TcpListener instead of direct SocketAddr binding
use tokio::net::TcpListener;
// Using stripe crate (renamed async-stripe v0.23.0 in Cargo.toml)
use stripe::{Client as StripeClient, PaymentIntent, CreatePaymentIntent as PaymentIntentCreateParams, Currency};
use sqlx::types::chrono::NaiveDateTime;
// CORS support
use tower_http::cors::{CorsLayer, Any};

// Module declarations
mod admin_auth;
mod admin_products;
mod square_payments;
mod lettre_email;
mod textbelt_sms;
mod easypost_shipping;
mod webhooks;

// --- Shared application state for all handlers ---
pub struct AppState {
    pub pool: Arc<sqlx::PgPool>,          // Shared Postgres connection pool
    pub stripe_client: StripeClient,      // Stripe API client
    pub jwt_secret: String,               // Secret for JWT signing/verification
}

// --- Main entrypoint for the backend server ---
#[tokio::main]
async fn main() {
    dotenv().ok();                        // Load .env file for secrets
    tracing_subscriber::fmt::init();      // Set up logging

    // --- Set up database pool ---
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    let pool = Arc::new(pool);

    // --- Set up Stripe client ---
    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    // Initialize Stripe client with async-stripe v0.23.0 API
    let stripe_client = StripeClient::new(stripe_secret);
    
    // --- JWT secret for authentication ---
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());

    // --- Shared app state ---
    let app_state = Arc::new(AppState {
        pool: pool.clone(),
        stripe_client,
        jwt_secret: jwt_secret.clone(),
    });

    // --- Configure CORS to allow requests from any origin ---
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // --- Build the Axum router with all routes and shared state ---
    let app = Router::new()
        .route("/", get(health_check))                                 // Health check endpoint
        .route("/api/products", get(get_products))                    // Public products endpoint
        .route("/api/create-payment-intent", post(create_payment_intent)) // Stripe payment intent
        .merge(admin_auth::admin_auth_routes(app_state.clone()))       // Admin authentication routes
        .merge(admin_products::admin_product_routes(app_state.clone()))// Admin product management
        .merge(square_payments::square_payment_routes(app_state.clone())) // Square payment processing
        .merge(lettre_email::lettre_email_routes(app_state.clone()))     // Lettre transactional emails
        .merge(textbelt_sms::textbelt_sms_routes(app_state.clone()))    // Textbelt SMS notifications
        .merge(easypost_shipping::easypost_shipping_routes(app_state.clone())) // EasyPost shipping
        .merge(webhooks::webhook_routes(app_state.clone()))            // Payment webhooks (Stripe, Square)
        .layer(cors)                                                   // Add CORS middleware
        .with_state(app_state);                                       // Attach shared state, converts Router<Arc<AppState>> -> Router<()>

    // --- Start the HTTP server using axum 0.7.4 API ---
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Backend running at http://{}", addr);
    
    // AXUM 0.7.4 UPDATE: Server setup pattern changed
    // OLD (Axum 0.6): axum::Server::bind(&addr).serve(app.into_make_service())
    // NEW (Axum 0.7+): axum::serve(listener, app)
    //
    // Changes made:
    // 1. axum::Server was removed - no longer exists in axum 0.7+
    // 2. Must create TcpListener manually and pass to axum::serve()
    // 3. Router<S>.with_state(S) returns Router<()>, which can be passed directly to axum::serve()
    // 4. No need for into_make_service() or into_service() - pass Router<()> directly
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", addr);

    // Router<()> (after with_state) can be passed directly to axum::serve() in Axum 0.7
    axum::serve(listener, app)
        .await
        .unwrap();
}

// --- Health check endpoint ---
async fn health_check() -> &'static str {
    "OK"
}

// --- Data types for Product, PaymentIntent, etc. ---
#[derive(Serialize, sqlx::FromRow)]
struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price: f64,
    inventory: i32,
    created_at: NaiveDateTime,
}

#[derive(Deserialize)]
struct CreatePaymentIntentRequest {
    amount: i64, // in cents
    currency: String,
}

#[derive(Serialize)]
struct CreatePaymentIntentResponse {
    client_secret: String,
}

// --- Example: create-payment-intent handler ---
// Accepts Stripe client and creates a PaymentIntent using the async-stripe v0.23.0 API
async fn create_payment_intent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<CreatePaymentIntentResponse>, (axum::http::StatusCode, String)> {
    // Create the params with required parameters in constructor
    let mut params = PaymentIntentCreateParams::new(
        payload.amount, 
        payload.currency.parse().unwrap_or(Currency::USD)
    );
    params.payment_method_types = Some(vec!["card".to_string()]);
    
    match PaymentIntent::create(&state.stripe_client, params).await {
        Ok(intent) => Ok(Json(CreatePaymentIntentResponse {
            client_secret: intent.client_secret.unwrap_or_default(),
        })),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe error: {e}"))),
    }
}

// --- Example: get_products handler ---
// Fetches all products from the database
async fn get_products(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Product>> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products ORDER BY id"
    )
    .fetch_all(&*state.pool)
    .await
    .unwrap_or_default();
    Json(products)
}