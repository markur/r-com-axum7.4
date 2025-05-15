use axum::{routing::get, Router, Json, routing::post, extract::State};
use serde::{Serialize, Deserialize};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use dotenv::dotenv;
use std::env;
use stripe::Client as StripeClient;
mod admin_auth;
use std::sync::Arc;
mod admin_products;

#[derive(Serialize, sqlx::FromRow)]
struct Product {
    id: i32,
    name: String,
    description: Option<String>,
    price: f64,
    inventory: i32,
    created_at: chrono::NaiveDateTime,
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

async fn health_check() -> &'static str {
    "OK"
}

async fn get_products(pool: axum::extract::State<sqlx::PgPool>) -> Json<Vec<Product>> {
    let products = sqlx::query_as::<_, Product>(
        "SELECT * FROM products ORDER BY id"
    )
    .fetch_all(&*pool)
    .await
    .unwrap_or_default();
    Json(products)
}

async fn create_payment_intent(
    State(stripe_client): State<StripeClient>,
    Json(payload): Json<CreatePaymentIntentRequest>,
) -> Result<Json<CreatePaymentIntentResponse>, (axum::http::StatusCode, String)> {
    let mut params = stripe::CreatePaymentIntent::new(payload.amount, payload.currency);
    params.payment_method_types = Some(vec!["card".to_string()]);
    match stripe::PaymentIntent::create(&stripe_client, params).await {
        Ok(intent) => Ok(Json(CreatePaymentIntentResponse {
            client_secret: intent.client_secret.unwrap_or_default(),
        })),
        Err(e) => Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Stripe error: {}", e))),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");
    let pool = Arc::new(pool);

    let stripe_secret = env::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY must be set");
    let stripe_client = StripeClient::new(stripe_secret);

    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());

    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/products", get(get_products))
        .route("/api/create-payment-intent", post(create_payment_intent))
        .merge(admin_auth::admin_auth_routes(pool.clone(), jwt_secret.clone()))
        .merge(admin_products::admin_product_routes(pool.clone()))
        .with_state(pool)
        .with_state(stripe_client);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Backend running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
} 