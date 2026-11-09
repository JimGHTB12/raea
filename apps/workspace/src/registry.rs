//! A project's compositions, components and controllers (docs/projects.feature PJ8): raea-app-registry's reading of
//! its files, with the project's controller overrides from .raea/project checked against the controllers there.
use std::path::Path;

use crate::Eval;
pub use raea_app_registry::{Component, Composition, Controller, Fix, How, Level, Lint, Prop, Registry, apply};

/// The folder's registry.
pub fn scan(dir: &Path) -> Registry {
    let ids: Vec<String> = crate::controllers::overrides(dir).into_iter().filter_map(|(path, _)| path.split('.').next().map(str::to_string)).collect();
    raea_app_registry::scan(dir, &ids)
}

/// What `raea context` reads of the workspace, as (section, text): its controllers with this project's overrides, and
/// the project's compositions, components and controllers as raea reads them.
pub fn context(from: &Path, eval: Eval) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Some(text) = crate::summary(from, eval) {
        out.push(("CONTROLLERS (raea controller ls; a project's own values marked)".to_string(), text));
    }
    if let Some(dir) = crate::doctor::registry_dir(from) {
        out.push(("PROJECT FILES (raea project scan)".to_string(), scan(&dir).summary()));
    }
    out
}
