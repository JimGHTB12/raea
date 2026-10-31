//! The store every project shares (~/.raea/npm/store): a package's tarball fetched once, checked against the registry's integrity hash and unpacked.

use super::*;

/// Where raea's npm keeps what it fetched: RAEA_NPM_HOME, or ~/.raea/npm.
pub(crate) fn home() -> PathBuf {
    std::env::var_os("RAEA_NPM_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".raea/npm"))
}

/// A package name as one path segment ("@scope/name" is "@scope+name").
pub(crate) fn flat(name: &str) -> String {
    name.replace('/', "+")
}

/// A package version's unpacked tarball in the store; `.raea-ok` in it once it's whole and checked.
pub(crate) fn stored(id: &NpmPackageId) -> PathBuf {
    home().join("store").join(format!("{}@{}", flat(&id.nv.name), id.nv.version))
}

pub(crate) const OK: &str = ".raea-ok";

/// Every package's tarball in the store: fetched, checked against its integrity hash and unpacked, those already there
/// left alone.
pub(crate) async fn fetch_all(s: &SerializedNpmResolutionSnapshot) -> R<Report> {
    let mut want: BTreeMap<PathBuf, (String, String, NpmPackageVersionDistInfoIntegrityOwned)> = BTreeMap::new();
    for p in &s.packages {
        let dir = stored(&p.id);
        let Some(dist) = &p.dist else { continue };
        if !dir.join(OK).is_file() {
            want.insert(dir, (p.id.nv.to_string(), dist.tarball.clone(), owned(dist.integrity())));
        }
    }
    let mut report = Report::default();
    if want.is_empty() {
        return Ok(report);
    }
    let client = client()?;
    let mut jobs = futures::stream::iter(want.into_iter().map(|(dir, (nv, url, integrity))| {
        let client = client.clone();
        async move {
            let bytes = client.get(&url).send().await.and_then(|r| r.error_for_status()).map_err(|e| format!("{nv}: {e}"))?.bytes().await.map_err(|e| format!("{nv}: {e}"))?;
            tokio::task::spawn_blocking(move || {
                let warning = check(&nv, &bytes, &integrity)?;
                unpack(&bytes, &dir).map_err(|e| format!("{nv}: unpacking: {e}"))?;
                Ok::<_, String>(warning)
            })
            .await
            .map_err(|e| e.to_string())?
        }
    }))
    .buffer_unordered(FETCHES);
    while let Some(r) = jobs.next().await {
        if let Some(w) = r? {
            report.warnings.push(w);
        }
        report.fetched += 1;
    }
    Ok(report)
}

pub(crate) enum NpmPackageVersionDistInfoIntegrityOwned {
    Sri(String, String),
    Sha1Hex(String),
    Unknown(String),
    None,
}

pub(crate) fn owned(i: NpmPackageVersionDistInfoIntegrity<'_>) -> NpmPackageVersionDistInfoIntegrityOwned {
    use NpmPackageVersionDistInfoIntegrityOwned as O;
    match i {
        NpmPackageVersionDistInfoIntegrity::Integrity { algorithm, base64_hash } => O::Sri(algorithm.to_string(), base64_hash.to_string()),
        NpmPackageVersionDistInfoIntegrity::LegacySha1Hex(h) => O::Sha1Hex(h.to_string()),
        NpmPackageVersionDistInfoIntegrity::UnknownIntegrity(s) => O::Unknown(s.to_string()),
        NpmPackageVersionDistInfoIntegrity::None => O::None,
    }
}

/// A tarball against the hash the registry gave: a mismatch refuses the install; no hash, or one of a kind raea
/// can't check, is a warning.
pub(crate) fn check(nv: &str, bytes: &[u8], integrity: &NpmPackageVersionDistInfoIntegrityOwned) -> R<Option<String>> {
    use base64::Engine;
    use sha2::Digest;
    use NpmPackageVersionDistInfoIntegrityOwned as O;
    let b64 = |d: &[u8]| base64::engine::general_purpose::STANDARD.encode(d);
    let (want, got) = match integrity {
        O::Sri(alg, hash) => match alg.as_str() {
            "sha512" => (hash.clone(), b64(&sha2::Sha512::digest(bytes))),
            "sha384" => (hash.clone(), b64(&sha2::Sha384::digest(bytes))),
            "sha256" => (hash.clone(), b64(&sha2::Sha256::digest(bytes))),
            "sha1" => (hash.clone(), b64(&sha1::Sha1::digest(bytes))),
            other => return Ok(Some(format!("{nv}: its integrity is {other}, which raea can't check"))),
        },
        O::Sha1Hex(h) => (h.to_ascii_lowercase(), hex::encode(sha1::Sha1::digest(bytes))),
        O::Unknown(s) => return Ok(Some(format!("{nv}: its integrity \"{s}\" isn't one raea reads; not checked"))),
        O::None => return Ok(Some(format!("{nv}: the registry gave no integrity hash; not checked"))),
    };
    if want != got {
        return Err(format!("{nv}: the tarball doesn't match its integrity hash (registry {want}, fetched {got}); refused"));
    }
    Ok(None)
}

/// A gzipped tarball unpacked into `dir`, its first path segment ("package/") dropped; only files and folders, never a
/// path that climbs out. Unpacked beside `dir` first and moved in whole.
pub(crate) fn unpack(bytes: &[u8], dir: &Path) -> std::io::Result<()> {
    let tmp = dir.with_extension(format!("part-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    let mut ar = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    for entry in ar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        let mut parts = path.components();
        parts.next();
        let rel: PathBuf = parts.as_path().to_path_buf();
        if rel.as_os_str().is_empty() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
            continue;
        }
        let to = tmp.join(&rel);
        match entry.header().entry_type() {
            tar::EntryType::Directory => std::fs::create_dir_all(&to)?,
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                if let Some(p) = to.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut f = std::fs::File::create(&to)?;
                std::io::copy(&mut entry, &mut f)?;
                // executable stays executable; npm's tarballs carry odd modes, so only that bit is kept
                let exec = entry.header().mode().map(|m| m & 0o111 != 0).unwrap_or(false);
                use std::os::unix::fs::PermissionsExt;
                f.set_permissions(std::fs::Permissions::from_mode(if exec { 0o755 } else { 0o644 }))?;
            }
            _ => {}
        }
    }
    std::fs::write(tmp.join(OK), b"")?;
    let _ = std::fs::remove_dir_all(dir);
    std::fs::rename(&tmp, dir)
}

pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("part-{}", std::process::id()));
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(tmp, path)
}

// ---- the project's node_modules -------------------------------------------------------------------------------------
