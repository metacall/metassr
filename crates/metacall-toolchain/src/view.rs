//! Views: the single loader directory and config tree MetaCall runs against.
//!
//! ```text
//! views/<id>/
//! ├── lib/                     # symlinks to every component's `lib` entries
//! ├── configurations/
//! │   ├── global.json          # { "py_loader": "py_loader.json", ... }
//! │   └── py_loader.json       # rendered from the artifact's manifest
//! └── view.json                # which store entries this view was built from
//! ```
//!
//! The id is a hash of the component ids and their directories, so every
//! project that resolves to the same set shares one view, and a host package
//! that moves gets a new one.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    error::{io_err, Error, Result},
    index::ArtifactKind,
    store::Installed,
};

const VIEW_SCHEMA: u32 = 1;
const VIEW_FILE: &str = "view.json";
const LIB_DIR: &str = "lib";
const CONFIG_DIR: &str = "configurations";
const GLOBAL_CONFIG: &str = "global.json";

/// A built view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub id: String,
    pub dir: PathBuf,
    /// Variables the loader configs set through their `environment` field.
    pub config_env: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ViewRecord {
    schema: u32,
    id: String,
    components: Vec<ViewComponent>,
    config_env: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ViewComponent {
    id: String,
    dir: PathBuf,
}

impl View {
    pub fn lib_dir(&self) -> PathBuf {
        self.dir.join(LIB_DIR)
    }

    pub fn global_config(&self) -> PathBuf {
        self.dir.join(CONFIG_DIR).join(GLOBAL_CONFIG)
    }

    /// The variables MetaCall core reads at `initialize()`. Set them before
    /// that call, and before any other thread starts.
    pub fn env(&self) -> Vec<(&'static str, OsString)> {
        let lib = self.lib_dir().into_os_string();
        vec![
            ("LOADER_LIBRARY_PATH", lib.clone()),
            ("SERIAL_LIBRARY_PATH", lib.clone()),
            ("DETOUR_LIBRARY_PATH", lib),
            ("CONFIGURATION_PATH", self.global_config().into_os_string()),
        ]
    }

    /// The config `environment` variables already set in this process. Core
    /// only applies a config value when the variable is unset, so these would
    /// silently override the view (a stray `PYTHONHOME`, for example).
    pub fn shadowed_env(&self) -> Vec<&str> {
        self.config_env
            .iter()
            .filter(|name| std::env::var_os(name).is_some())
            .map(String::as_str)
            .collect()
    }
}

/// The view id for a set of components.
pub fn view_id(components: &[Installed]) -> String {
    let mut parts: Vec<(&str, &Path)> = components
        .iter()
        .map(|c| (c.id(), c.dir.as_path()))
        .collect();
    parts.sort();
    let mut hasher = Sha256::new();
    hasher.update(format!("metacall-toolchain view v{VIEW_SCHEMA}\n"));
    for (id, dir) in parts {
        hasher.update(id.as_bytes());
        hasher.update([0]);
        hasher.update(dir.as_os_str().as_encoded_bytes());
        hasher.update([b'\n']);
    }
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Returns the view for `components` under `views_root`, building it if it
/// does not exist or one of its store entries is gone.
pub fn build(views_root: &Path, components: &[Installed]) -> Result<View> {
    let id = view_id(components);
    let dest = views_root.join(&id);
    if let Some(view) = existing(&dest) {
        return Ok(view);
    }

    fs::create_dir_all(views_root).map_err(io_err("create", views_root))?;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let tmp = views_root.join(format!(
        ".tmp-{id}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let result = write_view(&tmp, &id, components);
    if let Err(e) = result {
        let _ = fs::remove_dir_all(&tmp);
        return Err(e);
    }

    if dest.exists() {
        // Stale (a component vanished): replace it.
        let _ = fs::remove_dir_all(&dest);
    }
    if fs::rename(&tmp, &dest).is_err() {
        // Another process built the same view first.
        let _ = fs::remove_dir_all(&tmp);
    }
    existing(&dest).ok_or_else(|| Error::Io {
        op: "build view",
        path: dest,
        source: std::io::Error::other("view is incomplete after building"),
    })
}

/// A complete view at `dir` whose store entries all still exist.
fn existing(dir: &Path) -> Option<View> {
    let record: ViewRecord = serde_json::from_slice(&fs::read(dir.join(VIEW_FILE)).ok()?).ok()?;
    let valid = record.schema == VIEW_SCHEMA && record.components.iter().all(|c| c.dir.is_dir());
    valid.then(|| View {
        id: record.id,
        dir: dir.to_path_buf(),
        config_env: record.config_env,
    })
}

fn write_view(dir: &Path, id: &str, components: &[Installed]) -> Result<()> {
    let lib = dir.join(LIB_DIR);
    let configs = dir.join(CONFIG_DIR);
    fs::create_dir_all(&lib).map_err(io_err("create", &lib))?;
    fs::create_dir_all(&configs).map_err(io_err("create", &configs))?;

    let mut owners: BTreeMap<OsString, &str> = BTreeMap::new();
    let mut global = serde_json::Map::new();
    let mut config_env = vec![];

    for c in components {
        for entry in &c.manifest.lib {
            let src = c.dir.join(entry);
            let name = Path::new(entry)
                .file_name()
                .expect("manifest lib entries are validated as relative paths")
                .to_os_string();
            if let Some(first) = owners.insert(name.clone(), c.id()) {
                return Err(Error::ViewConflict {
                    name: name.to_string_lossy().into_owned(),
                    first: first.to_string(),
                    second: c.id().to_string(),
                });
            }
            let dst = lib.join(&name);
            link(&src, &dst).map_err(io_err("link", &dst))?;
        }

        let (ArtifactKind::Loader, Some(tag)) = (c.manifest.kind, &c.manifest.loader) else {
            continue;
        };
        let Some(config) = c.manifest.render_config(&c.dir) else {
            continue;
        };
        if let Some(vars) = config.get("environment").and_then(|e| e.as_array()) {
            config_env.extend(
                vars.iter()
                    .filter_map(|v| v.get("name")?.as_str())
                    .map(str::to_string),
            );
        }
        let file = format!("{tag}_loader.json");
        write_json(&configs.join(&file), &config)?;
        global.insert(format!("{tag}_loader"), file.into());
    }
    global.insert("log_level".into(), "Error".into());
    write_json(&configs.join(GLOBAL_CONFIG), &global)?;

    config_env.sort();
    config_env.dedup();
    let record = ViewRecord {
        schema: VIEW_SCHEMA,
        id: id.to_string(),
        components: components
            .iter()
            .map(|c| ViewComponent {
                id: c.id().to_string(),
                dir: c.dir.clone(),
            })
            .collect(),
        config_env,
    };
    write_json(&dir.join(VIEW_FILE), &record)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).expect("view types always serialize");
    fs::write(path, bytes).map_err(io_err("write", path))
}

#[cfg(unix)]
fn link(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

/// Symlinks need developer mode on Windows; fall back to hard links for
/// files and copies for directories.
#[cfg(windows)]
fn link(src: &Path, dst: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    if src.is_dir() {
        symlink_dir(src, dst).or_else(|_| copy_dir(src, dst))
    } else {
        symlink_file(src, dst)
            .or_else(|_| fs::hard_link(src, dst))
            .or_else(|_| fs::copy(src, dst).map(|_| ()))
    }
}

#[cfg(windows)]
fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use serde_json::json;

    fn component(
        root: &Path,
        id: &str,
        kind: ArtifactKind,
        tag: Option<&str>,
        files: &[&str],
        config: Option<serde_json::Value>,
    ) -> Installed {
        let dir = root.join(id);
        for f in files {
            let p = dir.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, b"x").unwrap();
        }
        Installed {
            dir,
            manifest: Manifest {
                schema: 1,
                id: id.into(),
                kind,
                core: "0.9.24".into(),
                loader: tag.map(Into::into),
                runtime_version: tag.map(|_| "1".into()),
                lib: files.iter().map(|s| s.to_string()).collect(),
                config,
            },
            external: false,
        }
    }

    fn fixture(root: &Path) -> Vec<Installed> {
        vec![
            component(
                root,
                "core",
                ArtifactKind::Core,
                None,
                &[
                    "lib/libmetacall.so",
                    "lib/libext_loader.so",
                    "lib/plugins/x.json",
                ],
                None,
            ),
            component(
                root,
                "py",
                ArtifactKind::Loader,
                Some("py"),
                &["lib/libpy_loader.so"],
                Some(json!({
                    "dependencies": { "python": ["${ARTIFACT_DIR}/runtime/libpython.so"] },
                    "environment": [{ "name": "PYTHONHOME", "value": "${ARTIFACT_DIR}/runtime" }]
                })),
            ),
            component(
                root,
                "ts",
                ArtifactKind::Loader,
                Some("ts"),
                &["lib/libts_loader.so"],
                None,
            ),
        ]
    }

    #[test]
    fn builds_links_and_configs() {
        let tmp = tempfile::tempdir().unwrap();
        let components = fixture(&tmp.path().join("store"));
        let view = build(&tmp.path().join("views"), &components).unwrap();

        let lib = view.lib_dir();
        let store = tmp.path().join("store");
        assert_eq!(
            fs::read_link(lib.join("libmetacall.so")).unwrap(),
            store.join("core/lib/libmetacall.so")
        );
        assert_eq!(
            fs::read_link(lib.join("libpy_loader.so")).unwrap(),
            store.join("py/lib/libpy_loader.so")
        );
        // Only listed entries are linked, so `plugins` here is the one file.
        assert!(lib.join("x.json").exists());

        let global: serde_json::Value =
            serde_json::from_slice(&fs::read(view.global_config()).unwrap()).unwrap();
        assert_eq!(
            global,
            json!({ "py_loader": "py_loader.json", "log_level": "Error" })
        );
        let py: serde_json::Value = serde_json::from_slice(
            &fs::read(view.dir.join("configurations/py_loader.json")).unwrap(),
        )
        .unwrap();
        let home = store.join("py/runtime");
        assert_eq!(py["environment"][0]["value"], json!(home.to_str().unwrap()));
        assert_eq!(view.config_env, ["PYTHONHOME"]);

        let env: BTreeMap<_, _> = view.env().into_iter().collect();
        assert_eq!(env["LOADER_LIBRARY_PATH"], lib.as_os_str());
        assert_eq!(env["CONFIGURATION_PATH"], view.global_config().as_os_str());
    }

    #[test]
    fn same_set_reuses_the_view_and_order_does_not_matter() {
        let tmp = tempfile::tempdir().unwrap();
        let mut components = fixture(&tmp.path().join("store"));
        let views = tmp.path().join("views");
        let a = build(&views, &components).unwrap();
        components.reverse();
        let b = build(&views, &components).unwrap();
        assert_eq!(a, b);

        components.pop();
        let c = build(&views, &components).unwrap();
        assert_ne!(a.id, c.id);
        assert_eq!(fs::read_dir(&views).unwrap().count(), 2, "no tmp dirs left");
    }

    #[test]
    fn rebuilds_when_a_component_disappears() {
        let tmp = tempfile::tempdir().unwrap();
        let components = fixture(&tmp.path().join("store"));
        let views = tmp.path().join("views");
        let view = build(&views, &components).unwrap();

        fs::remove_dir_all(tmp.path().join("store/py")).unwrap();
        assert!(
            existing(&view.dir).is_none(),
            "view is stale once a store entry is gone"
        );

        let components = fixture(&tmp.path().join("store"));
        let rebuilt = build(&views, &components).unwrap();
        assert_eq!(rebuilt.id, view.id);
        assert!(rebuilt.lib_dir().join("libpy_loader.so").exists());
    }

    #[test]
    fn conflicting_lib_names_are_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("store");
        let components = vec![
            component(
                &root,
                "py313",
                ArtifactKind::Loader,
                Some("py"),
                &["lib/libpy_loader.so"],
                None,
            ),
            component(
                &root,
                "py314",
                ArtifactKind::Loader,
                Some("py"),
                &["lib/libpy_loader.so"],
                None,
            ),
        ];
        let err = build(&tmp.path().join("views"), &components).unwrap_err();
        assert!(matches!(err, Error::ViewConflict { .. }), "{err}");
        let leftovers = fs::read_dir(tmp.path().join("views")).unwrap().count();
        assert_eq!(leftovers, 0);
    }
}
