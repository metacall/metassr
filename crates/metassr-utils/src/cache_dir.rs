use anyhow::Result;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

/// `CacheDir` represents a simple directory-based cache system.
/// It stores file entries in a specified directory and tracks files
/// that are currently in the cache's scope.
///
/// This is useful for caching purposes where files are written and
/// retrieved based on a pathname, making it easier to manage multiple
/// cached files in a structured directory.
///
/// **Example**
///
/// ```no_run
/// use metassr_utils::cache_dir::CacheDir;
///
/// let mut cache = CacheDir::new(".cache").unwrap();
/// cache.insert("example.txt", "Cache data".as_bytes()).unwrap();
/// println!("{:?}", cache.entries_in_scope());
/// ```

#[derive(Debug, Clone)]
pub struct CacheDir {
    dir_path: PathBuf,
    entries_in_scope: HashMap<String, PathBuf>,
}

impl CacheDir {
    /// Creates a new `CacheDir` at the specified path.
    ///
    /// If the directory does not exist, it will be created.
    ///
    /// **Arguments**
    ///
    /// * `path` - A reference to the directory where the cache will be stored.
    ///
    /// **Errors**
    ///
    /// Returns an error if the directory cannot be created or accessed.
    ///
    ///  **Example**
    ///
    /// ```no_run
    /// use metassr_utils::cache_dir::CacheDir;
    ///
    /// let cache = CacheDir::new(".cache").unwrap();
    /// ```
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

    /// Inserts a file into the cache.
    ///
    /// This method writes a file to the cache directory if it doesn't already exist.
    /// If the file exists and the content differs, it will be replaced with the new content.
    ///
    /// **Arguments**
    ///
    /// * `pathname` - The relative path where the file should be stored.
    /// * `buf` - The content to be written to the file.
    ///
    /// **Returns**
    ///
    /// The `PathBuf` of the written file.
    ///
    /// **Example**
    ///
    /// ```no_run
    /// use metassr_utils::cache_dir::CacheDir;
    /// 
    /// let mut cache = CacheDir::new(".cache").unwrap();
    /// cache.insert("data.txt", "Some data".as_bytes()).unwrap();
    /// ```
    pub fn insert(&mut self, pathname: &str, buf: &[u8]) -> Result<PathBuf> {

        // Set the path
        let path = format!("{}/{}", self.dir_path.to_str().unwrap(), pathname);
        let path = Path::new(&path);

        // Create file path if it doesn't exist
        if !path.exists() {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;

            let mut file = File::create(path)?;
            file.write_all(buf)?;
        } else {
            let mut file = File::options().read(true).write(true).open(path)?;
            let mut current_buf = Vec::new();

            // Check if the file buffer is changed or not to rewrite it
            file.read_to_end(&mut current_buf)?;
            if current_buf != buf {
                let mut file = File::create(path)?;
                file.write_all(buf)?;
            }
        }

        // Add the new file path to the cache entries
        self.entries_in_scope
            .insert(pathname.to_string(), path.canonicalize()?);

        Ok(path.to_path_buf())
    }

    /// Returns the path to the cache directory.
    ///
    /// **Example**
    ///
    /// ```no_run
    /// use metassr_utils::cache_dir::CacheDir;
    /// 
    /// let cache = CacheDir::new(".cache").unwrap();
    /// println!("Cache directory: {:?}", cache.dir_path());
    /// ```
    pub fn dir_path(&self) -> PathBuf {
        self.dir_path.clone()
    }

    /// Returns the current entries in scope.
    ///
    /// **Example**
    ///
    /// ```no_run
    /// use metassr_utils::cache_dir::CacheDir;
    /// 
    /// let cache = CacheDir::new(".cache").unwrap();
    /// println!("Entries in scope: {:?}", cache.entries_in_scope());
    /// ```
    pub fn entries_in_scope(&self) -> HashMap<String, PathBuf> {
        self.entries_in_scope.clone()
    }

    /// Returns all file entries in the cache directory.
    ///
    /// **Example**
    ///
    /// ```no_run
    /// use metassr_utils::cache_dir::CacheDir;
    /// 
    /// let cache = CacheDir::new(".cache").unwrap();
    /// println!("All entries: {:?}", cache.all_entries());
    /// ```
    pub fn all_entries(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.dir_path)
            .into_iter()
            .filter_map(|e| {
                // Check if the entry is a file
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
    use super::*;
    use crate::rand::Rand;
    use std::fs;
    use std::path::PathBuf;

    /// Helper function to create a `CacheDir` with a random name
    fn create_temp_cache_dir() -> Result<CacheDir> {
        let dir_path = PathBuf::from(format!(".cache-{}", Rand::new().val()));
        CacheDir::new(&dir_path)
    }

    /// Test case to verify that a new cache directory can be created and deleted.
    #[test]
    fn test_create_and_delete_temp_cache_dir() {
        let mut cache = create_temp_cache_dir().unwrap();
        let test_file_path = cache.insert("test_file.txt", b"Hello, world!").unwrap();

        assert!(test_file_path.exists(), "Inserted file should exist");

        // Cleanup
        fs::remove_file(test_file_path).unwrap();
        fs::remove_dir_all(cache.dir_path()).unwrap();
    }

    /// Test case to verify that a new file can be inserted into the cache.
    #[test]
    fn test_insert_new_file() {
        let mut cache = create_temp_cache_dir().unwrap();
        let file_path = cache
            .insert("test_file.txt", b"Hello, world!")
            .expect("File should be inserted into the cache");
        assert!(file_path.exists(), "Inserted file should exist");

        // Cleanup
        // let file_path = result.unwrap();
        fs::remove_file(file_path).unwrap();
        fs::remove_dir_all(cache.dir_path()).unwrap();
    }

    /// Test case to check the scope of entries in the cache.
    #[test]
    fn test_entries_in_scope() {
        let mut cache = create_temp_cache_dir().unwrap();
        cache.insert("test_file1.txt", b"Data 1").unwrap();
        cache.insert("test_file2.txt", b"Data 2").unwrap();
        let entries = cache.entries_in_scope();
        assert_eq!(entries.len(), 2, "There should be two entries in scope");

        // Cleanup
        for path in entries.values() {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir_all(cache.dir_path()).unwrap();
    }

    /// Test case to retrieve all entries in the cache.
    #[test]
    fn test_all_entries() {
        let mut cache = create_temp_cache_dir().unwrap();
        cache.insert("test_file1.txt", b"Data 1").unwrap();
        cache.insert("test_file2.txt", b"Data 2").unwrap();
        let entries = cache.all_entries();
        assert_eq!(entries.len(), 2, "Cache should contain two entries");

        // Cleanup
        for path in entries {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir_all(cache.dir_path()).unwrap();
    }

    /// Test case to verify file replacement when content changes.
    #[test]
    fn test_insert_file_content_change() {
        let mut cache = create_temp_cache_dir().unwrap();
        cache
            .insert("replace_file.txt", b"Original content")
            .unwrap();

        let path = cache.insert("replace_file.txt", b"New content").unwrap();

        let mut file = File::open(&path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        assert_eq!(content, "New content", "File content should be updated");

        // Cleanup
        fs::remove_file(path).unwrap();
        fs::remove_dir_all(cache.dir_path()).unwrap();
    }
}
