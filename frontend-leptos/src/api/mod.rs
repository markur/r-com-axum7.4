// API client for communicating with Axum backend

pub mod products;
pub mod cart;
pub mod checkout;

use gloo_net::http::Request;
use serde::de::DeserializeOwned;

/// Base URL for the API
const API_BASE: &str = "http://localhost:3000";

/// Generic API error type
#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
    pub status: u16,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Error {}: {}", self.status, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Helper function to make GET requests
pub async fn get<T: DeserializeOwned>(endpoint: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", API_BASE, endpoint);

    log::info!("GET {}", url);

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| ApiError {
            message: format!("Network error: {}", e),
            status: 0,
        })?;

    let status = response.status();

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ApiError {
            message: error_text,
            status,
        });
    }

    response.json::<T>().await.map_err(|e| ApiError {
        message: format!("Failed to parse response: {}", e),
        status,
    })
}

/// Helper function to make POST requests
pub async fn post<T: DeserializeOwned, B: serde::Serialize>(
    endpoint: &str,
    body: &B,
) -> Result<T, ApiError> {
    let url = format!("{}{}", API_BASE, endpoint);

    log::info!("POST {}", url);

    let response = Request::post(&url)
        .json(body)
        .map_err(|e| ApiError {
            message: format!("Failed to serialize request: {}", e),
            status: 0,
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            message: format!("Network error: {}", e),
            status: 0,
        })?;

    let status = response.status();

    if !response.ok() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ApiError {
            message: error_text,
            status,
        });
    }

    response.json::<T>().await.map_err(|e| ApiError {
        message: format!("Failed to parse response: {}", e),
        status,
    })
}
