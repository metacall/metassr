use super::DirectoryAnalyzer;
use anyhow::{anyhow, Result};
use std::{collections::HashMap, ffi::OsStr, marker::Sized, path::PathBuf};
use walkdir::WalkDir;

/// Wrappers for special entries that collected by the source analyzer
pub mod special_entries {
    use std::path::PathBuf;

    /// Represents a special entry for the `_app.[js, jsx, ts, tsx]` file.
    #[derive(Debug, Clone)]
    pub struct App(pub PathBuf);

    /// Represents a special entry for the `_head.[js, jsx, ts, tsx]` file.
    #[derive(Debug, Clone)]
    pub struct Head(pub PathBuf);
}

pub type PagesEntriesType = HashMap<String, PathBuf>;
pub type SpecialEntriesType = (Option<special_entries::App>, Option<special_entries::Head>);

/// A container holding the results of analyzing a source directory.
///
/// This struct holds the pages and special entries found in the source directory.
#[derive(Debug, Clone)]
pub struct SourceDirContainer {
    pub pages: PagesEntriesType,
    pub specials: SpecialEntriesType,
}

impl SourceDirContainer {
    /// Creates a new `SourceDirContainer` with the given pages and special entries.
    ///
    /// # Parameters
    ///
    /// - `pages`: A `HashMap` where keys are routes and values are paths to page files.
    /// - `specials`: A tuple containing optional special entries (`App` and `Head`).
    pub fn new(pages: PagesEntriesType, specials: SpecialEntriesType) -> Self {
        Self { pages, specials }
    }

    /// Retrieves the special entries from the container.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a tuple of `App` and `Head` if both are present,
    /// or an error if one or both are missing.
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
            "Couldn't find: {}. Create the files that have not been found.",
            not_found.join(", ")
        ))
    }

    /// Retrieves the pages entries from the container.
    ///
    /// # Returns
    ///
    /// Returns a `HashMap` where keys are routes and values are paths to page files.
    pub fn pages(&self) -> PagesEntriesType {
        self.pages.clone()
    }
}

/// A directory analyzer for a source directory.
///
/// This struct provides functionality to analyze a directory and extract pages and special entries.
#[derive(Debug)]
pub struct SourceDir(PathBuf);

impl SourceDir {
    /// Creates a new `SourceDir` instance.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the source directory.
    pub fn new<S>(path: &S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self(PathBuf::from(path))
    }
}

impl DirectoryAnalyzer for SourceDir {
    type Output = SourceDirContainer;

    /// Analyzes the source directory and extracts pages and special entries.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a `SourceDirContainer` with pages and special entries.
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
                Some(_) if list_of_specials.contains(&stem) => match stem {
                    "_app" => specials.0 = Some(special_entries::App(path.to_path_buf())),
                    "_head" => specials.1 = Some(special_entries::Head(path.to_path_buf())),
                    _ => (),
                },

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

        let container = SourceDirContainer::new(pages, specials);

        // Return an error if specials not found.
        if let Err(err) = container.specials() {
            return Err(anyhow!(err));
        }

        Ok(container)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rand::Rand;
    use std::fs;
    use std::path::PathBuf;

    /// Helper function to create a temporary source directory with a random name.
    fn create_temp_source_dir() -> Result<SourceDir> {
        let dir_path = PathBuf::from(format!("src-{}", Rand::new().val()));
        fs::create_dir_all(&dir_path)?;
        Ok(SourceDir::new(&dir_path))
    }

    /// Test case to verify the creation and analysis of a source directory.
    #[test]
    fn test_create_and_analyze_source_dir() {
        let source_dir = create_temp_source_dir().unwrap();
        let pages = vec!["page1.jsx", "page2.tsx"];
        let specials = vec!["_app.jsx", "_head.tsx"];

        for page in pages.iter() {
            let path = source_dir.0.join("pages").join(page);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, b"dummy content").unwrap();
        }

        for special in specials.iter() {
            let path = source_dir.0.join(special);
            fs::write(&path, b"dummy content").unwrap();
        }

        let result = source_dir.analyze().unwrap();
        assert_eq!(result.pages().len(), pages.len());
        assert!(result.specials().is_ok());

        // Cleanup
        for page in pages.iter() {
            let path = source_dir.0.join("pages").join(page);
            fs::remove_file(&path).unwrap();
        }
        for special in specials.iter() {
            let path = source_dir.0.join(special);
            fs::remove_file(&path).unwrap();
        }
        fs::remove_dir_all(source_dir.0).unwrap();
    }

    /// Test case to verify handling of missing special entries.
    #[test]
    fn test_missing_special_entries() {
        let source_dir = create_temp_source_dir().unwrap();
        let page_path = source_dir.0.join("pages/page1.jsx");
        fs::create_dir_all(page_path.parent().unwrap()).unwrap();
        fs::write(&page_path, b"dummy content").unwrap();

        let result = source_dir.analyze();
        assert!(
            result.is_err(),
            "Should return an error due to missing special entries"
        );

        // Cleanup
        fs::remove_file(&page_path).unwrap();
        fs::remove_dir_all(source_dir.0).unwrap();
    }
}
