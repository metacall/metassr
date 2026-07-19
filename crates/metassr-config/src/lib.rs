use std::path::Path;

pub struct MetaSSRConfig {
    _build: Option<BuildConfig>,
    _server: Option<ServerConfig>,
}

impl MetaSSRConfig {
    pub fn new() -> Self {
        Self {
            _build: Some(BuildConfig {
                _type: Some(String::from("SSSR")),
                _out_dir: Some(String::from("dist")),
            }),
            _server: Some(ServerConfig { _port: 8080 }),
        }
    }

    pub fn load(_root: &Path) -> anyhow::Result<()> {
        // find the config file and load it into memory
        //
        //validate syntax
        Ok(())
    }
}

impl Default for MetaSSRConfig {
    fn default() -> Self {
        Self::new()
    }
}

struct BuildConfig {
    _type: Option<String>,
    _out_dir: Option<String>,
}

struct ServerConfig {
    _port: u16,
}
