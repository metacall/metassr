use std::path::Path;

use anyhow::Result;
use metassr_api_handler::{
    scanner::{resolve_route_duplicates, scan_api_dir},
    DuplicateRoutesPolicy,
};

/// Validate the API routes of the project rooted at `root_path`.
///
/// Two files in `src/api` can map to the same route path (e.g. `hello.js` and
/// `hello.py` both map to `/api/hello`). With [`DuplicateRoutesPolicy::Error`]
/// this returns an error listing every conflicting route path, with
/// [`DuplicateRoutesPolicy::Warn`] it only warns and keeps going.
pub fn validate_api_routes(root_path: &Path, policy: DuplicateRoutesPolicy) -> Result<()> {
    let api_dir = root_path.join("src").join("api");

    if !api_dir.exists() {
        return Ok(());
    }

    resolve_route_duplicates(scan_api_dir(&api_dir), policy, root_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn no_api_directory_is_valid() {
        let temp_dir = TempDir::new().unwrap();

        assert!(validate_api_routes(temp_dir.path(), DuplicateRoutesPolicy::Error).is_ok());
    }

    #[test]
    fn unique_routes_are_valid() {
        let temp_dir = TempDir::new().unwrap();
        let api_dir = temp_dir.path().join("src/api");
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(api_dir.join("hello.js"), "function GET() {}").unwrap();

        assert!(validate_api_routes(temp_dir.path(), DuplicateRoutesPolicy::Error).is_ok());
    }

    #[test]
    fn duplicated_routes_fail_with_the_error_policy() {
        let temp_dir = TempDir::new().unwrap();
        let api_dir = temp_dir.path().join("src/api");
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(api_dir.join("hello.js"), "function GET() {}").unwrap();
        fs::write(api_dir.join("hello.py"), "def GET(request): pass").unwrap();

        let error = validate_api_routes(temp_dir.path(), DuplicateRoutesPolicy::Error)
            .expect_err("duplicated routes must fail the build");
        let message = error.to_string();

        assert!(message.contains("/api/hello"), "{message}");
        assert!(message.contains("hello.js"), "{message}");
        assert!(message.contains("hello.py"), "{message}");
    }

    #[test]
    fn duplicated_routes_pass_with_the_warn_policy() {
        let temp_dir = TempDir::new().unwrap();
        let api_dir = temp_dir.path().join("src/api");
        fs::create_dir_all(&api_dir).unwrap();
        fs::write(api_dir.join("hello.js"), "function GET() {}").unwrap();
        fs::write(api_dir.join("hello.py"), "def GET(request): pass").unwrap();

        assert!(validate_api_routes(temp_dir.path(), DuplicateRoutesPolicy::Warn).is_ok());
    }
}
