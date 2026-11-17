//! API request and response types for MetaSSR API routes.

use serde::{Deserialize, Serialize, Serializer};
use std::collections::{BTreeMap, HashMap};

/// Headers type alias for convenience.
pub type Headers = HashMap<String, String>;

fn serialize_sorted_map<S>(map: &HashMap<String, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let sorted: BTreeMap<&String, &String> = map.iter().collect();
    sorted.serialize(serializer)
}

/// Represents an incoming API request passed to handler functions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRequest {
    /// The URL path of the request.
    pub url: String,
    /// HTTP headers as key-value pairs.
    #[serde(serialize_with = "serialize_sorted_map")]
    pub headers: Headers,
    /// HTTP method (GET, POST, etc.).
    pub method: String,
    /// Request body (if any).
    pub body: Option<String>,
    /// URL path parameters (e.g., from dynamic routes).
    #[serde(serialize_with = "serialize_sorted_map")]
    pub params: HashMap<String, String>,
    /// Query string parameters.
    #[serde(serialize_with = "serialize_sorted_map")]
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

    #[test]
    fn test_deterministic_query_serialization_independent_of_insertion_order() {
        let mut query1 = HashMap::new();
        query1.insert("zeta".to_string(), "1".to_string());
        query1.insert("alpha".to_string(), "2".to_string());
        query1.insert("beta".to_string(), "3".to_string());
        query1.insert("gamma".to_string(), "4".to_string());

        let mut query2 = HashMap::new();
        query2.insert("alpha".to_string(), "2".to_string());
        query2.insert("gamma".to_string(), "4".to_string());
        query2.insert("zeta".to_string(), "1".to_string());
        query2.insert("beta".to_string(), "3".to_string());

        let req1 = ApiRequest {
            url: "/api/items".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: None,
            params: HashMap::new(),
            query: query1,
        };

        let req2 = ApiRequest {
            url: "/api/items".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: None,
            params: HashMap::new(),
            query: query2,
        };

        let json1 = serde_json::to_string(&req1).unwrap();
        let json2 = serde_json::to_string(&req2).unwrap();

        assert_eq!(json1, json2);
        assert!(json1.contains(r#""query":{"alpha":"2","beta":"3","gamma":"4","zeta":"1"}"#));
    }

    #[test]
    fn test_deterministic_params_serialization_independent_of_insertion_order() {
        let mut params1 = HashMap::new();
        params1.insert("id".to_string(), "123".to_string());
        params1.insert("category".to_string(), "rust".to_string());
        params1.insert("action".to_string(), "edit".to_string());
        params1.insert("format".to_string(), "json".to_string());

        let mut params2 = HashMap::new();
        params2.insert("format".to_string(), "json".to_string());
        params2.insert("id".to_string(), "123".to_string());
        params2.insert("action".to_string(), "edit".to_string());
        params2.insert("category".to_string(), "rust".to_string());

        let req1 = ApiRequest {
            url: "/api/items/123".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: None,
            params: params1,
            query: HashMap::new(),
        };

        let req2 = ApiRequest {
            url: "/api/items/123".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: None,
            params: params2,
            query: HashMap::new(),
        };

        let json1 = serde_json::to_string(&req1).unwrap();
        let json2 = serde_json::to_string(&req2).unwrap();

        assert_eq!(json1, json2);
        assert!(json1.contains(
            r#""params":{"action":"edit","category":"rust","format":"json","id":"123"}"#
        ));
    }

    #[test]
    fn test_deterministic_headers_serialization_independent_of_insertion_order() {
        let mut headers1 = HashMap::new();
        headers1.insert("User-Agent".to_string(), "test-agent".to_string());
        headers1.insert("Accept".to_string(), "*/*".to_string());
        headers1.insert("Authorization".to_string(), "Bearer secret".to_string());
        headers1.insert("Content-Type".to_string(), "application/json".to_string());

        let mut headers2 = HashMap::new();
        headers2.insert("Authorization".to_string(), "Bearer secret".to_string());
        headers2.insert("Content-Type".to_string(), "application/json".to_string());
        headers2.insert("User-Agent".to_string(), "test-agent".to_string());
        headers2.insert("Accept".to_string(), "*/*".to_string());

        let req1 = ApiRequest {
            url: "/api/data".to_string(),
            headers: headers1,
            method: "POST".to_string(),
            body: None,
            params: HashMap::new(),
            query: HashMap::new(),
        };

        let req2 = ApiRequest {
            url: "/api/data".to_string(),
            headers: headers2,
            method: "POST".to_string(),
            body: None,
            params: HashMap::new(),
            query: HashMap::new(),
        };

        let json1 = serde_json::to_string(&req1).unwrap();
        let json2 = serde_json::to_string(&req2).unwrap();

        assert_eq!(json1, json2);
        assert!(json1.contains(
            r#""headers":{"Accept":"*/*","Authorization":"Bearer secret","Content-Type":"application/json","User-Agent":"test-agent"}"#
        ));
    }

    #[test]
    fn test_equivalent_requests_with_different_insertion_orders_produce_identical_json() {
        let mut q1 = HashMap::new();
        q1.insert("b".to_string(), "2".to_string());
        q1.insert("a".to_string(), "1".to_string());

        let mut q2 = HashMap::new();
        q2.insert("a".to_string(), "1".to_string());
        q2.insert("b".to_string(), "2".to_string());

        let mut p1 = HashMap::new();
        p1.insert("y".to_string(), "20".to_string());
        p1.insert("x".to_string(), "10".to_string());

        let mut p2 = HashMap::new();
        p2.insert("x".to_string(), "10".to_string());
        p2.insert("y".to_string(), "20".to_string());

        let mut h1 = HashMap::new();
        h1.insert("h2".to_string(), "val2".to_string());
        h1.insert("h1".to_string(), "val1".to_string());

        let mut h2 = HashMap::new();
        h2.insert("h1".to_string(), "val1".to_string());
        h2.insert("h2".to_string(), "val2".to_string());

        let req1 = ApiRequest {
            url: "/api/test".to_string(),
            headers: h1,
            method: "POST".to_string(),
            body: Some(r#"{"ok":true}"#.to_string()),
            params: p1,
            query: q1,
        };

        let req2 = ApiRequest {
            url: "/api/test".to_string(),
            headers: h2,
            method: "POST".to_string(),
            body: Some(r#"{"ok":true}"#.to_string()),
            params: p2,
            query: q2,
        };

        let json1 = serde_json::to_string(&req1).unwrap();
        let json2 = serde_json::to_string(&req2).unwrap();

        assert_eq!(json1, json2);
    }

    #[test]
    fn test_repeated_serialization_produces_identical_output() {
        let mut query = HashMap::new();
        query.insert("z".to_string(), "3".to_string());
        query.insert("m".to_string(), "2".to_string());
        query.insert("a".to_string(), "1".to_string());

        let req = ApiRequest {
            url: "/api/repeated".to_string(),
            headers: HashMap::from([
                ("k2".to_string(), "v2".to_string()),
                ("k1".to_string(), "v1".to_string()),
            ]),
            method: "GET".to_string(),
            body: None,
            params: HashMap::from([
                ("p2".to_string(), "vp2".to_string()),
                ("p1".to_string(), "vp1".to_string()),
            ]),
            query,
        };

        let first = serde_json::to_string(&req).unwrap();
        for _ in 0..50 {
            let subsequent = serde_json::to_string(&req).unwrap();
            assert_eq!(first, subsequent);
        }
    }

    #[test]
    fn test_single_key_requests_continue_working() {
        let req = ApiRequest {
            url: "/api/single".to_string(),
            headers: HashMap::from([("single-header".to_string(), "val".to_string())]),
            method: "GET".to_string(),
            body: None,
            params: HashMap::from([("single-param".to_string(), "val".to_string())]),
            query: HashMap::from([("single-query".to_string(), "val".to_string())]),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""headers":{"single-header":"val"}"#));
        assert!(json.contains(r#""params":{"single-param":"val"}"#));
        assert!(json.contains(r#""query":{"single-query":"val"}"#));

        let parsed: ApiRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.url, "/api/single");
        assert_eq!(
            parsed.headers.get("single-header"),
            Some(&"val".to_string())
        );
        assert_eq!(parsed.params.get("single-param"), Some(&"val".to_string()));
        assert_eq!(parsed.query.get("single-query"), Some(&"val".to_string()));
    }

    #[test]
    fn test_empty_maps_continue_working() {
        let req = ApiRequest {
            url: "/api/empty".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            body: None,
            params: HashMap::new(),
            query: HashMap::new(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(
            json,
            r#"{"url":"/api/empty","headers":{},"method":"GET","body":null,"params":{},"query":{}}"#
        );

        let parsed: ApiRequest = serde_json::from_str(&json).unwrap();
        assert!(parsed.headers.is_empty());
        assert!(parsed.params.is_empty());
        assert!(parsed.query.is_empty());
    }
}
