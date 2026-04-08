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
use metacall::{load, metacall_handle};
use scanner::{scan_api_dir, ApiRouteFile};
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{debug, error, info, warn};
use types::{ApiRequest, ApiResponse};

// `load::Handle` holds a raw `*mut c_void` and is not `Send` by default.
// SAFETY: ScriptHandle is only ever accessed while holding the Mutex<ApiRoutes> lock.
struct ScriptHandle(load::Handle);
unsafe impl Send for ScriptHandle {}

/// Stores loaded API route scripts, each in its own isolated MetaCall handle.
pub struct ApiRoutes {
    loaded_scripts: HashMap<String, (PathBuf, ScriptHandle)>,
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

    /// Load a single JavaScript file into its own MetaCall handle.
    fn load_script(&mut self, file_path: &Path) -> Result<()> {
        let code = read_to_string(file_path)?;
        let path_str = file_path.to_string_lossy().to_string();
        let mut handle = load::Handle::new();

        // Each script gets its own handle so symbols can be cleared independently on reload.
        load::from_memory(load::Tag::NodeJS, &code, Some(&mut handle))
            .map_err(|e| anyhow!("Failed to load script {:?}: {:?}", file_path, e))?;

        self.loaded_scripts
            .insert(path_str, (file_path.to_path_buf(), ScriptHandle(handle)));
        Ok(())
    }

    /// Reload a changed script: drops the old handle (clearing its symbols) then reloads.
    pub fn reload_script(&mut self, file_path: &Path) -> Result<()> {
        let path_str = file_path.to_string_lossy().to_string();

        // Dropping the entry calls metacall_clear, unregistering the old symbols.
        self.loaded_scripts.remove(&path_str);

        let code = read_to_string(file_path)?;
        let mut handle = load::Handle::new();

        load::from_memory(load::Tag::NodeJS, &code, Some(&mut handle))
            .map_err(|e| anyhow!("Failed to reload script {:?}: {:?}", file_path, e))?;

        self.loaded_scripts
            .insert(path_str, (file_path.to_path_buf(), ScriptHandle(handle)));

        info!("Reloaded API script: {:?}", file_path);
        Ok(())
    }

    /// Call a handler function (GET, POST) on a loaded script.
    /// The function name should match the HTTP method (GET, POST, etc.)
    pub fn call_handler(
        &mut self,
        file_path: &str,
        method: &str,
        request: ApiRequest,
    ) -> Result<ApiResponse> {
        let request_json = serde_json::to_string(&request)?;

        debug!("Calling {}() with request: {}", method, request_json);

        let (_, script_handle) = self
            .loaded_scripts
            .get_mut(file_path)
            .ok_or_else(|| anyhow!("Script not loaded: {}", file_path))?;

        let result: String = metacall_handle(&mut script_handle.0, method, [request_json])
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

    let mut routes = api_routes.lock().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use std::fs;

    #[test]
    fn register_api_routes_skips_when_api_dir_missing() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();

        let base_router = Router::new();
        let (_router, routes) = register_api_routes(base_router, root).unwrap();

        assert!(routes.is_none());
    }

    #[test]
    fn register_api_routes_skips_when_api_dir_empty() {
        let tmp = tempfile::TempDir::new().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/api")).unwrap();

        let (router, routes) = register_api_routes(Router::new(), root).unwrap();
        let _ = router;
        assert!(routes.is_none());
    }

    #[test]
    fn call_handler_returns_error_when_method_is_missing() {
        let mut routes = ApiRoutes::new();
        let request = ApiRequest {
            url: "/api/hello".to_string(),
            headers: HashMap::new(),
            method: "GET".to_string(),
            query: HashMap::new(),
            body: None,
            params: HashMap::new(),
        };

        let result = routes.call_handler("src/api/hello.js", "GET", request);
        assert!(result.is_err());
    }
}
