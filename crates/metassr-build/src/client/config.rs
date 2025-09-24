use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Configuration for the client building process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Root directory of the project
    pub root_dir: PathBuf,
    /// Source directory (usually "src")
    pub src_dir: PathBuf,
    /// Distribution directory
    pub dist_dir: PathBuf,
    /// Cache directory for intermediate files
    pub cache_dir: PathBuf,
    /// Root element ID for hydration
    pub root_id: String,
    /// File extension for generated files
    pub output_extension: String,
    /// Custom bundler options
    pub bundler_options: BundlerOptions,
}

/// Configuration for the bundler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlerOptions {
    /// Whether to enable source maps
    pub source_maps: bool,
    /// Whether to minimize output
    pub minimize: bool,
    /// Target environment (web, node, etc.)
    pub target: String,
    /// Public path for assets
    pub public_path: String,
    /// Additional entry points
    pub additional_entries: HashMap<String, String>,
}

impl Default for BundlerOptions {
    fn default() -> Self {
        Self {
            source_maps: true,
            minimize: false,
            target: "web".to_string(),
            public_path: "".to_string(),
            additional_entries: HashMap::new(),
        }
    }
}

impl ClientConfig {
    /// Creates a new ClientConfig with default settings
    pub fn new<S>(root: &S, dist_dir: &str) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let root_dir = Path::new(root).to_path_buf();
        let src_dir = root_dir.join("src");
        let dist_path = root_dir.join(dist_dir);
        let cache_dir = dist_path.join("cache");

        Self::validate_directories(&root_dir, &src_dir)?;

        Ok(Self {
            root_dir,
            src_dir,
            dist_dir: dist_path,
            cache_dir,
            root_id: "root".to_string(),
            output_extension: "js".to_string(),
            bundler_options: BundlerOptions::default(),
        })
    }

    /// Creates a custom ClientConfig
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }

    /// Validates that required directories exist or can be created
    fn validate_directories(root_dir: &Path, src_dir: &Path) -> Result<()> {
        if !root_dir.exists() {
            return Err(anyhow::anyhow!(
                "Root directory does not exist: {}",
                root_dir.display()
            ));
        }

        if !src_dir.exists() {
            return Err(anyhow::anyhow!(
                "Source directory does not exist: {}",
                src_dir.display()
            ));
        }

        Ok(())
    }

    /// Ensures all required directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        if !self.dist_dir.exists() {
            std::fs::create_dir_all(&self.dist_dir)
                .with_context(|| format!("Failed to create dist directory: {}", self.dist_dir.display()))?;
        }

        if !self.cache_dir.exists() {
            std::fs::create_dir_all(&self.cache_dir)
                .with_context(|| format!("Failed to create cache directory: {}", self.cache_dir.display()))?;
        }

        Ok(())
    }

    /// Gets the cache directory path as a string
    pub fn cache_dir_str(&self) -> &str {
        self.cache_dir.to_str().unwrap_or_default()
    }

    /// Gets the dist directory path as a string
    pub fn dist_dir_str(&self) -> &str {
        self.dist_dir.to_str().unwrap_or_default()
    }
}

/// Builder for ClientConfig
#[derive(Default)]
pub struct ClientConfigBuilder {
    root_dir: Option<PathBuf>,
    src_dir: Option<PathBuf>,
    dist_dir: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    root_id: Option<String>,
    output_extension: Option<String>,
    bundler_options: Option<BundlerOptions>,
}

impl ClientConfigBuilder {
    pub fn root_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.root_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn src_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.src_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn dist_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.dist_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn cache_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.cache_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn root_id<S: Into<String>>(mut self, id: S) -> Self {
        self.root_id = Some(id.into());
        self
    }

    pub fn output_extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.output_extension = Some(ext.into());
        self
    }

    pub fn bundler_options(mut self, options: BundlerOptions) -> Self {
        self.bundler_options = Some(options);
        self
    }

    pub fn build(self) -> Result<ClientConfig> {
        let root_dir = self.root_dir.ok_or_else(|| anyhow::anyhow!("Root directory is required"))?;
        let src_dir = self.src_dir.unwrap_or_else(|| root_dir.join("src"));
        let dist_dir = self.dist_dir.unwrap_or_else(|| root_dir.join("dist"));
        let cache_dir = self.cache_dir.unwrap_or_else(|| dist_dir.join("cache"));

        ClientConfig::validate_directories(&root_dir, &src_dir)?;

        Ok(ClientConfig {
            root_dir,
            src_dir,
            dist_dir,
            cache_dir,
            root_id: self.root_id.unwrap_or_else(|| "root".to_string()),
            output_extension: self.output_extension.unwrap_or_else(|| "js".to_string()),
            bundler_options: self.bundler_options.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_client_config_creation() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("src")).unwrap();

        let config = ClientConfig::new(root, "dist").unwrap();
        assert_eq!(config.root_dir, root);
        assert_eq!(config.src_dir, root.join("src"));
        assert_eq!(config.dist_dir, root.join("dist"));
    }

    #[test]
    fn test_client_config_builder() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("src")).unwrap();

        let config = ClientConfig::builder()
            .root_dir(root)
            .root_id("app")
            .output_extension("mjs")
            .build()
            .unwrap();

        assert_eq!(config.root_id, "app");
        assert_eq!(config.output_extension, "mjs");
    }

    #[test]
    fn test_ensure_directories() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir(root.join("src")).unwrap();

        let config = ClientConfig::new(root, "dist").unwrap();
        config.ensure_directories().unwrap();

        assert!(config.dist_dir.exists());
        assert!(config.cache_dir.exists());
    }
}
