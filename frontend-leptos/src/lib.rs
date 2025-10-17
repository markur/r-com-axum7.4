// R-Com E-Commerce Frontend - Leptos Application
// Main library entry point

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

// Module declarations
mod app;
mod components;
mod pages;
mod api;
mod types;
mod utils;

// Re-export main app
pub use app::App;

// Hydrate function for CSR (Client-Side Rendering)
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // Set up console error panic hook for better debugging
    console_error_panic_hook::set_once();

    // Initialize logging
    _ = console_log::init_with_level(log::Level::Debug);

    // Mount the app
    leptos::mount_to_body(App);
}
