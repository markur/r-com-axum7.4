// Shopping cart type definitions

use serde::{Deserialize, Serialize};
use super::product::Product;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartItem {
    pub product: Product,
    pub quantity: u32,
}

impl CartItem {
    pub fn new(product: Product, quantity: u32) -> Self {
        Self { product, quantity }
    }

    /// Calculate subtotal for this cart item
    pub fn subtotal(&self) -> f64 {
        self.product.price * self.quantity as f64
    }

    /// Format subtotal as currency
    pub fn formatted_subtotal(&self) -> String {
        format!("${:.2}", self.subtotal())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Cart {
    pub items: Vec<CartItem>,
}

impl Cart {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add product to cart or increase quantity if already exists
    pub fn add_item(&mut self, product: Product, quantity: u32) {
        if let Some(item) = self.items.iter_mut().find(|i| i.product.id == product.id) {
            item.quantity += quantity;
        } else {
            self.items.push(CartItem::new(product, quantity));
        }
    }

    /// Remove item from cart by product ID
    pub fn remove_item(&mut self, product_id: i32) {
        self.items.retain(|item| item.product.id != product_id);
    }

    /// Update quantity for a specific product
    pub fn update_quantity(&mut self, product_id: i32, quantity: u32) {
        if quantity == 0 {
            self.remove_item(product_id);
        } else if let Some(item) = self.items.iter_mut().find(|i| i.product.id == product_id) {
            item.quantity = quantity;
        }
    }

    /// Clear all items from cart
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get total number of items in cart
    pub fn total_items(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    /// Calculate cart subtotal
    pub fn subtotal(&self) -> f64 {
        self.items.iter().map(|item| item.subtotal()).sum()
    }

    /// Calculate tax (8% for now)
    pub fn tax(&self) -> f64 {
        self.subtotal() * 0.08
    }

    /// Calculate total (subtotal + tax)
    pub fn total(&self) -> f64 {
        self.subtotal() + self.tax()
    }

    /// Format subtotal as currency
    pub fn formatted_subtotal(&self) -> String {
        format!("${:.2}", self.subtotal())
    }

    /// Format tax as currency
    pub fn formatted_tax(&self) -> String {
        format!("${:.2}", self.tax())
    }

    /// Format total as currency
    pub fn formatted_total(&self) -> String {
        format!("${:.2}", self.total())
    }

    /// Check if cart is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
