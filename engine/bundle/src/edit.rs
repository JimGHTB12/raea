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
    /// its folders as the resolver names modules: symlinks resolved (the Mac's temp folder is /var, a link to /private/var)
    pub fn new(e: Edit) -> Self {
        let real = |p: PathBuf| p.canonicalize().map(plain).unwrap_or(p);
        EditResolve(Edit { dir: real(e.dir), js: real(e.js), work: real(e.work) })
    }
}

/// A path without the verbatim `\\?\` prefix Windows' canonicalize() puts before a drive path. The resolver names
/// modules without it, so with it no importer was ever under the work folder: a live game's own `import "react"`
/// went to the engine's React instead of the live runtime its tsconfig paths name, and `raea play` threw "not a
/// function" before the first frame. Any other path, a `\\?\UNC\` one included, is kept as it is.
fn plain(p: PathBuf) -> PathBuf {
    match p.to_str().and_then(|s| s.strip_prefix(r"\\?\")).filter(|r| r.as_bytes().get(1) == Some(&b':')) {
        Some(r) => r.into(),
        None => p,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn its_folders_are_named_as_the_resolver_names_modules() {
        assert_eq!(plain(PathBuf::from(r"\\?\C:\Temp\raea-cli-1")), PathBuf::from(r"C:\Temp\raea-cli-1"));
        assert_eq!(plain(PathBuf::from(r"\\?\UNC\server\share")), PathBuf::from(r"\\?\UNC\server\share"));
        assert_eq!(plain(PathBuf::from("/private/var/x")), PathBuf::from("/private/var/x"));
        // a real folder: canonicalized on Windows that was \\?\C:\..., which no importer starts with
        let dir = std::env::temp_dir();
        let r = EditResolve::new(Edit { dir: dir.clone(), js: dir.clone(), work: dir.clone() });
        assert!(!r.0.work.display().to_string().starts_with(r"\\?\"), "{}", r.0.work.display());
        assert!(dir.join("entry.tsx").starts_with(&r.0.work) || cfg!(unix), "{} under {}", dir.display(), r.0.work.display());
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
        let bare = spec.starts_with('@') || spec.starts_with(|c: char| c.is_ascii_lowercase());
        if !bare || spec.starts_with("@raea/") || spec.contains(':') {
            return Ok(None);
        }
        let pkg = if spec.starts_with('@') { spec.splitn(3, '/').take(2).collect::<Vec<_>>().join("/") } else { spec.split('/').next().unwrap_or(spec).to_string() };
        let engine = crate::script::SINGLETONS.contains(&pkg.as_str()) && e.js.join("node_modules").join(&pkg).exists();
        let from_edit = Path::new(imp).starts_with(&e.work) && !imp.contains("/node_modules/");
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
