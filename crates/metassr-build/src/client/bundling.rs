use anyhow::{Context, Result};

use metassr_bundler::WebBundler;

use super::{config::ClientConfig, target::TargetCollection};

/// Handles the bundling process for client-side code
pub struct BundlingService {
    config: ClientConfig,
}

impl BundlingService {
    /// Creates a new bundling service
    pub fn new(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Bundles the provided targets
    pub fn bundle(&self, targets: &TargetCollection) -> Result<BundlingResult> {
        if targets.is_empty() {
            return Ok(BundlingResult::empty());
        }

        let bundler_targets = targets
            .to_bundler_targets()
            .context("Failed to convert targets for bundling")?;

        let bundler = WebBundler::new(&bundler_targets, self.config.dist_dir_str())
            .context("Failed to create web bundler")?;

        bundler
            .exec()
            .context("Bundling process failed")?;

        Ok(BundlingResult::success(bundler_targets.len()))
    }

    /// Validates that bundling targets are ready
    pub fn validate_targets(&self, targets: &TargetCollection) -> Result<ValidationResult> {
        let mut validation = ValidationResult::new();

        for target in targets.targets() {
            // Check if source file exists
            if !target.source_path.exists() {
                validation.add_error(format!(
                    "Source file does not exist: {}",
                    target.source_path.display()
                ));
            }

            // Check if content is not empty
            if target.content.is_empty() {
                validation.add_warning(format!(
                    "Target '{}' has empty content",
                    target.id
                ));
            }

            // Check for absolute path resolution
            if let Err(e) = target.absolute_source_path() {
                validation.add_error(format!(
                    "Cannot resolve absolute path for '{}': {}",
                    target.id, e
                ));
            }
        }

        Ok(validation)
    }

    /// Gets bundling statistics
    pub fn get_bundling_stats(&self, targets: &TargetCollection) -> BundlingStats {
        let target_stats = targets.stats();
        
        BundlingStats {
            total_targets: target_stats.total_targets,
            page_targets: target_stats.page_targets,
            special_targets: target_stats.special_targets,
            total_content_size: target_stats.total_content_size,
            dist_dir: self.config.dist_dir.clone(),
            bundler_options: self.config.bundler_options.clone(),
        }
    }
}

/// Result of a bundling operation
#[derive(Debug)]
pub struct BundlingResult {
    /// Whether the bundling was successful
    pub success: bool,
    /// Number of targets that were bundled
    pub bundled_targets: usize,
    /// Any messages from the bundling process
    pub messages: Vec<String>,
}

impl BundlingResult {
    /// Creates a successful bundling result
    pub fn success(bundled_targets: usize) -> Self {
        Self {
            success: true,
            bundled_targets,
            messages: vec![format!("Successfully bundled {} targets", bundled_targets)],
        }
    }

    /// Creates an empty bundling result (no targets to bundle)
    pub fn empty() -> Self {
        Self {
            success: true,
            bundled_targets: 0,
            messages: vec!["No targets to bundle".to_string()],
        }
    }

    /// Creates a failed bundling result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            bundled_targets: 0,
            messages: vec![error],
        }
    }
}

/// Result of target validation
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Validation errors that prevent bundling
    pub errors: Vec<String>,
    /// Validation warnings that don't prevent bundling
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Creates a new validation result
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an error to the validation result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Adds a warning to the validation result
    pub fn add_warning(&mut self, warning: String) {
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

/// Statistics about bundling operation
#[derive(Debug)]
pub struct BundlingStats {
    pub total_targets: usize,
    pub page_targets: usize,
    pub special_targets: usize,
    pub total_content_size: usize,
    pub dist_dir: std::path::PathBuf,
    pub bundler_options: super::config::BundlerOptions,
}

impl std::fmt::Display for BundlingStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Bundling Statistics:")?;
        writeln!(f, "  Total targets: {}", self.total_targets)?;
        writeln!(f, "  Page targets: {}", self.page_targets)?;
        writeln!(f, "  Special targets: {}", self.special_targets)?;
        writeln!(f, "  Total content size: {} bytes", self.total_content_size)?;
        writeln!(f, "  Output directory: {}", self.dist_dir.display())?;
        writeln!(f, "  Source maps: {}", self.bundler_options.source_maps)?;
        writeln!(f, "  Minimize: {}", self.bundler_options.minimize)?;
        write!(f, "  Target: {}", self.bundler_options.target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{config::ClientConfig, target::{BuildTarget, TargetCollection}};
    use std::{path::PathBuf, env};

    fn create_test_config() -> ClientConfig {
        let temp_dir = env::temp_dir().join("metassr_bundling_test");
        let src_dir = temp_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        
        let config = ClientConfig::new(&temp_dir, "dist").unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir); // Cleanup after creation
        config
    }

    fn create_test_target() -> BuildTarget {
        BuildTarget::new(
            "test".to_string(),
            PathBuf::from("src/test.js"),
            PathBuf::from("test.js"),
            "console.log('test')".to_string(),
            "/test".to_string(),
            false,
        )
    }

    #[test]
    fn test_bundling_service_creation() {
        let config = create_test_config();
        let service = BundlingService::new(config);
        
        // Service should be created successfully
        assert_eq!(service.config.root_id, "root");
    }

    #[test]
    fn test_empty_targets_bundling() {
        let config = create_test_config();
        let service = BundlingService::new(config);
        let targets = TargetCollection::new();
        
        let result = service.bundle(&targets).unwrap();
        assert!(result.success);
        assert_eq!(result.bundled_targets, 0);
    }

    #[test]
    fn test_validation_result() {
        let mut validation = ValidationResult::new();
        
        assert!(validation.is_valid());
        
        validation.add_error("Test error".to_string());
        assert!(!validation.is_valid());
        assert_eq!(validation.errors.len(), 1);
        
        validation.add_warning("Test warning".to_string());
        assert_eq!(validation.warnings.len(), 1);
        assert_eq!(validation.all_issues().len(), 2);
    }

    #[test]
    fn test_bundling_stats() {
        let config = create_test_config();
        let service = BundlingService::new(config);
        let mut targets = TargetCollection::new();
        targets.add_target(create_test_target()).unwrap();
        
        let stats = service.get_bundling_stats(&targets);
        assert_eq!(stats.total_targets, 1);
        assert_eq!(stats.page_targets, 1);
        assert_eq!(stats.special_targets, 0);
    }
}
