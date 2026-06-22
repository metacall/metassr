use crate::{
    shared::{APP_PATH_TAG, FUNC_ID_TAG, PAGE_PATH_TAG},
    traits::Generate,
};
use anyhow::Result;
use dunce;
use metassr_utils::{js_path::to_js_path, rand::Rand};
use std::{ffi::OsStr, path::PathBuf};

const RENDER_FILE_TEMPLATE: &str = include_str!("../scripts/render.js.template");

pub struct ServerRender {
    app_path: PathBuf,
    page_path: PathBuf,
}

impl ServerRender {
    pub fn new<'a, S>(app_path: &'a S, page_path: &'a S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            app_path: PathBuf::from(app_path),
            page_path: PathBuf::from(page_path),
        }
    }
}

impl Generate for ServerRender {
    type Output = (i64, String);
    fn generate(&self) -> Result<Self::Output> {
        let func_id = Rand::new().val();
        let mut app_path = dunce::canonicalize(&self.app_path)?;
        let mut page_path = dunce::canonicalize(&self.page_path)?;

        app_path.set_extension("");
        page_path.set_extension("");

        Ok((
            func_id,
            RENDER_FILE_TEMPLATE
                .replace(APP_PATH_TAG, &to_js_path(&app_path))
                .replace(PAGE_PATH_TAG, &to_js_path(&page_path))
                .replace(FUNC_ID_TAG, &func_id.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn generate_render_file() {
        let temp_dir = TempDir::new().unwrap();
        let app_path = temp_dir.path().join("_app.tsx");
        let pages_dir = temp_dir.path().join("pages");
        fs::create_dir_all(&pages_dir).unwrap();
        let page_path = pages_dir.join("home.jsx");

        fs::write(&app_path, "// app").unwrap();
        fs::write(&page_path, "// page").unwrap();

        let result =
            ServerRender::new(app_path.to_str().unwrap(), page_path.to_str().unwrap()).generate();

        assert!(result.is_ok());
        let (func_id, content) = result.unwrap();
        assert!(func_id != 0);
        assert!(!content.is_empty());
        println!("Generated: {:?}", (func_id, content));
    }
}
