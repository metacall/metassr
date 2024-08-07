use crate::bundler::{BundlingType, WebBundler};
use crate::traits::{Build, Exec, Generate};
use anyhow::{anyhow, Result};
use hydrator::Hydrator;
use metassr_utils::src_analyzer::special_entries;
use metassr_utils::{cache_dir::CacheDir, src_analyzer::SourceDir, traits::AnalyzeDir};
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
            fs::create_dir(dist_path.clone())?;
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

            // Page details
            let page = match Path::new(page) {
                path if path.file_stem() != Some(OsStr::new("index")) => {
                    let mut path = path.to_path_buf();
                    path.set_extension("");
                    let mut path = path.join("index.js");
                    path.set_extension("js");
                    path
                }

                path => {
                    let mut path = path.to_path_buf();
                    path.set_extension("js");
                    path
                }
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

        let bundler = WebBundler::new(&targets, &self.dist_path, BundlingType::Web);

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
    fn client_builder() {
        ClientBuilder::new("../../tests/web-app", "../../tests/web-app/dist")
            .unwrap()
            .build()
            .unwrap();
    }
}
