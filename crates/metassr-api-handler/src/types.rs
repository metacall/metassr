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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_api_request_serialization() {
        let request = ApiRequest {
            url: "/api/hello".to_string(),
            headers: HashMap::from([("Content-Type".to_string(), "application/json".to_string())]),
            method: "GET".to_string(),
            body: None,
            params: HashMap::new(),
            query: HashMap::from([("page".to_string(), "1".to_string())]),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ApiRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.url, "/api/hello");
        assert_eq!(parsed.method, "GET");
        assert!(parsed.body.is_none());
        assert_eq!(parsed.query.get("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_api_request_with_body() {
        let request = ApiRequest {
            url: "/api/users".to_string(),
            headers: HashMap::new(),
            method: "POST".to_string(),
            body: Some(r#"{"name": "test"}"#.to_string()),
            params: HashMap::new(),
            query: HashMap::new(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: ApiRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.body, Some(r#"{"name": "test"}"#.to_string()));
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse {
            status: 200,
            headers: HashMap::new(),
            body: json!({"message": "Hello!"}),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ApiResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.body["message"], "Hello!");
    }

    #[test]
    fn test_api_response_from_js_format() {
        // Test parsing the format that JS handlers return
        let js_response = r#"{"status": 201, "body": {"created": true}}"#;
        let parsed: ApiResponse = serde_json::from_str(js_response).unwrap();

        assert_eq!(parsed.status, 201);
        assert_eq!(parsed.body["created"], true);
        assert!(parsed.headers.is_empty()); // Default for missing headers
    }

    #[test]
    fn test_api_response_with_headers() {
        let response = ApiResponse {
            status: 200,
            headers: HashMap::from([("X-Custom-Header".to_string(), "value".to_string())]),
            body: json!({}),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: ApiResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed.headers.get("X-Custom-Header"),
            Some(&"value".to_string())
        );
    }
}
