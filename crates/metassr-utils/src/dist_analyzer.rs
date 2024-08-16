use crate::traits::AnalyzeDir;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    marker::Sized,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
/// A container contains analyzing result for `dist/` directory.
#[derive(Debug)]
pub struct DistDirContainer {
    pub pages: HashMap<String, PageEntry>,
}

/// The page entry, where each pages details stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEntry {
    pub scripts: Vec<PathBuf>,
    pub styles: Vec<PathBuf>,
    pub path: PathBuf,
}

impl PageEntry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            scripts: vec![],
            styles: vec![],
            path,
        }
    }
    pub fn push_script(&mut self, path: &Path) {
        self.scripts.push(path.to_path_buf());
    }
    pub fn push_style(&mut self, path: &Path) {
        self.styles.push(path.to_path_buf());
    }
}

/// A simple analyzer for `dist/` directory to extract script files and style files, a bundled files generated using `rspack`.
#[derive(Debug)]
pub struct DistDir(PathBuf);

impl DistDir {
    pub fn new<S>(path: &S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(anyhow!("Dist directory not found: {path:#?}"));
        }

        Ok(Self(path))
    }
}

impl AnalyzeDir for DistDir {
    type Output = DistDirContainer;
    fn analyze(&self) -> Result<Self::Output> {
        let pages_path = self.0.join("pages");
        let mut pages: HashMap<String, PageEntry> = HashMap::new();

        for entry in WalkDir::new(pages_path.clone())
            .into_iter()
            .filter_map(|e| {
                let exts = ["js", "css"];
                match e.ok() {
                    Some(e)
                        if e.path().is_file()
                            && exts.contains(&e.path().extension().unwrap().to_str().unwrap()) =>
                    {
                        Some(e)
                    }
                    _ => None,
                }
            })
        {
            let path = entry.path();
            let parent = path.parent().unwrap();

            let parent_stripped = match parent.strip_prefix(pages_path.clone()).unwrap() {
                p if p == Path::new("") => "#root",
                p => p.to_str().unwrap(),
            };
            let ext = path.extension().unwrap().to_str().unwrap();
            // let stem = path.file_stem().unwrap().to_str().unwrap();
            // let stripped = path.strip_prefix(src)?;
            if !pages.contains_key(parent_stripped) {
                pages.insert(
                    parent_stripped.to_owned(),
                    PageEntry::new(parent.to_path_buf().canonicalize().unwrap()),
                );
            };

            let page = pages.get_mut(parent_stripped).unwrap();
            match ext {
                "js" => (*page).push_script(path),
                "css" => {
                    (*page).push_style(path);
                }
                _ => (),
            }
        }

        Ok(Self::Output { pages })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        dbg!(&DistDir::new("../../tests/web-app/dist")
            .unwrap()
            .analyze()
            .unwrap());
    }
}
