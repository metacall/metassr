use crate::{
    shared::{APP_PATH_TAG, PAGE_PATH_TAG, ROOT_ID_TAG},
    traits::Generate,
};
use anyhow::Result;
use std::{ffi::OsStr, path::PathBuf};

const HYDRATED_FILE_TEMPLATE: &str = include_str!("../scripts/hydrate.js.template");

#[derive(Debug, Clone)]
pub struct Hydrator {
    app_path: PathBuf,
    page_path: PathBuf,
    root_id: String,
}

impl Hydrator {
    pub fn new<'a, S>(app_path: &'a S, page_path: &'a S, root_id: &'a str) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            app_path: PathBuf::from(app_path),
            page_path: PathBuf::from(page_path),
            root_id: root_id.to_string(),
        }
    }
}

impl Generate for Hydrator {
    type Output = String;
    fn generate(&self) -> Result<Self::Output> {
        Ok(HYDRATED_FILE_TEMPLATE
            .replace(
                APP_PATH_TAG,
                self.app_path.canonicalize()?.to_str().unwrap(),
            )
            .replace(
                PAGE_PATH_TAG,
                self.page_path.canonicalize()?.to_str().unwrap(),
            )
            .replace(ROOT_ID_TAG, &self.root_id))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn setup_test_files() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        let pages_dir = src_dir.join("pages");

        fs::create_dir_all(&pages_dir).unwrap();

        // Create minimal test files
        fs::write(
            src_dir.join("_app.tsx"),
            "export default function App() { return null; }",
        )
        .unwrap();

        fs::write(
            pages_dir.join("home.jsx"),
            "export default function Home() { return <div>Home</div>; }",
        )
        .unwrap();

        temp_dir
    }

    #[test]
    fn generate_hydrated_file() {
        let temp_dir = setup_test_files();
        let base_path = temp_dir.path();

        let result = Hydrator::new(
            &base_path.join("src/_app.tsx").to_str().unwrap(),
            &base_path.join("src/pages/home.jsx").to_str().unwrap(),
            "root",
        )
        .generate();

        assert!(result.is_ok());
    }
}
