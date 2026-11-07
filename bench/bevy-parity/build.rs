//! Each example in examples.list, copied from Bevy's checkout (BEVY_SRC, default ~/.cache/raea-vendor/bevy-0.20.0)
//! into a module: its inner doc comments and attributes become plain comments, its `App::new()` is the host's
//! `crate::parity_app()` (the fixed clock and the probe, then the example's own plugins) and its `main` is public.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    let here = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = std::env::var("BEVY_SRC")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/raea-vendor/bevy-0.20.0"));
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    println!("cargo:rerun-if-changed=examples.list");
    println!("cargo:rerun-if-env-changed=BEVY_SRC");
    // without Bevy's source (a fresh clone of the public repository) the workspace still builds: the host is built
    // with no examples, and says so when one is asked for
    let have = src.join("examples").is_dir();
    println!("cargo:rerun-if-changed={}", src.join("examples").display());
    if !have {
        println!("cargo:warning=raea-bevy-parity: no Bevy v0.20.0 at {} (set BEVY_SRC), so no examples are built in", src.display());
    }
    let list = if have { std::fs::read_to_string(here.join("examples.list")).unwrap() } else { String::new() };
    let (mut mods, mut arms) = (String::new(), String::new());
    for path in list.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let file = src.join("examples").join(format!("{path}.rs"));
        println!("cargo:rerun-if-changed={}", file.display());
        let code = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e} (clone bevy v0.20.0 to BEVY_SRC)", file.display()));
        let code: String = code
            .lines()
            .map(|l| {
                let t = l.trim_start();
                if t.starts_with("//!") || t.starts_with("#![") { format!("// {t}") } else { l.to_string() }
            })
            .collect::<Vec<_>>()
            .join("\n")
            // include_str!/include_bytes! paths are relative to the example, not to OUT_DIR
            .replace("include_str!(\"", &format!("include_str!(\"{}/", file.parent().unwrap().display()))
            .replace("include_bytes!(\"", &format!("include_bytes!(\"{}/", file.parent().unwrap().display()))
            .replace("App::new()", "crate::parity_app()")
            .replace("\nfn main()", "\npub fn main()");
        let m = format!("ex_{}", path.replace('/', "__"));
        std::fs::write(out.join(format!("{m}.rs")), code).unwrap();
        writeln!(mods, "#[allow(dead_code, unused_imports, non_snake_case, clippy::all)]\npub mod {m} {{ include!(concat!(env!(\"OUT_DIR\"), \"/{m}.rs\")); }}").unwrap();
        writeln!(arms, "        \"{path}\" => {m}::main(),").unwrap();
    }
    let reg = format!("{mods}\n/// Where the examples came from: empty when Bevy's source wasn't found at build time.\npub const BEVY_SRC: &str = {:?};\n/// Runs the example at `path`, false when the host doesn't build it.\npub fn run(path: &str) -> bool {{\n    match path {{\n{arms}        _ => return false,\n    }}\n    true\n}}\n", if have { src.display().to_string() } else { String::new() });
    std::fs::write(out.join("registry.rs"), reg).unwrap();
}
