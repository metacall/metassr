//! API request and response types for MetaSSR API routes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Headers type alias for convenience.
pub type Headers = HashMap<String, String>;

/// Represents an incoming API request passed to handler functions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRequest {
    /// The URL path of the request.
    pub url: String,
    /// HTTP headers as key-value pairs.
    pub headers: Headers,
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Request body (if any).
    pub body: Option<String>,
    /// URL path parameters (e.g., from dynamic routes).
    pub params: HashMap<String, String>,
    /// Query string parameters.
    pub query: HashMap<String, String>,
}

/// Represents an API response returned by handler functions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response headers (optional).
    #[serde(default)]
    pub headers: Headers,
    /// Response body as JSON value.
    pub body: serde_json::Value,
}
