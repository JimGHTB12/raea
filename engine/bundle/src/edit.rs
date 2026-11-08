//! A composition's bundle (raea's bundle_fresh), with the edit's own packages: its sources are copied into a
//! work folder whose node_modules is the engine's, so `import { motion } from "motion/react"` found nothing the edit
//! had installed. A bare import from the edit's sources resolves from the edit's own folder first (its node_modules,
//! then its parents', as Node and Vite would), then the engine's; the packages the engine carries (react, mediabunny,
//! bevy-react...) resolve to its copies from any package, so a bundle has one React however the edit installed its own.
use rolldown_plugin::{HookResolveIdArgs, HookResolveIdOutput, HookResolveIdReturn, HookUsage, Plugin, PluginContext, PluginContextResolveOptions};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// Where a composition's packages come from
#[derive(Debug, Clone, Default)]
pub struct Edit {
    /// the edit's own folder (not the work folder's copy)
    pub dir: PathBuf,
    /// sdk/react/js: the engine's node_modules
    pub js: PathBuf,
    /// the work folder (the entry's): the edit's copied sources and the SDK's
    pub work: PathBuf,
}

#[derive(Debug)]
pub struct EditResolve(Edit);

impl EditResolve {
    /// its folders as the resolver names modules: symlinks resolved (the Mac's temp folder is /var, a link to /private/var),
    /// and on Windows without canonicalize()'s verbatim `\\?\` prefix, which the resolver never writes: with it no
    /// importer was ever under the work folder, so a live game's own `import "react"` went to the engine's React instead of
    /// the live runtime its tsconfig paths name, and `raea play` threw "not a function" before the first frame
    pub fn new(e: Edit) -> Self {
        let real = |p: PathBuf| p.canonicalize().map(raea_io_home::plain).unwrap_or(p);
        EditResolve(Edit { dir: real(e.dir), js: real(e.js), work: real(e.work) })
    }

    /// whether an importer is one of the edit's own sources: under the work folder and not in a package. The importer
    /// is named as real as the folders, since the resolver writes a path as it was given: on GitHub's Windows runners
    /// TEMP is the 8.3 `C:\Users\RUNNER~1\..`, which canonicalize() spells `runneradmin`, so no importer was under the
    /// work folder and every live play threw "not a function" (the engine's React, with no `__h`) before its first frame
    fn from_edit(&self, imp: &str) -> bool {
        let named = Path::new(imp);
        let imp = named.canonicalize().map(raea_io_home::plain).unwrap_or_else(|_| named.to_path_buf());
        imp.starts_with(&self.0.work) && !imp.components().any(|c| c.as_os_str() == "node_modules")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_folders_are_named_as_the_resolver_names_modules() {
        assert_eq!(raea_io_home::plain(PathBuf::from(r"\\?\C:\Temp\raea-cli-1")), PathBuf::from(r"C:\Temp\raea-cli-1"));
        assert_eq!(raea_io_home::plain(PathBuf::from(r"\\?\UNC\server\share")), PathBuf::from(r"\\?\UNC\server\share"));
        assert_eq!(raea_io_home::plain(PathBuf::from("/private/var/x")), PathBuf::from("/private/var/x"));
        // a real folder: canonicalized on Windows that was \\?\C:\..., which no importer starts with
        let dir = std::env::temp_dir();
        let r = EditResolve::new(Edit { dir: dir.clone(), js: dir.clone(), work: dir.clone() });
        assert!(!r.0.work.display().to_string().starts_with(r"\\?\"), "{}", r.0.work.display());
        assert!(dir.join("entry.tsx").starts_with(&r.0.work) || cfg!(unix), "{} under {}", dir.display(), r.0.work.display());
    }

    /// an importer named by another spelling of the work folder (a symlink here; an 8.3 short name on Windows) is still
    /// the edit's, and a package under it isn't
    #[test]
    fn an_importer_by_another_name_for_the_work_folder_is_the_edits() {
        let d = std::env::temp_dir().join(format!("raea-edit-alias-{}", std::process::id()));
        let work = d.join("work");
        std::fs::create_dir_all(work.join("node_modules/pkg")).unwrap();
        std::fs::write(work.join("entry.tsx"), "").unwrap();
        std::fs::write(work.join("node_modules/pkg/index.js"), "").unwrap();
        let alias = d.join("alias");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&work, &alias).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&work, &alias).unwrap_or_else(|_| std::fs::create_dir_all(&alias).unwrap());
        let r = EditResolve::new(Edit { dir: work.clone(), js: work.clone(), work: alias.clone() });
        if alias.is_symlink() {
            assert!(r.from_edit(&work.join("entry.tsx").to_string_lossy()), "{} under {}", work.display(), r.0.work.display());
        }
        assert!(r.from_edit(&alias.join("entry.tsx").to_string_lossy()));
        assert!(!r.from_edit(&alias.join("node_modules").join("pkg").join("index.js").to_string_lossy()));
        let _ = std::fs::remove_dir_all(&d);
    }
}

/// `spec` resolved by Rolldown's resolver as if imported from a file in `dir`
async fn resolve_from(ctx: &PluginContext, spec: &str, dir: &Path) -> anyhow::Result<Option<HookResolveIdOutput>> {
    let importer = dir.join("_raea.js");
    let opts = PluginContextResolveOptions { skip_self: true, ..Default::default() };
    match ctx.resolve(spec, Some(&importer.to_string_lossy()), Some(opts)).await? {
        Ok(r) => Ok(Some(HookResolveIdOutput::from_id(r.id.as_str()))),
        Err(_) => Ok(None),
    }
}

/// Packages a composition's bundle takes from raea in place of their own, by the stand-in module under raea's home:
/// ones that need what a native render's JS hasn't (a DOM, web workers) to do what raea's engine does itself.
/// troika-three-text (drei's <Text>) lays text out in a worker onto a canvas; raea's R3F reader draws its Text as
/// raea's own text (sdk/r3f/src/input/troika.ts, text.ts; docs/r3f.feature R3F13).
const STAND_INS: &[(&str, &str)] = &[("troika-three-text", "sdk/r3f/src/input/troika.ts")];

impl Plugin for EditResolve {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("raea:edit")
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::ResolveId
    }

    async fn resolve_id(&self, ctx: &PluginContext, args: &HookResolveIdArgs<'_>) -> HookResolveIdReturn {
        let e = &self.0;
        let spec = args.specifier;
        let Some(imp) = args.importer else { return Ok(None) };
        // raea's home is sdk/react/js's grandparent's parent
        if let Some((_, own)) = STAND_INS.iter().find(|(p, _)| *p == spec) {
            if let Some(home) = e.js.ancestors().nth(3) {
                let f = home.join(own);
                if f.is_file() {
                    return Ok(Some(HookResolveIdOutput::from_id(f.to_string_lossy().as_ref())));
                }
            }
        }
        let bare = spec.starts_with('@') || spec.starts_with(|c: char| c.is_ascii_lowercase());
        if !bare || spec.starts_with("@raea/") || spec.contains(':') {
            return Ok(None);
        }
        let pkg = if spec.starts_with('@') { spec.splitn(3, '/').take(2).collect::<Vec<_>>().join("/") } else { spec.split('/').next().unwrap_or(spec).to_string() };
        let engine = crate::script::SINGLETONS.contains(&pkg.as_str()) && e.js.join("node_modules").join(&pkg).exists();
        let from_edit = self.from_edit(imp);
        match (from_edit, engine) {
            // the edit's sources: the engine's copy by the work folder's own resolution (its tsconfig paths point a live
            // bundle's react at the live runtime)
            (true, true) => Ok(None),
            (true, false) => resolve_from(ctx, spec, &e.dir).await,
            // a package (the edit's or the engine's) importing one the engine carries: the engine's
            (false, true) => resolve_from(ctx, spec, &e.js).await,
            (false, false) => Ok(None),
        }
    }
}
