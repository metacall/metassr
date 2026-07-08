//! Directory scanner for API route files.

use std::path::{Path, PathBuf};

use metacall::load::Tag;

/// Represents a discovered API route file.
#[derive(Debug, Clone)]
pub struct ApiRouteFile {
    /// The HTTP route path (e.g., "/api/users").
    pub route_path: String,
    /// The absolute file path to the script.
    pub file_path: PathBuf,
    /// The Programming Language MetaCall Tag used in the route.
    pub tag: Tag,
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
            let ext = path.extension().and_then(|s| s.to_str());

            match ext {
                Some("js") => {
                    if let Ok(relative_path) = path.strip_prefix(base_path) {
                        let route_path = build_route_path(relative_path, &path);
                        routes.push(ApiRouteFile {
                            route_path,
                            file_path: path,
                            tag: Tag::NodeJS,
                        });
                    }
                }
                Some("rb") => {
                    if let Ok(relative_path) = path.strip_prefix(base_path) {
                        let route_path = build_route_path(relative_path, &path);
                        routes.push(ApiRouteFile {
                            route_path,
                            file_path: path,
                            tag: Tag::Ruby,
                        });
                    }
                }
                Some("py") => {
                    if let Ok(relative_path) = path.strip_prefix(base_path) {
                        let route_path = build_route_path(relative_path, &path);
                        routes.push(ApiRouteFile {
                            route_path,
                            file_path: path,
                            tag: Tag::Python,
                        });
                    }
                }
                _ => {
                    panic!("language not supported")
                }
            }
        }
    }
}

/// Build the HTTP route path from the file path.
/// Example: api/users/list.js -> /api/users/list
fn build_route_path(relative_path: &Path, full_path: &Path) -> String {
    let route_parts: Vec<&str> = relative_path.iter().filter_map(|s| s.to_str()).collect();

    let mut route_path = String::from("/api");
    for (i, part) in route_parts.iter().enumerate() {
        if i == route_parts.len() - 1 {
            // Last part is the filename, remove extension
            if let Some(stem) = full_path.file_stem().and_then(|s| s.to_str()) {
                route_path.push('/');
                route_path.push_str(stem);
            }
        } else {
            // Directory name
            route_path.push('/');
            route_path.push_str(part);
        }
    }

    route_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_api_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create api/hello.js
        fs::write(temp_dir.path().join("hello.js"), "function GET() {}").unwrap();

        // Create api/users/list.js (nested)
        let users_dir = temp_dir.path().join("users");
        fs::create_dir(&users_dir).unwrap();
        fs::write(users_dir.join("list.js"), "function GET() {}").unwrap();

        // Create a non-js file (should be ignored)
        fs::write(temp_dir.path().join("readme.txt"), "ignore me").unwrap();

        temp_dir
    }

    #[test]
    fn test_scan_api_dir_finds_js_files() {
        let temp_dir = create_test_api_dir();
        let routes = scan_api_dir(temp_dir.path());

        assert_eq!(routes.len(), 2, "Should find exactly 2 JS files");
    }

    #[test]
    fn test_scan_api_dir_ignores_non_js_files() {
        let temp_dir = create_test_api_dir();
        let routes = scan_api_dir(temp_dir.path());

        // Should not contain any .txt files
        for route in &routes {
            assert!(
                route.file_path.extension().unwrap() == "js",
                "Should only find .js files"
            );
        }
    }

    #[test]
    fn test_scan_api_dir_builds_correct_routes() {
        let temp_dir = create_test_api_dir();
        let routes = scan_api_dir(temp_dir.path());

        let route_paths: Vec<_> = routes.iter().map(|r| r.route_path.as_str()).collect();

        assert!(
            route_paths.contains(&"/api/hello"),
            "Should have /api/hello route"
        );
        assert!(
            route_paths.contains(&"/api/users/list"),
            "Should have /api/users/list route"
        );
    }

    #[test]
    fn test_scan_nonexistent_dir_returns_empty() {
        let routes = scan_api_dir(Path::new("/nonexistent/path"));
        assert!(routes.is_empty(), "Should return empty for nonexistent dir");
    }

    #[test]
    fn test_build_route_path() {
        let relative = Path::new("hello.js");
        let full = Path::new("/some/path/hello.js");
        let route = build_route_path(relative, full);
        assert_eq!(route, "/api/hello");
    }

    #[test]
    fn test_build_route_path_nested() {
        let relative = Path::new("users/profile.js");
        let full = Path::new("/some/path/users/profile.js");
        let route = build_route_path(relative, full);
        assert_eq!(route, "/api/users/profile");
    }
}
