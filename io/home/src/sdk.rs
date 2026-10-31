//! What the SDK's import names mean: sdk/tsconfig.base.json's `paths`, the one list tsc, bun and both bundlers read
//! (docs/sdk.feature S1d). `raea` is the umbrella's prelude, `raea/<module>` a module, `@raea/<package>` a package.

use std::path::{Path, PathBuf};

/// The list itself, file and key, in sdk/ (`sdk` is the folder holding tsconfig.base.json)
pub const LIST: &str = "tsconfig.base.json";

/// `paths` as (pattern, target relative to `sdk`), in the file's order; empty when the file can't be read
pub fn paths(sdk: &Path) -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(sdk.join(LIST)) else { return vec![] };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else { return vec![] };
    let Some(map) = json["compilerOptions"]["paths"].as_object() else { return vec![] };
    map.iter().filter_map(|(k, v)| Some((k.clone(), v.get(0)?.as_str()?.trim_start_matches("./").to_string()))).collect()
}

/// `spec`'s file as tsc resolves it from the list: an exact key first, else the `prefix/*` key with the longest
/// prefix, its `*` taking the rest; then the target as given, with .ts or .tsx, or its folder's index
pub fn resolve(sdk: &Path, spec: &str) -> Option<PathBuf> {
    let list = paths(sdk);
    let target = match list.iter().find(|(k, _)| k == spec) {
        Some((_, t)) => t.clone(),
        None => {
            // (the prefix's length, the target) of each `prefix*` key `spec` extends, the longest prefix taken
            let wild = list.iter().filter_map(|(k, t)| k.strip_suffix('*').filter(|p| spec.len() > p.len() && spec.starts_with(p)).map(|p| (p.len(), t)));
            let (n, t) = wild.max_by_key(|&(n, _)| n)?;
            t.replacen('*', &spec[n..], 1)
        }
    };
    let base = sdk.join(&target);
    [base.clone(), sdk.join(format!("{target}.ts")), sdk.join(format!("{target}.tsx")), base.join("index.ts"), base.join("index.tsx")].into_iter().find(|p| p.is_file())
}

/// The packages the list names (`@raea/<name>` keys), each as (name, its source folder relative to `sdk`):
/// what a composition's bundle copies beside the umbrella
pub fn packages(sdk: &Path) -> Vec<(String, String)> {
    paths(sdk)
        .into_iter()
        .filter(|(k, _)| k.starts_with("@raea/") && !k.contains('*'))
        .filter_map(|(k, t)| Some((k, Path::new(&t).parent()?.to_string_lossy().into_owned())))
        .collect()
}
