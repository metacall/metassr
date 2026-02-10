//! MetaSSR API Handler - Polyglot API routes support.
//!
//! This crate provides API route handling for MetaSSR, allowing developers to
//! define backend API endpoints in `./src/api/` using JavaScript files.
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
//! module.exports = { GET, POST };
//! ```

pub mod scanner;
pub mod types;

use anyhow::{anyhow, Result};
use axum::{
    extract::Query,
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, MethodRouter},
    Router,
};
use metacall::{load, metacall};
use scanner::{scan_api_dir, ApiRouteFile};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, error, info, warn};
use types::{ApiRequest, ApiResponse};

/// Stores loaded API route scripts.
///
/// NOTE: Currently uses the shared global MetaCall context.
/// This is a testing behavior that might change in the future to use
/// a dedicated MetaCall runtime thread for better isolation.
pub struct ApiRoutes {
    /// Set of loaded script paths (to avoid reloading).
    loaded_scripts: HashMap<String, PathBuf>,
    /// List of discovered route files.
    routes: Vec<ApiRouteFile>,
}

impl ApiRoutes {
    /// Create a new empty ApiRoutes instance.
    pub fn new() -> Self {
        Self {
            loaded_scripts: HashMap::new(),
            routes: Vec::new(),
        }
    }

    /// Scan the given API directory and load all JavaScript files.
    ///
    /// # Arguments
    /// * `api_dir` - Path to the API directory (typically `./src/api/`)
    pub fn load_from_dir(&mut self, api_dir: &Path) -> Result<()> {
        let route_files = scan_api_dir(api_dir);

        if route_files.is_empty() {
            debug!("No API routes found in {:?}", api_dir);
            return Ok(());
        }

        info!("Found {} API route(s)", route_files.len());

        for route_file in &route_files {
            if let Err(e) = self.load_script(&route_file.file_path) {
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

    /// Load a single JavaScript file into MetaCall.
    fn load_script(&mut self, file_path: &Path) -> Result<()> {
        let code = read_to_string(file_path)?;
        let path_str = file_path.to_string_lossy().to_string();

        // NOTE: Using shared global MetaCall context.
        // This is a testing behavior that might change to use a dedicated
        // MetaCall runtime thread for better async handling and isolation.
        // Passing None as handle to use the global context.
        load::from_memory(load::Tag::NodeJS, &code, None)
            .map_err(|e| anyhow!("Failed to load script {:?}: {:?}", file_path, e))?;

        self.loaded_scripts
            .insert(path_str, file_path.to_path_buf());
        Ok(())
    }

    /// Call a handler function (GET, POST) on a loaded script.
    /// The function name should match the HTTP method (GET, POST, etc.)
    pub fn call_handler(
        &self,
        _file_path: &str,
        method: &str,
        request: ApiRequest,
    ) -> Result<ApiResponse> {
        let request_json = serde_json::to_string(&request)?;

        debug!("Calling {}() with request: {}", method, request_json);

        // Call the handler function with the request JSON
        // MetaCall looks up the function by name in all loaded scripts
        let result: String = metacall(method, [request_json])
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
pub fn register_api_routes(
    mut router: Router,
    root_path: &Path,
) -> Result<(Router, Option<Arc<std::sync::Mutex<ApiRoutes>>>)> {
    let api_dir = root_path.join("src").join("api");

    if !api_dir.exists() {
        debug!(
            "No API directory found at {:?}, skipping API routes",
            api_dir
        );
        return Ok((router, None));
    }

    let mut api_routes = ApiRoutes::new();
    api_routes.load_from_dir(&api_dir)?;

    if api_routes.routes().is_empty() {
        return Ok((router, None));
    }

    let api_routes = Arc::new(std::sync::Mutex::new(api_routes));

    // Clone routes info before moving api_routes
    let routes_info: Vec<_> = api_routes
        .lock()
        .unwrap()
        .routes()
        .iter()
        .map(|r| {
            (
                r.route_path.clone(),
                r.file_path.to_string_lossy().to_string(),
            )
        })
        .collect();

    for (route_path, file_path) in routes_info {
        let api_routes_clone = Arc::clone(&api_routes);
        let file_path_clone = file_path.clone();
        let route_path_clone = route_path.clone();

        // Create method router for GET and POST
        let method_router: MethodRouter = get({
            let api_routes = Arc::clone(&api_routes_clone);
            let file_path = file_path_clone.clone();
            let route_path = route_path_clone.clone();
            move |headers: HeaderMap, Query(query): Query<HashMap<String, String>>, body: String| {
                let api_routes = Arc::clone(&api_routes);
                let file_path = file_path.clone();
                let route_path = route_path.clone();
                async move {
                    handle_api_request(
                        api_routes,
                        headers,
                        Method::GET,
                        query,
                        body,
                        file_path,
                        route_path,
                    )
                }
            }
        })
        .post({
            let api_routes = Arc::clone(&api_routes_clone);
            let file_path = file_path_clone.clone();
            let route_path = route_path_clone.clone();
            move |headers: HeaderMap, Query(query): Query<HashMap<String, String>>, body: String| {
                let api_routes = Arc::clone(&api_routes);
                let file_path = file_path.clone();
                let route_path = route_path.clone();
                async move {
                    handle_api_request(
                        api_routes,
                        headers,
                        Method::POST,
                        query,
                        body,
                        file_path,
                        route_path,
                    )
                }
            }
        });

        router = router.route(&route_path, method_router);
        info!("Registered API route: {}", route_path);
    }

    Ok((router, Some(api_routes)))
}

/// Handle an incoming API request.
fn handle_api_request(
    api_routes: Arc<std::sync::Mutex<ApiRoutes>>,
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

    let routes = api_routes.lock().unwrap();
    match routes.call_handler(&file_path, method.as_str(), request) {
        Ok(response) => (
            StatusCode::from_u16(response.status).unwrap_or(StatusCode::OK),
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&response.body).unwrap_or_else(|_| "{}".to_string()),
        ),
        Err(error) => {
            error!("API handler error: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(axum::http::header::CONTENT_TYPE, "application/json")],
                format!("{{\"error\": \"{}\"}}", error),
            )
        }
    }
}
