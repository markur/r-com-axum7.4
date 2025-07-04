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
// Using stripe crate (renamed async-stripe in Cargo.toml)
use stripe::{Client as StripeClient, PaymentIntent, CreatePaymentIntent as PaymentIntentCreateParams, Currency};
use sqlx::types::chrono::NaiveDateTime;

// Module declarations
mod admin_auth;
mod admin_products;

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
    // Initialize Stripe client with async-stripe v0.21.0 API
    let stripe_client = StripeClient::new(stripe_secret);
    
    // --- JWT secret for authentication ---
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());

    // --- Shared app state ---
    let app_state = Arc::new(AppState {
        pool: pool.clone(),
        stripe_client,
        jwt_secret: jwt_secret.clone(),
    });

    // --- Build the Axum router with all routes and shared state ---
    let app: Router<Arc<AppState>> = Router::new()
        .route("/", get(health_check))                                 // Health check endpoint
        .route("/api/products", get(get_products))                    // Public products endpoint
        .route("/api/create-payment-intent", post(create_payment_intent)) // Stripe payment intent
        .merge(admin_auth::admin_auth_routes(app_state.clone()))       // Admin authentication routes
        .merge(admin_products::admin_product_routes(app_state.clone()))// Admin product management
        .with_state(app_state);                                       // Attach shared state

    // --- Start the HTTP server using axum 0.7 API ---
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Backend running at http://{}", addr);
    
    // Create a TCP listener and convert app to appropriate service
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let service = app.into_service();
    axum::serve(listener, service).await.unwrap();
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
// Accepts Stripe client and creates a PaymentIntent using the stripe-rust API
async fn create_payment_intent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<CreatePaymentIntentResponse>, (axum::http::StatusCode, String)> {
    // Create payment intent params according to async-stripe API
    let mut params = PaymentIntentCreateParams {
        amount: Some(payload.amount),
        currency: Some(payload.currency.parse().unwrap_or(Currency::USD)),
        ..Default::default()
    };
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