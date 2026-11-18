//! `manifest.json`: what an unpacked artifact contains and how to wire it.

use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    index::ArtifactKind,
};

/// The manifest schema version this build reads.
pub const MANIFEST_SCHEMA: u32 = 1;

/// Placeholder in a config template that is replaced with the artifact's
/// absolute root directory. Core does not resolve relative `dependencies`.
pub const ARTIFACT_DIR_VAR: &str = "${ARTIFACT_DIR}";

/// Shipped at the root of every artifact as `manifest.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub id: String,
    pub kind: ArtifactKind,
    pub core: String,
    /// The loader tag, for `kind: loader`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    /// Paths relative to the artifact root that must sit in MetaCall's loader
    /// directory. A view links each one under its file name.
    #[serde(default)]
    pub lib: Vec<String>,
    /// The loader config (`<tag>_loader.json`) as a template. String values
    /// may contain [`ARTIFACT_DIR_VAR`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

impl Manifest {
    /// Checks the manifest against the id it was installed under and the
    /// files actually present under `root`.
    pub fn validate(&self, expected_id: &str, root: &Path) -> Result<()> {
        let bad = |reason: String| Error::BadManifest {
            id: expected_id.to_string(),
            reason,
        };
        if self.schema != MANIFEST_SCHEMA {
            return Err(bad(format!(
                "schema {} (this build reads {MANIFEST_SCHEMA})",
                self.schema
            )));
        }
        if self.id != expected_id {
            return Err(bad(format!("manifest is for `{}`", self.id)));
        }
        match self.kind {
            ArtifactKind::Loader if self.loader.is_none() => {
                return Err(bad("loader manifest has no `loader` tag".into()))
            }
            ArtifactKind::Unknown => return Err(bad("unknown artifact kind".into())),
            _ => {}
        }
        for entry in &self.lib {
            if !is_safe_relative(Path::new(entry)) {
                return Err(bad(format!("lib entry `{entry}` is not a relative path")));
            }
            if root.join(entry).symlink_metadata().is_err() {
                return Err(bad(format!("lib entry `{entry}` does not exist")));
            }
        }
        Ok(())
    }

    /// The loader config with [`ARTIFACT_DIR_VAR`] replaced by `root` in
    /// every string value.
    pub fn render_config(&self, root: &Path) -> Option<serde_json::Value> {
        let root = root.to_string_lossy();
        let mut config = self.config.clone()?;
        replace_strings(&mut config, &|s| s.replace(ARTIFACT_DIR_VAR, &root));
        Some(config)
    }
}

fn replace_strings(value: &mut serde_json::Value, f: &impl Fn(&str) -> String) {
    match value {
        serde_json::Value::String(s) => *s = f(s),
        serde_json::Value::Array(items) => items.iter_mut().for_each(|v| replace_strings(v, f)),
        serde_json::Value::Object(map) => map.values_mut().for_each(|v| replace_strings(v, f)),
        _ => {}
    }
}

/// A non-empty relative path made only of normal components (no `..`, no
/// root, no drive prefix).
pub(crate) fn is_safe_relative(path: &Path) -> bool {
    path.components().next().is_some()
        && path
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn renders_artifact_dir_in_nested_strings() {
        let manifest = Manifest {
            schema: 1,
            id: "py".into(),
            kind: ArtifactKind::Loader,
            core: "0.9.24".into(),
            loader: Some("py".into()),
            runtime_version: Some("3.14".into()),
            lib: vec![],
            config: Some(json!({
                "dependencies": { "python": ["${ARTIFACT_DIR}/runtime/lib/libpython3.14.so"] },
                "environment": [{ "name": "PYTHONHOME", "value": "${ARTIFACT_DIR}/runtime" }],
                "log": 3
            })),
        };
        let rendered = manifest.render_config(Path::new("/store/py")).unwrap();
        assert_eq!(
            rendered,
            json!({
                "dependencies": { "python": ["/store/py/runtime/lib/libpython3.14.so"] },
                "environment": [{ "name": "PYTHONHOME", "value": "/store/py/runtime" }],
                "log": 3
            })
        );
    }

    #[test]
    fn safe_relative_paths() {
        assert!(is_safe_relative(Path::new("lib/libpy_loader.so")));
        assert!(!is_safe_relative(Path::new("../x")));
        assert!(!is_safe_relative(Path::new("/x")));
        assert!(!is_safe_relative(Path::new("")));
    }
}
