//! raea's npm: what a project's package.json depends on, resolved, fetched and laid out in its node_modules, with no
//! Node, npm or bun (Shane, 10-21: "we are sunsetting bun ... in the engine itself so it can be extended with npm
//! libraries"). Deno's resolver decides the versions (deno_npm: semver, dist-tags, peer dependencies, dedup); raea
//! fetches each package's tarball once into one store shared by every project (~/.raea/npm/store), checked against
//! the registry's integrity hash, and lays the project's node_modules out as pnpm and Deno do: each package a
//! copy-on-write clone under node_modules/.raea/<id>/node_modules/<name>, its own dependencies symlinked beside it, and
//! the project's direct ones symlinked at the top. Rolldown resolves through that layout as through npm's (engine/bundle:
//! a package sees only what it declares). The resolution is kept in raea.lock beside package.json, so an install with
//! nothing new asks the registry nothing.
//!
//! Install scripts never run: raea has no Node to run them, and a package that needs one (a native binary fetched in
//! postinstall) is named in the report. Neither are `bin` links made, for the same reason.
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use deno_npm::registry::{NpmPackageInfo, NpmPackageVersionDistInfoIntegrity, NpmRegistryApi, NpmRegistryPackageInfoLoadError};
use deno_npm::resolution::{AddPkgReqsOptions, NpmResolutionSnapshot, NpmVersionResolver, SerializedNpmResolutionSnapshot};
use deno_npm::{NpmPackageId, NpmSystemInfo};
use deno_semver::package::PackageReq;
use futures::future::{FutureExt, LocalBoxFuture, Shared};
use futures::StreamExt;
use serde_json::value::RawValue;

mod layout;
mod package;
mod registry;
mod store;

use layout::*;
use package::*;
use registry::*;
use store::*;

type R<T> = Result<T, String>;

/// The registry asked, unless RAEA_NPM_REGISTRY names another (a mirror, a private one).
const REGISTRY: &str = "https://registry.npmjs.org";
/// Tarballs fetched at once.
const FETCHES: usize = 16;

/// What an install did.
#[derive(Debug, Default)]
pub struct Report {
    /// packages in the project's node_modules now
    pub packages: usize,
    /// of those, laid out by this install (the rest were there already)
    pub added: usize,
    /// laid out before and not wanted now, removed
    pub removed: usize,
    /// tarballs fetched into the store (the rest were in it from another project or install)
    pub fetched: usize,
    pub warnings: Vec<String>,
}

impl Report {
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} package{} in node_modules: {} laid out, {} removed, {} fetched",
            self.packages,
            if self.packages == 1 { "" } else { "s" },
            self.added,
            self.removed,
            self.fetched
        );
        for w in &self.warnings {
            s.push_str("\nwarning: ");
            s.push_str(w);
        }
        s
    }
}

/// `add PKG[@RANGE]... [--dev]`, `remove NAME...`, `install [--offline]`, each in the project around the working folder
/// (the nearest folder up with a package.json; the working folder itself when there's none, for a first `add`), or
/// the one `--in DIR` names.
pub fn cli(args: &[String]) -> R<String> {
    let mut dir: Option<PathBuf> = None;
    let mut dev = false;
    let mut offline = false;
    let mut words = Vec::new();
    let mut it = args.iter().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--in" => dir = Some(PathBuf::from(it.next().ok_or("--in needs a folder")?)),
            "--dev" | "-D" => dev = true,
            "--offline" => offline = true,
            _ => words.push(a.clone()),
        }
    }
    let here = std::env::current_dir().map_err(|e| e.to_string())?;
    let root = dir.unwrap_or_else(|| here.ancestors().find(|d| d.join("package.json").is_file()).unwrap_or(&here).to_path_buf());
    let report = match args.first().map(String::as_str) {
        Some("add") if !words.is_empty() => add(&root, &words, dev)?,
        Some("remove") if !words.is_empty() => remove(&root, &words)?,
        Some("install") => install(&root, offline)?,
        _ => return Err("add PKG[@RANGE]... [--dev] | remove NAME... | install [--offline]  [--in DIR]".into()),
    };
    Ok(report.summary())
}

/// Adds packages to package.json ("react", "zod@^3", "@assistant-ui/react@latest", "alias@npm:pkg@1"), each at the
/// version it resolves to (^ of it, as npm writes, when no range was asked), then installs.
pub fn add(root: &Path, specs: &[String], dev: bool) -> R<Report> {
    let mut pj = PackageJson::read(root)?;
    let mut wanted = Vec::new();
    for spec in specs {
        let (alias, rest) = split_spec(spec);
        let req = req_of(&alias, if rest.is_empty() { "latest" } else { &rest })?;
        wanted.push((alias, rest, req));
    }
    // resolve what was asked first, so package.json says the version it got
    let reqs: Vec<PackageReq> = wanted.iter().map(|w| w.2.clone()).collect();
    let snap = runtime()?.block_on(resolve(root, &reqs, NpmResolutionSnapshot::default(), false))?;
    for (alias, rest, req) in &wanted {
        let nv = snap.package_reqs().get(req).ok_or_else(|| format!("{req} didn't resolve"))?;
        let range = match rest.as_str() {
            "" | "latest" => format!("^{}", nv.version),
            r if r.starts_with("npm:") => r.to_string(),
            r => r.to_string(),
        };
        let range = if alias.as_str() != nv.name.as_str() && !range.starts_with("npm:") { format!("npm:{}@{range}", nv.name) } else { range };
        pj.set(if dev { "devDependencies" } else { "dependencies" }, alias, &range);
        pj.unset(if dev { "dependencies" } else { "devDependencies" }, alias);
    }
    pj.write(root)?;
    install(root, false)
}

/// Takes packages out of package.json, then installs (which removes what nothing needs any more).
pub fn remove(root: &Path, names: &[String]) -> R<Report> {
    let mut pj = PackageJson::read(root)?;
    for n in names {
        if !pj.unset("dependencies", n) && !pj.unset("devDependencies", n) {
            return Err(format!("{n} isn't in {}'s package.json", root.display()));
        }
    }
    pj.write(root)?;
    install(root, false)
}

/// Makes the project's node_modules what its package.json says: resolved against raea.lock (the registry asked only for
/// what the lock doesn't have), fetched into the store, laid out, and the lock written.
pub fn install(root: &Path, offline: bool) -> R<Report> {
    // RAEA_OFFLINE: the packages raea already has, as --offline
    let offline = offline || raea_io_home::offline("npm").is_err();
    let pj = PackageJson::read(root)?;
    let mut warnings = Vec::new();
    let mut deps: Vec<(String, PackageReq)> = Vec::new();
    for (alias, spec) in pj.deps() {
        match req_of(&alias, &spec) {
            Ok(r) => deps.push((alias, r)),
            Err(e) => warnings.push(e),
        }
    }
    let rt = runtime()?;
    let lock = Lock::read(root);
    let wanted: HashSet<&PackageReq> = deps.iter().map(|d| &d.1).collect();
    let mut kept = lock.map(|l| l.snapshot).unwrap_or_default();
    kept.root_packages.retain(|req, _| wanted.contains(req));
    prune(&mut kept);
    let base = NpmResolutionSnapshot::new(kept.into_valid().map_err(|e| format!("raea.lock: {e}; delete it to resolve afresh"))?);
    let reqs: Vec<PackageReq> = deps.iter().map(|d| d.1.clone()).collect();
    let snap = rt.block_on(resolve(root, &reqs, base, offline))?;
    Lock { version: 1, snapshot: snap.as_valid_serialized().into_serialized() }.write(root)?;

    // this machine's packages (an optional one for another OS or CPU, left out)
    let here = snap.as_valid_serialized_for_system(&NpmSystemInfo::default()).into_serialized();
    let mut report = rt.block_on(fetch_all(&here))?;
    report.warnings.splice(0..0, warnings);
    let top: Vec<(String, NpmPackageId)> = deps.iter().filter_map(|(alias, req)| here.root_packages.get(req).map(|id| (alias.clone(), id.clone()))).collect();
    lay_out(root, &here, &top, &mut report)?;
    for p in &here.packages {
        if p.has_scripts {
            report.warnings.push(format!("{} has an install script, which raea doesn't run; if it fetches a native binary, it won't work here", p.id.nv));
        }
        if p.is_deprecated {
            report.warnings.push(format!("{} is deprecated", p.id.nv));
        }
    }
    Ok(report)
}

// ---- resolving -------------------------------------------------------------------------------------------------------

fn runtime() -> R<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())
}

/// `reqs` resolved onto `base` (what the lock already has resolves without the registry).
async fn resolve(root: &Path, reqs: &[PackageReq], base: NpmResolutionSnapshot, offline: bool) -> R<NpmResolutionSnapshot> {
    let missing: Vec<PackageReq> = reqs.iter().filter(|r| !base.package_reqs().contains_key(*r)).cloned().collect();
    if missing.is_empty() {
        return Ok(base);
    }
    let api = Registry::new(offline)?;
    let resolver = NpmVersionResolver::default();
    let out = base.add_pkg_reqs(&api, AddPkgReqsOptions { package_reqs: &missing, version_resolver: &resolver, should_dedup: true }, None).await;
    for (req, r) in missing.iter().zip(&out.results) {
        if let Err(e) = r {
            return Err(format!("{req}: {e} (in {})", root.display()));
        }
    }
    out.dep_graph_result.map_err(|e| e.to_string())
}

/// Drops the packages no root reaches (a lock's, once package.json stopped asking for some).
fn prune(s: &mut SerializedNpmResolutionSnapshot) {
    let by_id: HashMap<NpmPackageId, usize> = s.packages.iter().enumerate().map(|(i, p)| (p.id.clone(), i)).collect();
    let mut seen: HashSet<NpmPackageId> = HashSet::new();
    let mut todo: VecDeque<NpmPackageId> = s.root_packages.values().cloned().collect();
    while let Some(id) = todo.pop_front() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(&i) = by_id.get(&id) {
            todo.extend(s.packages[i].dependencies.values().cloned());
        }
    }
    s.packages.retain(|p| seen.contains(&p.id));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specs() {
        assert_eq!(split_spec("react"), ("react".into(), "".into()));
        assert_eq!(split_spec("@scope/pkg@^2"), ("@scope/pkg".into(), "^2".into()));
        assert_eq!(split_spec("alias@npm:pkg@1"), ("alias".into(), "npm:pkg@1".into()));
        assert_eq!(req_of("zod", "^3.22").unwrap().to_string(), "zod@^3.22");
        assert_eq!(req_of("x", "npm:@a/b@1").unwrap().name.as_str(), "@a/b");
        assert!(req_of("x", "github:a/b").is_err());
        assert!(req_of("x", "file:../x").is_err());
    }

    #[test]
    fn relative_paths() {
        assert_eq!(relative(Path::new("/p/node_modules/@s"), Path::new("/p/node_modules/.raea/a/node_modules/a")), PathBuf::from("../.raea/a/node_modules/a"));
    }

    #[test]
    fn package_json_keeps_its_text() {
        let dir = std::env::temp_dir().join(format!("raea-npm-pj-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), "{\n  \"name\": \"x\",\n  \"scripts\": {\"b\": \"1\", \"a\": \"2\"}\n}\n").unwrap();
        let mut pj = PackageJson::read(&dir).unwrap();
        pj.set("dependencies", "zod", "^3.0.0");
        pj.write(&dir).unwrap();
        let t = std::fs::read_to_string(dir.join("package.json")).unwrap();
        assert!(t.contains("\"scripts\": {\"b\": \"1\", \"a\": \"2\"}"), "{t}");
        assert!(t.contains("\"dependencies\": {\n    \"zod\": \"^3.0.0\"\n  }"), "{t}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
