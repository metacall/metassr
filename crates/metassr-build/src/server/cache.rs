use anyhow::{Context, Result};
use metassr_utils::cache_dir::CacheDir;

use super::{config::ServerConfig, target::ServerTargetCollection};

/// Service for managing cache operations during server build process
pub struct ServerCacheService {
    cache_dir: CacheDir,
    config: ServerConfig,
}

impl ServerCacheService {
    /// Creates a new server cache service
    pub fn new(config: ServerConfig) -> Result<Self> {
        let cache_dir = CacheDir::new(config.cache_dir_str())
            .context("Failed to initialize server cache directory")?;

        Ok(Self { cache_dir, config })
    }

    /// Stores a target in the cache and returns the cached path
    pub fn store_target(&mut self, cache_path: &str, content: &[u8]) -> Result<std::path::PathBuf> {
        self.cache_dir
            .insert(cache_path, content)
            .with_context(|| format!("Failed to cache target at: {}", cache_path))
    }

    /// Stores multiple targets in the cache
    pub fn store_targets(&mut self, targets: &ServerTargetCollection) -> Result<ServerCacheStats> {
        let mut stored_files = 0;
        let mut total_bytes = 0;

        for target in targets.targets() {
            let cache_path = format!("pages/{}", target.id);

            self.cache_dir
                .insert(&cache_path, target.content_bytes())
                .with_context(|| format!("Failed to cache server target: {}", target.id))?;

            stored_files += 1;
            total_bytes += target.metadata.content_size;
        }

        Ok(ServerCacheStats {
            stored_files,
            total_bytes,
            cache_dir: self.cache_dir.path().to_path_buf(),
        })
    }

    /// Retrieves the cache entries suitable for bundling
    pub fn get_bundler_entries(&self) -> Result<std::collections::HashMap<String, String>> {
        let entries = self
            .cache_dir
            .entries_in_scope()
            .iter()
            .map(|(entry_name, path)| {
                let fullpath = path.canonicalize().with_context(|| {
                    format!("Failed to canonicalize cache path: {}", path.display())
                })?;

                Ok((entry_name.to_owned(), format!("{}", fullpath.display())))
            })
            .collect::<Result<std::collections::HashMap<String, String>>>()?;

        Ok(entries)
    }

    /// Gets the cache directory reference
    pub fn cache_dir(&self) -> &CacheDir {
        &self.cache_dir
    }

    /// Gets a mutable reference to the cache directory
    pub fn cache_dir_mut(&mut self) -> &mut CacheDir {
        &mut self.cache_dir
    }

    /// Clears the cache
    pub fn clear_cache(&mut self) -> Result<()> {
        let cache_path = self.config.cache_dir_str();

        if std::path::Path::new(cache_path).exists() {
            std::fs::remove_dir_all(cache_path)
                .context("Failed to remove server cache directory")?;
        }

        self.cache_dir =
            CacheDir::new(cache_path).context("Failed to recreate server cache directory")?;

        Ok(())
    }

    /// Gets cache statistics
    pub fn get_cache_stats(&self) -> ServerCacheStats {
        let entries = self.cache_dir.entries_in_scope();
        let stored_files = entries.len();
        let total_bytes = entries
            .values()
            .filter_map(|path| std::fs::metadata(path).ok())
            .map(|metadata| metadata.len() as usize)
            .sum();

        ServerCacheStats {
            stored_files,
            total_bytes,
            cache_dir: self.cache_dir.path().to_path_buf(),
        }
    }

    /// Validates cache integrity
    pub fn validate_cache(&self) -> Result<ServerCacheValidation> {
        let mut validation = ServerCacheValidation::new();
        let entries = self.cache_dir.entries_in_scope();

        for (entry_name, path) in entries.iter() {
            if !path.exists() {
                validation.add_missing_file(entry_name.clone(), path.clone());
            } else if path.is_dir() {
                validation.add_warning(format!(
                    "Cache entry '{}' is a directory, expected a file",
                    entry_name
                ));
            } else {
                // Check if file is readable
                if let Err(e) = std::fs::read(path) {
                    validation.add_error(format!("Cannot read cache file '{}': {}", entry_name, e));
                }
            }
        }

        Ok(validation)
    }
}

/// Statistics about server cache operations
#[derive(Debug)]
pub struct ServerCacheStats {
    pub stored_files: usize,
    pub total_bytes: usize,
    pub cache_dir: std::path::PathBuf,
}

impl std::fmt::Display for ServerCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server Cache: {} files, {} bytes in {}",
            self.stored_files,
            self.total_bytes,
            self.cache_dir.display()
        )
    }
}

/// Result of server cache validation
#[derive(Debug)]
pub struct ServerCacheValidation {
    missing_files: Vec<(String, std::path::PathBuf)>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ServerCacheValidation {
    fn new() -> Self {
        Self {
            missing_files: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_missing_file(&mut self, entry: String, path: std::path::PathBuf) {
        self.missing_files.push((entry, path));
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Checks if cache validation passed
    pub fn is_valid(&self) -> bool {
        self.missing_files.is_empty() && self.errors.is_empty()
    }

    /// Gets all validation issues
    pub fn issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        for (entry, path) in &self.missing_files {
            issues.push(format!(
                "Missing cache file '{}' at {}",
                entry,
                path.display()
            ));
        }

        issues.extend(self.errors.iter().cloned());
        issues.extend(self.warnings.iter().cloned());

        issues
    }

    /// Gets the number of missing files
    pub fn missing_file_count(&self) -> usize {
        self.missing_files.len()
    }

    /// Gets the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Gets the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

impl std::fmt::Display for ServerCacheValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_valid() {
            write!(f, "Server cache validation passed")
        } else {
            write!(
                f,
                "Server cache validation failed: {} missing files, {} errors, {} warnings",
                self.missing_file_count(),
                self.error_count(),
                self.warning_count()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::target::ServerTarget;
    use std::{env, path::PathBuf};

    fn create_test_config() -> ServerConfig {
        let temp_dir = env::temp_dir().join("metassr_server_cache_test");
        let src_dir = temp_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        ServerConfig::new(&temp_dir, "dist").unwrap()
    }

    #[test]
    fn test_server_cache_service_creation() {
        let config = create_test_config();
        config.ensure_directories().unwrap();

        let cache_service = ServerCacheService::new(config);
        assert!(cache_service.is_ok());

        // Cleanup
        let _ = std::fs::remove_dir_all(env::temp_dir().join("metassr_server_cache_test"));
    }

    #[test]
    fn test_server_cache_stats() {
        let config = create_test_config();
        config.ensure_directories().unwrap();

        let cache_service = ServerCacheService::new(config).unwrap();
        let stats = cache_service.get_cache_stats();

        assert_eq!(stats.stored_files, 0); // Empty cache initially

        // Cleanup
        let _ = std::fs::remove_dir_all(env::temp_dir().join("metassr_server_cache_test"));
    }

    #[test]
    fn test_server_cache_validation() {
        let config = create_test_config();
        config.ensure_directories().unwrap();

        let cache_service = ServerCacheService::new(config).unwrap();
        let validation = cache_service.validate_cache().unwrap();

        assert!(validation.is_valid()); // Empty cache should be valid

        // Cleanup
        let _ = std::fs::remove_dir_all(env::temp_dir().join("metassr_server_cache_test"));
    }

    #[test]
    fn test_server_cache_validation_display() {
        let mut validation = ServerCacheValidation::new();
        validation.add_error("Test error".to_string());
        validation.add_missing_file("test".to_string(), PathBuf::from("/missing"));

        let display_text = format!("{}", validation);
        assert!(display_text.contains("Server cache validation failed"));
        assert!(display_text.contains("1 missing files"));
        assert!(display_text.contains("1 errors"));
    }
}
