use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    path::PathBuf,
};

use metassr_fs_analyzer::src_dir::Page;

/// Represents a build target with metadata
#[derive(Debug, Clone)]
pub struct BuildTarget {
    /// Unique identifier for the target
    pub id: String,
    /// Source file path
    pub source_path: PathBuf,
    /// Output file path relative to dist directory
    pub output_path: PathBuf,
    /// Generated content for this target
    pub content: String,
    /// Metadata about the target
    pub metadata: TargetMetadata,
}

/// Metadata associated with a build target
#[derive(Debug, Clone)]
pub struct TargetMetadata {
    /// Route associated with this target
    pub route: String,
    /// Whether this is a special entry (like _app)
    pub is_special: bool,
    /// File type/extension
    pub file_type: String,
    /// Size of the content in bytes
    pub content_size: usize,
}

impl BuildTarget {
    /// Creates a new build target
    pub fn new(
        id: String,
        source_path: PathBuf,
        output_path: PathBuf,
        content: String,
        route: String,
        is_special: bool,
    ) -> Self {
        let content_size = content.len();
        let file_type = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            id,
            source_path,
            output_path,
            content,
            metadata: TargetMetadata {
                route,
                is_special,
                file_type,
                content_size,
            },
        }
    }

    /// Gets the content as bytes
    pub fn content_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    /// Gets the absolute source path if it exists
    pub fn absolute_source_path(&self) -> Result<PathBuf> {
        self.source_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {}", self.source_path.display()))
    }

    /// Gets the output path as a string
    pub fn output_path_str(&self) -> &str {
        self.output_path.to_str().unwrap_or_default()
    }
}

/// Collection of build targets with utilities for managing them
#[derive(Debug, Default)]
pub struct TargetCollection {
    targets: Vec<BuildTarget>,
    target_map: HashMap<String, usize>,
}

impl TargetCollection {
    /// Creates a new empty target collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a target to the collection
    pub fn add_target(&mut self, target: BuildTarget) -> Result<()> {
        if self.target_map.contains_key(&target.id) {
            return Err(anyhow::anyhow!("Target with id '{}' already exists", target.id));
        }

        let index = self.targets.len();
        self.target_map.insert(target.id.clone(), index);
        self.targets.push(target);
        Ok(())
    }

    /// Gets a target by ID
    pub fn get_target(&self, id: &str) -> Option<&BuildTarget> {
        self.target_map.get(id).and_then(|&index| self.targets.get(index))
    }

    /// Gets a mutable reference to a target by ID
    pub fn get_target_mut(&mut self, id: &str) -> Option<&mut BuildTarget> {
        self.target_map.get(id).and_then(|&index| self.targets.get_mut(index))
    }

    /// Gets all targets
    pub fn targets(&self) -> &[BuildTarget] {
        &self.targets
    }

    /// Gets all targets mutably
    pub fn targets_mut(&mut self) -> &mut [BuildTarget] {
        &mut self.targets
    }

    /// Removes a target by ID
    pub fn remove_target(&mut self, id: &str) -> Option<BuildTarget> {
        if let Some(&index) = self.target_map.get(id) {
            self.target_map.remove(id);
            // Update indices for targets after the removed one
            for (_, target_index) in self.target_map.iter_mut() {
                if *target_index > index {
                    *target_index -= 1;
                }
            }
            Some(self.targets.remove(index))
        } else {
            None
        }
    }

    /// Gets the number of targets
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Filters targets by a predicate
    pub fn filter_targets<F>(&self, predicate: F) -> Vec<&BuildTarget>
    where
        F: Fn(&BuildTarget) -> bool,
    {
        self.targets.iter().filter(|target| predicate(target)).collect()
    }

    /// Gets targets that are pages (not special entries)
    pub fn page_targets(&self) -> Vec<&BuildTarget> {
        self.filter_targets(|target| !target.metadata.is_special)
    }

    /// Gets special targets (like _app)
    pub fn special_targets(&self) -> Vec<&BuildTarget> {
        self.filter_targets(|target| target.metadata.is_special)
    }

    /// Converts targets to a HashMap suitable for bundling
    pub fn to_bundler_targets(&self) -> Result<HashMap<String, String>> {
        let mut bundler_targets = HashMap::new();
        
        for target in &self.targets {
            let absolute_path = target.absolute_source_path()?;
            bundler_targets.insert(
                target.id.clone(),
                absolute_path.to_string_lossy().to_string(),
            );
        }
        
        Ok(bundler_targets)
    }

    /// Gets statistics about the collection
    pub fn stats(&self) -> TargetCollectionStats {
        let total_targets = self.targets.len();
        let page_targets = self.page_targets().len();
        let special_targets = self.special_targets().len();
        let total_content_size = self.targets.iter().map(|t| t.metadata.content_size).sum();

        TargetCollectionStats {
            total_targets,
            page_targets,
            special_targets,
            total_content_size,
        }
    }
}

/// Statistics about a target collection
#[derive(Debug)]
pub struct TargetCollectionStats {
    pub total_targets: usize,
    pub page_targets: usize,
    pub special_targets: usize,
    pub total_content_size: usize,
}

impl std::fmt::Display for TargetCollectionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Targets: {} total ({} pages, {} special), Content size: {} bytes",
            self.total_targets,
            self.page_targets,
            self.special_targets,
            self.total_content_size
        )
    }
}

/// Builder for creating build targets from pages
pub struct TargetBuilder;

impl TargetBuilder {
    /// Creates a build target from a page and generated content
    pub fn from_page(
        page: &Page,
        content: String,
        output_extension: &str,
    ) -> Result<BuildTarget> {
        let page_path = crate::utils::setup_page_path(&page.info.route, output_extension);
        let target_id = format!("pages/{}", page_path.display());
        
        Ok(BuildTarget::new(
            target_id,
            page.info.path.clone(),
            page_path,
            content,
            page.info.route.clone(),
            false, // Pages are not special entries
        ))
    }

    /// Creates a build target for a special entry
    pub fn from_special(
        id: String,
        source_path: PathBuf,
        content: String,
        route: String,
    ) -> BuildTarget {
        let output_path = PathBuf::from(&id);
        
        BuildTarget::new(
            id,
            source_path,
            output_path,
            content,
            route,
            true, // This is a special entry
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_target_collection() {
        let mut collection = TargetCollection::new();
        
        let target = BuildTarget::new(
            "test".to_string(),
            PathBuf::from("src/test.js"),
            PathBuf::from("test.js"),
            "console.log('test')".to_string(),
            "/test".to_string(),
            false,
        );

        collection.add_target(target).unwrap();
        assert_eq!(collection.len(), 1);
        assert!(collection.get_target("test").is_some());
    }

    #[test]
    fn test_target_filtering() {
        let mut collection = TargetCollection::new();
        
        let page_target = BuildTarget::new(
            "page".to_string(),
            PathBuf::from("src/page.js"),
            PathBuf::from("page.js"),
            "page content".to_string(),
            "/page".to_string(),
            false,
        );

        let special_target = BuildTarget::new(
            "app".to_string(),
            PathBuf::from("src/_app.js"),
            PathBuf::from("_app.js"),
            "app content".to_string(),
            "/_app".to_string(),
            true,
        );

        collection.add_target(page_target).unwrap();
        collection.add_target(special_target).unwrap();

        assert_eq!(collection.page_targets().len(), 1);
        assert_eq!(collection.special_targets().len(), 1);
    }

    #[test]
    fn test_target_stats() {
        let mut collection = TargetCollection::new();
        
        let target = BuildTarget::new(
            "test".to_string(),
            PathBuf::from("src/test.js"),
            PathBuf::from("test.js"),
            "hello world".to_string(), // 11 bytes
            "/test".to_string(),
            false,
        );

        collection.add_target(target).unwrap();
        let stats = collection.stats();
        
        assert_eq!(stats.total_targets, 1);
        assert_eq!(stats.page_targets, 1);
        assert_eq!(stats.special_targets, 0);
        assert_eq!(stats.total_content_size, 11);
    }
}
