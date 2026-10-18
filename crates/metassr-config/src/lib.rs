use serde::Deserialize;
use std::path::Path;

const KNOWN_KEYS: &[&str] = &[
    "build",
    "server",
    "debug",
    "dev",
    "middleware",
    "images",
    "lint",
    "plugins",
];

const BUILD_KEYS: &[&str] = &["type", "out_dir"];
const SERVER_KEYS: &[&str] = &["port"];
const DEBUG_KEYS: &[&str] = &["mode", "log_file"];
const DEV_KEYS: &[&str] = &["hmr", "server_port", "ws_port"];

fn known_section_keys(section: &str) -> Option<&'static [&'static str]> {
    match section {
        "build" => Some(BUILD_KEYS),
        "server" => Some(SERVER_KEYS),
        "debug" => Some(DEBUG_KEYS),
        "dev" => Some(DEV_KEYS),
        _ => None,
    }
}

fn unknown_keys(table: &toml::Table) -> Vec<String> {
    let mut unknown = Vec::new();

    for (section, value) in table {
        if !KNOWN_KEYS.contains(&section.as_str()) {
            unknown.push(section.clone());
            continue;
        }

        let Some(known_keys) = known_section_keys(section) else {
            continue;
        };
        let Some(section_table) = value.as_table() else {
            continue;
        };

        for key in section_table.keys() {
            if !known_keys.contains(&key.as_str()) {
                unknown.push(format!("{section}.{key}"));
            }
        }
    }

    unknown.sort();
    unknown
}

#[derive(Debug, Deserialize)]
pub struct MetaSSRConfig {
    pub build: Option<BuildConfig>,
    pub server: Option<ServerConfig>,
    pub debug: Option<DebugConfig>,
    pub dev: Option<DevConfig>,
    #[allow(dead_code)]
    pub middleware: Option<toml::Value>,
    #[allow(dead_code)]
    pub images: Option<toml::Value>,
    #[allow(dead_code)]
    pub lint: Option<toml::Value>,
    #[allow(dead_code)]
    pub plugins: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub r#type: Option<String>,
    pub out_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct DebugConfig {
    pub mode: Option<String>,
    pub log_file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DevConfig {
    pub hmr: Option<bool>,
    pub server_port: Option<u16>,
    pub ws_port: Option<u16>,
}

impl MetaSSRConfig {
    pub fn load(root: &Path) -> anyhow::Result<Option<Self>> {
        let config_path = root.join("metassr.toml");
        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)?;

        let table: toml::Table = content.parse()?;
        for key in unknown_keys(&table) {
            println!("warning: metassr.toml: unknown key \"{key}\"");
        }

        let config: MetaSSRConfig = toml::from_str(&content)?;
        Ok(Some(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_valid_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let toml_content = r#"
[build]
type = "ssr"
out_dir = "dist"

[server]
port = 3000

[dev]
hmr = true
ws_port = 3001
"#;
        fs::write(dir.path().join("metassr.toml"), toml_content).unwrap();
        let config = MetaSSRConfig::load(dir.path()).unwrap().unwrap();
        let build = config.build.as_ref().unwrap();
        assert_eq!(build.r#type.as_deref(), Some("ssr"));
        assert_eq!(build.out_dir.as_deref(), Some("dist"));
        assert_eq!(config.server.as_ref().unwrap().port, Some(3000));
        let dev = config.dev.as_ref().unwrap();
        assert_eq!(dev.hmr, Some(true));
        assert_eq!(dev.ws_port, Some(3001));
    }

    #[test]
    fn load_missing_file_returns_none() {
        let dir = tempfile::TempDir::new().unwrap();
        assert!(MetaSSRConfig::load(dir.path()).unwrap().is_none());
    }

    #[test]
    fn load_partial_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let toml_content = r#"
[server]
port = 8080
"#;
        fs::write(dir.path().join("metassr.toml"), toml_content).unwrap();
        let config = MetaSSRConfig::load(dir.path()).unwrap().unwrap();
        assert_eq!(config.server.as_ref().unwrap().port, Some(8080));
        assert!(config.build.is_none());
        assert!(config.debug.is_none());
        assert!(config.dev.is_none());
    }

    #[test]
    fn load_unknown_key_does_not_error() {
        let dir = tempfile::TempDir::new().unwrap();
        let toml_content = r#"
[build]
type = "ssr"

[unknown_section]
foo = "bar"
"#;
        fs::write(dir.path().join("metassr.toml"), toml_content).unwrap();
        let config = MetaSSRConfig::load(dir.path()).unwrap();
        assert!(config.is_some());
    }

    #[test]
    fn reports_unknown_nested_keys_with_their_full_path() {
        let table: toml::Table = r#"
[build]
type = "ssr"
outdir = "output"
"#
        .parse()
        .unwrap();

        assert_eq!(unknown_keys(&table), vec!["build.outdir"]);
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let dir = tempfile::TempDir::new().unwrap();
        fs::write(dir.path().join("metassr.toml"), "[[[invalid").unwrap();
        assert!(MetaSSRConfig::load(dir.path()).is_err());
    }
}
