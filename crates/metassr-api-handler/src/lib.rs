//! MetaSSR API Handler - Polyglot API routes support.
//!
//! This crate provides API route handling for MetaSSR, allowing developers to
//! define backend API endpoints in `./src/api/` using JavaScript or TypeScript files.
//!
//! # Example
//!
//! Create a file at `./src/api/hello.js`:
//!
//! ```javascript
//! function GET(req) {
//!     return JSON.stringify({
//!         status: 200,
//!         body: { message: "Hello from API!" }
//!     });
//! }
//!
//! function POST(req) {
//!     const data = JSON.parse(req.body || "{}");
//!     return JSON.stringify({
//!         status: 201,
//!         body: { received: data }
//!     });
//! }
//!
//! module.exports = { GET, POST, PUT, DELETE };
//! ```

pub mod scanner;
pub mod types;

use anyhow::{anyhow, Result};
use axum::{
    extract::Query,
    http::{header, HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::MethodRouter,
    Router,
};
use lock_freedom::map::Map;
use metacall::{
    load::{self, Handle, Tag},
    metacall_handle,
};
use scanner::{scan_api_dir, ApiRouteFile};
use std::{collections::HashMap, ffi::CString, path::Path, sync::Arc};
use tracing::{debug, error, info, warn};
use types::{ApiRequest, ApiResponse};

/// Stores loaded API route scripts and their MetaCall handles.
pub struct ApiRoutes {
    handles: Map<String, Handle>,
    /// List of discovered route files.
    routes: Vec<ApiRouteFile>,
}

impl ApiRoutes {
    /// Create a new empty ApiRoutes instance.
    pub fn new() -> Self {
        Self {
            handles: Map::new(),
            routes: Vec::new(),
        }
    }

    /// Scan the given API directory and load all script files.
    ///
    /// # Arguments
    /// * `api_dir` - Path to the API directory (typically `./src/api/`)
    pub async fn load_from_dir(&mut self, api_dir: &Path) -> Result<()> {
        let route_files = scan_api_dir(api_dir);

        if route_files.is_empty() {
            debug!("No API routes found in {:?}", api_dir);
            return Ok(());
        }

        info!("Found {} API route(s)", route_files.len());

        for route_file in &route_files {
            if let Err(e) = self
                .load_script(&route_file.file_path, route_file.tag)
                .await
            {
                warn!("Failed to load API route {:?}: {}", route_file.file_path, e);
            } else {
                info!(
                    "  Loaded: {} -> {:?}",
                    route_file.route_path, route_file.file_path
                );
            }
        }

        self.routes = route_files;
        Ok(())
    }

    /// Load a single script into MetaCall.
    async fn load_script(&mut self, file_path: &Path, tag: Tag) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();
        if self.handles.get(&path_str).is_some() {
            return Ok(());
        }
        let mut handle = Handle::new();
        load::from_file(tag, [path_str.clone()], Some(&mut handle))
            .map_err(|e| anyhow!("Failed to load script {:?}: {:?}", file_path, e))?;

        self.handles.insert(path_str, handle);

        Ok(())
    }
    /// Check whether a loaded script handle exports a function with the given name.
    ///
    /// # Safety
    /// This function calls the MetaCall C FFI `metacall_handle_function`. It performs
    /// null-pointer checks on both the handle pointer and function pointer, and only
    /// accesses the handle within the `self.handles` guard lifetime.
    pub fn has_method(&self, file_path: &str, method: &str) -> bool {
        let guard = match self.handles.get(file_path) {
            Some(g) => g,
            None => return false,
        };
        let raw_handle = guard.val().as_mut_raw_ptr();
        if raw_handle.is_null() {
            return false;
        }
        let c_method = match CString::new(method) {
            Ok(c) => c,
            Err(_) => return false,
        };
        // SAFETY: `raw_handle` is verified non-null and guarded by `self.handles`.
        // `c_method` is a valid null-terminated C string. `metacall_handle_function`
        // inspects the handle's symbol table without mutating it.
        let func_ptr =
            unsafe { metacall::bindings::metacall_handle_function(raw_handle, c_method.as_ptr()) };
        !func_ptr.is_null()
    }

    /// Returns a list of supported HTTP methods (GET, POST, PUT, DELETE) exported by the script.
    pub fn exported_methods(&self, file_path: &str) -> Vec<Method> {
        const CANDIDATES: [Method; 4] = [Method::GET, Method::POST, Method::PUT, Method::DELETE];
        CANDIDATES
            .into_iter()
            .filter(|m| self.has_method(file_path, m.as_str()))
            .collect()
    }

    /// Reload a changed script: drops the old handle (clearing its symbols) then reloads.
    pub fn reload_script(&self, file_path: &Path, tag: Tag) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();

        {
            self.handles.remove(&path_str);
        }

        let code = std::fs::read_to_string(file_path)?;

        let mut handle = Handle::new();
        load::from_memory(tag, code, Some(&mut handle))
            .map_err(|e| anyhow!("Failed to reload script {:?}: {:?}", file_path, e))?;

        self.handles.insert(path_str, handle);

        info!("Reloaded API script: {:?}", file_path);
        Ok(())
    }

    /// Call a handler function (GET, POST, PUT, DELETE) on a loaded script.
    /// The function name should match the HTTP method (GET, POST, etc.)
    pub async fn call_handler(
        &self,
        file_path: &str,
        method: &str,
        request: ApiRequest,
    ) -> Result<ApiResponse> {
        if !self.has_method(file_path, method) {
            return Err(anyhow!("Method {} not exported", method));
        }

        let request_json = serde_json::to_string(&request)?;

        debug!("Calling {}() with request: {}", method, request_json);

        // Call the handler function with the request JSON
        // MetaCall looks up the function by name in all loaded scripts
        let guard = &self
            .handles
            .get(file_path)
            .ok_or_else(|| anyhow!("No loaded handle for API script: {}", file_path))?;
        let result: String = metacall_handle(guard.val(), method, vec![request_json])
            .map_err(|e| anyhow!("Failed to call {}: {:?}", method, e))?;

        let response: ApiResponse = serde_json::from_str(&result)
            .map_err(|e| anyhow!("Failed to parse response: {} (raw: {})", e, result))?;

        Ok(response)
    }

    /// Get the list of discovered routes.
    pub fn routes(&self) -> &[ApiRouteFile] {
        &self.routes
    }
}

impl Default for ApiRoutes {
    fn default() -> Self {
        Self::new()
    }
}

/// Register API routes on an Axum router.
///
/// # Arguments
/// * `router` - The Axum router to add routes to
/// * `root_path` - Root path of the project (to find `./src/api/`)
pub async fn register_api_routes(
    mut router: Router,
    root_path: &Path,
) -> Result<(Router, Option<Arc<ApiRoutes>>)> {
    let api_dir = root_path.join("src").join("api");

    if !api_dir.exists() {
        debug!(
            "No API directory found at {:?}, skipping API routes",
            api_dir
        );
        return Ok((router, None));
    }

    let mut api_routes = ApiRoutes::new();
    api_routes.load_from_dir(&api_dir).await?;

    if api_routes.routes().is_empty() {
        return Ok((router, None));
    }

    // Clone routes info before moving api_routes
    let routes_info: Vec<_> = api_routes
        .routes()
        .iter()
        .map(|r| {
            (
                r.route_path.clone(),
                r.file_path.to_string_lossy().to_string(),
            )
        })
        .collect();

    let api_routes = Arc::new(api_routes);

    for (route_path, file_path) in routes_info {
        let make_handler = |method: Method| {
            let api_routes = Arc::clone(&api_routes);
            let file_path = file_path.clone();
            let route_path = route_path.clone();
            move |headers: HeaderMap, Query(query): Query<HashMap<String, String>>, body: String| {
                let api_routes = Arc::clone(&api_routes);
                let file_path = file_path.clone();
                let route_path = route_path.clone();
                async move {
                    handle_api_request(
                        api_routes, headers, method, query, body, file_path, route_path,
                    )
                    .await
                }
            }
        };

        // Create method router for supported API handler exports.
        let exported = api_routes.exported_methods(&file_path);
        let mut method_router = MethodRouter::new();
        if exported.contains(&Method::GET) {
            method_router = method_router.get(make_handler(Method::GET));
        }
        if exported.contains(&Method::POST) {
            method_router = method_router.post(make_handler(Method::POST));
        }
        if exported.contains(&Method::PUT) {
            method_router = method_router.put(make_handler(Method::PUT));
        }
        if exported.contains(&Method::DELETE) {
            method_router = method_router.delete(make_handler(Method::DELETE));
        }
        router = router.route(&route_path, method_router);
        info!("Registered API route: {}", route_path);
    }

    Ok((router, Some(api_routes)))
}

/// Build the outgoing response headers.
///
/// Starts from an `application/json` content type and then applies the headers
/// returned by the handler. Handler headers win, so they can override the
/// default content type or add cookies, CORS and cache directives. Entries with
/// an invalid header name or value are skipped with a warning.
fn merge_response_headers(extra: &HashMap<String, String>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    for (name, value) in extra {
        match (
            header::HeaderName::try_from(name.as_str()),
            header::HeaderValue::try_from(value.as_str()),
        ) {
            (Ok(name), Ok(value)) => {
                headers.insert(name, value);
            }
            _ => warn!("Ignoring invalid API response header: {name:?}: {value:?}"),
        }
    }

    headers
}

/// Handle an incoming API request.
async fn handle_api_request(
    api_routes: Arc<ApiRoutes>,
    headers: HeaderMap,
    method: Method,
    query: HashMap<String, String>,
    body: String,
    file_path: String,
    route_path: String,
) -> impl IntoResponse {
    let headers_map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(key, value)| {
            value
                .to_str()
                .ok()
                .map(|v| (key.to_string(), v.to_string()))
        })
        .collect();

    let request = ApiRequest {
        url: route_path,
        headers: headers_map,
        method: method.to_string(),
        query,
        body: if body.is_empty() { None } else { Some(body) },
        params: HashMap::new(),
    };

    let method_str = method.as_str();

    let result = api_routes
        .call_handler(&file_path, method_str, request)
        .await;

    match result {
        Ok(response) => (
            StatusCode::from_u16(response.status).unwrap_or(StatusCode::OK),
            merge_response_headers(&response.headers),
            serde_json::to_string(&response.body).unwrap_or_else(|_| "{}".to_string()),
        ),
        Err(e) => {
            error!("API handler error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                merge_response_headers(&HashMap::new()),
                format!("{{\"error\": \"{}\"}}", e),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use std::fs;
    use std::sync::{Mutex, Once};
    use tower_service::Service;

    static INIT_METACALL: Once = Once::new();
    static METACALL_TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn init_metacall_for_tests() -> std::sync::MutexGuard<'static, ()> {
        let guard = METACALL_TEST_MUTEX.lock().unwrap();
        INIT_METACALL.call_once(|| {
            if !metacall::is_initialized() {
                if let Ok(destroy_guard) = metacall::initialize() {
                    std::mem::forget(destroy_guard);
                }
            }
        });
        guard
    }

    #[tokio::test]
    async fn register_api_routes_skips_when_api_dir_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();

        let base_router = Router::new();
        let (_router, routes) = register_api_routes(base_router, root).await.unwrap();

        assert!(routes.is_none());
    }

    #[tokio::test]
    async fn register_api_routes_skips_when_api_dir_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/api")).unwrap();

        let base_router = Router::new();
        let (router, routes) = register_api_routes(base_router, root).await.unwrap();
        let _ = router;
        assert!(routes.is_none());
    }

    #[tokio::test]
    async fn call_handler_returns_error_when_method_is_missing() {
        let routes = ApiRoutes::new();
        let request = ApiRequest {
            url: "/api/hello".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            query: HashMap::new(),
            body: None,
            params: HashMap::new(),
        };

        let result = routes
            .call_handler("src/api/hello.js", "GET", request)
            .await;
        assert!(result.is_err());
    }

    #[test]
    fn handles_map_insert_get_remove() {
        let routes = ApiRoutes::new();
        let path = "src/api/hello.js".to_string();

        assert!(routes.handles.get(&path).is_none());

        routes.handles.insert(path.clone(), Handle::new());
        assert!(routes.handles.get(&path).is_some());

        routes.handles.remove(&path);
        assert!(routes.handles.get(&path).is_none());
    }

    #[test]
    fn merge_response_headers_defaults_to_json() {
        let headers = merge_response_headers(&HashMap::new());

        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn merge_response_headers_forwards_handler_headers() {
        let extra = HashMap::from([
            ("Set-Cookie".to_string(), "session=abc".to_string()),
            ("Cache-Control".to_string(), "no-store".to_string()),
        ]);

        let headers = merge_response_headers(&extra);

        assert_eq!(headers.get("set-cookie").unwrap(), "session=abc");
        assert_eq!(headers.get("cache-control").unwrap(), "no-store");
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[test]
    fn merge_response_headers_lets_handler_override_content_type() {
        let extra = HashMap::from([("Content-Type".to_string(), "text/plain".to_string())]);

        let headers = merge_response_headers(&extra);

        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), "text/plain");
    }

    #[test]
    fn merge_response_headers_skips_invalid_entries() {
        let extra = HashMap::from([("Invalid Header".to_string(), "value".to_string())]);

        let headers = merge_response_headers(&extra);

        assert_eq!(headers.len(), 1);
        assert!(headers.contains_key(header::CONTENT_TYPE));
    }
    #[tokio::test]
    async fn unexported_method_returns_405_with_allow_header() {
        let _lock = init_metacall_for_tests();
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let api_dir = root.join("src/api");
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            api_dir.join("fast.js"),
            r#"
function GET(_req) {
    return JSON.stringify({ status: 200, body: { ok: true } });
}
module.exports = { GET };
"#,
        )
        .unwrap();

        let (mut router, _) = register_api_routes(Router::new(), root).await.unwrap();

        // GET should return 200
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/fast")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // POST should return 405 with Allow header
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/fast")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        let allow = res.headers().get(header::ALLOW).unwrap().to_str().unwrap();
        assert!(allow.contains("GET"));
        assert!(!allow.contains("POST"));

        // DELETE should return 405 with Allow header
        let req = axum::http::Request::builder()
            .method("DELETE")
            .uri("/api/fast")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn module_exporting_none_of_http_methods_returns_405() {
        let _lock = init_metacall_for_tests();
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let api_dir = root.join("src/api");
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            api_dir.join("none.js"),
            r#"
function helper() {}
module.exports = { helper };
"#,
        )
        .unwrap();

        let (mut router, _) = register_api_routes(Router::new(), root).await.unwrap();

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/none")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn module_exporting_multiple_methods_lists_all_in_allow() {
        let _lock = init_metacall_for_tests();
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let api_dir = root.join("src/api");
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            api_dir.join("multi.js"),
            r#"
function GET(_req) {
    return JSON.stringify({ status: 200, body: { ok: true } });
}
function POST(_req) {
    return JSON.stringify({ status: 201, body: { created: true } });
}
module.exports = { GET, POST };
"#,
        )
        .unwrap();

        let (mut router, _) = register_api_routes(Router::new(), root).await.unwrap();

        // GET -> 200
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/multi")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // POST -> 201
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/multi")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);

        // DELETE -> 405 with Allow containing GET and POST
        let req = axum::http::Request::builder()
            .method("DELETE")
            .uri("/api/multi")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        let allow = res.headers().get(header::ALLOW).unwrap().to_str().unwrap();
        assert!(allow.contains("GET"));
        assert!(allow.contains("POST"));
        assert!(!allow.contains("DELETE"));
    }

    #[tokio::test]
    async fn failing_handler_returns_500_not_405() {
        let _lock = init_metacall_for_tests();
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let api_dir = root.join("src/api");
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            api_dir.join("crash.js"),
            r#"
function GET(_req) {
    throw new Error("genuine runtime crash");
}
module.exports = { GET };
"#,
        )
        .unwrap();

        let (mut router, _) = register_api_routes(Router::new(), root).await.unwrap();

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/crash")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn nested_route_unexported_method_returns_405() {
        let _lock = init_metacall_for_tests();
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        let api_dir = root.join("src/api/users");
        fs::create_dir_all(&api_dir).unwrap();

        fs::write(
            api_dir.join("profile.js"),
            r#"
function GET(_req) {
    return JSON.stringify({ status: 200, body: { ok: true } });
}
module.exports = { GET };
"#,
        )
        .unwrap();

        let (mut router, _) = register_api_routes(Router::new(), root).await.unwrap();

        // GET /api/users/profile -> 200
        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/users/profile")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // POST /api/users/profile -> 405
        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/users/profile")
            .body(axum::body::Body::empty())
            .unwrap();
        let res = Service::call(&mut router, req).await.unwrap();
        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
        let allow = res.headers().get(header::ALLOW).unwrap().to_str().unwrap();
        assert!(allow.contains("GET"));
        assert!(!allow.contains("POST"));
    }
}
