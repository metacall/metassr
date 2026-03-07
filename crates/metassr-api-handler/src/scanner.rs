//! Directory scanner for API route files.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a discovered API route file.
#[derive(Debug, Clone)]
pub struct ApiRouteFile {
    /// The HTTP route path (e.g., "/api/users").
    pub route_path: String,
    /// The absolute file path to the script.
    pub file_path: PathBuf,
}

/// Scan the api directory and return list of discovered route files.
/// Only scans for .js files (NodeJS support only for now).
pub fn scan_api_dir(api_dir: &Path) -> Vec<ApiRouteFile> {
    let mut routes = Vec::new();

    if !api_dir.exists() {
        tracing::debug!("API directory {:?} not found, skipping", api_dir);
        return routes;
    }

    scan_api_dir_recursive(api_dir, api_dir, &mut routes);

    // Deduplicate routes to prevent Axum from panicking on duplicate route registration.
    // Note: read_dir order is nondeterministic, so we sort first to ensure the winner
    // is always deterministic. Lexicographic order puts `users/index.js` before `users.js`,
    // so the index file is kept and the flat sibling is skipped.
    routes.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    routes.retain(|r| {
        if let Some(kept) = seen.get(&r.route_path) {
            tracing::warn!(
                "Duplicate API route '{}': keeping {:?}, skipping {:?}.",
                r.route_path,
                kept,
                r.file_path,
            );
            false
        } else {
            seen.insert(r.route_path.clone(), r.file_path.clone());
            true
        }
    });

    routes
}

/// Recursively scan directories for API files.
fn scan_api_dir_recursive(base_path: &Path, current_path: &Path, routes: &mut Vec<ApiRouteFile>) {
    let entries = match current_path.read_dir() {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Failed to read directory {:?}: {}", current_path, e);
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // Recursively scan subdirectories
            scan_api_dir_recursive(base_path, &path, routes);
        } else if path.is_file() {
            // Only support .js files for now (NodeJS)
            let is_js = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext == "js")
                .unwrap_or(false);

            if is_js {
                if let Ok(relative_path) = path.strip_prefix(base_path) {
                    let route_path = build_route_path(relative_path, &path);
                    routes.push(ApiRouteFile {
                        route_path,
                        file_path: path,
                    });
                }
            }
        }
    }
}

/// Build the HTTP route path from the file path.
///
/// Files named `index.js` map to the route of their parent directory:
/// - `api/users/index.js`  -> `/api/users`
/// - `api/hello.js`        -> `/api/hello`
/// - `api/users/list.js`   -> `/api/users/list`
fn build_route_path(relative_path: &Path, full_path: &Path) -> String {
    let route_parts: Vec<&str> = relative_path.iter().filter_map(|s| s.to_str()).collect();

    let mut route_path = String::from("/api");
    for (i, part) in route_parts.iter().enumerate() {
        if i == route_parts.len() - 1 {
            // Skip "index" stem so `users/index.js` maps to `/api/users`.
            if let Some(stem) = full_path.file_stem().and_then(|s| s.to_str()) {
                if stem != "index" {
                    route_path.push('/');
                    route_path.push_str(stem);
                }
            }
        } else {
            route_path.push('/');
            route_path.push_str(part);
        }
    }

    route_path
}

#[cfg(test)]
mod tests {
    use super::build_route_path;
    use std::path::Path;

    /// Test case for a regular file mapping to a named route.
    #[test]
    fn test_regular_file_maps_to_named_route() {
        let relative = Path::new("hello.js");
        let full = Path::new("/project/src/api/hello.js");
        assert_eq!(build_route_path(relative, full), "/api/hello");
    }

    /// Test case for an index.js at the api root mapping to /api.
    #[test]
    fn test_index_file_at_root_maps_to_api_root() {
        let relative = Path::new("index.js");
        let full = Path::new("/project/src/api/index.js");
        assert_eq!(build_route_path(relative, full), "/api");
    }

    /// Test case for a nested regular file mapping to its full path.
    #[test]
    fn test_nested_regular_file_maps_to_full_path() {
        let relative = Path::new("users/list.js");
        let full = Path::new("/project/src/api/users/list.js");
        assert_eq!(build_route_path(relative, full), "/api/users/list");
    }

    /// Test case for a nested index.js mapping to its parent directory route.
    #[test]
    fn test_nested_index_file_maps_to_parent_directory_route() {
        let relative = Path::new("users/index.js");
        let full = Path::new("/project/src/api/users/index.js");
        assert_eq!(build_route_path(relative, full), "/api/users");
    }

    /// Test case for a deeply nested index.js mapping to its parent directory route.
    #[test]
    fn test_deeply_nested_index_file_maps_to_parent_directory_route() {
        let relative = Path::new("v1/users/index.js");
        let full = Path::new("/project/src/api/v1/users/index.js");
        assert_eq!(build_route_path(relative, full), "/api/v1/users");
    }
}
