use std::path::Path;

use anyhow::Result;
use metassr_fs_analyzer::{src_dir::SourceDir, DirectoryAnalyzer};
use metassr_utils::routes::{resolve_duplicates, DuplicateRoutesPolicy, RouteFile, RouteKind};

use crate::utils::setup_page_path;

/// Map a `src/pages` key (`index.tsx`, `blog/$article.tsx`) to the axum route
/// it produces (`/`, `/blog/:article`).
///
/// Reuses [`setup_page_path`] so the mapping matches the build output: `index`
/// pages become the parent route, other pages are wrapped in a subdirectory.
fn source_page_route(key: &str) -> String {
    let output = setup_page_path(key, "js");
    let route = match output.parent() {
        Some(parent) if parent != Path::new("") => parent.to_string_lossy().replace('\\', "/"),
        _ => "#root".to_string(),
    };

    match route.as_str() {
        "#root" => "/".to_string(),
        route => format!("/{}", route.replace('$', ":")),
    }
}

/// Validate the page routes of the project rooted at `root_path`.
///
/// Two pages can claim the same URL shape even when their files differ, e.g.
/// `blog/$article.tsx` and `blog/$slug.tsx` both map to `/blog/:param`, or
/// `blog/$article.tsx` and `blog/$article/index.tsx` both map to the same
/// route. With [`DuplicateRoutesPolicy::Error`] this returns an error listing
/// every conflict; with [`DuplicateRoutesPolicy::Warn`] it only warns.
pub fn validate_page_routes(root_path: &Path, policy: DuplicateRoutesPolicy) -> Result<()> {
    let src = SourceDir::new(&root_path.join("src")).analyze()?;

    let candidates = src
        .pages
        .into_iter()
        .map(|(key, path)| RouteFile {
            route_path: source_page_route(&key),
            file_path: path,
        })
        .collect();

    resolve_duplicates(candidates, policy, RouteKind::Page, root_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup(files: &[&str]) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(src.join("pages")).unwrap();
        fs::write(src.join("_app.tsx"), "").unwrap();
        fs::write(src.join("_head.tsx"), "").unwrap();

        for file in files {
            let path = src.join("pages").join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }

        tmp
    }

    #[test]
    fn maps_source_keys_to_routes() {
        assert_eq!(source_page_route("index.tsx"), "/");
        assert_eq!(source_page_route("about.tsx"), "/about");
        assert_eq!(source_page_route("blog/index.tsx"), "/blog");
        assert_eq!(source_page_route("blog/$article.tsx"), "/blog/:article");
    }

    #[test]
    fn parameter_name_collision_fails_with_error_policy() {
        let tmp = setup(&["blog/$article.tsx", "blog/$slug.tsx"]);

        let error = validate_page_routes(tmp.path(), DuplicateRoutesPolicy::Error)
            .expect_err("colliding page parameters must fail the build");
        let message = error.to_string();

        assert!(message.contains("Duplicate page routes"), "{message}");
        assert!(message.contains("$article.tsx"), "{message}");
        assert!(message.contains("$slug.tsx"), "{message}");
    }

    #[test]
    fn index_and_non_index_collision_is_detected() {
        let tmp = setup(&["blog/$article.tsx", "blog/$article/index.tsx"]);

        assert!(validate_page_routes(tmp.path(), DuplicateRoutesPolicy::Error).is_err());
    }

    #[test]
    fn distinct_page_routes_are_allowed() {
        let tmp = setup(&["blog/$slug.tsx", "users/$id.tsx", "about.tsx"]);

        assert!(validate_page_routes(tmp.path(), DuplicateRoutesPolicy::Error).is_ok());
    }

    #[test]
    fn warn_policy_allows_collisions() {
        let tmp = setup(&["blog/$article.tsx", "blog/$slug.tsx"]);

        assert!(validate_page_routes(tmp.path(), DuplicateRoutesPolicy::Warn).is_ok());
    }
}
