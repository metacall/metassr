use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use metassr_fs_analyzer::src_dir::PagesEntriesType;

use super::{
    cache::ServerCacheService,
    config::ServerConfig,
    render::{ServerRender, ServerRenderConfig},
    target::{ServerTarget, ServerTargetBuilder, ServerTargetCollection},
};
use crate::utils::setup_page_path;
use crate::traits::Generate;

/// Service for generating server-side targets
pub struct ServerTargetService {
    config: ServerConfig,
}

impl ServerTargetService {
    /// Creates a new server target service
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Generates server targets from app and pages
    pub fn generate_targets(
        &self,
        app_path: &Path,
        pages: &PagesEntriesType,
        cache_service: &mut ServerCacheService,
    ) -> Result<ServerTargetCollection> {
        let mut targets = ServerTargetCollection::new();

        for (page_name, page_path) in pages.iter() {
            let target = self.generate_page_target(app_path, page_name, page_path, cache_service)
                .with_context(|| format!("Failed to generate target for page: {}", page_name))?;

            targets.add_target(target)
                .with_context(|| format!("Failed to add target for page: {}", page_name))?;
        }

        Ok(targets)
    }

    /// Generates a single page target
    fn generate_page_target(
        &self,
        app_path: &Path,
        page_name: &str,
        page_path: &Path,
        cache_service: &mut ServerCacheService,
    ) -> Result<ServerTarget> {
        // Create render configuration
        let render_config = ServerRenderConfig::builder()
            .app_path(app_path)
            .page_path(page_path)
            .build()
            .with_context(|| format!("Failed to create render config for page: {}", page_name))?;

        // Generate render script
        let renderer = ServerRender::new(render_config);
        let (func_id, render_script) = renderer.generate()
            .with_context(|| format!("Failed to generate render script for page: {}", page_name))?;

        // Setup cache path
        let page_file = setup_page_path(page_name, &self.config.server_extension);
        let cache_path = format!("pages/{}", page_file.display());

        // Store in cache
        let cached_path = cache_service.store_target(&cache_path, render_script.as_bytes())
            .with_context(|| format!("Failed to cache target for page: {}", page_name))?;

        // Create target
        let target = ServerTargetBuilder::from_page(
            page_name,
            page_path,
            func_id,
            render_script,
            cached_path,
            &self.config.server_extension,
        ).with_context(|| format!("Failed to create target for page: {}", page_name))?;

        Ok(target)
    }

    /// Validates that target generation is possible
    pub fn validate_generation(&self, app_path: &Path, pages: &PagesEntriesType) -> Result<TargetGenerationValidation> {
        let mut validation = TargetGenerationValidation::new();

        // Validate app path
        if !app_path.exists() {
            validation.add_error(format!("App component not found: {}", app_path.display()));
        }

        // Validate pages
        for (page_name, page_path) in pages.iter() {
            if !page_path.exists() {
                validation.add_error(format!(
                    "Page '{}' not found: {}",
                    page_name,
                    page_path.display()
                ));
            }

            // Check for valid file extensions
            if let Some(ext) = page_path.extension() {
                let ext_str = ext.to_string_lossy();
                if !["js", "jsx", "ts", "tsx"].contains(&ext_str.as_ref()) {
                    validation.add_warning(format!(
                        "Page '{}' has unusual extension: {}",
                        page_name, ext_str
                    ));
                }
            } else {
                validation.add_warning(format!(
                    "Page '{}' has no file extension",
                    page_name
                ));
            }
        }

        Ok(validation)
    }

    /// Gets statistics about target generation
    pub fn get_generation_stats(&self, pages: &PagesEntriesType) -> TargetGenerationStats {
        let total_pages = pages.len();
        let page_types = pages
            .values()
            .filter_map(|path| path.extension())
            .filter_map(|ext| ext.to_str())
            .fold(std::collections::HashMap::new(), |mut acc, ext| {
                *acc.entry(ext.to_string()).or_insert(0) += 1;
                acc
            });

        TargetGenerationStats {
            total_pages,
            page_types,
            server_extension: self.config.server_extension.clone(),
        }
    }
}

/// Result of target generation validation
#[derive(Debug, Default)]
pub struct TargetGenerationValidation {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl TargetGenerationValidation {
    fn new() -> Self {
        Self::default()
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Checks if validation passed (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Gets all issues (errors + warnings)
    pub fn all_issues(&self) -> Vec<&String> {
        self.errors.iter().chain(self.warnings.iter()).collect()
    }
}

/// Statistics about target generation
#[derive(Debug)]
pub struct TargetGenerationStats {
    pub total_pages: usize,
    pub page_types: std::collections::HashMap<String, usize>,
    pub server_extension: String,
}

impl std::fmt::Display for TargetGenerationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Target Generation Statistics:")?;
        writeln!(f, "  Total pages: {}", self.total_pages)?;
        writeln!(f, "  Server extension: {}", self.server_extension)?;
        write!(f, "  Page types: ")?;
        
        for (ext, count) in &self.page_types {
            write!(f, "{}({}) ", ext, count)?;
        }
        
        Ok(())
    }
}

pub struct TargetsGenerator<'a> {
    service: ServerTargetService,
    app: PathBuf,
    pages: PagesEntriesType,
    cache_service: &'a mut ServerCacheService,
}

impl<'a> TargetsGenerator<'a> {
    /// Creates a new targets generator
    pub fn new(
        app: PathBuf,
        pages: PagesEntriesType,
        cache: &'a mut metassr_utils::cache_dir::CacheDir,
    ) -> Self {
        // This is a compatibility layer - in practice you'd want to pass the proper config
        let config = ServerConfig::new(".", "dist").unwrap_or_else(|_| {
            // Fallback config if we can't create one
            ServerConfig::builder()
                .root_dir(".")
                .build()
                .unwrap()
        });
        
        // Create a cache service wrapper
        // Note: This is a simplification for compatibility
        let cache_service = unsafe {
            std::mem::transmute::<&'a mut metassr_utils::cache_dir::CacheDir, &'a mut ServerCacheService>(cache)
        };

        Self {
            service: ServerTargetService::new(config),
            app,
            pages,
            cache_service,
        }
    }

    /// Generates targets using the legacy interface
    pub fn generate(&mut self) -> Result<super::target::Targets> {
        let targets = self.service.generate_targets(&self.app, &self.pages, self.cache_service)?;
        
        // Convert to legacy Targets format
        let mut legacy_targets = super::target::Targets::new();
        for target in targets.targets() {
            legacy_targets.insert(target.func_id, &target.cached_path);
        }
        
        Ok(legacy_targets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::config::ServerConfig;
    use std::{collections::HashMap, env};

    fn create_test_env() -> (PathBuf, PathBuf, PathBuf) {
        let temp_dir = env::temp_dir().join("metassr_server_targets_test");
        let src_dir = temp_dir.join("src");
        let pages_dir = src_dir.join("pages");
        
        std::fs::create_dir_all(&pages_dir).unwrap();
        
        let app_path = src_dir.join("_app.tsx");
        let page_path = pages_dir.join("home.tsx");
        
        std::fs::write(&app_path, "export default function App() { return null; }").unwrap();
        std::fs::write(&page_path, "export default function Home() { return null; }").unwrap();
        
        (temp_dir, app_path, page_path)
    }

    #[test]
    fn test_server_target_service_creation() {
        let (temp_dir, _, _) = create_test_env();
        
        let config = ServerConfig::new(&temp_dir, "dist").unwrap();
        let service = ServerTargetService::new(config);
        
        assert_eq!(service.config.server_extension, "server.js");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_target_generation_validation() {
        let (temp_dir, app_path, page_path) = create_test_env();
        
        let config = ServerConfig::new(&temp_dir, "dist").unwrap();
        let service = ServerTargetService::new(config);
        
        let mut pages = HashMap::new();
        pages.insert("home".to_string(), page_path);
        
        let validation = service.validate_generation(&app_path, &pages).unwrap();
        assert!(validation.is_valid());
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generation_stats() {
        let (temp_dir, _, page_path) = create_test_env();
        
        let config = ServerConfig::new(&temp_dir, "dist").unwrap();
        let service = ServerTargetService::new(config);
        
        let mut pages = HashMap::new();
        pages.insert("home".to_string(), page_path);
        
        let stats = service.get_generation_stats(&pages);
        assert_eq!(stats.total_pages, 1);
        assert!(stats.page_types.contains_key("tsx"));
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
