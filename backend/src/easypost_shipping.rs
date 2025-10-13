// EasyPost Shipping Integration
// Handles shipping rates, label creation, tracking, and address validation
// API Docs: https://www.easypost.com/docs/api

use axum::{Json, Router, routing::{post, get}, extract::{State, Path}, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::AppState;
use reqwest;

// EasyPost configuration
pub struct ShippingConfig {
    pub easypost_api_key: String,
    pub easypost_api_url: String,
}

impl ShippingConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            easypost_api_key: std::env::var("EASYPOST_API_KEY").ok()?,
            easypost_api_url: std::env::var("EASYPOST_API_URL")
                .unwrap_or_else(|_| "https://api.easypost.com/v2".to_string()),
        })
    }
}

// Add shipping config to AppState
impl AppState {
    pub fn shipping_config(&self) -> Option<ShippingConfig> {
        ShippingConfig::from_env()
    }
}

// ===== Request Structures =====

#[derive(Deserialize)]
pub struct GetRatesRequest {
    pub from_address: Address,
    pub to_address: Address,
    pub parcel: Parcel,
}

#[derive(Deserialize)]
pub struct CreateShipmentRequest {
    pub from_address: Address,
    pub to_address: Address,
    pub parcel: Parcel,
    pub rate_id: Option<String>, // If provided, buy this specific rate
}

#[derive(Deserialize)]
pub struct ValidateAddressRequest {
    pub address: Address,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Address {
    pub name: Option<String>,
    pub street1: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: Option<String>, // Defaults to "US"
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Parcel {
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub weight: f64, // in ounces
}

// ===== Response Structures =====

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct ShippingRatesResponse {
    pub success: bool,
    pub rates: Vec<ShippingRate>,
    pub shipment_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ShippingRate {
    pub id: String,
    pub carrier: String,
    pub service: String,
    pub rate: String,
    pub currency: String,
    pub delivery_days: Option<i32>,
    pub delivery_date: Option<String>,
}

#[derive(Serialize)]
pub struct CreateShipmentResponse {
    pub success: bool,
    pub shipment_id: String,
    pub tracking_code: String,
    pub label_url: String,
    pub postage_label: PostageLabel,
}

#[derive(Serialize, Deserialize)]
pub struct PostageLabel {
    pub label_url: String,
    pub label_pdf_url: Option<String>,
    pub label_zpl_url: Option<String>,
}

#[derive(Serialize)]
pub struct TrackingResponse {
    pub success: bool,
    pub tracking_code: String,
    pub status: String,
    pub carrier: String,
    pub tracking_details: Vec<TrackingDetail>,
}

#[derive(Serialize, Deserialize)]
pub struct TrackingDetail {
    pub datetime: String,
    pub status: String,
    pub message: String,
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Serialize)]
pub struct AddressValidationResponse {
    pub success: bool,
    pub is_valid: bool,
    pub original_address: Address,
    pub verified_address: Option<Address>,
    pub messages: Vec<String>,
}

// ===== EasyPost API Response Structures =====

#[derive(Deserialize)]
struct EasyPostShipment {
    id: String,
    rates: Vec<EasyPostRate>,
    postage_label: Option<EasyPostLabel>,
    tracking_code: Option<String>,
}

#[derive(Deserialize)]
struct EasyPostRate {
    id: String,
    carrier: String,
    service: String,
    rate: String,
    currency: String,
    delivery_days: Option<i32>,
    delivery_date: Option<String>,
}

#[derive(Deserialize)]
struct EasyPostLabel {
    label_url: String,
    label_pdf_url: Option<String>,
    label_zpl_url: Option<String>,
}

#[derive(Deserialize)]
struct EasyPostTracker {
    tracking_code: String,
    status: String,
    carrier: String,
    tracking_details: Vec<EasyPostTrackingDetail>,
}

#[derive(Deserialize)]
struct EasyPostTrackingDetail {
    datetime: String,
    status: String,
    message: String,
    tracking_location: Option<EasyPostLocation>,
}

#[derive(Deserialize)]
struct EasyPostLocation {
    city: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct EasyPostAddress {
    street1: String,
    street2: Option<String>,
    city: String,
    state: String,
    zip: String,
    country: Option<String>,
    verifications: Option<EasyPostVerifications>,
}

#[derive(Deserialize)]
struct EasyPostVerifications {
    delivery: Option<EasyPostDeliveryVerification>,
}

#[derive(Deserialize)]
struct EasyPostDeliveryVerification {
    success: bool,
    errors: Option<Vec<EasyPostError>>,
}

#[derive(Deserialize)]
struct EasyPostError {
    message: String,
}

// ===== Routes =====

pub fn easypost_shipping_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/shipping/rates", post(get_shipping_rates))
        .route("/api/shipping/create-label", post(create_shipping_label))
        .route("/api/shipping/track/:tracking_code", get(track_shipment))
        .route("/api/shipping/validate-address", post(validate_address))
        .with_state(app_state)
}

// ===== API Handlers =====

// Get shipping rates
async fn get_shipping_rates(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GetRatesRequest>,
) -> Result<Json<ShippingRatesResponse>, (StatusCode, String)> {
    let config = state.shipping_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Shipping not configured".to_string()))?;

    // Create shipment to get rates
    let client = reqwest::Client::new();
    let url = format!("{}/shipments", config.easypost_api_url);

    let mut shipment_data = serde_json::json!({
        "shipment": {
            "to_address": {
                "street1": payload.to_address.street1,
                "city": payload.to_address.city,
                "state": payload.to_address.state,
                "zip": payload.to_address.zip,
                "country": payload.to_address.country.unwrap_or_else(|| "US".to_string()),
            },
            "from_address": {
                "street1": payload.from_address.street1,
                "city": payload.from_address.city,
                "state": payload.from_address.state,
                "zip": payload.from_address.zip,
                "country": payload.from_address.country.unwrap_or_else(|| "US".to_string()),
            },
            "parcel": {
                "length": payload.parcel.length,
                "width": payload.parcel.width,
                "height": payload.parcel.height,
                "weight": payload.parcel.weight,
            }
        }
    });

    // Add optional fields
    if let Some(street2) = &payload.to_address.street2 {
        shipment_data["shipment"]["to_address"]["street2"] = serde_json::json!(street2);
    }
    if let Some(name) = &payload.to_address.name {
        shipment_data["shipment"]["to_address"]["name"] = serde_json::json!(name);
    }

    let response = client
        .post(&url)
        .basic_auth(&config.easypost_api_key, Some(""))
        .json(&shipment_data)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("EasyPost API error: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::BAD_REQUEST, format!("EasyPost error: {}", error_text)));
    }

    let shipment: EasyPostShipment = response.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)))?;

    let rates: Vec<ShippingRate> = shipment.rates.into_iter().map(|r| ShippingRate {
        id: r.id,
        carrier: r.carrier,
        service: r.service,
        rate: r.rate,
        currency: r.currency,
        delivery_days: r.delivery_days,
        delivery_date: r.delivery_date,
    }).collect();

    Ok(Json(ShippingRatesResponse {
        success: true,
        rates,
        shipment_id: shipment.id,
    }))
}

// Create shipping label
async fn create_shipping_label(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateShipmentRequest>,
) -> Result<Json<CreateShipmentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let config = state.shipping_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "Shipping not configured".to_string() })))?;

    // First, create shipment to get rates (if rate_id not provided)
    let client = reqwest::Client::new();

    let shipment_id = if let Some(rate_id) = payload.rate_id {
        // Buy the specified rate
        let url = format!("{}/shipments/buy", config.easypost_api_url);
        let buy_data = serde_json::json!({
            "rate": { "id": rate_id }
        });

        let response = client
            .post(&url)
            .basic_auth(&config.easypost_api_key, Some(""))
            .json(&buy_data)
            .send()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("EasyPost API error: {}", e) })))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("EasyPost error: {}", error_text) })));
        }

        let shipment: EasyPostShipment = response.json().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to parse response: {}", e) })))?;

        shipment.id
    } else {
        // Create new shipment and buy lowest rate
        let url = format!("{}/shipments", config.easypost_api_url);

        let shipment_data = serde_json::json!({
            "shipment": {
                "to_address": {
                    "street1": payload.to_address.street1,
                    "city": payload.to_address.city,
                    "state": payload.to_address.state,
                    "zip": payload.to_address.zip,
                    "country": payload.to_address.country.unwrap_or_else(|| "US".to_string()),
                },
                "from_address": {
                    "street1": payload.from_address.street1,
                    "city": payload.from_address.city,
                    "state": payload.from_address.state,
                    "zip": payload.from_address.zip,
                    "country": payload.from_address.country.unwrap_or_else(|| "US".to_string()),
                },
                "parcel": {
                    "length": payload.parcel.length,
                    "width": payload.parcel.width,
                    "height": payload.parcel.height,
                    "weight": payload.parcel.weight,
                }
            }
        });

        let response = client
            .post(&url)
            .basic_auth(&config.easypost_api_key, Some(""))
            .json(&shipment_data)
            .send()
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("EasyPost API error: {}", e) })))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("EasyPost error: {}", error_text) })));
        }

        let shipment: EasyPostShipment = response.json().await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to parse response: {}", e) })))?;

        // Buy lowest rate
        if let Some(rate) = shipment.rates.first() {
            let buy_url = format!("{}/shipments/{}/buy", config.easypost_api_url, shipment.id);
            let buy_data = serde_json::json!({
                "rate": { "id": rate.id }
            });

            let buy_response = client
                .post(&buy_url)
                .basic_auth(&config.easypost_api_key, Some(""))
                .json(&buy_data)
                .send()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to buy label: {}", e) })))?;

            if !buy_response.status().is_success() {
                let error_text = buy_response.text().await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: format!("Failed to buy label: {}", error_text) })));
            }

            shipment.id
        } else {
            return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: "No rates available".to_string() })));
        }
    };

    // Retrieve the shipment with label
    let get_url = format!("{}/shipments/{}", config.easypost_api_url, shipment_id);
    let get_response = client
        .get(&get_url)
        .basic_auth(&config.easypost_api_key, Some(""))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to retrieve shipment: {}", e) })))?;

    let final_shipment: EasyPostShipment = get_response.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: format!("Failed to parse shipment: {}", e) })))?;

    let label = final_shipment.postage_label
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: "No label generated".to_string() })))?;

    Ok(Json(CreateShipmentResponse {
        success: true,
        shipment_id: final_shipment.id,
        tracking_code: final_shipment.tracking_code.unwrap_or_default(),
        label_url: label.label_url.clone(),
        postage_label: PostageLabel {
            label_url: label.label_url,
            label_pdf_url: label.label_pdf_url,
            label_zpl_url: label.label_zpl_url,
        },
    }))
}

// Track shipment
async fn track_shipment(
    State(state): State<Arc<AppState>>,
    Path(tracking_code): Path<String>,
) -> Result<Json<TrackingResponse>, (StatusCode, String)> {
    let config = state.shipping_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Shipping not configured".to_string()))?;

    let client = reqwest::Client::new();
    let url = format!("{}/trackers/{}", config.easypost_api_url, tracking_code);

    let response = client
        .get(&url)
        .basic_auth(&config.easypost_api_key, Some(""))
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("EasyPost API error: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::NOT_FOUND, format!("Tracking not found: {}", error_text)));
    }

    let tracker: EasyPostTracker = response.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)))?;

    let details: Vec<TrackingDetail> = tracker.tracking_details.into_iter().map(|d| TrackingDetail {
        datetime: d.datetime,
        status: d.status,
        message: d.message,
        city: d.tracking_location.as_ref().and_then(|l| l.city.clone()),
        state: d.tracking_location.as_ref().and_then(|l| l.state.clone()),
    }).collect();

    Ok(Json(TrackingResponse {
        success: true,
        tracking_code: tracker.tracking_code,
        status: tracker.status,
        carrier: tracker.carrier,
        tracking_details: details,
    }))
}

// Validate address
async fn validate_address(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ValidateAddressRequest>,
) -> Result<Json<AddressValidationResponse>, (StatusCode, String)> {
    let config = state.shipping_config()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Shipping not configured".to_string()))?;

    let client = reqwest::Client::new();
    let url = format!("{}/addresses", config.easypost_api_url);

    let address_data = serde_json::json!({
        "address": {
            "street1": payload.address.street1,
            "city": payload.address.city,
            "state": payload.address.state,
            "zip": payload.address.zip,
            "country": payload.address.country.clone().unwrap_or_else(|| "US".to_string()),
            "verify": ["delivery"]
        }
    });

    let response = client
        .post(&url)
        .basic_auth(&config.easypost_api_key, Some(""))
        .json(&address_data)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("EasyPost API error: {}", e)))?;

    if !response.status().is_success() {
        let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err((StatusCode::BAD_REQUEST, format!("EasyPost error: {}", error_text)));
    }

    let verified: EasyPostAddress = response.json().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse response: {}", e)))?;

    let is_valid = verified.verifications
        .as_ref()
        .and_then(|v| v.delivery.as_ref())
        .map(|d| d.success)
        .unwrap_or(false);

    let messages = verified.verifications
        .and_then(|v| v.delivery)
        .and_then(|d| d.errors)
        .map(|errors| errors.into_iter().map(|e| e.message).collect())
        .unwrap_or_default();

    let verified_address = if is_valid {
        Some(Address {
            name: payload.address.name.clone(),
            street1: verified.street1,
            street2: verified.street2,
            city: verified.city,
            state: verified.state,
            zip: verified.zip,
            country: verified.country,
            phone: payload.address.phone.clone(),
            email: payload.address.email.clone(),
        })
    } else {
        None
    };

    Ok(Json(AddressValidationResponse {
        success: true,
        is_valid,
        original_address: payload.address,
        verified_address,
        messages,
    }))
}
