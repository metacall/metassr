//! The entry point: `$METACALL_HOME` with its index cache, store and views.

use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::{
    error::{io_err, Error, Result},
    fetch::{self, FetchPolicy, Fetched, HttpTransport, IndexCache, Transport},
    index::{Artifact, Index, TrustedKeys},
    resolve::{resolve, Requirements, Resolution},
    store::{Installed, Store},
    target::{glibc_version, DottedVersion, Target},
    view::{self, View},
};

/// Receives download progress. The toolchain itself never prints.
pub trait Progress {
    /// A download of `total` bytes (0 when unknown) is starting.
    fn start(&self, _artifact: &Artifact, _total: u64) {}
    fn advance(&self, _bytes: u64) {}
    fn finish(&self) {}
}

/// Reports nothing.
pub struct NoProgress;

impl Progress for NoProgress {}

/// How to open a [`Toolchain`].
pub struct Options {
    /// Root directory. Defaults to `$METACALL_HOME`, then `~/.metacall`.
    pub home: Option<PathBuf>,
    /// Where `index.json` (and `index.json.sig` next to it) live. Defaults to
    /// `$METACALL_INDEX_URL`. Without one, only the cached index is usable.
    pub index_url: Option<String>,
    /// Keys the index must be signed with.
    pub keys: TrustedKeys,
    /// Defaults to the target this binary was built for.
    pub target: Option<Target>,
    /// Defaults to [`HttpTransport`].
    pub transport: Option<Box<dyn Transport>>,
}

impl Options {
    pub fn new(keys: TrustedKeys) -> Self {
        Options {
            home: None,
            index_url: None,
            keys,
            target: None,
            transport: None,
        }
    }
}

/// Everything the toolchain manages under one home directory.
pub struct Toolchain {
    home: PathBuf,
    target: Target,
    glibc: Option<DottedVersion>,
    index_url: Option<String>,
    keys: TrustedKeys,
    transport: Box<dyn Transport>,
    store: Store,
    index_cache: IndexCache,
}

impl Toolchain {
    pub fn open(options: Options) -> Result<Toolchain> {
        let home = match options
            .home
            .or_else(|| std::env::var_os("METACALL_HOME").map(PathBuf::from))
        {
            Some(home) => home,
            None => default_home()?,
        };
        let target = match options.target.or_else(Target::current) {
            Some(target) => target,
            None => {
                return Err(Error::UnsupportedHost(format!(
                    "no MetaCall builds for {}-{}",
                    std::env::consts::OS,
                    std::env::consts::ARCH
                )))
            }
        };
        let index_url = options.index_url.or_else(|| {
            std::env::var("METACALL_INDEX_URL")
                .ok()
                .filter(|u| !u.is_empty())
        });
        Ok(Toolchain {
            store: Store::open(home.join("store"))?,
            index_cache: IndexCache::new(home.join("index")),
            glibc: glibc_version(),
            transport: options
                .transport
                .unwrap_or_else(|| Box::new(HttpTransport::default())),
            keys: options.keys,
            home,
            target,
            index_url,
        })
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn target(&self) -> Target {
        self.target
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The signed index, from the cache or the network per `policy`.
    pub fn index(&self, policy: FetchPolicy) -> Result<Index> {
        self.index_cache.get(
            policy,
            self.index_url.as_deref(),
            &self.keys,
            self.transport.as_ref(),
        )
    }

    /// Resolves against what is installed, and the index when given.
    pub fn resolve(&self, req: &Requirements, index: Option<&Index>) -> Result<Resolution> {
        Ok(resolve(req, &self.store.installed()?, index, self.target))
    }

    /// Downloads, verifies and installs one artifact. A no-op if it is
    /// already in the store.
    pub fn install(&self, artifact: &Artifact, progress: &dyn Progress) -> Result<Installed> {
        if let Some(existing) = self.store.get(&artifact.id)? {
            return Ok(existing);
        }
        if !artifact.runs_on(self.target, self.glibc.as_ref()) {
            return Err(Error::UnsupportedHost(format!(
                "{} needs {}{} (this host: {}{})",
                artifact.id,
                artifact.target,
                artifact
                    .min_glibc
                    .as_ref()
                    .map(|v| format!(" with glibc {v}+"))
                    .unwrap_or_default(),
                self.target,
                self.glibc
                    .as_ref()
                    .map(|v| format!(" with glibc {v}"))
                    .unwrap_or_default(),
            )));
        }

        let downloads = self.home.join("cache").join("downloads");
        fs::create_dir_all(&downloads).map_err(io_err("create", &downloads))?;
        let part = downloads.join(format!("{}.{}.part", artifact.id, std::process::id()));
        let result = self.download(artifact, &part, progress).and_then(|()| {
            self.store
                .install_archive(&artifact.id, &part, &artifact.sha256)
        });
        let _ = fs::remove_file(&part);
        result
    }

    /// Streams the artifact to `dest`, checking size and sha256 as it goes.
    fn download(&self, artifact: &Artifact, dest: &Path, progress: &dyn Progress) -> Result<()> {
        let url = &artifact.url;
        let mut reader = match fetch::get(self.transport.as_ref(), url, None)? {
            Fetched::Body { reader, .. } => reader,
            Fetched::NotModified => return Err(fetch::fetch_error(url, "unexpected 304")),
        };
        let mut file = File::create(dest).map_err(io_err("create", dest))?;
        let mut hasher = Sha256::new();
        let mut received = 0u64;
        let mut buf = vec![0u8; 64 * 1024];
        progress.start(artifact, artifact.size);
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| fetch::fetch_error(url, e))?;
            if n == 0 {
                break;
            }
            received += n as u64;
            if artifact.size > 0 && received > artifact.size {
                return Err(fetch::fetch_error(
                    url,
                    format!("more than the {} bytes the index lists", artifact.size),
                ));
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(io_err("write", dest))?;
            progress.advance(n as u64);
        }
        progress.finish();
        file.sync_all().map_err(io_err("write", dest))?;

        if artifact.size > 0 && received != artifact.size {
            return Err(fetch::fetch_error(
                url,
                format!("got {received} bytes, the index lists {}", artifact.size),
            ));
        }
        let actual = format!("{:x}", hasher.finalize());
        if actual != artifact.sha256 {
            return Err(Error::HashMismatch {
                id: artifact.id.clone(),
                expected: artifact.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }

    /// The view for a complete resolution, built or reused.
    pub fn view(&self, resolution: &Resolution) -> Result<View> {
        if !resolution.is_complete() {
            let missing: Vec<String> = resolution.missing.iter().map(|m| m.to_string()).collect();
            return Err(Error::Incomplete(missing.join(", ")));
        }
        view::build(&self.home.join("views"), &resolution.ready)
    }
}

fn default_home() -> Result<PathBuf> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(var)
        .filter(|h| !h.is_empty())
        .map(|h| PathBuf::from(h).join(".metacall"))
        .ok_or_else(|| Error::UnsupportedHost(format!("${var} is not set; set METACALL_HOME")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fetch::tests::{FakeTransport, Signer, URL},
        resolve::LoaderSpec,
        ArtifactKind, Manifest,
    };
    use std::{cell::Cell, sync::Arc};

    /// A `.tar.zst` holding a manifest and the listed files.
    fn artifact_tarball(manifest: &Manifest, files: &[&str]) -> Vec<u8> {
        let mut tar = tar::Builder::new(Vec::new());
        let mut add = |path: &str, data: &[u8]| {
            let mut h = tar::Header::new_gnu();
            h.set_size(data.len() as u64);
            h.set_mode(0o644);
            h.set_cksum();
            tar.append_data(&mut h, path, data).unwrap();
        };
        add("manifest.json", &serde_json::to_vec(manifest).unwrap());
        for f in files {
            add(f, b"\x7fELF");
        }
        zstd::encode_all(&tar.into_inner().unwrap()[..], 3).unwrap()
    }

    fn sha(data: &[u8]) -> String {
        format!("{:x}", Sha256::digest(data))
    }

    /// Wraps a shared fake so tests can inspect requests after handing the
    /// transport to the toolchain.
    struct Shared(Arc<FakeTransport>);

    impl Transport for Shared {
        fn get(&self, url: &str, etag: Option<&str>) -> std::io::Result<Fetched> {
            self.0.get(url, etag)
        }
    }

    struct Setup {
        _tmp: tempfile::TempDir,
        toolchain: Toolchain,
        transport: Arc<FakeTransport>,
        payload: PathBuf,
    }

    /// A host package with a builtin core, and a signed index offering py.
    fn setup(py_bytes: impl FnOnce(Vec<u8>) -> Vec<u8>, listed_size: Option<u64>) -> Setup {
        let tmp = tempfile::tempdir().unwrap();
        let signer = Signer::new();
        let transport = Arc::new(FakeTransport::default());

        let py = Manifest {
            schema: 1,
            id: "py_loader-3.14-core0.9.24-linux-x64-gnu".into(),
            kind: ArtifactKind::Loader,
            core: "0.9.24".into(),
            loader: Some("py".into()),
            runtime_version: Some("3.14".into()),
            lib: vec!["lib/libpy_loader.so".into()],
            config: Some(
                serde_json::json!({ "dependencies": { "python": ["${ARTIFACT_DIR}/runtime/libpython.so"] } }),
            ),
        };
        let good = artifact_tarball(&py, &["lib/libpy_loader.so", "runtime/libpython.so"]);
        let served = py_bytes(good.clone());
        let index = serde_json::json!({
            "schema": 1, "generated": "now",
            "cores": { "0.9.24": { "commit": "c", "loaders": {
                "py": { "versions": ["3.14"], "default": "3.14" } } } },
            "artifacts": [{
                "id": py.id, "kind": "loader", "loader": "py", "runtime_version": "3.14",
                "core": "0.9.24", "target": "linux-x64-gnu", "url": "https://mirror.invalid/py.tar.zst",
                "sha256": sha(&good), "size": listed_size.unwrap_or(good.len() as u64)
            }]
        })
        .to_string();
        transport.serve(URL, index.clone(), Some("\"1\""));
        transport.serve(&format!("{URL}.sig"), signer.sign(index.as_bytes()), None);
        transport.serve("https://mirror.invalid/py.tar.zst", served, None);

        let mut options = Options::new(signer.keys());
        options.home = Some(tmp.path().join("home"));
        options.index_url = Some(URL.into());
        options.target = Some(Target::LinuxX64Gnu);
        options.transport = Some(Box::new(Shared(transport.clone())));
        let toolchain = Toolchain::open(options).unwrap();

        let payload = tmp.path().join("payload");
        fs::create_dir_all(payload.join("lib")).unwrap();
        fs::write(payload.join("lib/libmetacall.so"), b"\x7fELF").unwrap();
        let core = Manifest {
            schema: 1,
            id: "core-0.9.24-linux-x64-gnu".into(),
            kind: ArtifactKind::Core,
            core: "0.9.24".into(),
            loader: None,
            runtime_version: None,
            lib: vec!["lib/libmetacall.so".into()],
            config: None,
        };
        toolchain
            .store()
            .register_external(&core, &payload)
            .unwrap();

        Setup {
            _tmp: tmp,
            toolchain,
            transport,
            payload,
        }
    }

    fn needs_py() -> Requirements {
        Requirements::default().with("python".parse::<LoaderSpec>().unwrap())
    }

    #[derive(Default)]
    struct Count(Cell<u64>);

    impl Progress for Count {
        fn advance(&self, bytes: u64) {
            self.0.set(self.0.get() + bytes);
        }
    }

    #[test]
    fn resolve_install_and_view_end_to_end() {
        let s = setup(|b| b, None);
        let tc = &s.toolchain;

        let offline = tc.resolve(&needs_py(), None).unwrap();
        assert_eq!(offline.missing.len(), 1);
        assert!(
            offline.missing[0].artifact.is_none(),
            "no index, no artifact"
        );
        assert!(matches!(tc.view(&offline), Err(Error::Incomplete(_))));

        let index = tc.index(FetchPolicy::Cached).unwrap();
        let res = tc.resolve(&needs_py(), Some(&index)).unwrap();
        let artifact = res.missing[0].artifact.clone().unwrap();
        let progress = Count::default();
        tc.install(&artifact, &progress).unwrap();
        assert_eq!(progress.0.get(), artifact.size);

        let res = tc.resolve(&needs_py(), None).unwrap();
        assert!(res.is_complete());
        let view = tc.view(&res).unwrap();
        let lib = view.lib_dir();
        assert_eq!(
            fs::read_link(lib.join("libmetacall.so")).unwrap(),
            fs::canonicalize(&s.payload)
                .unwrap()
                .join("lib/libmetacall.so")
        );
        assert!(lib.join("libpy_loader.so").exists());

        // Installing again touches neither the network nor the store.
        let before = s.transport.requests().len();
        tc.install(&artifact, &NoProgress).unwrap();
        assert_eq!(s.transport.requests().len(), before);
        assert_eq!(
            fs::read_dir(tc.home().join("cache/downloads"))
                .unwrap()
                .count(),
            0,
            "partial downloads are cleaned up"
        );
    }

    #[test]
    fn rejects_a_corrupted_download() {
        let s = setup(
            |mut b| {
                let last = b.len() - 1;
                b[last] ^= 0xff;
                b
            },
            None,
        );
        let index = s.toolchain.index(FetchPolicy::Cached).unwrap();
        let artifact = &index.artifacts[0];
        assert!(matches!(
            s.toolchain.install(artifact, &NoProgress),
            Err(Error::HashMismatch { .. })
        ));
        assert!(s.toolchain.store().get(&artifact.id).unwrap().is_none());
    }

    #[test]
    fn stops_a_download_larger_than_listed() {
        let s = setup(|b| b, Some(10));
        let index = s.toolchain.index(FetchPolicy::Cached).unwrap();
        let err = s
            .toolchain
            .install(&index.artifacts[0], &NoProgress)
            .unwrap_err();
        assert!(matches!(err, Error::Fetch { .. }), "{err}");
    }

    #[test]
    fn refuses_artifacts_for_another_host() {
        let s = setup(|b| b, None);
        let index = s.toolchain.index(FetchPolicy::Cached).unwrap();
        let mut other = index.artifacts[0].clone();
        other.target = "darwin-arm64".into();
        other.id = "py-darwin".into();
        assert!(matches!(
            s.toolchain.install(&other, &NoProgress),
            Err(Error::UnsupportedHost(_))
        ));
    }
}
