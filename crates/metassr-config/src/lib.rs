pub struct MetaSSRConfig {
    build: Option<toml::Value>,
    server: Option<toml::Value>,
    port: Option<toml::Value>,
}

impl MetaSSRConfig {
    fn new() -> Self {
        Self {
            build: todo!(),
            server: todo!(),
            port: todo!(),
        }
    }
}

impl Default for MetaSSRConfig {
    fn default() -> Self {
        Self::new()
    }
}
