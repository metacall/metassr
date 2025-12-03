use crate::{
    shared::{APP_PATH_TAG, FUNC_ID_TAG, PAGE_PATH_TAG},
    traits::Generate,
};
use anyhow::Result;
use metassr_utils::rand::Rand;
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
        let mut app_path = self.app_path.canonicalize()?;
        let mut page_path = self.page_path.canonicalize()?;

        app_path.set_extension("");
        page_path.set_extension("");

        Ok((
            func_id,
            RENDER_FILE_TEMPLATE
                .replace(APP_PATH_TAG, app_path.to_str().unwrap())
                .replace(PAGE_PATH_TAG, page_path.to_str().unwrap())
                .replace(FUNC_ID_TAG, &func_id.to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_test_files() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        let pages_dir = src_dir.join("pages");

        fs::create_dir_all(&pages_dir).unwrap();

        // Create minimal _app.tsx
        let mut app_file = fs::File::create(src_dir.join("_app.tsx")).unwrap();
        writeln!(app_file, "import React from 'react';").unwrap();
        writeln!(app_file, "export default function App({{ children }}) {{").unwrap();
        writeln!(
            app_file,
            "  return <div className=\"app\">{{children}}</div>;"
        )
        .unwrap();
        writeln!(app_file, "}}").unwrap();

        // Create minimal home.jsx
        let mut home_file = fs::File::create(pages_dir.join("home.jsx")).unwrap();
        writeln!(home_file, "import React from 'react';").unwrap();
        writeln!(home_file, "export default function Home() {{").unwrap();
        writeln!(
            home_file,
            "  return <div className=\"home\">Home Page</div>;"
        )
        .unwrap();
        writeln!(home_file, "}}").unwrap();

        temp_dir
    }

    #[test]
    fn generate_render_file() {
        let temp_dir = setup_test_files();
        let base = temp_dir.path();

        let app_path = base.join("src/_app.tsx");
        let page_path = base.join("src/pages/home.jsx");

        let result =
            ServerRender::new(app_path.to_str().unwrap(), page_path.to_str().unwrap()).generate();

        match result {
            Ok(output) => {
                assert!(!output.1.is_empty(), "Generated output should not be empty");
                println!("✓ Generated render code successfully");
                println!("{:?}", output);
            }
            Err(e) => {
                panic!("Failed to generate render file: {:?}", e);
            }
        }
    }
}
