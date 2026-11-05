//! A project's node_modules laid out as pnpm and Deno do: each package a copy-on-write clone under .raea/, its dependencies symlinked beside it.

use super::*;

/// The folder a package is laid out under in node_modules/.raea: its id, peers and all, as one path segment.
pub(crate) fn folder(id: &NpmPackageId) -> String {
    let s = flat(&id.as_serialized());
    if s.len() <= 120 {
        return s;
    }
    use sha2::Digest;
    let h = hex::encode(sha2::Sha256::digest(s.as_bytes()));
    format!("{}@{}_{}", flat(&id.nv.name), id.nv.version, &h[..16])
}

/// node_modules made the snapshot's: each package cloned from the store into .raea/<folder>/node_modules/<name> with its
/// dependencies linked beside it, the project's own at the top, and what the snapshot no longer has removed.
pub(crate) fn lay_out(root: &Path, s: &SerializedNpmResolutionSnapshot, top: &[(String, NpmPackageId)], report: &mut Report) -> R<()> {
    let nm = root.join("node_modules");
    let vs = nm.join(".raea");
    std::fs::create_dir_all(&vs).map_err(|e| format!("{}: {e}", vs.display()))?;
    let io = |what: &Path, e: std::io::Error| format!("{}: {e}", what.display());
    let have: HashSet<&NpmPackageId> = s.packages.iter().map(|p| &p.id).collect();
    let home_of = |id: &NpmPackageId| vs.join(folder(id)).join("node_modules").join(id.nv.name.as_str());
    let mut keep: HashSet<String> = HashSet::new();
    for p in &s.packages {
        let f = folder(&p.id);
        keep.insert(f.clone());
        let pkg = home_of(&p.id);
        let store = stored(&p.id);
        if !pkg.join(OK).is_file() {
            if p.dist.is_none() {
                report.warnings.push(format!("{}: a linked package, which raea doesn't lay out", p.id.nv));
                continue;
            }
            let _ = std::fs::remove_dir_all(vs.join(&f));
            std::fs::create_dir_all(pkg.parent().unwrap()).map_err(|e| io(&pkg, e))?;
            clone_dir(&store, &pkg).map_err(|e| io(&pkg, e))?;
            report.added += 1;
        }
        for (alias, dep) in &p.dependencies {
            if !have.contains(dep) || alias.as_str() == p.id.nv.name.as_str() {
                continue;
            }
            let at = vs.join(&f).join("node_modules").join(alias.as_str());
            link(&at, &home_of(dep)).map_err(|e| io(&at, e))?;
        }
    }
    report.packages = s.packages.len();
    // the project's own, at the top
    let mut tops: HashSet<PathBuf> = HashSet::new();
    for (alias, id) in top {
        let at = nm.join(alias);
        // a package another tool installed there (bun's, npm's) gives way to raea's
        if at.is_dir() && !at.is_symlink() {
            std::fs::remove_dir_all(&at).map_err(|e| io(&at, e))?;
        }
        link(&at, &home_of(id)).map_err(|e| io(&at, e))?;
        tops.insert(at);
    }
    // what's not wanted any more: .raea folders, and top-level links into .raea
    for e in std::fs::read_dir(&vs).map_err(|e| io(&vs, e))?.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !keep.contains(&name) {
            let _ = std::fs::remove_dir_all(e.path());
            report.removed += 1;
        }
    }
    let mut entries: Vec<PathBuf> = Vec::new();
    for e in std::fs::read_dir(&nm).map_err(|e| io(&nm, e))?.flatten() {
        let p = e.path();
        if e.file_name().to_string_lossy().starts_with('@') && p.is_dir() && !p.is_symlink() {
            entries.extend(std::fs::read_dir(&p).map_err(|e| io(&p, e))?.flatten().map(|e| e.path()));
        } else {
            entries.push(p);
        }
    }
    for p in entries {
        let into_raea = std::fs::read_link(&p).map(|t| t.components().any(|c| c.as_os_str() == ".raea")).unwrap_or(false);
        if into_raea && !tops.contains(&p) {
            let _ = unlink(&p);
            // a scope (@assistant-ui) whose last package went: gone too (remove_dir fails while it holds anything)
            if let Some(scope) = p.parent().filter(|s| *s != nm && s.file_name().is_some_and(|n| n.to_string_lossy().starts_with('@'))) {
                let _ = std::fs::remove_dir(scope);
            }
        }
    }
    Ok(())
}

/// A symlink at `at` to `to`, relative (the project can move), replacing what was there. On Windows it's a junction to
/// the absolute folder, as pnpm makes there: a symlink needs Developer Mode or an administrator, a junction neither.
pub(crate) fn link(at: &Path, to: &Path) -> std::io::Result<()> {
    let parent = at.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    let rel = relative(parent, to);
    let want = if cfg!(windows) { to.to_path_buf() } else { rel.clone() };
    if std::fs::read_link(at).ok().as_deref() == Some(want.as_path()) {
        return Ok(());
    }
    if at.is_symlink() {
        unlink(at)?;
    } else if at.is_file() {
        std::fs::remove_file(at)?;
    } else if at.is_dir() {
        std::fs::remove_dir_all(at)?;
    }
    #[cfg(unix)]
    return std::os::unix::fs::symlink(rel, at);
    #[cfg(windows)]
    return junction::create(to, at);
}

/// A link made by `link` removed, never what it points at: a junction is a folder to Windows, so it goes by remove_dir
/// (which leaves the target alone).
fn unlink(at: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    if std::fs::remove_dir(at).is_ok() {
        return Ok(());
    }
    std::fs::remove_file(at)
}

/// `to` as a path from the folder `from` (both absolute).
pub(crate) fn relative(from: &Path, to: &Path) -> PathBuf {
    let f: Vec<Component> = from.components().collect();
    let t: Vec<Component> = to.components().collect();
    let common = f.iter().zip(&t).take_while(|(a, b)| a == b).count();
    let mut out = PathBuf::new();
    for _ in common..f.len() {
        out.push("..");
    }
    for c in &t[common..] {
        out.push(c.as_os_str());
    }
    out
}

/// A folder copied: an APFS clone on macOS (no bytes copied until one side changes), else file by file.
pub(crate) fn clone_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::ffi::OsStrExt;
        let a = std::ffi::CString::new(from.as_os_str().as_bytes())?;
        let b = std::ffi::CString::new(to.as_os_str().as_bytes())?;
        if unsafe { libc::clonefile(a.as_ptr(), b.as_ptr(), 0) } == 0 {
            return Ok(());
        }
    }
    copy_dir(from, to)
}

pub(crate) fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let (a, b) = (e.path(), to.join(e.file_name()));
        if e.file_type()?.is_dir() {
            copy_dir(&a, &b)?;
        } else {
            std::fs::copy(&a, &b)?;
        }
    }
    Ok(())
}

// ---- package.json and raea.lock --------------------------------------------------------------------------------------
