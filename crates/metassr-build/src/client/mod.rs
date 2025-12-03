use crate::traits::{Build, Generate};
use crate::utils::setup_page_path;
use anyhow::{anyhow, Result};
use hydrator::Hydrator;

use metassr_bundler::WebBundler;
use metassr_fs_analyzer::{
    src_dir::{special_entries, SourceDir},
    DirectoryAnalyzer,
};
use metassr_utils::cache_dir::CacheDir;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub mod hydrator;

pub struct ClientBuilder {
    src_path: PathBuf,
    dist_path: PathBuf,
}

impl ClientBuilder {
    pub fn new<S>(root: &S, dist_dir: &str) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let root = Path::new(root);
        let src_path = root.join("src");
        let dist_path = root.join(dist_dir);

        if !src_path.exists() {
            return Err(anyhow!("src directory not found."));
        }
        if !dist_path.exists() {
            fs::create_dir(&dist_path)?;
        }
        Ok(Self {
            src_path,
            dist_path,
        })
    }
}

impl Build for ClientBuilder {
    type Output = ();
    fn build(&self) -> Result<Self::Output> {
        let mut cache_dir = CacheDir::new(&format!("{}/cache", self.dist_path.display()))?;
        let src = SourceDir::new(&self.src_path).analyze()?;

        let pages = src.pages();
        let (special_entries::App(app_path), _) = src.specials()?;

        for (page, page_path) in pages.iter() {
            let hydrator = Hydrator::new(&app_path, page_path, "root").generate()?;
            let page = setup_page_path(page, "js");

            cache_dir.insert(&format!("pages/{}", page.display()), hydrator.as_bytes())?;
        }

        let targets = cache_dir
            .entries_in_scope()
            .iter()
            .map(|(entry_name, path)| {
                let fullpath = path.canonicalize().unwrap();

                (entry_name.to_owned(), format!("{}", fullpath.display()))
            })
            .collect::<HashMap<String, String>>();

        let bundler = WebBundler::new(&targets, &self.dist_path)?;
        if let Err(e) = bundler.exec() {
            return Err(anyhow!("Bundling failed: {e}"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn setup_test_project() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_dir = temp_dir.path().join("src");
        let pages_dir = src_dir.join("pages");
        let dist_dir = temp_dir.path().join("dist");

        fs::create_dir_all(&pages_dir).unwrap();
        fs::create_dir_all(&dist_dir).unwrap();

        // Create package.json
        let mut package_json = fs::File::create(temp_dir.path().join("package.json")).unwrap();
        writeln!(package_json, "{{").unwrap();
        writeln!(package_json, "  \"name\": \"test-app\",").unwrap();
        writeln!(package_json, "  \"version\": \"1.0.0\",").unwrap();
        writeln!(package_json, "  \"type\": \"module\"").unwrap();
        writeln!(package_json, "}}").unwrap();

        // Create _app.tsx
        let mut app_file = fs::File::create(src_dir.join("_app.tsx")).unwrap();
        writeln!(app_file, "import React from 'react';").unwrap();
        writeln!(app_file, "export default function App({{ children }}) {{").unwrap();
        writeln!(app_file, "  return <div>{{children}}</div>;").unwrap();
        writeln!(app_file, "}}").unwrap();

        // Create _head.tsx
        let mut head_file = fs::File::create(src_dir.join("_head.tsx")).unwrap();
        writeln!(head_file, "import React from 'react';").unwrap();
        writeln!(head_file, "export default function Head() {{").unwrap();
        writeln!(head_file, "  return (").unwrap();
        writeln!(head_file, "    <>").unwrap();
        writeln!(head_file, "      <title>Test App</title>").unwrap();
        writeln!(
            head_file,
            "      <meta name=\"description\" content=\"Test application\" />"
        )
        .unwrap();
        writeln!(head_file, "    </>").unwrap();
        writeln!(head_file, "  );").unwrap();
        writeln!(head_file, "}}").unwrap();

        // Create index page
        let mut index_file = fs::File::create(pages_dir.join("index.jsx")).unwrap();
        writeln!(index_file, "import React from 'react';").unwrap();
        writeln!(index_file, "export default function Index() {{").unwrap();
        writeln!(index_file, "  return <div>Index Page</div>;").unwrap();
        writeln!(index_file, "}}").unwrap();
        temp_dir
    }

    #[test]
    fn client_builder() {
        let temp_dir = setup_test_project();
        let project_path = temp_dir.path();
        let dist_path = project_path.join("dist");

        let result =
            ClientBuilder::new(project_path.to_str().unwrap(), dist_path.to_str().unwrap());

        match result {
            Ok(builder) => match builder.build() {
                Ok(_) => {
                    println!("✓ Client built successfully");
                }
                Err(e) => {
                    panic!("Build failed with unexpected error: {:?}", e);
                }
            },
            Err(e) => {
                panic!("ClientBuilder creation failed: {:?}", e);
            }
        }
    }
}
