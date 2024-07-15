use anyhow::{anyhow, Result};
use bundler::ClientBundler;
use hydrator::Hydrator;
use metassr_utils::{
    cache_dir::CacheDir,
    src_analyzer::{AnalyzeDir, SourceDir},
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

pub mod bundler;
pub mod hydrator;

pub struct ClientBuilder {
    src_path: PathBuf,
    dist_path: PathBuf,
}

impl ClientBuilder {
    pub fn new<'a, S>(root: &'a S, dist_dir: &str) -> Result<Self>
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
            fs::create_dir(dist_path.clone())?;
        }
        Ok(Self {
            src_path,
            dist_path,
        })
    }

    pub fn build(&self) -> Result<()> {
        let mut cache_dir = CacheDir::new(&format!("{}/.cache-metassr", self.dist_path.display()))?;

        let src = SourceDir::new(&self.src_path).analyze()?;
        let pages = src.clone().pages;
        let app_path = src
            .specials
            .get("_app")
            .expect("Error: Cannot detect '_app' in src directory")
            .as_ref()
            .unwrap();

        for (page, page_path) in pages.iter() {
            let hydrator = Hydrator::new(&app_path, &page_path, "root").generate()?;

            // Page details
            let page = match Path::new(page) {
                path if path.file_stem() != Some(OsStr::new("index")) => {
                    let mut path = path.to_path_buf();
                    path.set_extension("");
                    path.join("index.js")
                }

                path => path.to_path_buf(),
            };

            cache_dir.insert(&format!("pages/{}", page.display()), hydrator.as_bytes())?;
        }

        let targets = cache_dir
            .entries_in_scope()
            .iter()
            .map(|(id, path)| {
                let fullpath = path.canonicalize().unwrap();

                (id.to_owned(), format!("{}", fullpath.display()))
            })
            .collect::<HashMap<String, String>>();

        let bundler = ClientBundler::new(&targets, &self.dist_path);

        println!("{targets:#?}");

        if let Err(e) = bundler.bundling() {
            return Err(anyhow!("Bundling failed: {e}"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_builder() {
        ClientBuilder::new("../../tests/web-app", "../../tests/web-app/dist")
            .unwrap()
            .build()
            .unwrap();
    }
}
