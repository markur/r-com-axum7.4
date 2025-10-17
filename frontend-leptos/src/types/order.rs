// Order and checkout type definitions

use serde::{Deserialize, Serialize};
use super::cart::CartItem;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShippingAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: i32,
    pub items: Vec<CartItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub shipping: f64,
    pub total: f64,
    pub status: OrderStatus,
    pub shipping_address: ShippingAddress,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::Shipped => "Shipped",
            Self::Delivered => "Delivered",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Pending => "badge",
            Self::Processing => "badge-primary",
            Self::Shipped => "badge-info",
            Self::Delivered => "badge-success",
            Self::Cancelled => "badge-error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutRequest {
    pub items: Vec<CartItem>,
    pub shipping_address: ShippingAddress,
    pub payment_method: PaymentMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentMethod {
    Stripe,
    Square,
}
