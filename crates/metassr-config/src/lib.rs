pub struct MetaSSRConfig {
    build: Option<BuildConfig>,
    server: Option<ServerConfig>,
}

impl MetaSSRConfig {
    fn new() -> Self {
        Self {
            build: Some(BuildConfig {
                _type: Some(String::from("SSSR")),
                out_dir: Some(String::from("dist")),
            }),
            server: Some(ServerConfig { port: 8080 }),
        }
    }
}

impl Default for MetaSSRConfig {
    fn default() -> Self {
        Self::new()
    }
}

struct BuildConfig {
    _type: Option<String>,
    out_dir: Option<String>,
}

struct ServerConfig {
    port: u16,
}
