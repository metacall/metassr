//! Network access: a small transport trait, an HTTP implementation, and the
//! verified index cache.

use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    error::{io_err, Error, Result},
    index::{Index, TrustedKeys},
};

/// Largest index or signature accepted, as a guard against a bad mirror.
const MAX_INDEX_BYTES: u64 = 16 * 1024 * 1024;

/// The result of a GET.
pub enum Fetched {
    /// The server confirmed the cached copy (the `ETag` matched).
    NotModified,
    Body {
        reader: Box<dyn Read + Send>,
        etag: Option<String>,
    },
}

/// How the toolchain reaches the network. Tests swap in an in-memory one.
pub trait Transport: Send + Sync {
    /// GETs `url`, sending `If-None-Match` when `etag` is given. Non-success
    /// statuses are errors.
    fn get(&self, url: &str, etag: Option<&str>) -> io::Result<Fetched>;
}

/// HTTPS via `ureq` and rustls. Honours `HTTPS_PROXY`/`NO_PROXY`.
pub struct HttpTransport {
    agent: ureq::Agent,
}

impl Default for HttpTransport {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .user_agent(concat!("metacall-toolchain/", env!("CARGO_PKG_VERSION")))
            .timeout_connect(Some(Duration::from_secs(30)))
            .http_status_as_error(false)
            .build();
        HttpTransport {
            agent: config.into(),
        }
    }
}

impl Transport for HttpTransport {
    fn get(&self, url: &str, etag: Option<&str>) -> io::Result<Fetched> {
        let mut request = self.agent.get(url);
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        let response = request.call().map_err(io::Error::other)?;
        let status = response.status();
        if status == 304 {
            return Ok(Fetched::NotModified);
        }
        if !status.is_success() {
            return Err(io::Error::other(format!("HTTP {status}")));
        }
        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let reader = response
            .into_body()
            .into_with_config()
            .limit(u64::MAX)
            .reader();
        Ok(Fetched::Body {
            reader: Box::new(reader),
            etag,
        })
    }
}

/// How much the index lookup may use the network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchPolicy {
    /// Use the cached index if there is a valid one, otherwise fetch.
    Cached,
    /// Ask the server (with the cached `ETag`) for a newer index.
    Refresh,
    /// Never touch the network.
    Offline,
}

/// The last verified index under `$METACALL_HOME/index/`.
pub(crate) struct IndexCache {
    dir: PathBuf,
}

impl IndexCache {
    pub(crate) fn new(dir: PathBuf) -> Self {
        IndexCache { dir }
    }

    fn data_path(&self) -> PathBuf {
        self.dir.join("index.json")
    }

    fn sig_path(&self) -> PathBuf {
        self.dir.join("index.json.sig")
    }

    fn etag_path(&self) -> PathBuf {
        self.dir.join("etag")
    }

    /// The cached index, verified again (the files are only as trustworthy
    /// as the user's home directory).
    fn load(&self, keys: &TrustedKeys) -> Option<Index> {
        let data = fs::read(self.data_path()).ok()?;
        let sig = fs::read_to_string(self.sig_path()).ok()?;
        Index::from_signed(&data, &sig, keys).ok()
    }

    pub(crate) fn get(
        &self,
        policy: FetchPolicy,
        url: Option<&str>,
        keys: &TrustedKeys,
        transport: &dyn Transport,
    ) -> Result<Index> {
        let cached = self.load(keys);
        match (policy, cached) {
            (FetchPolicy::Offline | FetchPolicy::Cached, Some(index)) => Ok(index),
            (FetchPolicy::Offline, None) => Err(Error::NoCachedIndex),
            (_, cached) => {
                let url = url.ok_or(Error::NoIndexUrl)?;
                // Only send the ETag when there is a verified copy to fall back on.
                let etag = cached
                    .as_ref()
                    .and_then(|_| fs::read_to_string(self.etag_path()).ok());
                match get(transport, url, etag.as_deref())? {
                    Fetched::NotModified => {
                        cached.ok_or_else(|| fetch_error(url, "unexpected 304 Not Modified"))
                    }
                    Fetched::Body { reader, etag } => {
                        self.store(url, reader, etag, keys, transport)
                    }
                }
            }
        }
    }

    /// Verifies a fresh download, then replaces the cache with it.
    fn store(
        &self,
        url: &str,
        reader: Box<dyn Read + Send>,
        etag: Option<String>,
        keys: &TrustedKeys,
        transport: &dyn Transport,
    ) -> Result<Index> {
        let data = read_limited(reader, url)?;
        let sig_url = format!("{url}.sig");
        let sig = match get(transport, &sig_url, None)? {
            Fetched::Body { reader, .. } => read_limited(reader, &sig_url)?,
            Fetched::NotModified => return Err(fetch_error(&sig_url, "unexpected 304")),
        };
        let sig =
            String::from_utf8(sig).map_err(|_| fetch_error(&sig_url, "signature is not text"))?;
        let index = Index::from_signed(&data, &sig, keys)?;

        fs::create_dir_all(&self.dir).map_err(io_err("create", &self.dir))?;
        // Signature last: a crash between the writes leaves a pair that
        // fails verification, which just means fetching again.
        let _ = fs::remove_file(self.etag_path());
        write_atomic(&self.data_path(), &data)?;
        write_atomic(&self.sig_path(), sig.as_bytes())?;
        if let Some(etag) = etag {
            write_atomic(&self.etag_path(), etag.as_bytes())?;
        }
        Ok(index)
    }
}

pub(crate) fn get(transport: &dyn Transport, url: &str, etag: Option<&str>) -> Result<Fetched> {
    transport.get(url, etag).map_err(|e| fetch_error(url, e))
}

pub(crate) fn fetch_error(url: &str, reason: impl std::fmt::Display) -> Error {
    Error::Fetch {
        url: url.to_string(),
        reason: reason.to_string(),
    }
}

fn read_limited(reader: impl Read, url: &str) -> Result<Vec<u8>> {
    let mut data = vec![];
    reader
        .take(MAX_INDEX_BYTES + 1)
        .read_to_end(&mut data)
        .map_err(|e| fetch_error(url, e))?;
    if data.len() as u64 > MAX_INDEX_BYTES {
        return Err(fetch_error(url, "response is too large"));
    }
    Ok(data)
}

pub(crate) fn write_atomic(path: &Path, data: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, data).map_err(io_err("write", &tmp))?;
    fs::rename(&tmp, path).map_err(io_err("rename into", path))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::{
        collections::HashMap,
        io::{BufRead, BufReader, Cursor, Write},
        net::TcpListener,
        sync::Mutex,
        thread,
    };

    /// A served body and its `ETag`.
    type Served = (Vec<u8>, Option<String>);

    /// Serves fixed bodies and records every request.
    #[derive(Default)]
    pub(crate) struct FakeTransport {
        pub(crate) files: Mutex<HashMap<String, Served>>,
        pub(crate) requests: Mutex<Vec<(String, Option<String>)>>,
    }

    impl FakeTransport {
        pub(crate) fn serve(&self, url: &str, body: impl Into<Vec<u8>>, etag: Option<&str>) {
            self.files
                .lock()
                .unwrap()
                .insert(url.into(), (body.into(), etag.map(Into::into)));
        }

        pub(crate) fn requests(&self) -> Vec<(String, Option<String>)> {
            self.requests.lock().unwrap().clone()
        }
    }

    impl Transport for FakeTransport {
        fn get(&self, url: &str, etag: Option<&str>) -> io::Result<Fetched> {
            self.requests
                .lock()
                .unwrap()
                .push((url.into(), etag.map(Into::into)));
            let files = self.files.lock().unwrap();
            let (body, served_etag) = files.get(url).ok_or_else(|| io::Error::other("HTTP 404"))?;
            if etag.is_some() && etag == served_etag.as_deref() {
                return Ok(Fetched::NotModified);
            }
            Ok(Fetched::Body {
                reader: Box::new(Cursor::new(body.clone())),
                etag: served_etag.clone(),
            })
        }
    }

    pub(crate) struct Signer(minisign::KeyPair);

    impl Signer {
        pub(crate) fn new() -> Self {
            Signer(minisign::KeyPair::generate_unencrypted_keypair().unwrap())
        }

        pub(crate) fn keys(&self) -> TrustedKeys {
            TrustedKeys::from_base64([self.0.pk.to_base64().as_str()]).unwrap()
        }

        pub(crate) fn sign(&self, data: &[u8]) -> String {
            minisign::sign(Some(&self.0.pk), &self.0.sk, Cursor::new(data), None, None)
                .unwrap()
                .to_string()
        }
    }

    pub(crate) const URL: &str = "https://mirror.invalid/index.json";

    fn index_json(generated: &str) -> String {
        serde_json::json!({ "schema": 1, "generated": generated, "cores": {}, "artifacts": [] })
            .to_string()
    }

    fn publish(t: &FakeTransport, s: &Signer, generated: &str, etag: &str) {
        let data = index_json(generated);
        t.serve(URL, data.clone(), Some(etag));
        t.serve(&format!("{URL}.sig"), s.sign(data.as_bytes()), None);
    }

    #[test]
    fn cached_policy_fetches_once() {
        let tmp = tempfile::tempdir().unwrap();
        let (t, s) = (FakeTransport::default(), Signer::new());
        publish(&t, &s, "one", "\"e1\"");
        let cache = IndexCache::new(tmp.path().join("index"));

        let a = cache
            .get(FetchPolicy::Cached, Some(URL), &s.keys(), &t)
            .unwrap();
        let b = cache
            .get(FetchPolicy::Cached, Some(URL), &s.keys(), &t)
            .unwrap();
        assert_eq!((a.generated.as_str(), b.generated.as_str()), ("one", "one"));
        assert_eq!(t.requests().len(), 2, "index + signature, then nothing");
    }

    #[test]
    fn refresh_uses_the_etag() {
        let tmp = tempfile::tempdir().unwrap();
        let (t, s) = (FakeTransport::default(), Signer::new());
        publish(&t, &s, "one", "\"e1\"");
        let cache = IndexCache::new(tmp.path().join("index"));
        cache
            .get(FetchPolicy::Cached, Some(URL), &s.keys(), &t)
            .unwrap();

        cache
            .get(FetchPolicy::Refresh, Some(URL), &s.keys(), &t)
            .unwrap();
        assert_eq!(t.requests().last().unwrap().1.as_deref(), Some("\"e1\""));
        assert_eq!(t.requests().len(), 3, "304: no signature download");

        publish(&t, &s, "two", "\"e2\"");
        let fresh = cache
            .get(FetchPolicy::Refresh, Some(URL), &s.keys(), &t)
            .unwrap();
        assert_eq!(fresh.generated, "two");
        let offline = cache
            .get(FetchPolicy::Offline, None, &s.keys(), &t)
            .unwrap();
        assert_eq!(offline.generated, "two");
    }

    #[test]
    fn a_badly_signed_index_is_not_cached() {
        let tmp = tempfile::tempdir().unwrap();
        let (t, s) = (FakeTransport::default(), Signer::new());
        t.serve(URL, index_json("evil"), None);
        t.serve(
            &format!("{URL}.sig"),
            Signer::new().sign(index_json("evil").as_bytes()),
            None,
        );
        let cache = IndexCache::new(tmp.path().join("index"));
        assert!(matches!(
            cache.get(FetchPolicy::Cached, Some(URL), &s.keys(), &t),
            Err(Error::BadSignature)
        ));
        assert!(matches!(
            cache.get(FetchPolicy::Offline, None, &s.keys(), &t),
            Err(Error::NoCachedIndex)
        ));
    }

    #[test]
    fn a_tampered_cache_is_fetched_again() {
        let tmp = tempfile::tempdir().unwrap();
        let (t, s) = (FakeTransport::default(), Signer::new());
        publish(&t, &s, "one", "\"e1\"");
        let cache = IndexCache::new(tmp.path().join("index"));
        cache
            .get(FetchPolicy::Cached, Some(URL), &s.keys(), &t)
            .unwrap();
        fs::write(cache.data_path(), index_json("tampered")).unwrap();

        assert!(matches!(
            cache.get(FetchPolicy::Offline, None, &s.keys(), &t),
            Err(Error::NoCachedIndex)
        ));
        let index = cache
            .get(FetchPolicy::Cached, Some(URL), &s.keys(), &t)
            .unwrap();
        assert_eq!(index.generated, "one");
    }

    #[test]
    fn needs_a_url_to_fetch() {
        let tmp = tempfile::tempdir().unwrap();
        let (t, s) = (FakeTransport::default(), Signer::new());
        let cache = IndexCache::new(tmp.path().join("index"));
        assert!(matches!(
            cache.get(FetchPolicy::Cached, None, &s.keys(), &t),
            Err(Error::NoIndexUrl)
        ));
    }

    /// One real HTTP round trip through ureq: 200 with an ETag, then 304.
    #[test]
    fn http_transport_handles_etags() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/index.json", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut conditional = false;
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line
                        .to_ascii_lowercase()
                        .starts_with("if-none-match: \"abc\"")
                    {
                        conditional = true;
                    }
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let mut stream = stream;
                let reply = if conditional {
                    "HTTP/1.1 304 Not Modified\r\nConnection: close\r\n\r\n".to_string()
                } else {
                    "HTTP/1.1 200 OK\r\nETag: \"abc\"\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello"
                        .to_string()
                };
                stream.write_all(reply.as_bytes()).unwrap();
            }
        });

        let http = HttpTransport::default();
        match http.get(&url, None).unwrap() {
            Fetched::Body { mut reader, etag } => {
                let mut body = String::new();
                reader.read_to_string(&mut body).unwrap();
                assert_eq!((body.as_str(), etag.as_deref()), ("hello", Some("\"abc\"")));
            }
            Fetched::NotModified => panic!("expected a body"),
        }
        assert!(matches!(
            http.get(&url, Some("\"abc\"")).unwrap(),
            Fetched::NotModified
        ));
        server.join().unwrap();
    }
}
