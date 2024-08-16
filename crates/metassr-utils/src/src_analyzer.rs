use crate::traits::AnalyzeDir;
use anyhow::{anyhow, Result};
use std::{collections::HashMap, ffi::OsStr, marker::Sized, path::PathBuf};
use walkdir::WalkDir;

pub mod special_entries {
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    pub struct Head(pub PathBuf);
    #[derive(Debug, Clone)]
    pub struct App(pub PathBuf);
}

pub type PagesEntriesType = HashMap<String, PathBuf>;
pub type SpecialEntriesType = (Option<special_entries::App>, Option<special_entries::Head>);
#[derive(Debug, Clone)]

pub struct SourceDirContainer {
    pub pages: PagesEntriesType,
    pub specials: SpecialEntriesType,
}

impl SourceDirContainer {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            specials: (None, None),
        }
    }

    pub fn specials(&self) -> Result<(special_entries::App, special_entries::Head)> {
        let (app, head) = self.specials.clone();
        if let (Some(app), Some(head)) = (app.clone(), head.clone()) {
            return Ok((app, head));
        }
        let mut not_found = vec![];
        if app.is_none() {
            not_found.push("_app.[js,jsx,ts,tsx]")
        }
        if head.is_none() {
            not_found.push("_head.[js,jsx,ts,tsx]")
        }
        Err(anyhow!(
            "Couldn't found: {}. Create the files that have not been found.",
            not_found.join(", ")
        ))
    }

    pub fn pages(&self) -> PagesEntriesType {
        self.pages.clone()
    }
}

#[derive(Debug)]
pub struct SourceDir(PathBuf);

impl SourceDir {
    pub fn new<S>(path: &S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self(PathBuf::from(path))
    }
}

impl AnalyzeDir for SourceDir {
    type Output = SourceDirContainer;
    fn analyze(&self) -> Result<Self::Output> {
        let src = self.0.to_str().unwrap();

        let list_of_specials = ["_app", "_head"];
        let mut pages: HashMap<String, PathBuf> = HashMap::new();
        let mut specials: SpecialEntriesType = (None, None);

        for entry in WalkDir::new(src)
            .into_iter()
            .filter_map(|e| match e.ok() {
                Some(e) if e.path().is_file() => Some(e),
                _ => None,
            })
            .skip_while(|e| {
                // Check if the entry is a js/ts file.
                let exts: Vec<&str> = vec!["js", "jsx", "tsx", "ts"];
                !exts.contains(&e.path().extension().unwrap().to_str().unwrap())
            })
        {
            let path = entry.path();
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let stripped = path.strip_prefix(src)?;

            match stripped.iter().next() {
                Some(_) if list_of_specials.contains(&stem) => {
                    dbg!(&stem);
                    match stem {
                        "_app" => specials.0 = Some(special_entries::App(path.to_path_buf())),
                        "_head" => specials.1 = Some(special_entries::Head(path.to_path_buf())),
                        _ => (),
                    }
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
