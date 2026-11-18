//! Turning "this project needs py" into a concrete set of store entries.

use std::{collections::BTreeMap, fmt, str::FromStr};

use crate::{
    index::{Artifact, ArtifactKind, Index},
    store::Installed,
    target::{DottedVersion, Target},
};

/// What a project needs. Builtin loaders are listed like any other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Requirements {
    /// A core version, or `None` for the installed (else newest indexed) one.
    pub core: Option<String>,
    /// Loader tag to runtime version. `None` means "any": an installed build
    /// if there is one, otherwise the index default.
    pub loaders: BTreeMap<String, Option<String>>,
}

impl Requirements {
    pub fn with(mut self, spec: LoaderSpec) -> Self {
        self.loaders.insert(spec.tag, spec.version);
        self
    }
}

/// A user-facing loader request: `python`, `py@3.13`, `node`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoaderSpec {
    /// The MetaCall loader tag (`py`, `node`, `ts`, ...).
    pub tag: String,
    pub version: Option<String>,
}

/// Common names for each MetaCall loader tag.
const ALIASES: &[(&str, &[&str])] = &[
    ("py", &["python", "python3"]),
    ("node", &["nodejs", "js", "javascript"]),
    ("ts", &["typescript"]),
    ("rb", &["ruby"]),
    ("rs", &["rust"]),
    ("cs", &["csharp", "dotnet"]),
    ("java", &[]),
    ("go", &["golang"]),
    ("wasm", &["webassembly"]),
];

impl LoaderSpec {
    /// The canonical tag for a name or alias, if known.
    pub fn tag_for(name: &str) -> Option<&'static str> {
        let name = name.to_ascii_lowercase();
        ALIASES
            .iter()
            .find(|(tag, aliases)| *tag == name || aliases.contains(&name.as_str()))
            .map(|(tag, _)| *tag)
    }
}

impl FromStr for LoaderSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, version) = match s.split_once('@') {
            Some((n, v)) if !v.is_empty() && v != "*" => (n, Some(v.to_string())),
            Some((n, _)) => (n, None),
            None => (s, None),
        };
        let tag = LoaderSpec::tag_for(name).ok_or_else(|| format!("unknown loader `{name}`"))?;
        Ok(LoaderSpec {
            tag: tag.to_string(),
            version,
        })
    }
}

impl fmt::Display for LoaderSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version {
            Some(v) => write!(f, "{}@{v}", self.tag),
            None => f.write_str(&self.tag),
        }
    }
}

/// A component the resolution needs but the store does not have.
#[derive(Debug, Clone, PartialEq)]
pub struct Missing {
    /// The loader tag, or `None` for core itself.
    pub loader: Option<String>,
    /// The version that was asked for or picked from the index, if known.
    pub version: Option<String>,
    /// Where to get it. `None` when the index has no build for this host (or
    /// no index was given).
    pub artifact: Option<Artifact>,
}

impl fmt::Display for Missing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.loader.as_deref().unwrap_or("core");
        match &self.version {
            Some(v) => write!(f, "{name} {v}"),
            None => f.write_str(name),
        }
    }
}

/// The outcome of [`resolve`].
#[derive(Debug, Clone)]
pub struct Resolution {
    /// The core version every component is built against, once known.
    pub core: Option<String>,
    /// Installed components: core first, then loaders by tag.
    pub ready: Vec<Installed>,
    pub missing: Vec<Missing>,
}

impl Resolution {
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty() && self.core.is_some()
    }
}

/// Picks one store entry per requirement, all built against the same core.
///
/// The index is optional: without it, only installed components can be
/// chosen and anything else is reported missing with no artifact. With it,
/// loader `requires` (ts needs node) are followed and unpinned loaders that
/// are not installed resolve to the index default.
pub fn resolve(
    req: &Requirements,
    installed: &[Installed],
    index: Option<&Index>,
    target: Target,
) -> Resolution {
    let core = req
        .core
        .clone()
        .or_else(|| best_installed(installed, ArtifactKind::Core, None, None))
        .or_else(|| index.and_then(|i| i.latest_core()).map(str::to_string));
    let Some(core) = core else {
        return Resolution {
            core: None,
            ready: vec![],
            missing: vec![Missing {
                loader: None,
                version: None,
                artifact: None,
            }],
        };
    };

    let mut ready = vec![];
    let mut missing = vec![];

    match find(installed, ArtifactKind::Core, &core, None, None) {
        Some(c) => ready.push(c.clone()),
        None => missing.push(Missing {
            loader: None,
            version: Some(core.clone()),
            artifact: index.and_then(|i| i.core_artifact(&core, target)).cloned(),
        }),
    }

    for (tag, version) in with_requires(&req.loaders, index, &core) {
        let version = version
            .or_else(|| best_installed(installed, ArtifactKind::Loader, Some(&core), Some(&tag)))
            .or_else(|| {
                index
                    .and_then(|i| i.loader(&core, &tag))
                    .map(|l| l.default.clone())
            });
        let found = version
            .as_deref()
            .and_then(|v| find(installed, ArtifactKind::Loader, &core, Some(&tag), Some(v)));
        match found {
            Some(l) => ready.push(l.clone()),
            None => missing.push(Missing {
                artifact: index.and_then(|i| {
                    i.loader_artifact(&core, &tag, version.as_deref(), target)
                        .cloned()
                }),
                loader: Some(tag),
                version,
            }),
        }
    }

    Resolution {
        core: Some(core),
        ready,
        missing,
    }
}

/// The requested loaders plus everything they require, per the index.
fn with_requires(
    loaders: &BTreeMap<String, Option<String>>,
    index: Option<&Index>,
    core: &str,
) -> BTreeMap<String, Option<String>> {
    let mut all = loaders.clone();
    let mut queue: Vec<String> = loaders.keys().cloned().collect();
    while let Some(tag) = queue.pop() {
        let requires = index
            .and_then(|i| i.loader(core, &tag))
            .map(|l| l.requires.clone())
            .unwrap_or_default();
        for dep in requires {
            if !all.contains_key(&dep) {
                all.insert(dep.clone(), None);
                queue.push(dep);
            }
        }
    }
    all
}

fn matches(
    i: &Installed,
    kind: ArtifactKind,
    core: Option<&str>,
    tag: Option<&str>,
    version: Option<&str>,
) -> bool {
    let m = &i.manifest;
    m.kind == kind
        && core.is_none_or(|c| m.core == c)
        && tag.is_none_or(|t| m.loader.as_deref() == Some(t))
        && version.is_none_or(|v| m.runtime_version.as_deref() == Some(v))
}

fn find<'a>(
    installed: &'a [Installed],
    kind: ArtifactKind,
    core: &str,
    tag: Option<&str>,
    version: Option<&str>,
) -> Option<&'a Installed> {
    installed
        .iter()
        .filter(|i| matches(i, kind, Some(core), tag, version))
        // Prefer the host package's copy when both exist.
        .max_by_key(|i| i.external)
}

/// The version to use when none was asked for: the host package's build if
/// installed, else the highest installed version.
fn best_installed(
    installed: &[Installed],
    kind: ArtifactKind,
    core: Option<&str>,
    tag: Option<&str>,
) -> Option<String> {
    let version_of = |i: &Installed| match kind {
        ArtifactKind::Core => Some(i.manifest.core.clone()),
        _ => i.manifest.runtime_version.clone(),
    };
    let rank = |v: &str| v.parse::<DottedVersion>().ok();
    installed
        .iter()
        .filter(|i| matches(i, kind, core, tag, None))
        .filter_map(|i| version_of(i).map(|v| (i.external, rank(&v), v)))
        .max_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)))
        .map(|(_, _, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use std::path::PathBuf;

    fn inst(
        kind: ArtifactKind,
        core: &str,
        tag: Option<&str>,
        ver: Option<&str>,
        ext: bool,
    ) -> Installed {
        let id = match tag {
            Some(t) => format!("{t}_loader-{}-core{core}", ver.unwrap_or("x")),
            None => format!("core-{core}"),
        };
        Installed {
            dir: PathBuf::from(format!("/store/{id}")),
            manifest: Manifest {
                schema: 1,
                id,
                kind,
                core: core.into(),
                loader: tag.map(Into::into),
                runtime_version: ver.map(Into::into),
                lib: vec![],
                config: None,
            },
            external: ext,
        }
    }

    fn core(v: &str, ext: bool) -> Installed {
        inst(ArtifactKind::Core, v, None, None, ext)
    }

    fn loader(core: &str, tag: &str, v: &str, ext: bool) -> Installed {
        inst(ArtifactKind::Loader, core, Some(tag), Some(v), ext)
    }

    fn index() -> Index {
        let sha = "0".repeat(64);
        let art = |id: &str, kind: &str, loader: Option<&str>, v: Option<&str>| {
            serde_json::json!({ "id": id, "kind": kind, "loader": loader, "runtime_version": v,
                "core": "0.9.24", "target": "linux-x64-gnu", "url": "https://x.invalid", "sha256": sha, "size": 1 })
        };
        let json = serde_json::json!({
            "schema": 1, "generated": "now",
            "cores": { "0.9.24": { "commit": "c", "loaders": {
                "py": { "versions": ["3.14", "3.13"], "default": "3.14" },
                "node": { "versions": ["24.20"], "default": "24.20", "builtin": true },
                "ts": { "versions": ["5.x"], "default": "5.x", "builtin": true, "requires": ["node"] }
            }}},
            "artifacts": [
                art("core-0.9.24-linux-x64-gnu", "core", None, None),
                art("py_loader-3.14-core0.9.24-linux-x64-gnu", "loader", Some("py"), Some("3.14")),
                art("py_loader-3.13-core0.9.24-linux-x64-gnu", "loader", Some("py"), Some("3.13")),
            ]
        });
        Index::from_trusted_bytes(json.to_string().as_bytes()).unwrap()
    }

    fn req(specs: &[&str]) -> Requirements {
        specs
            .iter()
            .fold(Requirements::default(), |r, s| r.with(s.parse().unwrap()))
    }

    fn ids(res: &Resolution) -> Vec<&str> {
        res.ready.iter().map(|i| i.id()).collect()
    }

    #[test]
    fn parses_specs_and_aliases() {
        let s: LoaderSpec = "python@3.13".parse().unwrap();
        assert_eq!((s.tag.as_str(), s.version.as_deref()), ("py", Some("3.13")));
        let s: LoaderSpec = "TypeScript".parse().unwrap();
        assert_eq!((s.tag.as_str(), s.version), ("ts", None));
        let s: LoaderSpec = "py@*".parse().unwrap();
        assert_eq!(s.version, None);
        assert!("cobol".parse::<LoaderSpec>().is_err());
    }

    #[test]
    fn builtins_resolve_offline() {
        let installed = [
            core("0.9.24", true),
            loader("0.9.24", "node", "24.20", true),
        ];
        let res = resolve(&req(&["node"]), &installed, None, Target::LinuxX64Gnu);
        assert!(res.is_complete());
        assert_eq!(ids(&res), ["core-0.9.24", "node_loader-24.20-core0.9.24"]);
    }

    #[test]
    fn missing_loader_points_at_the_index_default() {
        let installed = [
            core("0.9.24", true),
            loader("0.9.24", "node", "24.20", true),
        ];
        let res = resolve(
            &req(&["node", "python"]),
            &installed,
            Some(&index()),
            Target::LinuxX64Gnu,
        );
        assert!(!res.is_complete());
        assert_eq!(res.missing.len(), 1);
        let m = &res.missing[0];
        assert_eq!(
            (m.loader.as_deref(), m.version.as_deref()),
            (Some("py"), Some("3.14"))
        );
        assert_eq!(
            m.artifact.as_ref().unwrap().id,
            "py_loader-3.14-core0.9.24-linux-x64-gnu"
        );
    }

    #[test]
    fn unpinned_prefers_an_installed_version_over_the_default() {
        let installed = [core("0.9.24", true), loader("0.9.24", "py", "3.13", false)];
        let res = resolve(
            &req(&["py"]),
            &installed,
            Some(&index()),
            Target::LinuxX64Gnu,
        );
        assert!(res.is_complete());
        assert_eq!(ids(&res)[1], "py_loader-3.13-core0.9.24");
    }

    #[test]
    fn pins_are_exact() {
        let installed = [core("0.9.24", true), loader("0.9.24", "py", "3.13", false)];
        let res = resolve(
            &req(&["py@3.14"]),
            &installed,
            Some(&index()),
            Target::LinuxX64Gnu,
        );
        assert_eq!(res.missing[0].version.as_deref(), Some("3.14"));
        let res = resolve(
            &req(&["py@3.12"]),
            &installed,
            Some(&index()),
            Target::LinuxX64Gnu,
        );
        assert!(res.missing[0].artifact.is_none(), "index has no 3.12");
    }

    #[test]
    fn loaders_for_another_core_are_never_mixed_in() {
        let installed = [core("0.9.24", true), loader("0.9.23", "py", "3.14", false)];
        let res = resolve(&req(&["py"]), &installed, None, Target::LinuxX64Gnu);
        assert_eq!(ids(&res), ["core-0.9.24"]);
        assert_eq!(res.missing[0].loader.as_deref(), Some("py"));
    }

    #[test]
    fn follows_requires_from_the_index() {
        let installed = [
            core("0.9.24", true),
            loader("0.9.24", "node", "24.20", true),
            loader("0.9.24", "ts", "5.x", true),
        ];
        let res = resolve(
            &req(&["ts"]),
            &installed,
            Some(&index()),
            Target::LinuxX64Gnu,
        );
        assert!(res.is_complete());
        assert_eq!(
            ids(&res)[1..],
            ["node_loader-24.20-core0.9.24", "ts_loader-5.x-core0.9.24"]
        );
    }

    #[test]
    fn prefers_the_host_core_and_falls_back_to_the_index() {
        let installed = [core("0.9.30", false), core("0.9.24", true)];
        let res = resolve(&req(&[]), &installed, None, Target::LinuxX64Gnu);
        assert_eq!(res.core.as_deref(), Some("0.9.24"));

        let res = resolve(&req(&[]), &[], Some(&index()), Target::LinuxX64Gnu);
        assert_eq!(res.core.as_deref(), Some("0.9.24"));
        assert_eq!(
            res.missing[0].artifact.as_ref().unwrap().id,
            "core-0.9.24-linux-x64-gnu"
        );

        let res = resolve(&req(&[]), &[], None, Target::LinuxX64Gnu);
        assert!(res.core.is_none() && !res.is_complete());
    }
}
