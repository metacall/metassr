use anyhow::Result;
use std::{
    collections::HashMap,
    ffi::OsStr,
    marker::Sized,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
pub trait AnalyzeDir {
    type Output;
    fn analyze(&self) -> Result<Self::Output>;
}

#[derive(Debug, Clone)]
pub struct SourceDirContainer {
    pub pages: HashMap<String, PathBuf>,
    pub specials: HashMap<String, Option<PathBuf>>,
}

#[derive(Debug)]
pub struct SourceDir<'a>(&'a Path);

impl<'a> SourceDir<'a> {
    pub fn new<S>(path: &'a S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self(Path::new(path))
    }
}

impl<'a> AnalyzeDir for SourceDir<'a> {
    type Output = SourceDirContainer;
    fn analyze(&self) -> Result<Self::Output> {
        let src = self.0.to_str().unwrap();

        let mut pages: HashMap<String, PathBuf> = HashMap::new();
        let mut specials: HashMap<String, Option<PathBuf>> =
            HashMap::from([("_app".to_owned(), None), ("_head".to_owned(), None)]);

        for entry in WalkDir::new(src).into_iter().filter_map(|e| {
            // Check if the entry is a js/ts file.
            let exts: Vec<&str> = vec!["js", "jsx", "tsx", "ts"];
            e.ok().and_then(|e| {
                match e.path().is_file()
                    && exts.contains(&e.path().extension().unwrap().to_str().unwrap())
                {
                    true => Some(e),
                    false => None,
                }
            })
        }) {
            let path = entry.path();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let stripped = path.strip_prefix(src)?;

            match stripped.iter().next() {
                Some(_) if specials.contains_key(stem) => {
                    specials.insert(stem.to_owned(), Some(path.to_path_buf()));
                }

                Some(p) if p == OsStr::new("pages") => {
                    let route = path
                        .strip_prefix([src, "/pages"].concat())?
                        .to_str()
                        .unwrap();
                    pages.insert(route.to_owned(), path.to_path_buf());
                }

                _ => (),
            }
        }

        Ok(Self::Output { pages, specials })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        dbg!(&SourceDir::new("../../tests/web-app/src").analyze().unwrap());
    }
}
