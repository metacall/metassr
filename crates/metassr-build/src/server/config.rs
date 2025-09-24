use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Building type for server-side rendering
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BuildingType {
    ServerSideRendering,
    StaticSiteGeneration,
}

impl Default for BuildingType {
    fn default() -> Self {
        BuildingType::ServerSideRendering
    }
}

/// Configuration for the server building process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Root directory of the project
    pub root_dir: PathBuf,
    /// Source directory (usually "src")
    pub src_dir: PathBuf,
    /// Distribution directory
    pub dist_dir: PathBuf,
    /// Cache directory for intermediate files
    pub cache_dir: PathBuf,
    /// Building type (SSR or SSG)
    pub building_type: BuildingType,
    /// File extension for generated server files
    pub server_extension: String,
    /// Custom bundler options
    pub bundler_options: ServerBundlerOptions,
    /// Manifest generation options
    pub manifest_options: ManifestOptions,
}

/// Configuration for server-side bundling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerBundlerOptions {
    /// Whether to enable source maps
    pub source_maps: bool,
    /// Whether to minimize output
    pub minimize: bool,
    /// Target environment (node, etc.)
    pub target: String,
    /// Additional entry points
    pub additional_entries: HashMap<String, String>,
}

impl Default for ServerBundlerOptions {
    fn default() -> Self {
        Self {
            source_maps: true,
            minimize: false,
            target: "node".to_string(),
            additional_entries: HashMap::new(),
        }
    }
}

/// Configuration for manifest generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestOptions {
    /// Whether to generate manifest
    pub generate_manifest: bool,
    /// Custom manifest filename
    pub manifest_filename: String,
    /// Whether to include development info
    pub include_dev_info: bool,
}

impl Default for ManifestOptions {
    fn default() -> Self {
        Self {
            generate_manifest: true,
            manifest_filename: "manifest.json".to_string(),
            include_dev_info: false,
        }
    }
}

impl ServerConfig {
    /// Creates a new ServerConfig with default settings
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
            building_type: BuildingType::default(),
            server_extension: "server.js".to_string(),
            bundler_options: ServerBundlerOptions::default(),
            manifest_options: ManifestOptions::default(),
        })
    }

    /// Creates a custom ServerConfig
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
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

    /// Sets the building type
    pub fn with_building_type(mut self, building_type: BuildingType) -> Self {
        self.building_type = building_type;
        self
    }

    /// Sets the server extension
    pub fn with_server_extension<S: Into<String>>(mut self, extension: S) -> Self {
        self.server_extension = extension.into();
        self
    }

    /// Sets the bundler options
    pub fn with_bundler_options(mut self, options: ServerBundlerOptions) -> Self {
        self.bundler_options = options;
        self
    }

    /// Sets the manifest options
    pub fn with_manifest_options(mut self, options: ManifestOptions) -> Self {
        self.manifest_options = options;
        self
    }
}

/// Builder for ServerConfig
#[derive(Default)]
pub struct ServerConfigBuilder {
    root_dir: Option<PathBuf>,
    src_dir: Option<PathBuf>,
    dist_dir: Option<PathBuf>,
    cache_dir: Option<PathBuf>,
    building_type: Option<BuildingType>,
    server_extension: Option<String>,
    bundler_options: Option<ServerBundlerOptions>,
    manifest_options: Option<ManifestOptions>,
}

impl ServerConfigBuilder {
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

    pub fn building_type(mut self, building_type: BuildingType) -> Self {
        self.building_type = Some(building_type);
        self
    }

    pub fn server_extension<S: Into<String>>(mut self, ext: S) -> Self {
        self.server_extension = Some(ext.into());
        self
    }

    pub fn bundler_options(mut self, options: ServerBundlerOptions) -> Self {
        self.bundler_options = Some(options);
        self
    }

    pub fn manifest_options(mut self, options: ManifestOptions) -> Self {
        self.manifest_options = Some(options);
        self
    }

    pub fn build(self) -> Result<ServerConfig> {
        let root_dir = self.root_dir.ok_or_else(|| anyhow::anyhow!("Root directory is required"))?;
        let src_dir = self.src_dir.unwrap_or_else(|| root_dir.join("src"));
        let dist_dir = self.dist_dir.unwrap_or_else(|| root_dir.join("dist"));
        let cache_dir = self.cache_dir.unwrap_or_else(|| dist_dir.join("cache"));

        ServerConfig::validate_directories(&root_dir, &src_dir)?;

        Ok(ServerConfig {
            root_dir,
            src_dir,
            dist_dir,
            cache_dir,
            building_type: self.building_type.unwrap_or_default(),
            server_extension: self.server_extension.unwrap_or_else(|| "server.js".to_string()),
            bundler_options: self.bundler_options.unwrap_or_default(),
            manifest_options: self.manifest_options.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_env() -> PathBuf {
        let temp_dir = env::temp_dir().join("metassr_server_config_test");
        let src_dir = temp_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        temp_dir
    }

    #[test]
    fn test_server_config_creation() {
        let test_dir = create_test_env();
        
        let config = ServerConfig::new(&test_dir, "dist").unwrap();
        assert_eq!(config.root_dir, test_dir);
        assert_eq!(config.src_dir, test_dir.join("src"));
        assert_eq!(config.dist_dir, test_dir.join("dist"));
        assert_eq!(config.building_type, BuildingType::ServerSideRendering);
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_server_config_builder() {
        let test_dir = create_test_env();
        
        let config = ServerConfig::builder()
            .root_dir(&test_dir)
            .building_type(BuildingType::StaticSiteGeneration)
            .server_extension("ssr.js")
            .build()
            .unwrap();

        assert_eq!(config.building_type, BuildingType::StaticSiteGeneration);
        assert_eq!(config.server_extension, "ssr.js");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_ensure_directories() {
        let test_dir = create_test_env();
        
        let config = ServerConfig::new(&test_dir, "dist").unwrap();
        config.ensure_directories().unwrap();

        assert!(config.dist_dir.exists());
        assert!(config.cache_dir.exists());
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_config_methods() {
        let test_dir = create_test_env();
        
        let config = ServerConfig::new(&test_dir, "dist")
            .unwrap()
            .with_building_type(BuildingType::StaticSiteGeneration)
            .with_server_extension("custom.js");

        assert_eq!(config.building_type, BuildingType::StaticSiteGeneration);
        assert_eq!(config.server_extension, "custom.js");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
