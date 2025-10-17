// Product type definitions
// These match the backend Product struct

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub inventory: i32,
    #[serde(rename = "created_at")]
    pub created_at: String,  // Backend sends NaiveDateTime as string
}

impl Product {
    /// Check if product is in stock
    pub fn is_in_stock(&self) -> bool {
        self.inventory > 0
    }

    /// Format price as USD currency
    pub fn formatted_price(&self) -> String {
        format!("${:.2}", self.price)
    }

    /// Get product image URL (placeholder for now)
    pub fn image_url(&self) -> String {
        format!("https://via.placeholder.com/400x300?text={}",
            urlencoding::encode(&self.name))
    }

    /// Get stock status badge text
    pub fn stock_status(&self) -> &'static str {
        match self.inventory {
            0 => "Out of Stock",
            1..=5 => "Low Stock",
            _ => "In Stock",
        }
    }

    /// Get stock status CSS class
    pub fn stock_status_class(&self) -> &'static str {
        match self.inventory {
            0 => "badge-error",
            1..=5 => "badge-warning",
            _ => "badge-success",
        }
    }
}

// Product filter options
#[derive(Debug, Clone, PartialEq)]
pub enum ProductSortOrder {
    NameAsc,
    NameDesc,
    PriceAsc,
    PriceDesc,
    Newest,
}

impl ProductSortOrder {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NameAsc => "Name (A-Z)",
            Self::NameDesc => "Name (Z-A)",
            Self::PriceAsc => "Price (Low to High)",
            Self::PriceDesc => "Price (High to Low)",
            Self::Newest => "Newest First",
        }
    }
}
