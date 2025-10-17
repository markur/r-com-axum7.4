// Cart API (currently client-side only, can add backend sync later)

use crate::types::{Cart, Product};

/// Load cart from localStorage
pub fn load_cart() -> Cart {
    if let Ok(Some(storage)) = web_sys::window().map(|w| w.local_storage().ok().flatten()) {
        if let Ok(Some(cart_json)) = storage.get_item("cart") {
            if let Ok(cart) = serde_json::from_str::<Cart>(&cart_json) {
                return cart;
            }
        }
    }
    Cart::new()
}

/// Save cart to localStorage
pub fn save_cart(cart: &Cart) {
    if let Ok(Some(storage)) = web_sys::window().map(|w| w.local_storage().ok().flatten()) {
        if let Ok(cart_json) = serde_json::to_string(cart) {
            let _ = storage.set_item("cart", &cart_json);
        }
    }
}

/// Add product to cart
pub fn add_to_cart(cart: &mut Cart, product: Product, quantity: u32) {
    cart.add_item(product, quantity);
    save_cart(cart);
}

/// Remove product from cart
pub fn remove_from_cart(cart: &mut Cart, product_id: i32) {
    cart.remove_item(product_id);
    save_cart(cart);
}

/// Update product quantity in cart
pub fn update_cart_quantity(cart: &mut Cart, product_id: i32, quantity: u32) {
    cart.update_quantity(product_id, quantity);
    save_cart(cart);
}

/// Clear entire cart
pub fn clear_cart(cart: &mut Cart) {
    cart.clear();
    save_cart(cart);
}
