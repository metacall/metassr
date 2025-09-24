use crate::{
    shared::{APP_PATH_TAG, FUNC_ID_TAG, PAGE_PATH_TAG},
    traits::Generate,
};
use anyhow::{Context, Result};
use metassr_utils::rand::Rand;
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

const RENDER_FILE_TEMPLATE: &str = include_str!("../scripts/render.js.template");

/// Configuration for server-side render script generation
#[derive(Debug, Clone)]
pub struct ServerRenderConfig {
    /// Path to the app component
    pub app_path: PathBuf,
    /// Path to the page component
    pub page_path: PathBuf,
    /// Function ID for MetaCall execution
    pub func_id: Option<i64>,
    /// Additional template variables
    pub template_vars: HashMap<String, String>,
    /// Custom template content
    pub custom_template: Option<String>,
}

impl ServerRenderConfig {
    /// Creates a new server render config
    pub fn new<S>(app_path: &S, page_path: &S) -> Self
    where
        S: AsRef<OsStr> + ?Sized,
    {
        Self {
            app_path: PathBuf::from(app_path),
            page_path: PathBuf::from(page_path),
            func_id: None,
            template_vars: HashMap::new(),
            custom_template: None,
        }
    }

    /// Builder pattern for configuration
    pub fn builder() -> ServerRenderConfigBuilder {
        ServerRenderConfigBuilder::default()
    }

    /// Sets the function ID
    pub fn with_func_id(mut self, func_id: i64) -> Self {
        self.func_id = Some(func_id);
        self
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

    /// Sets a custom template
    pub fn with_template<S: Into<String>>(mut self, template: S) -> Self {
        self.custom_template = Some(template.into());
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

        Ok(())
    }

    /// Gets or generates a function ID
    pub fn get_func_id(&self) -> i64 {
        self.func_id.unwrap_or_else(|| Rand::new().val())
    }
}

/// Builder for ServerRenderConfig
#[derive(Default)]
pub struct ServerRenderConfigBuilder {
    app_path: Option<PathBuf>,
    page_path: Option<PathBuf>,
    func_id: Option<i64>,
    template_vars: HashMap<String, String>,
    custom_template: Option<String>,
}

impl ServerRenderConfigBuilder {
    pub fn app_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.app_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn page_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.page_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn func_id(mut self, func_id: i64) -> Self {
        self.func_id = Some(func_id);
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

    pub fn custom_template<S: Into<String>>(mut self, template: S) -> Self {
        self.custom_template = Some(template.into());
        self
    }

    pub fn build(self) -> Result<ServerRenderConfig> {
        let app_path = self.app_path.ok_or_else(|| anyhow::anyhow!("App path is required"))?;
        let page_path = self.page_path.ok_or_else(|| anyhow::anyhow!("Page path is required"))?;

        let config = ServerRenderConfig {
            app_path,
            page_path,
            func_id: self.func_id,
            template_vars: self.template_vars,
            custom_template: self.custom_template,
        };

        config.validate()?;
        Ok(config)
    }
}

/// Generates server-side render scripts
#[derive(Debug)]
pub struct ServerRender {
    config: ServerRenderConfig,
}

impl ServerRender {
    /// Creates a new server render with the given configuration
    pub fn new(config: ServerRenderConfig) -> Self {
        Self { config }
    }

    /// Creates a server render with simple parameters (legacy compatibility)
    pub fn simple<S>(app_path: &S, page_path: &S) -> Result<Self>
    where
        S: AsRef<OsStr> + ?Sized,
    {
        let config = ServerRenderConfig::new(app_path, page_path);
        config.validate()?;
        Ok(Self::new(config))
    }

    /// Generates the render script content
    fn generate_content(&self) -> Result<(i64, String)> {
        let func_id = self.config.get_func_id();

        let mut app_path = self.config.app_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize app path: {}", self.config.app_path.display()))?;

        let mut page_path = self.config.page_path
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize page path: {}", self.config.page_path.display()))?;

        // Remove extensions for JavaScript imports
        app_path.set_extension("");
        page_path.set_extension("");

        let app_path_str = app_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("App path contains invalid UTF-8"))?;

        let page_path_str = page_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Page path contains invalid UTF-8"))?;

        // Use custom template if provided, otherwise use default
        let template = self.config.custom_template
            .as_deref()
            .unwrap_or(RENDER_FILE_TEMPLATE);

        let mut content = template
            .replace(APP_PATH_TAG, app_path_str)
            .replace(PAGE_PATH_TAG, page_path_str)
            .replace(FUNC_ID_TAG, &func_id.to_string());

        // Apply additional template variables
        for (key, value) in &self.config.template_vars {
            let placeholder = format!("%{}%", key.to_uppercase());
            content = content.replace(&placeholder, value);
        }

        Ok((func_id, content))
    }

    /// Gets the configuration
    pub fn config(&self) -> &ServerRenderConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn with_config(mut self, config: ServerRenderConfig) -> Result<Self> {
        config.validate()?;
        self.config = config;
        Ok(self)
    }
}

impl Generate for ServerRender {
    type Output = (i64, String);
    
    fn generate(&self) -> Result<Self::Output> {
        self.generate_content()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_files() -> (PathBuf, PathBuf) {
        let temp_dir = env::temp_dir().join("metassr_server_render_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let app_path = temp_dir.join("_app.tsx");
        let page_path = temp_dir.join("home.jsx");
        
        // Create test files
        std::fs::write(&app_path, "export default function App() { return null; }").unwrap();
        std::fs::write(&page_path, "export default function Page() { return null; }").unwrap();
        
        (app_path, page_path)
    }

    #[test]
    fn test_server_render_config_creation() {
        let (app_path, page_path) = create_test_files();
        
        let config = ServerRenderConfig::new(&app_path, &page_path);
        assert!(config.func_id.is_none());
        assert!(config.template_vars.is_empty());
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_server_render_simple_creation() {
        let (app_path, page_path) = create_test_files();
        
        let render = ServerRender::simple(&app_path, &page_path).unwrap();
        assert!(render.config().func_id.is_none());
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }

    #[test]
    fn test_generate_render_script() {
        let (app_path, page_path) = create_test_files();
        
        let render = ServerRender::simple(&app_path, &page_path).unwrap();
        let (func_id, content) = render.generate().unwrap();
        
        assert!(func_id > 0);
        assert!(content.contains(&func_id.to_string()));
        
        // Cleanup
        let _ = std::fs::remove_file(&app_path);
        let _ = std::fs::remove_file(&page_path);
    }
}
