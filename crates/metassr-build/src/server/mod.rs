pub mod renderer;

pub mod cache;
pub mod config;
pub mod generation;
pub mod manifest;
mod pages_generator;
mod render;
mod render_exec;
pub mod target;
mod targets;

use crate::traits::Build;
use anyhow::{Context, Result};

use cache::ServerCacheService;
use config::{BuildingType, ServerConfig};
use generation::ServerTargetService;
use manifest::ManifestGenerator;
use metassr_bundler::WebBundler;
use metassr_fs_analyzer::{
    dist_dir::DistDir,
    src_dir::{special_entries, SourceDir},
    DirectoryAnalyzer,
};
use pages_generator::PagesGenerator;
use renderer::head::HeadRenderer;
use target::{ServerTargetCollection, Targets};
use tracing::debug;

use std::{ffi::OsStr, path::Path};

/// Enhanced server-side builder with improved architecture and error handling
pub struct ServerSideBuilder {
    config: ServerConfig,
}

impl ServerSideBuilder {
    /// Creates a new ServerSideBuilder with the given configuration
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Creates a ServerSideBuilder with simple parameters (legacy compatibility)
    pub fn simple<S>(root: &S, dist_dir: &str, building_type: BuildingType) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let config = ServerConfig::new(root, dist_dir)?
            .with_building_type(building_type);
        Ok(Self::new(config))
    }

    /// Creates a ServerSideBuilder with custom configuration
    pub fn with_config(config: ServerConfig) -> Result<Self> {
        config.ensure_directories()?;
        Ok(Self::new(config))
    }

    /// Gets the current configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn with_updated_config(mut self, config: ServerConfig) -> Result<Self> {
        config.ensure_directories()?;
        self.config = config;
        Ok(self)
    }

    /// Builds the server-side code with detailed logging and error handling
    fn build_internal(&self) -> Result<ServerBuildResult> {
        // Ensure directories exist
        self.config.ensure_directories()
            .context("Failed to ensure required directories exist")?;

        // Initialize services
        let mut cache_service = ServerCacheService::new(self.config.clone())
            .context("Failed to initialize server cache service")?;
        
        let target_service = ServerTargetService::new(self.config.clone());

        // Analyze source directory
        let src = SourceDir::new(&self.config.src_dir)
            .analyze()
            .context("Failed to analyze source directory")?;

        let pages = src.clone().pages();
        let (special_entries::App(app), special_entries::Head(head)) = src.specials()
            .context("Failed to find required special entries (like _app and _head)")?;

        // Validate target generation
        let validation = target_service.validate_generation(&app, &pages.as_map())
            .context("Failed to validate target generation")?;

        if !validation.is_valid() {
            return Err(anyhow::anyhow!(
                "Target generation validation failed: {:?}",
                validation.errors
            ));
        }

        // Generate targets
        let targets = target_service.generate_targets(&app, &pages.as_map(), &mut cache_service)
            .context("Failed to generate server targets")?;

        // Bundle the targets
        let bundling_result = self.bundle_targets(&targets)
            .context("Failed to bundle server targets")?;

        // Generate manifest
        let manifest_result = if self.config.manifest_options.generate_manifest {
            Some(self.generate_manifest(&targets, &cache_service, &head)
                .context("Failed to generate manifest")?)
        } else {
            None
        };

        // Handle Static Site Generation if required
        let ssg_result = if self.config.building_type == BuildingType::StaticSiteGeneration {
            Some(self.generate_static_pages(&targets, &head, &cache_service)
                .context("Failed to generate static pages")?)
        } else {
            None
        };

        Ok(ServerBuildResult {
            targets_processed: targets.len(),
            cache_stats: cache_service.get_cache_stats(),
            bundling_result,
            manifest_result,
            ssg_result,
            target_stats: targets.stats(),
        })
    }

    /// Bundles the server targets
    fn bundle_targets(&self, targets: &ServerTargetCollection) -> Result<ServerBundlingResult> {
        let bundling_targets = targets.ready_for_bundling(&self.config.dist_dir)
            .context("Failed to prepare targets for bundling")?;

        let bundler = WebBundler::new(&bundling_targets, &self.config.dist_dir)
            .context("Failed to create web bundler for server targets")?;

        bundler.exec()
            .context("Server bundling process failed")?;

        Ok(ServerBundlingResult {
            bundled_targets: bundling_targets.len(),
            target_names: bundling_targets.keys().cloned().collect(),
        })
    }

    /// Generates the manifest
    fn generate_manifest(
        &self,
        targets: &ServerTargetCollection,
        cache_service: &ServerCacheService,
        head: &Path,
    ) -> Result<ServerManifestResult> {
        if !self.config.manifest_options.generate_manifest {
            return Ok(ServerManifestResult::skipped());
        }

        let dist = DistDir::new(&self.config.dist_dir)
            .context("Failed to analyze dist directory")?
            .analyze()
            .context("Failed to analyze dist directory structure")?;

        // Convert targets to legacy format for compatibility
        let mut legacy_targets = targets::Targets::new();
        for target in targets.targets() {
            legacy_targets.insert(target.func_id, &target.cached_path);
        }

        let manifest = ManifestGenerator::new(
            legacy_targets,
            cache_service.cache_dir().clone(),
            dist,
        ).generate(head)
            .context("Failed to generate manifest")?;

        manifest.write(&self.config.dist_dir)
            .context("Failed to write manifest to disk")?;

        // Render head
        HeadRenderer::new(&manifest.global.head, cache_service.cache_dir().clone())
            .render(true)
            .context("Failed to render head component")?;

        Ok(ServerManifestResult {
            manifest_written: true,
            head_rendered: true,
            manifest_path: self.config.dist_dir.join(&self.config.manifest_options.manifest_filename),
        })
    }

    /// Generates static pages for SSG
    fn generate_static_pages(
        &self,
        targets: &ServerTargetCollection,
        head: &Path,
        cache_service: &ServerCacheService,
    ) -> Result<ServerSSGResult> {
        // Convert targets to legacy format for compatibility
        let mut legacy_targets = Targets::new();
        for target in targets.targets() {
            legacy_targets.insert(target.func_id, &target.cached_path);
        }

        PagesGenerator::new(legacy_targets, head, &self.config.dist_dir, cache_service.cache_dir().clone())?
            .generate()
            .context("Failed to generate static pages")?;

        Ok(ServerSSGResult {
            pages_generated: targets.len(),
            output_dir: self.config.dist_dir.clone(),
        })
    }
}

impl Build for ServerSideBuilder {
    type Output = ();

    fn build(&self) -> Result<Self::Output> {
        let result = self.build_internal()
            .context("Server build process failed")?;

        // Log build statistics
        debug!("Server build completed successfully");
        debug!("Building type: {:?}", self.config.building_type);
        debug!("Processed {} targets", result.targets_processed);
        debug!("Cache: {}", result.cache_stats);
        debug!("Bundling: {} targets bundled", result.bundling_result.bundled_targets);
        
        if let Some(manifest) = &result.manifest_result {
            if manifest.manifest_written {
                debug!("Manifest generated at: {}", manifest.manifest_path.display());
            }
        }

        if let Some(ssg) = &result.ssg_result {
            debug!("Generated {} static pages", ssg.pages_generated);
        }

        Ok(())
    }
}

/// Result of a server build operation
#[derive(Debug)]
pub struct ServerBuildResult {
    pub targets_processed: usize,
    pub cache_stats: cache::ServerCacheStats,
    pub bundling_result: ServerBundlingResult,
    pub manifest_result: Option<ServerManifestResult>,
    pub ssg_result: Option<ServerSSGResult>,
    pub target_stats: target::ServerTargetCollectionStats,
}

/// Result of server bundling operation
#[derive(Debug)]
pub struct ServerBundlingResult {
    pub bundled_targets: usize,
    pub target_names: Vec<String>,
}

/// Result of server manifest generation
#[derive(Debug)]
pub struct ServerManifestResult {
    pub manifest_written: bool,
    pub head_rendered: bool,
    pub manifest_path: std::path::PathBuf,
}

impl ServerManifestResult {
    fn skipped() -> Self {
        Self {
            manifest_written: false,
            head_rendered: false,
            manifest_path: std::path::PathBuf::new(),
        }
    }
}

/// Result of static site generation
#[derive(Debug)]
pub struct ServerSSGResult {
    pub pages_generated: usize,
    pub output_dir: std::path::PathBuf,
}

impl std::fmt::Display for ServerBuildResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Server Build Result:")?;
        writeln!(f, "  Targets processed: {}", self.targets_processed)?;
        writeln!(f, "  {}", self.cache_stats)?;
        writeln!(f, "  Bundled targets: {}", self.bundling_result.bundled_targets)?;
        
        if let Some(manifest) = &self.manifest_result {
            if manifest.manifest_written {
                writeln!(f, "  Manifest: Generated")?;
            }
        }
        
        if let Some(ssg) = &self.ssg_result {
            writeln!(f, "  Static pages: {}", ssg.pages_generated)?;
        }
        
        write!(f, "  {}", self.target_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_env() -> std::path::PathBuf {
        let temp_dir = env::temp_dir().join("metassr_server_test");
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
        
        // Create _head.tsx
        let head_content = r#"
import React from 'react';

export default function Head() {
    return (
        <>
            <title>Test App</title>
            <meta name="description" content="Test description" />
        </>
    );
}
"#;
        std::fs::write(src_dir.join("_head.tsx"), head_content).unwrap();
        
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
    fn test_server_builder_creation() {
        let test_dir = create_test_env();
        
        let config = ServerConfig::new(&test_dir, "dist").unwrap();
        let builder = ServerSideBuilder::new(config);
        
        assert_eq!(builder.config().building_type, BuildingType::ServerSideRendering);
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_server_builder_simple() {
        let test_dir = create_test_env();
        
        let builder = ServerSideBuilder::simple(&test_dir, "dist", BuildingType::StaticSiteGeneration).unwrap();
        assert_eq!(builder.config().building_type, BuildingType::StaticSiteGeneration);
        
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
            
        let builder = ServerSideBuilder::with_config(config).unwrap();
        assert_eq!(builder.config().building_type, BuildingType::StaticSiteGeneration);
        assert_eq!(builder.config().server_extension, "ssr.js");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
