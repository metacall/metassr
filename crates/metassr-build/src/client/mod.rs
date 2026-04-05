use crate::traits::{Build, Generate};
use crate::utils::setup_page_path;
use anyhow::{anyhow, Result};
use dunce;
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
                let fullpath = dunce::canonicalize(path).unwrap();

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

    #[test]
    #[ignore = "requires full bundling infrastructure and test fixtures"]
    fn client_builder() {
        // This test requires:
        // 1. A valid project structure with package.json
        // 2. Node.js/bundler available
        // 3. Proper test fixtures
        //
        // Run with: cargo test client_builder -- --ignored
        todo!("Set up proper test fixtures for client builder")
    }
}
