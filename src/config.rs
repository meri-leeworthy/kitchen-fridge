//! Support for compile-time configuration options

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

/// Part of the ProdID string that describes the organization (example of a ProdID string: `-//ABC Corporation//My Product//EN`).
/// Feel free to override it when initing this library.
pub static ORG_NAME: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new("My organization".to_string())));

/// Part of the ProdID string that describes the product name (example of a ProdID string: `-//ABC Corporation//My Product//EN`).
/// Feel free to override it when initing this library.
pub static PRODUCT_NAME: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new("KitchenFridge".to_string())));
