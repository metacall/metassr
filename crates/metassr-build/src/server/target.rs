use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Represents a server-side build target with metadata
#[derive(Debug, Clone)]
pub struct ServerTarget {
    /// Unique identifier for the target
    pub id: String,
    /// Function ID for MetaCall execution
    pub func_id: i64,
    /// Source file path
    pub source_path: PathBuf,
    /// Cached script path
    pub cached_path: PathBuf,
    /// Generated render script content
    pub content: String,
    /// Metadata about the target
    pub metadata: ServerTargetMetadata,
}

/// Metadata associated with a server target
#[derive(Debug, Clone)]
pub struct ServerTargetMetadata {
    /// Route associated with this target
    pub route: String,
    /// Whether this is a special entry (like _app)
    pub is_special: bool,
    /// File type/extension
    pub file_type: String,
    /// Size of the content in bytes
    pub content_size: usize,
    /// Page name
    pub page_name: String,
}

impl ServerTarget {
    /// Creates a new server target
    pub fn new(
        id: String,
        func_id: i64,
        source_path: PathBuf,
        cached_path: PathBuf,
        content: String,
        route: String,
        page_name: String,
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
            func_id,
            source_path,
            cached_path,
            content,
            metadata: ServerTargetMetadata {
                route,
                is_special,
                file_type,
                content_size,
                page_name,
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

    /// Gets the absolute cached path if it exists
    pub fn absolute_cached_path(&self) -> Result<PathBuf> {
        self.cached_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize cached path: {}", self.cached_path.display()))
    }

    /// Gets the cached path as a string
    pub fn cached_path_str(&self) -> &str {
        self.cached_path.to_str().unwrap_or_default()
    }
}

/// Collection of server build targets with utilities for managing them
#[derive(Debug, Default, Clone)]
pub struct ServerTargetCollection {
    targets: Vec<ServerTarget>,
    target_map: HashMap<String, usize>,
    func_id_map: HashMap<i64, usize>,
}

impl ServerTargetCollection {
    /// Creates a new empty target collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a target to the collection
    pub fn add_target(&mut self, target: ServerTarget) -> Result<()> {
        if self.target_map.contains_key(&target.id) {
            return Err(anyhow::anyhow!("Target with id '{}' already exists", target.id));
        }

        if self.func_id_map.contains_key(&target.func_id) {
            return Err(anyhow::anyhow!("Target with func_id '{}' already exists", target.func_id));
        }

        let index = self.targets.len();
        self.target_map.insert(target.id.clone(), index);
        self.func_id_map.insert(target.func_id, index);
        self.targets.push(target);
        Ok(())
    }

    /// Gets a target by ID
    pub fn get_target(&self, id: &str) -> Option<&ServerTarget> {
        self.target_map.get(id).and_then(|&index| self.targets.get(index))
    }

    /// Gets a target by function ID
    pub fn get_target_by_func_id(&self, func_id: i64) -> Option<&ServerTarget> {
        self.func_id_map.get(&func_id).and_then(|&index| self.targets.get(index))
    }

    /// Gets all targets
    pub fn targets(&self) -> &[ServerTarget] {
        &self.targets
    }

    /// Gets all targets mutably
    pub fn targets_mut(&mut self) -> &mut [ServerTarget] {
        &mut self.targets
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
    pub fn filter_targets<F>(&self, predicate: F) -> Vec<&ServerTarget>
    where
        F: Fn(&ServerTarget) -> bool,
    {
        self.targets.iter().filter(|target| predicate(target)).collect()
    }

    /// Gets targets that are pages (not special entries)
    pub fn page_targets(&self) -> Vec<&ServerTarget> {
        self.filter_targets(|target| !target.metadata.is_special)
    }

    /// Gets special targets (like _app)
    pub fn special_targets(&self) -> Vec<&ServerTarget> {
        self.filter_targets(|target| target.metadata.is_special)
    }

    /// Converts targets to a HashMap suitable for bundling
    pub fn ready_for_bundling(&self, dist_path: &Path) -> Result<HashMap<String, String>> {
        let mut bundler_targets = HashMap::new();
        
        for target in &self.targets {
            let mut name = target.cached_path
                .strip_prefix(dist_path)
                .with_context(|| format!(
                    "Couldn't strip prefix '{}' from path '{}'",
                    dist_path.display(),
                    target.cached_path.display()
                ))?
                .to_path_buf();
            
            name.set_extension("");
            
            let absolute_path = target.absolute_cached_path()?;
            bundler_targets.insert(
                name.to_string_lossy().to_string(),
                absolute_path.to_string_lossy().to_string(),
            );
        }
        
        Ok(bundler_targets)
    }

    /// Gets targets ready for execution (func_id mapping)
    pub fn ready_for_exec(&self) -> HashMap<String, i64> {
        self.targets
            .iter()
            .map(|target| (target.cached_path_str().to_string(), target.func_id))
            .collect()
    }

    /// Gets statistics about the collection
    pub fn stats(&self) -> ServerTargetCollectionStats {
        let total_targets = self.targets.len();
        let page_targets = self.page_targets().len();
        let special_targets = self.special_targets().len();
        let total_content_size = self.targets.iter().map(|t| t.metadata.content_size).sum();

        ServerTargetCollectionStats {
            total_targets,
            page_targets,
            special_targets,
            total_content_size,
        }
    }

    /// Creates an iterator over (PathBuf, i64) pairs
    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, i64)> {
        self.targets.iter().map(|target| (&target.cached_path, target.func_id))
    }
}

/// Statistics about a server target collection
#[derive(Debug)]
pub struct ServerTargetCollectionStats {
    pub total_targets: usize,
    pub page_targets: usize,
    pub special_targets: usize,
    pub total_content_size: usize,
}

impl std::fmt::Display for ServerTargetCollectionStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Server Targets: {} total ({} pages, {} special), Content size: {} bytes",
            self.total_targets,
            self.page_targets,
            self.special_targets,
            self.total_content_size
        )
    }
}

/// Builder for creating server build targets from pages
pub struct ServerTargetBuilder;

impl ServerTargetBuilder {
    /// Creates a server target from page information and render script
    pub fn from_page(
        page_name: &str,
        page_path: &Path,
        func_id: i64,
        render_script: String,
        cached_path: PathBuf,
        server_extension: &str,
    ) -> Result<ServerTarget> {
        let target_id = format!("pages/{}.{}", page_name, server_extension);
        let route = format!("/{}", page_name);
        
        Ok(ServerTarget::new(
            target_id,
            func_id,
            page_path.to_path_buf(),
            cached_path,
            render_script,
            route,
            page_name.to_string(),
            false, // Pages are not special entries
        ))
    }

    /// Creates a server target for a special entry
    pub fn from_special(
        id: String,
        func_id: i64,
        source_path: PathBuf,
        cached_path: PathBuf,
        content: String,
        route: String,
        name: String,
    ) -> ServerTarget {
        ServerTarget::new(
            id,
            func_id,
            source_path,
            cached_path,
            content,
            route,
            name,
            true, // This is a special entry
        )
    }
}

/// Legacy compatibility - wraps the new ServerTargetCollection to match old Targets interface
pub struct Targets(ServerTargetCollection);

impl Targets {
    pub fn new() -> Self {
        Self(ServerTargetCollection::new())
    }

    pub fn insert(&mut self, func_id: i64, path: &Path) {
        // This is a simplified version for compatibility
        // In practice, you'd want to create a proper ServerTarget
        let target = ServerTarget::new(
            path.to_string_lossy().to_string(),
            func_id,
            path.to_path_buf(),
            path.to_path_buf(),
            String::new(),
            String::new(),
            String::new(),
            false,
        );
        let _ = self.0.add_target(target);
    }

    pub fn ready_for_bundling(&self, dist_path: &PathBuf) -> HashMap<String, String> {
        self.0.ready_for_bundling(dist_path).unwrap_or_default()
    }

    pub fn ready_for_exec(&self) -> HashMap<String, i64> {
        self.0.ready_for_exec()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, i64)> {
        self.0.iter()
    }
}

impl Default for Targets {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Targets {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_target() -> ServerTarget {
        let temp_dir = env::temp_dir().join("metassr_server_target_test");
        let source_path = temp_dir.join("test.tsx");
        let cached_path = temp_dir.join("test.server.js");
        
        ServerTarget::new(
            "test".to_string(),
            12345,
            source_path,
            cached_path,
            "console.log('test')".to_string(),
            "/test".to_string(),
            "test".to_string(),
            false,
        )
    }

    #[test]
    fn test_server_target_collection() {
        let mut collection = ServerTargetCollection::new();
        
        let target = create_test_target();
        let func_id = target.func_id;
        let id = target.id.clone();
        
        collection.add_target(target).unwrap();
        assert_eq!(collection.len(), 1);
        assert!(collection.get_target(&id).is_some());
        assert!(collection.get_target_by_func_id(func_id).is_some());
    }

    #[test]
    fn test_target_filtering() {
        let mut collection = ServerTargetCollection::new();
        
        let page_target = create_test_target();
        let special_target = ServerTarget::new(
            "app".to_string(),
            54321,
            PathBuf::from("src/_app.tsx"),
            PathBuf::from("_app.server.js"),
            "app content".to_string(),
            "/_app".to_string(),
            "_app".to_string(),
            true,
        );

        collection.add_target(page_target).unwrap();
        collection.add_target(special_target).unwrap();

        assert_eq!(collection.page_targets().len(), 1);
        assert_eq!(collection.special_targets().len(), 1);
    }

    #[test]
    fn test_target_stats() {
        let mut collection = ServerTargetCollection::new();
        
        let target = create_test_target();
        collection.add_target(target).unwrap();
        let stats = collection.stats();
        
        assert_eq!(stats.total_targets, 1);
        assert_eq!(stats.page_targets, 1);
        assert_eq!(stats.special_targets, 0);
        assert_eq!(stats.total_content_size, 17); // "console.log('test')" length
    }

    #[test]
    fn test_legacy_targets_compatibility() {
        let mut targets = Targets::new();
        let path = PathBuf::from("/test/path");
        
        targets.insert(12345, &path);
        
        let exec_map = targets.ready_for_exec();
        assert!(exec_map.contains_key(path.to_str().unwrap()));
    }
}
