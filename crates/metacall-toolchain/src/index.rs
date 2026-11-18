//! The signed artifact index: what can be installed, for which core and target.

use std::collections::{BTreeMap, BTreeSet};

use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};

use crate::{
    error::{malformed_index, Error, Result},
    target::{DottedVersion, Target},
};

/// The index schema version this build reads.
pub const INDEX_SCHEMA: u32 = 1;

/// The list of artifacts published for each MetaCall core release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub schema: u32,
    pub generated: String,
    pub cores: BTreeMap<String, CoreRelease>,
    pub artifacts: Vec<Artifact>,
}

/// One MetaCall core release and the loaders built against it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreRelease {
    pub commit: String,
    pub loaders: BTreeMap<String, LoaderRelease>,
}

/// The runtime versions available for one loader tag (`py`, `node`, ...).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderRelease {
    pub versions: Vec<String>,
    pub default: String,
    /// Shipped with the host package instead of downloaded.
    #[serde(default)]
    pub builtin: bool,
    /// Other loader tags this loader needs at runtime (ts needs node).
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub status: Status,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Stable,
    Experimental,
    /// A status added by a newer index. Treated like `experimental`.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactKind {
    Core,
    Loader,
    /// A kind added by a newer index. Never selected by this build.
    #[serde(other)]
    Unknown,
}

/// One downloadable tarball.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub kind: ArtifactKind,
    /// The loader tag, for `kind: loader`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<String>,
    /// The runtime version (`3.14` for Python), for `kind: loader`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_version: Option<String>,
    /// The core version this artifact is (or belongs to).
    pub core: String,
    /// Kept as a string so an index listing newer targets still parses.
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_glibc: Option<String>,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

impl Artifact {
    /// Whether this artifact can run on the current host: right target, and a
    /// new enough glibc when the artifact declares a minimum.
    pub fn runs_on(&self, target: Target, glibc: Option<&DottedVersion>) -> bool {
        if self.target != target.as_str() {
            return false;
        }
        match (&self.min_glibc, glibc) {
            (Some(min), Some(have)) => min.parse::<DottedVersion>().is_ok_and(|min| *have >= min),
            _ => true,
        }
    }
}

/// The minisign public keys an index may be signed with. More than one key
/// allows rotation.
#[derive(Debug, Clone)]
pub struct TrustedKeys(Vec<PublicKey>);

impl TrustedKeys {
    /// Keys in minisign base64 form (the second line of a `.pub` file).
    pub fn from_base64<'a>(keys: impl IntoIterator<Item = &'a str>) -> Result<Self> {
        keys.into_iter()
            .map(|k| {
                PublicKey::from_base64(k.trim()).map_err(|e| Error::MalformedSignature {
                    what: "public key",
                    reason: e.to_string(),
                })
            })
            .collect::<Result<Vec<_>>>()
            .map(TrustedKeys)
    }

    /// Verifies `data` against a minisign signature file's contents.
    pub fn verify(&self, data: &[u8], signature: &str) -> Result<()> {
        let sig = Signature::decode(signature).map_err(|e| Error::MalformedSignature {
            what: "signature",
            reason: e.to_string(),
        })?;
        if self.0.iter().any(|k| k.verify(data, &sig, false).is_ok()) {
            Ok(())
        } else {
            Err(Error::BadSignature)
        }
    }
}

impl Index {
    /// Verifies the signature over the exact bytes, then parses and validates
    /// them. Nothing is parsed before the signature checks out.
    pub fn from_signed(data: &[u8], signature: &str, keys: &TrustedKeys) -> Result<Index> {
        keys.verify(data, signature)?;
        Index::from_trusted_bytes(data)
    }

    /// Parses an index that was already verified (for example, a cached copy
    /// that was checked when it was written).
    pub fn from_trusted_bytes(data: &[u8]) -> Result<Index> {
        #[derive(Deserialize)]
        struct Header {
            schema: u32,
        }
        let header: Header = serde_json::from_slice(data).map_err(malformed_index)?;
        if header.schema != INDEX_SCHEMA {
            return Err(Error::UnsupportedSchema {
                found: header.schema,
                supported: INDEX_SCHEMA,
            });
        }
        let index: Index = serde_json::from_slice(data).map_err(malformed_index)?;
        index.validate()?;
        Ok(index)
    }

    fn validate(&self) -> Result<()> {
        let mut ids = BTreeSet::new();
        for a in &self.artifacts {
            validate_id(&a.id)?;
            if !ids.insert(a.id.as_str()) {
                return Err(malformed_index(format!("duplicate artifact id `{}`", a.id)));
            }
            if a.sha256.len() != 64
                || !a
                    .sha256
                    .bytes()
                    .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
            {
                return Err(malformed_index(format!(
                    "artifact `{}` has a bad sha256 (want 64 lowercase hex chars)",
                    a.id
                )));
            }
            if a.kind == ArtifactKind::Loader && (a.loader.is_none() || a.runtime_version.is_none())
            {
                return Err(malformed_index(format!(
                    "loader artifact `{}` needs `loader` and `runtime_version`",
                    a.id
                )));
            }
        }
        for (core, release) in &self.cores {
            for (tag, loader) in &release.loaders {
                if !loader.versions.contains(&loader.default) {
                    return Err(malformed_index(format!(
                        "core {core}: default `{}` for `{tag}` is not in its versions",
                        loader.default
                    )));
                }
            }
        }
        Ok(())
    }

    /// The newest core version listed.
    pub fn latest_core(&self) -> Option<&str> {
        self.cores
            .keys()
            .filter_map(|v| v.parse::<DottedVersion>().ok().map(|d| (d, v.as_str())))
            .max_by(|a, b| a.0.cmp(&b.0))
            .map(|(_, v)| v)
    }

    pub fn loader(&self, core: &str, tag: &str) -> Option<&LoaderRelease> {
        self.cores.get(core)?.loaders.get(tag)
    }

    /// The core artifact for `core` on `target`.
    pub fn core_artifact(&self, core: &str, target: Target) -> Option<&Artifact> {
        self.artifacts
            .iter()
            .find(|a| a.kind == ArtifactKind::Core && a.core == core && a.target == target.as_str())
    }

    /// The loader artifact for `tag` at `version` (or the index default when
    /// `None`), built against `core`, for `target`.
    pub fn loader_artifact(
        &self,
        core: &str,
        tag: &str,
        version: Option<&str>,
        target: Target,
    ) -> Option<&Artifact> {
        let version = match version {
            Some(v) => v,
            None => self.loader(core, tag)?.default.as_str(),
        };
        self.artifacts.iter().find(|a| {
            a.kind == ArtifactKind::Loader
                && a.core == core
                && a.target == target.as_str()
                && a.loader.as_deref() == Some(tag)
                && a.runtime_version.as_deref() == Some(version)
        })
    }
}

/// Artifact ids become directory names in the store, so they must be a single
/// safe path component.
pub(crate) fn validate_id(id: &str) -> Result<()> {
    let ok = !id.is_empty()
        && !id.starts_with('.')
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'));
    if ok {
        Ok(())
    } else {
        Err(malformed_index(format!(
            "artifact id `{id}` is not a safe file name"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const SHA: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    fn fixture() -> String {
        serde_json::json!({
            "schema": 1,
            "generated": "2026-11-17T00:00:00Z",
            "cores": {
                "0.9.24": { "commit": "a7011dc2", "loaders": {
                    "py": { "versions": ["3.14", "3.13"], "default": "3.14" },
                    "node": { "versions": ["24.20"], "default": "24.20", "builtin": true },
                    "rb": { "versions": ["3.4"], "default": "3.4", "status": "preview" }
                }},
                "0.9.3": { "commit": "old", "loaders": {} }
            },
            "artifacts": [
                { "id": "core-0.9.24-linux-x64-gnu", "kind": "core", "core": "0.9.24",
                  "target": "linux-x64-gnu", "url": "https://example.invalid/core", "sha256": SHA, "size": 1 },
                { "id": "py_loader-3.14-core0.9.24-linux-x64-gnu", "kind": "loader", "loader": "py",
                  "runtime_version": "3.14", "core": "0.9.24", "target": "linux-x64-gnu",
                  "min_glibc": "2.28", "url": "https://example.invalid/py314", "sha256": SHA, "size": 1 },
                { "id": "py_loader-3.13-core0.9.24-linux-x64-gnu", "kind": "loader", "loader": "py",
                  "runtime_version": "3.13", "core": "0.9.24", "target": "linux-x64-gnu",
                  "url": "https://example.invalid/py313", "sha256": SHA, "size": 1 },
                { "id": "py_loader-3.14-core0.9.24-sunos-sparc", "kind": "loader", "loader": "py",
                  "runtime_version": "3.14", "core": "0.9.24", "target": "sunos-sparc",
                  "url": "https://example.invalid/sparc", "sha256": SHA, "size": 1 },
                { "id": "wasm-thing", "kind": "plugin", "core": "0.9.24", "target": "linux-x64-gnu",
                  "url": "https://example.invalid/w", "sha256": SHA, "size": 1, "added_later": true }
            ]
        })
        .to_string()
    }

    fn keypair() -> (minisign::KeyPair, TrustedKeys) {
        let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let keys = TrustedKeys::from_base64([kp.pk.to_base64().as_str()]).unwrap();
        (kp, keys)
    }

    fn sign(kp: &minisign::KeyPair, data: &[u8]) -> String {
        minisign::sign(Some(&kp.pk), &kp.sk, Cursor::new(data), None, None)
            .unwrap()
            .to_string()
    }

    #[test]
    fn verifies_and_parses_a_signed_index() {
        let (kp, keys) = keypair();
        let data = fixture();
        let index =
            Index::from_signed(data.as_bytes(), &sign(&kp, data.as_bytes()), &keys).unwrap();
        assert_eq!(index.artifacts.len(), 5);
        assert_eq!(index.latest_core(), Some("0.9.24"));
        assert_eq!(
            index.loader("0.9.24", "rb").unwrap().status,
            Status::Unknown
        );
        assert_eq!(index.artifacts[4].kind, ArtifactKind::Unknown);
    }

    #[test]
    fn rejects_tampered_data() {
        let (kp, keys) = keypair();
        let data = fixture();
        let sig = sign(&kp, data.as_bytes());
        let tampered = data.replace("example.invalid/py314", "evil.invalid/py314");
        assert!(matches!(
            Index::from_signed(tampered.as_bytes(), &sig, &keys),
            Err(Error::BadSignature)
        ));
    }

    #[test]
    fn rejects_untrusted_key_and_accepts_any_trusted_one() {
        let (signer, _) = keypair();
        let (_, other) = keypair();
        let data = fixture();
        let sig = sign(&signer, data.as_bytes());
        assert!(matches!(
            Index::from_signed(data.as_bytes(), &sig, &other),
            Err(Error::BadSignature)
        ));

        assert!(
            TrustedKeys::from_base64([""]).is_err(),
            "empty key must not decode"
        );
        let both = TrustedKeys::from_base64([
            minisign::KeyPair::generate_unencrypted_keypair()
                .unwrap()
                .pk
                .to_base64()
                .as_str(),
            signer.pk.to_base64().as_str(),
        ])
        .unwrap();
        assert!(Index::from_signed(data.as_bytes(), &sig, &both).is_ok());
    }

    #[test]
    fn rejects_garbage_signature() {
        let (_, keys) = keypair();
        assert!(matches!(
            Index::from_signed(fixture().as_bytes(), "not a signature", &keys),
            Err(Error::MalformedSignature { .. })
        ));
    }

    #[test]
    fn rejects_other_schemas() {
        let data = fixture().replace("\"schema\":1", "\"schema\":2");
        assert!(matches!(
            Index::from_trusted_bytes(data.as_bytes()),
            Err(Error::UnsupportedSchema {
                found: 2,
                supported: 1
            })
        ));
    }

    #[test]
    fn rejects_unsafe_ids_and_bad_hashes() {
        for bad in ["../escape", ".hidden", "a/b", ""] {
            let data = fixture().replace("\"wasm-thing\"", &format!("\"{bad}\""));
            assert!(
                matches!(
                    Index::from_trusted_bytes(data.as_bytes()),
                    Err(Error::MalformedIndex(_))
                ),
                "id `{bad}` should be rejected"
            );
        }
        let data = fixture().replacen(SHA, "ABC", 1);
        assert!(matches!(
            Index::from_trusted_bytes(data.as_bytes()),
            Err(Error::MalformedIndex(_))
        ));
    }

    #[test]
    fn rejects_duplicate_ids_and_unknown_defaults() {
        let data = fixture().replace("\"wasm-thing\"", "\"core-0.9.24-linux-x64-gnu\"");
        assert!(matches!(
            Index::from_trusted_bytes(data.as_bytes()),
            Err(Error::MalformedIndex(_))
        ));
        let data = fixture().replace("\"default\":\"3.14\"", "\"default\":\"3.12\"");
        assert!(matches!(
            Index::from_trusted_bytes(data.as_bytes()),
            Err(Error::MalformedIndex(_))
        ));
    }

    #[test]
    fn looks_up_artifacts() {
        let index = Index::from_trusted_bytes(fixture().as_bytes()).unwrap();
        let t = Target::LinuxX64Gnu;
        assert_eq!(
            index.core_artifact("0.9.24", t).unwrap().id,
            "core-0.9.24-linux-x64-gnu"
        );
        assert!(index.core_artifact("0.9.24", Target::DarwinArm64).is_none());
        assert_eq!(
            index.loader_artifact("0.9.24", "py", None, t).unwrap().id,
            "py_loader-3.14-core0.9.24-linux-x64-gnu"
        );
        assert_eq!(
            index
                .loader_artifact("0.9.24", "py", Some("3.13"), t)
                .unwrap()
                .id,
            "py_loader-3.13-core0.9.24-linux-x64-gnu"
        );
        assert!(index
            .loader_artifact("0.9.24", "py", Some("3.12"), t)
            .is_none());
        assert!(index.loader_artifact("0.9.3", "py", None, t).is_none());
    }

    #[test]
    fn checks_glibc_minimum() {
        let index = Index::from_trusted_bytes(fixture().as_bytes()).unwrap();
        let py = index
            .loader_artifact("0.9.24", "py", None, Target::LinuxX64Gnu)
            .unwrap();
        let v = |s: &str| s.parse::<DottedVersion>().unwrap();
        assert!(py.runs_on(Target::LinuxX64Gnu, Some(&v("2.38"))));
        assert!(!py.runs_on(Target::LinuxX64Gnu, Some(&v("2.17"))));
        assert!(py.runs_on(Target::LinuxX64Gnu, None));
        assert!(!py.runs_on(Target::DarwinArm64, None));
    }
}
