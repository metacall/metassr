use crate::{
    shared::{APP_PATH_TAG, PAGE_PATH_TAG, ROOT_ID_TAG},
    traits::Generate,
};
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

const HYDRATED_FILE_TEMPLATE: &str = include_str!("../scripts/hydrate.js.template");

/// Configuration for hydration generation
#[derive(Debug, Clone)]
pub struct HydrationConfig {
    /// Path to the app component
    pub app_path: PathBuf,
    /// Path to the page component
    pub page_path: PathBuf,
    /// Root element ID for hydration
    pub root_id: String,
    /// Additional template variables
    pub template_vars: HashMap<String, String>,
    /// Whether to use strict mode
    pub strict_mode: bool,
}

impl HydrationConfig {
    /// Creates a new hydration config
    pub fn new<S>(app_path: &S, page_path: &S, root_id: &str) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            app_path: PathBuf::from(app_path),
            page_path: PathBuf::from(page_path),
            root_id: root_id.to_string(),
            template_vars: HashMap::new(),
            strict_mode: true,
        }
    }

    /// Builder pattern for configuration
    pub fn builder() -> HydrationConfigBuilder {
        HydrationConfigBuilder::default()
    }

    /// Adds a template variable
    pub fn with_var<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.template_vars.insert(key.into(), value.into());
        self
    }

    /// Sets strict mode
    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    /// Validates the configuration
    pub fn validate(&self) -> Result<()> {
        if !self.app_path.exists() {
            return Err(anyhow::anyhow!(
                "App component not found: {}",
                self.app_path.display()
            ));
        }

        if !self.page_path.exists() {
            return Err(anyhow::anyhow!(
                "Page component not found: {}",
                self.page_path.display()
            ));
        }

        if self.root_id.is_empty() {
            return Err(anyhow::anyhow!("Root ID cannot be empty"));
        }

        Ok(())
    }
}

/// Builder for HydrationConfig
#[derive(Default)]
pub struct HydrationConfigBuilder {
    app_path: Option<PathBuf>,
    page_path: Option<PathBuf>,
    root_id: Option<String>,
    template_vars: HashMap<String, String>,
    strict_mode: bool,
}

impl HydrationConfigBuilder {
    pub fn app_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.app_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn page_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.page_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn root_id<S: Into<String>>(mut self, id: S) -> Self {
        self.root_id = Some(id.into());
        self
    }

    pub fn template_var<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.template_vars.insert(key.into(), value.into());
        self
    }

    pub fn strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    pub fn build(self) -> Result<HydrationConfig> {
        let app_path = self.app_path.ok_or_else(|| anyhow::anyhow!("App path is required"))?;
        let page_path = self.page_path.ok_or_else(|| anyhow::anyhow!("Page path is required"))?;
        let root_id = self.root_id.unwrap_or_else(|| "root".to_string());

        let config = HydrationConfig {
            app_path,
            page_path,
            root_id,
            template_vars: self.template_vars,
            strict_mode: self.strict_mode,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Generates hydration scripts for client-side rendering
#[derive(Debug)]
pub struct Hydrator {
    config: HydrationConfig,
}

impl Hydrator {
    /// Creates a new hydrator with the given configuration
    pub fn new(config: HydrationConfig) -> Self {
        Self { config }
    }

    /// Creates a hydrator with simple parameters (legacy compatibility)
    pub fn simple<S>(app_path: &S, page_path: &S, root_id: &str) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let config = HydrationConfig::new(app_path, page_path, root_id);
        config.validate()?;
        Ok(Self::new(config))
    }

    /// Generates the hydration script content
    fn generate_content(&self) -> Result<String> {
        let app_path_canonical = self.config.app_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize app path: {}", self.config.app_path.display()))?;

        let page_path_canonical = self.config.page_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize page path: {}", self.config.page_path.display()))?;

        let app_path_str = app_path_canonical
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("App path contains invalid UTF-8"))?;

        let page_path_str = page_path_canonical
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Page path contains invalid UTF-8"))?;

        let mut content = HYDRATED_FILE_TEMPLATE
            .replace(APP_PATH_TAG, app_path_str)
            .replace(PAGE_PATH_TAG, page_path_str)
            .replace(ROOT_ID_TAG, &self.config.root_id);

        // Apply additional template variables
        for (key, value) in &self.config.template_vars {
            let placeholder = format!("%{}%", key.to_uppercase());
            content = content.replace(&placeholder, value);
        }

        // Handle strict mode
        if !self.config.strict_mode {
            content = content.replace("React.StrictMode", "React.Fragment");
        }

        Ok(content)
    }

    /// Gets the configuration
    pub fn config(&self) -> &HydrationConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn with_config(mut self, config: HydrationConfig) -> Result<Self> {
        config.validate()?;
        self.config = config;
        Ok(self)
    }
}

impl Generate for Hydrator {
    type Output = String;
    
    fn generate(&self) -> Result<Self::Output> {
        self.generate_content()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_files() -> (PathBuf, PathBuf) {
        let temp_dir = env::temp_dir().join("metassr_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let app_path = temp_dir.join("_app.tsx");
        let page_path = temp_dir.join("home.jsx");
        
        // Create test files
        std::fs::write(&app_path, "export default function App() { return null; }").unwrap();
        std::fs::write(&page_path, "export default function Page() { return null; }").unwrap();
        
        (app_path, page_path)
    }

    #[test]
    fn test_hydration_config_creation() {
        let (app_path, page_path) = create_test_files();
        
        let config = HydrationConfig::new(&app_path, &page_path, "root");
        assert_eq!(config.root_id, "root");
        assert!(config.strict_mode);
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_hydration_config_builder() {
        let (app_path, page_path) = create_test_files();
        
        let config = HydrationConfig::builder()
            .app_path(&app_path)
            .page_path(&page_path)
            .root_id("app")
            .template_var("CUSTOM_VAR", "custom_value")
            .strict_mode(false)
            .build()
            .unwrap();

        assert_eq!(config.root_id, "app");
        assert!(!config.strict_mode);
        assert_eq!(config.template_vars.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_hydrator_simple_creation() {
        let (app_path, page_path) = create_test_files();
        
        let hydrator = Hydrator::simple(&app_path, &page_path, "root").unwrap();
        assert_eq!(hydrator.config().root_id, "root");
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_generate_hydrated_file() {
        let (app_path, page_path) = create_test_files();
        
        let hydrator = Hydrator::simple(&app_path, &page_path, "root").unwrap();
        let content = hydrator.generate().unwrap();
        
        assert!(content.contains("hydrateRoot"));
        assert!(content.contains("document.getElementById(\"root\")"));
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_template_variables() {
        let (app_path, page_path) = create_test_files();
        
        let config = HydrationConfig::new(&app_path, &page_path, "root")
            .with_var("CUSTOM_VAR", "test_value");
        
        let hydrator = Hydrator::new(config);
        
        // Verify the config has the variable
        assert_eq!(hydrator.config().template_vars.get("CUSTOM_VAR"), Some(&"test_value".to_string()));
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }
}
