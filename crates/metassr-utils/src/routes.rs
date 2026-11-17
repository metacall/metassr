//! Shared route conflict detection for API and page routes.
//!
//! Axum (through `matchit`) identifies a route by the *shape* of its path, not
//! by the parameter names. `/api/:foo` and `/api/:bar` match exactly the same
//! URLs, so registering both panics at startup. This module normalizes routes
//! to a shape, groups the conflicts and applies a [`DuplicateRoutesPolicy`].

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// What to do when two or more files map to the same route shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateRoutesPolicy {
    /// Fail with an actionable error. Used by production builds, where an
    /// invalid route layout must be rejected instead of surprising the user.
    Error,
    /// Warn and keep the first route (sorted order), dropping the duplicates.
    /// Used by the development server so it can still boot and iterate.
    Warn,
}

/// The kind of route being validated. Only used in diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteKind {
    Api,
    Page,
}

impl RouteKind {
    fn plural(self) -> &'static str {
        match self {
            RouteKind::Api => "API routes",
            RouteKind::Page => "page routes",
        }
    }
}

/// A file that claims a route.
pub trait RouteCandidate {
    /// The axum-style route path, e.g. `/api/users/:id`.
    fn route_path(&self) -> &str;
    /// The file that produced the route, used in diagnostics.
    fn file_path(&self) -> &Path;
}

/// A generic route candidate.
#[derive(Debug, Clone)]
pub struct RouteFile {
    pub route_path: String,
    pub file_path: PathBuf,
}

impl RouteCandidate for RouteFile {
    fn route_path(&self) -> &str {
        &self.route_path
    }

    fn file_path(&self) -> &Path {
        &self.file_path
    }
}

/// A route shape claimed by more than one file.
struct DuplicateRoute {
    shape: String,
    /// `(route_path, file)` pairs, with paths shortened for display.
    entries: Vec<(String, String)>,
}

/// Canonical route shape used for conflict detection.
///
/// Mirrors `matchit` (the router used by axum): a wildcard starts at `:` or `*`
/// and runs to the next `/`. The parameter name does not affect matching, so
/// `/api/:foo` and `/api/:bar` both become `/api/:`.
pub fn route_shape(route_path: &str) -> String {
    let mut shape = String::with_capacity(route_path.len());
    let mut chars = route_path.chars();

    while let Some(c) = chars.next() {
        if c == ':' || c == '*' {
            shape.push(c);
            for next in chars.by_ref() {
                if next == '/' {
                    shape.push('/');
                    break;
                }
            }
        } else {
            shape.push(c);
        }
    }

    shape
}

/// Detect candidates that claim the same route shape and resolve them
/// according to the policy.
///
/// Candidates are sorted by `(shape, route_path, file_path)`, so the result and
/// the "first route wins" behavior are deterministic. Under
/// [`DuplicateRoutesPolicy::Warn`] the first candidate of every conflicting
/// shape is kept and the rest are dropped; under
/// [`DuplicateRoutesPolicy::Error`] an error listing every conflict is
/// returned.
///
/// `root_path` is only used to shorten paths in messages.
pub fn resolve_duplicates<T: RouteCandidate>(
    mut candidates: Vec<T>,
    policy: DuplicateRoutesPolicy,
    kind: RouteKind,
    root_path: &Path,
) -> Result<Vec<T>> {
    candidates.sort_by_cached_key(|candidate| {
        (
            route_shape(candidate.route_path()),
            candidate.route_path().to_owned(),
            candidate.file_path().to_path_buf(),
        )
    });

    let mut resolved: Vec<T> = Vec::with_capacity(candidates.len());
    let mut duplicates: Vec<DuplicateRoute> = Vec::new();
    let mut iter = candidates.into_iter().peekable();

    while let Some(first) = iter.next() {
        let shape = route_shape(first.route_path());
        let mut group = vec![first];

        while let Some(next) = iter.peek() {
            if route_shape(next.route_path()) == shape {
                group.push(iter.next().unwrap());
            } else {
                break;
            }
        }

        if group.len() == 1 {
            resolved.push(group.pop().unwrap());
            continue;
        }

        let entries = group
            .iter()
            .map(|candidate| {
                (
                    candidate.route_path().to_owned(),
                    display_path(root_path, candidate.file_path()),
                )
            })
            .collect();

        let mut group = group.into_iter();
        resolved.push(group.next().unwrap());
        duplicates.push(DuplicateRoute { shape, entries });
    }

    if duplicates.is_empty() {
        return Ok(resolved);
    }

    match policy {
        DuplicateRoutesPolicy::Error => Err(anyhow!(duplicate_routes_error(kind, &duplicates))),
        DuplicateRoutesPolicy::Warn => {
            for duplicate in &duplicates {
                let files = duplicate
                    .entries
                    .iter()
                    .map(|(_, file)| file.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");

                tracing::warn!(
                    "Duplicate {} found for shape '{}' in: {}. Keeping '{}' and skipping the rest.",
                    kind.plural(),
                    duplicate.shape,
                    files,
                    duplicate.entries[0].1
                );
            }

            Ok(resolved)
        }
    }
}

/// Build the error message reporting every duplicated route shape.
fn duplicate_routes_error(kind: RouteKind, duplicates: &[DuplicateRoute]) -> String {
    let conflicts = if duplicates.len() == 1 {
        "conflict"
    } else {
        "conflicts"
    };

    let mut message = format!(
        "Duplicate {} found: {} {}",
        kind.plural(),
        duplicates.len(),
        conflicts
    );

    for duplicate in duplicates {
        message.push_str(&format!("\n  {} (same URL shape):", duplicate.shape));

        for (route_path, file) in &duplicate.entries {
            if route_path == &duplicate.shape {
                message.push_str(&format!("\n    {file}"));
            } else {
                message.push_str(&format!("\n    {route_path} <- {file}"));
            }
        }
    }

    message.push_str(
        "\nKeep only one file per route shape. Paths that differ only in parameter names match the same URLs.",
    );

    message
}

/// Prefer a path relative to the project root for friendlier error messages.
fn display_path(root_path: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(root_path)
        .unwrap_or(file_path)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(route: &str, file: &str) -> RouteFile {
        RouteFile {
            route_path: route.to_string(),
            file_path: PathBuf::from(file),
        }
    }

    #[test]
    fn shape_ignores_parameter_names() {
        assert_eq!(route_shape("/api/:foo"), "/api/:");
        assert_eq!(route_shape("/api/:bar"), "/api/:");
        assert_eq!(route_shape("/api/:org/members/:id"), "/api/:/members/:");
        assert_eq!(route_shape("/blog/:article"), route_shape("/blog/:slug"));
    }

    #[test]
    fn shape_keeps_literal_segments() {
        assert_eq!(route_shape("/api/users/:id"), "/api/users/:");
        assert_ne!(route_shape("/api/users/:id"), route_shape("/api/posts/:id"));
        assert_ne!(route_shape("/api/:id"), route_shape("/api/*rest"));
    }

    #[test]
    fn error_reports_parameter_name_collision() {
        let candidates = vec![
            candidate("/api/:foo", "/project/src/api/$foo.js"),
            candidate("/api/:bar", "/project/src/api/$bar.js"),
        ];

        let error = resolve_duplicates(
            candidates,
            DuplicateRoutesPolicy::Error,
            RouteKind::Api,
            Path::new("/project"),
        )
        .expect_err("same shape must conflict");

        let message = error.to_string();
        assert!(
            message.contains("Duplicate API routes found: 1 conflict"),
            "{message}"
        );
        assert!(message.contains("/api/"), "{message}");
        assert!(message.contains("$foo.js"), "{message}");
        assert!(message.contains("$bar.js"), "{message}");
    }

    #[test]
    fn error_reports_static_duplicates() {
        let candidates = vec![
            candidate("/api/hello", "/project/src/api/hello.js"),
            candidate("/api/hello", "/project/src/api/hello.py"),
        ];

        let error = resolve_duplicates(
            candidates,
            DuplicateRoutesPolicy::Error,
            RouteKind::Api,
            Path::new("/project"),
        )
        .expect_err("same path must conflict");

        let message = error.to_string();
        assert!(message.contains("/api/hello"), "{message}");
        assert!(message.contains("hello.js"), "{message}");
        assert!(message.contains("hello.py"), "{message}");
    }

    #[test]
    fn warn_keeps_first_sorted_candidate() {
        let candidates = vec![
            candidate("/api/:foo", "/project/src/api/$foo.js"),
            candidate("/api/:bar", "/project/src/api/$bar.js"),
            candidate("/api/users", "/project/src/api/users.js"),
        ];

        let resolved = resolve_duplicates(
            candidates,
            DuplicateRoutesPolicy::Warn,
            RouteKind::Api,
            Path::new("/project"),
        )
        .expect("warn policy must not fail");

        let routes: Vec<&str> = resolved.iter().map(|r| r.route_path.as_str()).collect();
        assert_eq!(routes, vec!["/api/:bar", "/api/users"]);
    }

    #[test]
    fn distinct_shapes_are_allowed() {
        let candidates = vec![
            candidate("/api/users/:id", "/project/src/api/users/$id.js"),
            candidate("/api/posts/:id", "/project/src/api/posts/$id.js"),
            candidate("/api/users/list", "/project/src/api/users/list.js"),
        ];

        let resolved = resolve_duplicates(
            candidates,
            DuplicateRoutesPolicy::Error,
            RouteKind::Api,
            Path::new("/project"),
        )
        .expect("distinct shapes must not conflict");

        assert_eq!(resolved.len(), 3);
    }

    #[test]
    fn noop_when_unique() {
        let candidates = vec![candidate("/api/hello", "/project/src/api/hello.js")];

        let resolved = resolve_duplicates(
            candidates,
            DuplicateRoutesPolicy::Error,
            RouteKind::Api,
            Path::new("/project"),
        )
        .expect("unique routes must pass");

        assert_eq!(resolved.len(), 1);
    }
}
