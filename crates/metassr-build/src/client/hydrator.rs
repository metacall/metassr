use std::{ffi::OsStr, path::PathBuf};

use anyhow::Result;

const HYDRATED_FILE_TEMPLATE: &str = include_str!("./scripts/hydrate.js.template");

const APP_PATH_TAG: &str = "%APP_PATH%";
const PAGE_PATH_TAG: &str = "%PAGE_PATH%";
const ROOT_ID_TAG: &str = "%ROOT_ID%";

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

    pub fn generate(&self) -> Result<String> {
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
    #[test]
    fn generate_hydrated_file() {
        println!(
            "{}",
            Hydrator::new("src/_app.tsx", "src/pages/home.jsx", "root")
                .generate()
                .unwrap()
        );
    }
}
