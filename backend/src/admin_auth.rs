use axum::{Json, extract::State, http::StatusCode, routing::{post, Router}, async_trait, extract::{FromRequestParts}, http::{request::Parts}, headers::{authorization::Bearer, Authorization}, TypedHeader};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use argon2::{self, Config};
use oath::{totp_raw_now, HashType};
use jsonwebtoken::{encode, EncodingKey, Header, decode, DecodingKey, Validation, TokenData};
use rand::Rng;
use base32::{Alphabet, encode as base32_encode};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct TotpVerifyRequest {
    pub username: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub qr_url: String,
}

#[derive(Serialize)]
pub struct JwtResponse {
    pub token: String,
}

#[derive(sqlx::FromRow)]
struct AdminUser {
    id: i32,
    username: String,
    password_hash: String,
    totp_secret: Option<String>,
}

pub struct AuthenticatedAdmin {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedAdmin
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header".to_string()))?;
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "supersecretjwtkey".to_string());
        let token_data: TokenData<Claims> = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".to_string()))?;
        Ok(AuthenticatedAdmin {
            username: token_data.claims.sub,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn admin_auth_routes(pool: Arc<PgPool>, jwt_secret: String) -> Router {
    Router::new()
        .route("/api/admin/register", post(move |body| register_admin(pool.clone(), body)))
        .route("/api/admin/login", post(move |body| login_admin(pool.clone(), body)))
        .route("/api/admin/totp/setup", post(move |body| totp_setup(pool.clone(), body)))
        .route("/api/admin/totp/verify", post(move |body| totp_verify(pool.clone(), jwt_secret.clone(), body)))
}

async fn register_admin(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let salt = rand::thread_rng().gen::<[u8; 16]>();
    let password_hash = argon2::hash_encoded(req.password.as_bytes(), &salt, &Config::default())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Hash error: {}", e)))?;
    sqlx::query("INSERT INTO admin_users (username, password_hash) VALUES ($1, $2)")
        .bind(&req.username)
        .bind(&password_hash)
        .execute(&*pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
    Ok(StatusCode::CREATED)
}

async fn login_admin(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TotpSetupResponse>, (StatusCode, String)> {
    let user: AdminUser = sqlx::query_as("SELECT * FROM admin_users WHERE username = $1")
        .bind(&req.username)
        .fetch_one(&*pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username or password".to_string()))?;
    if !argon2::verify_encoded(&user.password_hash, req.password.as_bytes())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid password".to_string()))?
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid password".to_string()));
    }
    // If TOTP not set up, return secret and QR code
    if user.totp_secret.is_none() {
        let secret_bytes: [u8; 20] = rand::thread_rng().gen();
        let secret = base32_encode(Alphabet::RFC4648 { padding: false }, &secret_bytes);
        let qr_url = format!(
            "otpauth://totp/AdminPortal:{}?secret={}&issuer=RustEcomAdmin",
            user.username, secret
        );
        sqlx::query("UPDATE admin_users SET totp_secret = $1 WHERE id = $2")
            .bind(&secret)
            .bind(user.id)
            .execute(&*pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
        return Ok(Json(TotpSetupResponse { secret, qr_url }));
    }
    // If TOTP is set up, just return a dummy response
    Ok(Json(TotpSetupResponse { secret: "".to_string(), qr_url: "".to_string() }))
}

async fn totp_setup(
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TotpSetupResponse>, (StatusCode, String)> {
    // For explicit TOTP setup (if needed)
    let user: AdminUser = sqlx::query_as("SELECT * FROM admin_users WHERE username = $1")
        .bind(&req.username)
        .fetch_one(&*pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username".to_string()))?;
    let secret_bytes: [u8; 20] = rand::thread_rng().gen();
    let secret = base32_encode(Alphabet::RFC4648 { padding: false }, &secret_bytes);
    let qr_url = format!(
        "otpauth://totp/AdminPortal:{}?secret={}&issuer=RustEcomAdmin",
        user.username, secret
    );
    sqlx::query("UPDATE admin_users SET totp_secret = $1 WHERE id = $2")
        .bind(&secret)
        .bind(user.id)
        .execute(&*pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;
    Ok(Json(TotpSetupResponse { secret, qr_url }))
}

async fn totp_verify(
    State(pool): State<Arc<PgPool>>,
    jwt_secret: String,
    Json(req): Json<TotpVerifyRequest>,
) -> Result<Json<JwtResponse>, (StatusCode, String)> {
    let user: AdminUser = sqlx::query_as("SELECT * FROM admin_users WHERE username = $1")
        .bind(&req.username)
        .fetch_one(&*pool)
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid username".to_string()))?;
    let secret = user.totp_secret.ok_or((StatusCode::UNAUTHORIZED, "TOTP not set up".to_string()))?;
    let code = totp_raw_now(secret.as_bytes(), 6, 0, 30, &HashType::SHA1);
    if code != req.code {
        return Err((StatusCode::UNAUTHORIZED, "Invalid TOTP code".to_string()));
    }
    // Issue JWT
    let claims = Claims {
        sub: user.username,
        exp: (chrono::Utc::now() + chrono::Duration::hours(8)).timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_bytes()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("JWT error: {}", e)))?;
    Ok(Json(JwtResponse { token }))
} 