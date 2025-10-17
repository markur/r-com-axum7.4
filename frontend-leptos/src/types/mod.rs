// Shared type definitions

pub mod product;
pub mod cart;
pub mod user;
pub mod order;

// Re-export commonly used types
pub use product::Product;
pub use cart::{Cart, CartItem};
pub use user::User;
pub use order::Order;
