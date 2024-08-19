use anyhow::Result;
use std::{
    collections::HashMap, ffi::OsStr, fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}
};
use walkdir::WalkDir;

/// A simple cache directory.

#[derive(Debug, Clone)]
pub struct CacheDir {
    dir_path: PathBuf,
    entries_in_scope: HashMap<String, PathBuf>,
}

impl CacheDir {
    pub fn new<S: AsRef<OsStr> + ?Sized>(path: &S) -> Result<Self> {
        let dir_path = PathBuf::from(path);

        if !dir_path.exists() {
            fs::create_dir(dir_path.clone())?;
        }

        Ok(Self {
            dir_path,
            entries_in_scope: HashMap::new(),
        })
    }

    pub fn insert(&mut self, pathname: &str, buf: &[u8]) -> Result<PathBuf> {
        let id = pathname;
        let pathname = format!("{}/{}", self.dir_path.to_str().unwrap(), pathname);
        let path = Path::new(&pathname);

        // Create fille path if it isn't exist
        if !path.exists() {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;

            let mut file = File::create(path)?;
            file.write_all(buf)?;
        } else {
            let mut file = File::options().read(true).write(true).open(path)?;
            let mut current_buf = Vec::new();

            file.read_to_end(&mut current_buf)?;
            if current_buf != buf {
                let mut file = File::create(path)?;
                file.write_all(buf)?;
            }
        }

        // Replace the file if its content was changed

        // Adding the new filepath
        self.entries_in_scope
            .insert(id.to_string(), path.canonicalize()?.as_path().into());
        Ok(path.to_path_buf())
    }

    pub fn dir_path(&self) -> PathBuf {
        self.dir_path.clone()
    }

    pub fn entries_in_scope(&self) -> HashMap<String, PathBuf> {
        self.entries_in_scope.clone()
    }
    pub fn all_entries(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.dir_path)
            .into_iter()
            .filter_map(|e| {
                // Check if the entry is a js/ts file.
                e.ok().and_then(|e| match e.path().is_file() {
                    true => Some(e.into_path()),
                    false => None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::CacheDir;

    #[test]
    fn create_new_tmpfile() {
        let mut cache = CacheDir::new(".cache-metassr").unwrap();
        cache
            .insert("pages/home.jsx", "hello world".as_bytes())
            .map_err(|e| println!("{e}"))
            .unwrap();
        cache
            .insert("styles/index.jsx", "hello world".as_bytes())
            .map_err(|e| println!("{e}"))
            .unwrap();
        println!("{:?}", cache.entries_in_scope());
    }
}
