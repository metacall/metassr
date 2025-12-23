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
    use tempfile::TempDir;

    #[test]
    fn generate_hydrated_file() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("_app.tsx");
        let pages_dir = temp_dir.path().join("pages");
        fs::create_dir_all(&pages_dir).unwrap();
        let page_path = pages_dir.join("home.jsx");

        fs::write(&app_path, "// app").unwrap();
        fs::write(&page_path, "// page").unwrap();

        let result = Hydrator::new(
            app_path.to_str().unwrap(),
            page_path.to_str().unwrap(),
            "root",
        )
        .generate();

        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(!content.is_empty());
        println!("Generated: {:?}", content);
    }
}
