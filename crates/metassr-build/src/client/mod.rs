use crate::traits::{Build, Generate};
use anyhow::{Context, Result};

use metassr_fs_analyzer::src_dir::special_entries;
use metassr_fs_analyzer::{src_dir::SourceDir, DirectoryAnalyzer};

pub mod bundling;
pub mod cache;
pub mod config;
pub mod hydrator;
pub mod target;

use bundling::BundlingService;
use cache::CacheService;
use config::ClientConfig;
use hydrator::{HydrationConfig, Hydrator};
use target::{TargetBuilder, TargetCollection};
use tracing::debug;

/// Enhanced client builder with improved architecture and error handling
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Creates a new ClientBuilder with the given configuration
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Creates a ClientBuilder with simple parameters (legacy compatibility)
    pub fn simple<S>(root: &S, dist_dir: &str) -> Result<Self>
    where
        S: AsRef<std::ffi::OsStr> + ?Sized,
    {
        let config = ClientConfig::new(root, dist_dir)?;
        Ok(Self::new(config))
    }

    /// Creates a ClientBuilder with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        config.ensure_directories()?;
        Ok(Self::new(config))
    }

    /// Gets the current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn with_updated_config(mut self, config: ClientConfig) -> Result<Self> {
        config.ensure_directories()?;
        self.config = config;
        Ok(self)
    }

    /// Builds the client-side code with detailed logging and error handling
    fn build_internal(&self) -> Result<ClientBuildResult> {
        // Ensure directories exist
        self.config.ensure_directories()
            .context("Failed to ensure required directories exist")?;

        // Initialize services
        let mut cache_service = CacheService::new(self.config.clone())
            .context("Failed to initialize cache service")?;
        
        let bundling_service = BundlingService::new(self.config.clone());

        // Analyze source directory
        let src = SourceDir::new(&self.config.src_dir)
            .analyze()
            .context("Failed to analyze source directory")?;

        let pages = src.pages();
        let (special_entries::App(app_path), _) = src.specials()
            .context("Failed to find required special entries (like _app)")?;

        // Generate targets
        let mut targets = TargetCollection::new();
        
        for page in pages.iter() {
            // Create hydration configuration for this page
            let hydration_config = HydrationConfig::builder()
                .app_path(&app_path)
                .page_path(&page.info.path)
                .root_id(&self.config.root_id)
                .strict_mode(true)
                .build()
                .with_context(|| format!("Failed to create hydration config for page: {}", page.id))?;

            // Generate hydration script
            let hydrator = Hydrator::new(hydration_config);
            let hydration_content = hydrator.generate()
                .with_context(|| format!("Failed to generate hydration script for page: {}", page.id))?;

            // Create build target
            let target = TargetBuilder::from_page(page, hydration_content, &self.config.output_extension)
                .with_context(|| format!("Failed to create build target for page: {}", page.id))?;

            targets.add_target(target)
                .with_context(|| format!("Failed to add target for page: {}", page.id))?;
        }

        // Validate targets before proceeding
        let validation = bundling_service.validate_targets(&targets)
            .context("Failed to validate build targets")?;

        if !validation.is_valid() {
            return Err(anyhow::anyhow!(
                "Target validation failed: {:?}",
                validation.errors
            ));
        }

        // Store targets in cache
        let cache_stats = cache_service.store_targets(&targets)
            .context("Failed to store targets in cache")?;

        // Bundle the cached files
        let bundling_result = bundling_service.bundle(&targets)
            .context("Failed to bundle client code")?;

        if !bundling_result.success {
            return Err(anyhow::anyhow!(
                "Bundling failed: {:?}",
                bundling_result.messages
            ));
        }

        Ok(ClientBuildResult {
            targets_processed: targets.len(),
            cache_stats,
            bundling_result,
            target_stats: targets.stats(),
        })
    }
}

impl Build for ClientBuilder {
    type Output = ();

    fn build(&self) -> Result<Self::Output> {
        let result = self.build_internal()
            .context("Client build process failed")?;

        // Log build statistics
        debug!("Client build completed successfully");
        debug!("Processed {} targets", result.targets_processed);
        debug!("Cache: {}", result.cache_stats);
        debug!("Bundling: {} targets bundled", result.bundling_result.bundled_targets);

        Ok(())
    }
}

/// Result of a client build operation
#[derive(Debug)]
pub struct ClientBuildResult {
    pub targets_processed: usize,
    pub cache_stats: cache::CacheStats,
    pub bundling_result: bundling::BundlingResult,
    pub target_stats: target::TargetCollectionStats,
}

impl std::fmt::Display for ClientBuildResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Client Build Result:")?;
        writeln!(f, "  Targets processed: {}", self.targets_processed)?;
        writeln!(f, "  {}", self.cache_stats)?;
        writeln!(f, "  Bundled targets: {}", self.bundling_result.bundled_targets)?;
        write!(f, "  {}", self.target_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_env() -> std::path::PathBuf {
        let temp_dir = env::temp_dir().join("metassr_client_test");
        let src_dir = temp_dir.join("src");
        let pages_dir = src_dir.join("pages");
        
        // Create directories
        std::fs::create_dir_all(&pages_dir).unwrap();
        
        // Create _app.tsx
        let app_content = r#"
import React from 'react';

export default function App({ Component }) {
    return <Component />;
}
"#;
        std::fs::write(src_dir.join("_app.tsx"), app_content).unwrap();
        
        // Create a test page
        let page_content = r#"
import React from 'react';

export default function Home() {
    return <div>Home Page</div>;
}
"#;
        std::fs::write(pages_dir.join("home.tsx"), page_content).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_client_builder_creation() {
        let test_dir = create_test_env();
        
        let config = ClientConfig::new(&test_dir, "dist").unwrap();
        let builder = ClientBuilder::new(config);
        
        assert_eq!(builder.config().root_id, "root");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_client_builder_simple() {
        let test_dir = create_test_env();
        
        let builder = ClientBuilder::simple(&test_dir, "dist").unwrap();
        assert_eq!(builder.config().root_id, "root");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_client_config_builder() {
        let test_dir = create_test_env();
        
        let config = ClientConfig::builder()
            .root_dir(&test_dir)
            .root_id("app")
            .build()
            .unwrap();
            
        let builder = ClientBuilder::with_config(config).unwrap();
        assert_eq!(builder.config().root_id, "app");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
