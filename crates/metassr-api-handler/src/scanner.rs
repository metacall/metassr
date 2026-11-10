//! Directory scanner for API route files.

use anyhow::Result;
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
/// Scans for `.js` (NodeJS), `.ts` (TypeScript), `.py` (Python) and `.rb` (Ruby) files.
pub fn scan_api_dir(api_dir: &Path) -> Vec<ApiRouteFile> {
    let mut routes = Vec::new();

    if !api_dir.exists() {
        tracing::debug!("API directory {:?} not found, skipping", api_dir);
        return routes;
    }

    scan_api_dir_recursive(api_dir, api_dir, &mut routes);

    // Sort by route path, then file path, so that duplicate detection is
    // deterministic regardless of directory iteration order. This also
    // decides which file "wins" when duplicates are skipped.
    routes.sort_by(|a, b| {
        a.route_path
            .cmp(&b.route_path)
            .then_with(|| a.file_path.cmp(&b.file_path))
    });

    routes
}

/// What to do when two or more files map to the same route path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateRoutesPolicy {
    /// Fail with an actionable error. Used by production builds, where an
    /// invalid route layout must be rejected instead of surprising the user.
    Error,
    /// Warn and keep the first route (sorted order), dropping the duplicates.
    /// Used by the development server so it can still boot and iterate.
    Skip,
}

/// Detect route files that map to the same route path and resolve them
/// according to the policy.
///
/// Returns the route files to use (duplicates removed under [`DuplicateRoutesPolicy::Skip`])
/// or a descriptive error listing every conflicting file once per route path
/// (under [`DuplicateRoutesPolicy::Error`]).
///
/// `root_path` is used only to shorten paths in messages.
pub fn resolve_route_duplicates(
    mut route_files: Vec<ApiRouteFile>,
    policy: DuplicateRoutesPolicy,
    root_path: &Path,
) -> Result<Vec<ApiRouteFile>> {
    use std::collections::BTreeMap;

    // Group routes by route path, preserving (already sorted) order.
    let mut groups: BTreeMap<String, Vec<ApiRouteFile>> = BTreeMap::new();
    for route_file in route_files.drain(..) {
        groups
            .entry(route_file.route_path.clone())
            .or_default()
            .push(route_file);
    }

    for (route_path, group) in groups.iter_mut() {
        if group.len() <= 1 {
            continue;
        }

        let files = group
            .iter()
            .map(|r| display_path(root_path, &r.file_path))
            .collect::<Vec<_>>()
            .join(", ");

        match policy {
            DuplicateRoutesPolicy::Error => {
                anyhow::bail!(
                    "Duplicate API route '{}' is registered by {} files: {}. \
                     Keep only one file per route path (the extension selects the loader).",
                    route_path,
                    group.len(),
                    files
                );
            }
            DuplicateRoutesPolicy::Skip => {
                tracing::warn!(
                    "Duplicate API route '{}' found in files: {}. \
                     Keeping '{}' and skipping the rest.",
                    route_path,
                    files,
                    display_path(root_path, &group[0].file_path)
                );

                // Keep only the first (sorted) one.
                group.truncate(1);
            }
        }
    }

    Ok(groups.into_values().flatten().collect())
}

/// Prefer a path relative to the project root for friendlier error messages.
fn display_path(root_path: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(root_path)
        .unwrap_or(file_path)
        .to_string_lossy()
        .into_owned()
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
            // The file extension selects the MetaCall loader
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
                // Declaration files (`*.d.ts`) only hold types, they are not routes
                Some("ts") if !is_declaration_file(&path) => {
                    if let Ok(relative_path) = path.strip_prefix(base_path) {
                        let route_path = build_route_path(relative_path, &path);
                        routes.push(ApiRouteFile {
                            route_path,
                            file_path: path,
                            tag: Tag::TypeScript,
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
                _ => {}
            }
        }
    }
}

/// Whether the path is a TypeScript declaration file (`*.d.ts`).
fn is_declaration_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
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
    fn test_scan_api_dir_finds_ts_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create api/hello.ts and api/users/list.ts (nested)
        fs::write(temp_dir.path().join("hello.ts"), "function GET() {}").unwrap();
        let users_dir = temp_dir.path().join("users");
        fs::create_dir(&users_dir).unwrap();
        fs::write(users_dir.join("list.ts"), "function GET() {}").unwrap();

        // Create a non-route file (should be ignored)
        fs::write(temp_dir.path().join("readme.txt"), "ignore me").unwrap();

        let routes = scan_api_dir(temp_dir.path());

        assert_eq!(routes.len(), 2, "Should find exactly 2 TS files");
        for route in &routes {
            assert!(
                matches!(route.tag, Tag::TypeScript),
                "TS files should use the TypeScript loader"
            );
        }

        let route_paths: Vec<_> = routes.iter().map(|r| r.route_path.as_str()).collect();
        assert!(route_paths.contains(&"/api/hello"));
        assert!(route_paths.contains(&"/api/users/list"));
    }

    #[test]
    fn test_scan_api_dir_ignores_ts_declaration_files() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("hello.ts"), "function GET() {}").unwrap();
        fs::write(
            temp_dir.path().join("types.d.ts"),
            "export type Id = string;",
        )
        .unwrap();

        let routes = scan_api_dir(temp_dir.path());

        assert_eq!(routes.len(), 1, "Should skip .d.ts declaration files");
        assert_eq!(routes[0].route_path, "/api/hello");
    }

    #[test]
    fn test_scan_api_dir_assigns_loader_by_extension() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("a.js"), "").unwrap();
        fs::write(temp_dir.path().join("b.ts"), "").unwrap();
        fs::write(temp_dir.path().join("c.py"), "").unwrap();

        let routes = scan_api_dir(temp_dir.path());

        let tag_of = |route_path: &str| {
            routes
                .iter()
                .find(|r| r.route_path == route_path)
                .map(|r| r.tag)
        };
        assert!(matches!(tag_of("/api/a"), Some(Tag::NodeJS)));
        assert!(matches!(tag_of("/api/b"), Some(Tag::TypeScript)));
        assert!(matches!(tag_of("/api/c"), Some(Tag::Python)));
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

    /// Create hello.js + hello.py + world.js in a temp api dir and return the
    /// scanned routes (sorted, so hello.js comes before hello.py).
    fn create_duplicate_routes() -> (TempDir, Vec<ApiRouteFile>) {
        let temp_dir = TempDir::new().unwrap();
        let api = temp_dir.path();
        fs::write(api.join("hello.js"), "function GET() {}").unwrap();
        fs::write(api.join("hello.py"), "def GET(request): pass").unwrap();
        fs::write(api.join("world.js"), "function GET() {}").unwrap();
        let routes = scan_api_dir(api);
        (temp_dir, routes)
    }

    #[test]
    fn test_scan_api_dir_sorts_routes_deterministically() {
        let (_tmp, routes) = create_duplicate_routes();

        let paths: Vec<&str> = routes.iter().map(|r| r.route_path.as_str()).collect();
        assert_eq!(paths, vec!["/api/hello", "/api/hello", "/api/world"]);
    }

    #[test]
    fn test_resolve_duplicates_errors_and_lists_files() {
        let (tmp, routes) = create_duplicate_routes();

        let result = resolve_route_duplicates(routes, DuplicateRoutesPolicy::Error, tmp.path());

        let err = result.err().expect("should fail with an actionable error");
        let msg = err.to_string();

        assert!(msg.contains("Duplicate API route '/api/hello'"), "{msg}");
        assert!(msg.contains("hello.js"), "{msg}");
        assert!(msg.contains("hello.py"), "{msg}");
        assert!(!msg.contains("world"), "{msg}");
    }

    #[test]
    fn test_resolve_duplicates_skip_keeps_first_sorted_route() {
        let (tmp, routes) = create_duplicate_routes();

        let resolved = resolve_route_duplicates(routes, DuplicateRoutesPolicy::Skip, tmp.path())
            .expect("skip policy should not fail");

        let paths: Vec<&str> = resolved.iter().map(|r| r.route_path.as_str()).collect();
        assert_eq!(paths, vec!["/api/hello", "/api/world"]);

        // The lexicographically first file wins (hello.js before hello.py).
        assert_eq!(resolved[0].file_path.extension().unwrap(), "js");
    }

    #[test]
    fn test_resolve_duplicates_noop_when_unique() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("hello.js"), "function GET() {}").unwrap();
        let routes = scan_api_dir(temp_dir.path());

        for policy in [DuplicateRoutesPolicy::Error, DuplicateRoutesPolicy::Skip] {
            let resolved =
                resolve_route_duplicates(routes.clone(), policy, temp_dir.path()).unwrap();
            assert_eq!(resolved.len(), 1);
        }
    }
}
