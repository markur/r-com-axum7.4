// Utility functions

/// Format a number as USD currency
pub fn format_currency(amount: f64) -> String {
    format!("${:.2}", amount)
}

/// Truncate text to a maximum length with ellipsis
pub fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len])
    }
}

/// Get value from localStorage
pub fn get_local_storage(key: &str) -> Option<String> {
    web_sys::window()?
        .local_storage()
        .ok()??
        .get_item(key)
        .ok()?
}

/// Set value in localStorage
pub fn set_local_storage(key: &str, value: &str) -> Result<(), String> {
    web_sys::window()
        .ok_or("No window")?
        .local_storage()
        .map_err(|_| "No localStorage".to_string())?
        .ok_or("localStorage not available")?
        .set_item(key, value)
        .map_err(|_| "Failed to set item".to_string())
}

/// Remove value from localStorage
pub fn remove_local_storage(key: &str) -> Result<(), String> {
    web_sys::window()
        .ok_or("No window")?
        .local_storage()
        .map_err(|_| "No localStorage".to_string())?
        .ok_or("localStorage not available")?
        .remove_item(key)
        .map_err(|_| "Failed to remove item".to_string())
}
