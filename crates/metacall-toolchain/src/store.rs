//! The store: one immutable directory per installed artifact.
//!
//! ```text
//! store/
//! ├── .lock                 # held while installing or removing
//! ├── .tmp/                 # unpack area, renamed into place when complete
//! ├── py_loader-3.14-.../   # downloaded: the unpacked tarball
//! │   └── manifest.json
//! └── core-0.9.24-.../      # external: files stay in the host package
//!     ├── manifest.json
//!     └── external.json     # { "root": "/abs/path/into/host/package" }
//! ```

use std::{
    fs::{self, File},
    io::{self, Read, Seek},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    error::{io_err, Error, Result},
    index::validate_id,
    manifest::{is_safe_relative, Manifest},
};

const MANIFEST_FILE: &str = "manifest.json";
const EXTERNAL_FILE: &str = "external.json";
const LOCK_FILE: &str = ".lock";
const TMP_DIR: &str = ".tmp";

/// An artifact present in the store.
#[derive(Debug, Clone)]
pub struct Installed {
    pub manifest: Manifest,
    /// The artifact's root: its store directory, or the host package
    /// directory for an external entry.
    pub dir: PathBuf,
    /// Registered from the host package rather than downloaded.
    pub external: bool,
}

impl Installed {
    pub fn id(&self) -> &str {
        &self.manifest.id
    }
}

#[derive(Serialize, Deserialize)]
struct ExternalRecord {
    root: PathBuf,
}

/// The artifact store under `$METACALL_HOME/store`.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
}

/// Held for the duration of a store mutation. The OS releases the lock when
/// the file is closed, including when the process dies.
struct StoreLock(#[allow(dead_code)] File);

impl Store {
    pub fn open(root: impl Into<PathBuf>) -> Result<Store> {
        let root = root.into();
        fs::create_dir_all(root.join(TMP_DIR)).map_err(io_err("create", &root))?;
        Ok(Store { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn lock(&self) -> Result<StoreLock> {
        let path = self.root.join(LOCK_FILE);
        let file = File::create(&path).map_err(io_err("open", &path))?;
        file.lock().map_err(io_err("lock", &path))?;
        Ok(StoreLock(file))
    }

    fn entry_dir(&self, id: &str) -> Result<PathBuf> {
        validate_id(id)?;
        Ok(self.root.join(id))
    }

    /// The installed artifact `id`, if present.
    pub fn get(&self, id: &str) -> Result<Option<Installed>> {
        let dir = self.entry_dir(id)?;
        if !dir.is_dir() {
            return Ok(None);
        }
        let manifest: Manifest = read_json(&dir.join(MANIFEST_FILE), id)?;
        let external_path = dir.join(EXTERNAL_FILE);
        let (root, external) = if external_path.exists() {
            let record: ExternalRecord = read_json(&external_path, id)?;
            (record.root, true)
        } else {
            (dir, false)
        };
        manifest.validate(id, &root)?;
        Ok(Some(Installed {
            manifest,
            dir: root,
            external,
        }))
    }

    /// Every artifact in the store, sorted by id.
    pub fn installed(&self) -> Result<Vec<Installed>> {
        let mut ids: Vec<String> = fs::read_dir(&self.root)
            .map_err(io_err("read", &self.root))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| !name.starts_with('.'))
            .collect();
        ids.sort();
        ids.iter()
            .filter_map(|id| self.get(id).transpose())
            .collect()
    }

    /// Installs a `.tar.zst` artifact after checking its sha256. Returns the
    /// existing entry if `id` is already installed.
    pub fn install_archive(&self, id: &str, archive: &Path, sha256: &str) -> Result<Installed> {
        let dest = self.entry_dir(id)?;
        let _lock = self.lock()?;
        if let Some(existing) = self.get(id)? {
            return Ok(existing);
        }

        let mut file = File::open(archive).map_err(io_err("open", archive))?;
        let actual = sha256_of(&mut file).map_err(io_err("read", archive))?;
        if !actual.eq_ignore_ascii_case(sha256) {
            return Err(Error::HashMismatch {
                id: id.to_string(),
                expected: sha256.to_string(),
                actual,
            });
        }
        file.rewind().map_err(io_err("read", archive))?;

        let tmp = self.fresh_tmp(id)?;
        let result = (|| {
            let decoder = zstd::Decoder::new(file).map_err(io_err("decompress", archive))?;
            unpack(decoder, &tmp, id)?;
            let manifest: Manifest = read_json(&tmp.join(MANIFEST_FILE), id)?;
            manifest.validate(id, &tmp)?;
            fs::rename(&tmp, &dest).map_err(io_err("rename into", &dest))
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&tmp);
        }
        result?;
        self.get(id)?
            .ok_or_else(|| Error::NotInstalled(id.to_string()))
    }

    /// Records an artifact whose files live in the host package (the npm
    /// payload's core, node and ts). Re-registering replaces the record, so a
    /// package upgrade that moves the files is picked up.
    pub fn register_external(&self, manifest: &Manifest, root: &Path) -> Result<Installed> {
        let id = manifest.id.as_str();
        let dest = self.entry_dir(id)?;
        let root = fs::canonicalize(root).map_err(io_err("resolve", root))?;
        manifest.validate(id, &root)?;

        let _lock = self.lock()?;
        if dest.exists() && !dest.join(EXTERNAL_FILE).exists() {
            // A downloaded artifact with the same id wins over the host copy.
            return self.get(id)?.ok_or_else(|| Error::NotInstalled(id.into()));
        }
        let tmp = self.fresh_tmp(id)?;
        let result = (|| {
            write_json(&tmp.join(MANIFEST_FILE), manifest)?;
            write_json(
                &tmp.join(EXTERNAL_FILE),
                &ExternalRecord { root: root.clone() },
            )?;
            if dest.exists() {
                fs::remove_dir_all(&dest).map_err(io_err("remove", &dest))?;
            }
            fs::rename(&tmp, &dest).map_err(io_err("rename into", &dest))
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(&tmp);
        }
        result?;
        self.get(id)?.ok_or_else(|| Error::NotInstalled(id.into()))
    }

    /// Deletes a downloaded artifact and returns the bytes freed. External
    /// entries are refused.
    pub fn remove(&self, id: &str) -> Result<u64> {
        let dir = self.entry_dir(id)?;
        let _lock = self.lock()?;
        if !dir.is_dir() {
            return Err(Error::NotInstalled(id.to_string()));
        }
        if dir.join(EXTERNAL_FILE).exists() {
            return Err(Error::External(id.to_string()));
        }
        let size = dir_size(&dir).map_err(io_err("read", &dir))?;
        fs::remove_dir_all(&dir).map_err(io_err("remove", &dir))?;
        Ok(size)
    }

    /// A clean unpack directory for `id`. Only called while holding the
    /// lock, so leftovers can only come from an interrupted earlier run.
    fn fresh_tmp(&self, id: &str) -> Result<PathBuf> {
        let tmp = self.root.join(TMP_DIR).join(id);
        if tmp.symlink_metadata().is_ok() {
            fs::remove_dir_all(&tmp).map_err(io_err("clean", &tmp))?;
        }
        fs::create_dir_all(&tmp).map_err(io_err("create", &tmp))?;
        Ok(tmp)
    }
}

fn sha256_of(reader: &mut impl Read) -> io::Result<String> {
    let mut hasher = Sha256::new();
    io::copy(reader, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Unpacks a tar stream into `dst`, accepting only regular files,
/// directories and symlinks that stay inside `dst`.
fn unpack(reader: impl Read, dst: &Path, id: &str) -> Result<()> {
    let unsafe_entry = |reason: String| Error::UnsafeArchive {
        id: id.to_string(),
        reason,
    };
    let io = |e: io::Error| unsafe_entry(format!("unreadable archive: {e}"));

    let mut archive = tar::Archive::new(reader);
    archive.set_preserve_permissions(false); // drops setuid/setgid/sticky
    archive.set_unpack_xattrs(false);
    for entry in archive.entries().map_err(io)? {
        let mut entry = entry.map_err(io)?;
        let path = entry.path().map_err(io)?.into_owned();
        if !is_safe_relative(&path) {
            return Err(unsafe_entry(format!(
                "entry `{}` escapes the artifact",
                path.display()
            )));
        }
        let kind = entry.header().entry_type();
        if kind.is_pax_global_extensions() {
            continue;
        }
        if kind.is_symlink() {
            let target = entry.link_name().map_err(io)?.ok_or_else(|| {
                unsafe_entry(format!("symlink `{}` has no target", path.display()))
            })?;
            if !link_stays_inside(&path, &target) {
                return Err(unsafe_entry(format!(
                    "symlink `{}` -> `{}` points outside the artifact",
                    path.display(),
                    target.display()
                )));
            }
        } else if !kind.is_file() && !kind.is_dir() {
            return Err(unsafe_entry(format!(
                "entry `{}` is not a file, directory or symlink",
                path.display()
            )));
        }
        if !entry.unpack_in(dst).map_err(io)? {
            return Err(unsafe_entry(format!(
                "entry `{}` escapes the artifact",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Whether a relative symlink at `link` (relative to the artifact root)
/// resolves to a path inside the artifact.
fn link_stays_inside(link: &Path, target: &Path) -> bool {
    let mut depth: Vec<&std::ffi::OsStr> = link
        .parent()
        .into_iter()
        .flat_map(|p| p.components())
        .filter_map(|c| match c {
            Component::Normal(n) => Some(n),
            _ => None,
        })
        .collect();
    for c in target.components() {
        match c {
            Component::Normal(n) => depth.push(n),
            Component::CurDir => {}
            Component::ParentDir => {
                if depth.pop().is_none() {
                    return false;
                }
            }
            Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

fn dir_size(dir: &Path) -> io::Result<u64> {
    let mut total = 0;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.path().symlink_metadata()?;
        total += if meta.is_dir() {
            dir_size(&entry.path())?
        } else {
            meta.len()
        };
    }
    Ok(total)
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path, id: &str) -> Result<T> {
    let bytes = fs::read(path).map_err(|e| Error::BadManifest {
        id: id.to_string(),
        reason: format!("cannot read {}: {e}", path.display()),
    })?;
    serde_json::from_slice(&bytes).map_err(|e| Error::BadManifest {
        id: id.to_string(),
        reason: format!("{}: {e}", path.display()),
    })
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value).expect("manifest types always serialize");
    fs::write(path, bytes).map_err(io_err("write", path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::ArtifactKind;
    use std::{sync::Arc, thread};

    enum E<'a> {
        File(&'a str, &'a [u8]),
        Dir(&'a str),
        Symlink(&'a str, &'a str),
        Hardlink(&'a str, &'a str),
        Fifo(&'a str),
    }

    /// Builds a `.tar.zst` with raw header names, so tests can write paths
    /// the `tar` builder itself would refuse.
    fn archive(dir: &Path, name: &str, entries: &[E]) -> (PathBuf, String) {
        let mut tar = Vec::new();
        {
            let mut b = tar::Builder::new(&mut tar);
            for e in entries {
                let mut h = tar::Header::new_gnu();
                let (path, kind, data, link): (&str, _, &[u8], Option<&str>) = match e {
                    E::File(p, d) => (p, tar::EntryType::Regular, d, None),
                    E::Dir(p) => (p, tar::EntryType::Directory, b"", None),
                    E::Symlink(p, t) => (p, tar::EntryType::Symlink, b"", Some(t)),
                    E::Hardlink(p, t) => (p, tar::EntryType::Link, b"", Some(t)),
                    E::Fifo(p) => (p, tar::EntryType::Fifo, b"", None),
                };
                let raw = &mut h.as_old_mut().name;
                raw[..path.len()].copy_from_slice(path.as_bytes());
                if let Some(t) = link {
                    let raw = &mut h.as_old_mut().linkname;
                    raw[..t.len()].copy_from_slice(t.as_bytes());
                }
                h.set_entry_type(kind);
                h.set_mode(if matches!(e, E::Dir(_)) {
                    0o755
                } else {
                    0o4755
                });
                h.set_size(data.len() as u64);
                h.set_cksum();
                b.append(&h, data).unwrap();
            }
            b.finish().unwrap();
        }
        let path = dir.join(name);
        let compressed = zstd::encode_all(&tar[..], 3).unwrap();
        fs::write(&path, &compressed).unwrap();
        let sha = sha256_of(&mut &compressed[..]).unwrap();
        (path, sha)
    }

    fn manifest_json(id: &str, lib: &[&str]) -> Vec<u8> {
        serde_json::to_vec(&Manifest {
            schema: 1,
            id: id.into(),
            kind: ArtifactKind::Loader,
            core: "0.9.24".into(),
            loader: Some("py".into()),
            runtime_version: Some("3.14".into()),
            lib: lib.iter().map(|s| s.to_string()).collect(),
            config: None,
        })
        .unwrap()
    }

    const ID: &str = "py_loader-3.14-core0.9.24-linux-x64-gnu";

    fn good_archive(dir: &Path) -> (PathBuf, String) {
        let manifest = manifest_json(ID, &["lib/libpy_loader.so"]);
        archive(
            dir,
            "py.tar.zst",
            &[
                E::File("manifest.json", &manifest),
                E::Dir("lib"),
                E::File("lib/libpy_loader.so", b"\x7fELF"),
                E::Symlink("lib/libpy_loader.so.1", "libpy_loader.so"),
                E::Dir("runtime/lib"),
                E::Symlink("runtime/lib/loader", "../../lib/libpy_loader.so"),
            ],
        )
    }

    #[test]
    fn installs_lists_and_removes() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("store")).unwrap();
        let (archive, sha) = good_archive(tmp.path());

        let installed = store.install_archive(ID, &archive, &sha).unwrap();
        assert!(!installed.external);
        assert_eq!(installed.dir, store.root().join(ID));
        assert!(installed.dir.join("lib/libpy_loader.so").is_file());
        assert_eq!(
            fs::read_link(installed.dir.join("lib/libpy_loader.so.1")).unwrap(),
            Path::new("libpy_loader.so")
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(installed.dir.join("lib/libpy_loader.so"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o7000, 0, "setuid must be dropped");
        }

        // Installing again is a no-op.
        store.install_archive(ID, &archive, &sha).unwrap();
        let ids: Vec<_> = store
            .installed()
            .unwrap()
            .iter()
            .map(|i| i.id().to_string())
            .collect();
        assert_eq!(ids, [ID]);

        assert!(store.remove(ID).unwrap() > 0);
        assert!(store.get(ID).unwrap().is_none());
        assert!(matches!(store.remove(ID), Err(Error::NotInstalled(_))));
    }

    #[test]
    fn rejects_a_tampered_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("store")).unwrap();
        let (archive, _) = good_archive(tmp.path());
        let wrong = "1".repeat(64);
        assert!(matches!(
            store.install_archive(ID, &archive, &wrong),
            Err(Error::HashMismatch { .. })
        ));
        assert!(store.get(ID).unwrap().is_none());
    }

    #[test]
    fn rejects_entries_that_escape_or_are_special() {
        let manifest = manifest_json(ID, &[]);
        let cases: Vec<(&str, Vec<E>)> = vec![
            ("dotdot", vec![E::File("../evil", b"x")]),
            ("absolute", vec![E::File("/tmp/evil", b"x")]),
            ("symlink out", vec![E::Symlink("lib/x", "../../outside")]),
            ("absolute symlink", vec![E::Symlink("x", "/etc/passwd")]),
            ("hardlink", vec![E::Hardlink("x", "manifest.json")]),
            ("fifo", vec![E::Fifo("pipe")]),
        ];
        for (name, mut entries) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let store = Store::open(tmp.path().join("store")).unwrap();
            entries.insert(0, E::File("manifest.json", &manifest));
            let (archive, sha) = archive(tmp.path(), "bad.tar.zst", &entries);
            let result = store.install_archive(ID, &archive, &sha);
            assert!(
                matches!(result, Err(Error::UnsafeArchive { .. })),
                "{name}: got {result:?}"
            );
            assert!(
                store.get(ID).unwrap().is_none(),
                "{name}: nothing installed"
            );
            assert!(
                !tmp.path().join("evil").exists(),
                "{name}: nothing written outside"
            );
            assert_eq!(
                fs::read_dir(store.root().join(TMP_DIR)).unwrap().count(),
                0,
                "{name}: tmp cleaned"
            );
        }
    }

    #[test]
    fn rejects_bad_manifests() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("store")).unwrap();
        let other = manifest_json("someone-else", &[]);
        let missing_lib = manifest_json(ID, &["lib/nope.so"]);
        for (name, entries) in [
            ("no manifest", vec![E::Dir("lib")]),
            ("wrong id", vec![E::File("manifest.json", &other)]),
            ("missing lib", vec![E::File("manifest.json", &missing_lib)]),
        ] {
            let (archive, sha) = archive(tmp.path(), "m.tar.zst", &entries);
            let result = store.install_archive(ID, &archive, &sha);
            assert!(
                matches!(result, Err(Error::BadManifest { .. })),
                "{name}: {result:?}"
            );
            assert!(store.get(ID).unwrap().is_none());
        }
    }

    #[test]
    fn concurrent_installs_of_the_same_artifact_agree() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(tmp.path().join("store")).unwrap());
        let (archive, sha) = good_archive(tmp.path());
        let archive = Arc::new(archive);
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let (store, archive, sha) = (store.clone(), archive.clone(), sha.clone());
                thread::spawn(move || store.install_archive(ID, &archive, &sha).map(|i| i.dir))
            })
            .collect();
        for h in handles {
            assert_eq!(h.join().unwrap().unwrap(), store.root().join(ID));
        }
        assert_eq!(store.installed().unwrap().len(), 1);
    }

    #[test]
    fn external_entries_point_into_the_host_package_and_cannot_be_removed() {
        let tmp = tempfile::tempdir().unwrap();
        let store = Store::open(tmp.path().join("store")).unwrap();
        let payload = tmp.path().join("payload");
        fs::create_dir_all(payload.join("lib")).unwrap();
        fs::write(payload.join("lib/libmetacall.so"), b"\x7fELF").unwrap();
        let manifest = Manifest {
            schema: 1,
            id: "core-0.9.24-linux-x64-gnu".into(),
            kind: ArtifactKind::Core,
            core: "0.9.24".into(),
            loader: None,
            runtime_version: None,
            lib: vec!["lib/libmetacall.so".into()],
            config: None,
        };

        let installed = store.register_external(&manifest, &payload).unwrap();
        assert!(installed.external);
        assert_eq!(installed.dir, fs::canonicalize(&payload).unwrap());
        assert!(matches!(
            store.remove(&manifest.id),
            Err(Error::External(_))
        ));

        // A moved payload is picked up by registering again.
        let moved = tmp.path().join("payload-2");
        fs::rename(&payload, &moved).unwrap();
        let again = store.register_external(&manifest, &moved).unwrap();
        assert_eq!(again.dir, fs::canonicalize(&moved).unwrap());

        // A manifest that lists files the payload lacks is refused.
        let mut broken = manifest.clone();
        broken.lib.push("lib/libmissing.so".into());
        assert!(matches!(
            store.register_external(&broken, &moved),
            Err(Error::BadManifest { .. })
        ));
    }

    #[test]
    fn link_check() {
        let p = Path::new;
        assert!(link_stays_inside(p("a/b/link"), p("../c")));
        assert!(link_stays_inside(p("a/link"), p("./b/../c")));
        assert!(!link_stays_inside(p("a/link"), p("../../c")));
        assert!(!link_stays_inside(p("link"), p("/abs")));
    }
}
