//! The registry Deno's resolver asks: each package's info fetched once a run, and the HTTP client.

use super::*;

#[derive(Clone, Debug)]
pub(crate) enum Missing {
    NotThere,
    Failed(String),
}

/// The registry, each package's document asked once per install (concurrent asks for one share it) and kept in the store's
/// registry/ folder with its ETag: asked again, an unchanged one is a 304, and with no network the kept one is used.
pub(crate) struct Registry {
    pub(crate) client: reqwest::Client,
    pub(crate) base: String,
    pub(crate) dir: PathBuf,
    pub(crate) offline: bool,
    pub(crate) asked: RefCell<HashMap<String, Shared<LocalBoxFuture<'static, Result<Arc<NpmPackageInfo>, Missing>>>>>,
}

impl Registry {
    pub(crate) fn new(offline: bool) -> R<Self> {
        let dir = home().join("registry");
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok(Registry {
            client: client()?,
            base: std::env::var("RAEA_NPM_REGISTRY").unwrap_or_else(|_| REGISTRY.into()).trim_end_matches('/').to_string(),
            dir,
            offline,
            asked: RefCell::new(HashMap::new()),
        })
    }
}

#[async_trait::async_trait(?Send)]
impl NpmRegistryApi for Registry {
    async fn package_info(&self, name: &str) -> Result<Arc<NpmPackageInfo>, NpmRegistryPackageInfoLoadError> {
        let ask = {
            let mut asked = self.asked.borrow_mut();
            asked
                .entry(name.to_string())
                .or_insert_with(|| packument(self.client.clone(), format!("{}/{}", self.base, name.replace('/', "%2f")), self.dir.join(flat(name)), self.offline).boxed_local().shared())
                .clone()
        };
        ask.await.map_err(|e| match e {
            Missing::NotThere => NpmRegistryPackageInfoLoadError::PackageNotExists { package_name: name.to_string() },
            Missing::Failed(m) => NpmRegistryPackageInfoLoadError::LoadError(Arc::new(deno_error::JsErrorBox::generic(m))),
        })
    }
}

/// One package's document (the abbreviated one npm installs from), kept at `kept`.json with its ETag in `kept`.etag.
async fn packument(client: reqwest::Client, url: String, kept: PathBuf, offline: bool) -> Result<Arc<NpmPackageInfo>, Missing> {
    let json = kept.with_extension("json");
    let etag_file = kept.with_extension("etag");
    let parse = |bytes: &[u8]| serde_json::from_slice::<NpmPackageInfo>(bytes).map(Arc::new).map_err(|e| Missing::Failed(format!("{url}: {e}")));
    let from_disk = || std::fs::read(&json).ok();
    if offline {
        return from_disk().ok_or_else(|| Missing::Failed(format!("{url}: not fetched before, and --offline"))).and_then(|b| parse(&b));
    }
    let mut req = client.get(&url).header("Accept", "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8");
    if let (Some(etag), true) = (std::fs::read_to_string(&etag_file).ok(), json.is_file()) {
        req = req.header("If-None-Match", etag.trim());
    }
    let resp = match req.send().await {
        Ok(r) => r,
        // no network: what was fetched before
        Err(e) => return from_disk().ok_or_else(|| Missing::Failed(format!("{url}: {e}"))).and_then(|b| parse(&b)),
    };
    match resp.status().as_u16() {
        304 => from_disk().ok_or_else(|| Missing::Failed(format!("{url}: 304 with nothing kept"))).and_then(|b| parse(&b)),
        404 => Err(Missing::NotThere),
        200 => {
            let etag = resp.headers().get("etag").and_then(|v| v.to_str().ok()).map(str::to_string);
            let bytes = resp.bytes().await.map_err(|e| Missing::Failed(format!("{url}: {e}")))?;
            let info = parse(&bytes)?;
            if write_atomic(&json, &bytes).is_ok() {
                match etag {
                    Some(t) => drop(std::fs::write(&etag_file, t)),
                    None => drop(std::fs::remove_file(&etag_file)),
                }
            }
            Ok(info)
        }
        s => Err(Missing::Failed(format!("{url}: HTTP {s}"))),
    }
}

pub(crate) fn client() -> R<reqwest::Client> {
    reqwest::Client::builder().user_agent(concat!("raea-io-npm/", env!("CARGO_PKG_VERSION"))).build().map_err(|e| e.to_string())
}

// ---- the store -------------------------------------------------------------------------------------------------------
