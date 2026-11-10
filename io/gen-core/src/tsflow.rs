//! A TypeScript flow (raea/gen's `flow()` as the default export) read with bun under the SDK's import names. Those are
//! sdk/tsconfig.base.json's `paths`, but raea/gen itself is the private side's (sdk/gen), so the public file leaves it
//! out, and since the public-first split (11-06) a flow outside raea's tree couldn't import it: the launch film's flows
//! imported ~/raea/sdk/gen by its absolute path instead, which the app's lint flags (11-09). So the runner hands bun its
//! own override: the SDK's file with the SDK's folder as `baseUrl`, and raea/gen added when the SDK has it.

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value as J, json};

/// raea/gen's entry, under the SDK
const GEN: &str = "gen/src/index.ts";

/// The override bun reads for a flow: the SDK's tsconfig with `baseUrl` the SDK's folder (the override is written
/// elsewhere, and `paths` are relative) and raea/gen and @raea/gen mapped when sdk/gen is there. None when the SDK's
/// file can't be read as JSON; the caller then passes that file as it is.
pub fn override_for(sdk: &Path) -> Option<J> {
    let text = std::fs::read_to_string(sdk.join(raea_io_home::sdk::LIST)).ok()?;
    let mut j: J = serde_json::from_str(&text).ok()?;
    let opts = j.as_object_mut()?.entry("compilerOptions").or_insert_with(|| json!({})).as_object_mut()?;
    opts.insert("baseUrl".into(), json!(sdk.display().to_string()));
    if sdk.join(GEN).is_file() {
        let paths = opts.entry("paths").or_insert_with(|| json!({})).as_object_mut()?;
        for name in ["raea/gen", "@raea/gen"] {
            paths.entry(name).or_insert_with(|| json!([format!("./{GEN}")]));
        }
    }
    Some(j)
}

/// The flow file's default export as JSON (called first when it's a function), run with bun under the SDK at `sdk`.
pub fn run(file: &Path, sdk: &Path) -> Result<J, String> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let id = format!("{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed));
    let tmp = std::env::temp_dir();
    let entry = tmp.join(format!("raea-gen-flow-{id}.ts"));
    let js = "const m = await import(process.argv[2]);\nconst f = typeof m.default === \"function\" ? await m.default() : await m.default;\nconsole.log(JSON.stringify(f));\n";
    std::fs::write(&entry, js).map_err(|e| format!("{}: {e}", entry.display()))?;
    let config = override_for(sdk).and_then(|j| {
        let c = tmp.join(format!("raea-gen-tsconfig-{id}.json"));
        std::fs::write(&c, j.to_string()).ok().map(|()| c)
    });
    let out = std::process::Command::new("bun")
        .arg("--tsconfig-override")
        .arg(config.clone().unwrap_or_else(|| sdk.join(raea_io_home::sdk::LIST)))
        .arg(&entry)
        .arg(file)
        .output();
    let _ = std::fs::remove_file(&entry);
    if let Some(c) = config {
        let _ = std::fs::remove_file(c);
    }
    let out = out.map_err(|e| format!("a TypeScript flow runs with bun, which didn't start ({e})"))?;
    // bun reports an override that isn't the file's own folder as an internal error; it isn't one
    let err: String = String::from_utf8_lossy(&out.stderr).lines().filter(|l| !l.contains("directory mismatch")).collect::<Vec<_>>().join("\n");
    if !out.status.success() {
        return Err(err.trim().to_string());
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("its default export isn't a flow ({e})"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A folder of its own under the temp dir, emptied first.
    fn dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("raea-gen-tsflow-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// An SDK with one public name, and raea/gen when `with_gen` (a `flow` that hands back what it's given).
    fn sdk(name: &str, with_gen: bool) -> PathBuf {
        let d = dir(name);
        std::fs::write(d.join(raea_io_home::sdk::LIST), r#"{"compilerOptions":{"strict":true,"paths":{"raea":["./raea/src/prelude.ts"]}}}"#).unwrap();
        if with_gen {
            std::fs::create_dir_all(d.join("gen/src")).unwrap();
            std::fs::write(d.join(GEN), "export const flow = (f: () => unknown) => f;\n").unwrap();
        }
        d
    }

    #[test]
    fn the_override_maps_raea_gen_when_the_sdk_has_it_and_keeps_the_public_names() {
        let with = sdk("with", true);
        let o = override_for(&with).unwrap();
        let opts = &o["compilerOptions"];
        assert_eq!(opts["baseUrl"], json!(with.display().to_string()));
        assert_eq!(opts["paths"]["raea/gen"], json!(["./gen/src/index.ts"]));
        assert_eq!(opts["paths"]["@raea/gen"], json!(["./gen/src/index.ts"]));
        assert_eq!(opts["paths"]["raea"], json!(["./raea/src/prelude.ts"]));
        assert_eq!(opts["strict"], json!(true));
        // a public install without sdk/gen: no name for a file that isn't there
        let without = override_for(&sdk("without", false)).unwrap();
        assert!(without["compilerOptions"]["paths"].get("raea/gen").is_none());
        // a file that isn't JSON: none, and the caller passes it as it is
        let bad = dir("bad");
        std::fs::write(bad.join(raea_io_home::sdk::LIST), "{ not json").unwrap();
        assert!(override_for(&bad).is_none());
    }

    #[test]
    fn a_flow_outside_raeas_tree_imports_raea_gen() {
        if std::process::Command::new("bun").arg("--version").output().is_err() {
            eprintln!("skipped: no bun on this machine");
            return;
        }
        let s = sdk("run", true);
        let project = dir("project");
        let file = project.join("kite.flow.ts");
        std::fs::write(&file, "import { flow } from \"raea/gen\";\nexport default flow(() => ({ flow: 1, steps: {}, out: [] }));\n").unwrap();
        assert_eq!(run(&file, &s).unwrap(), json!({ "flow": 1, "steps": {}, "out": [] }));
        // an import name the SDK hasn't: bun's own error, said
        std::fs::write(&file, "import { flow } from \"raea/nothing\";\nexport default flow;\n").unwrap();
        assert!(run(&file, &s).unwrap_err().contains("raea/nothing"));
    }
}
