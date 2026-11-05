// raea-bundle's library (bundle/: Rolldown, lightningcss, the Z3 compile), built with raea-runtime-react and put beside the
// binaries (target/<profile>/libraea_bundle.dylib, .so on Linux, raea_bundle.dll on Windows), where raea loads it at run
// time (raea-runtime-bundle's inproc). It's never a Cargo dependency: its crates' features would reach raea's own
// (f60e0b7). Only builds that bundle need it (the `modules` feature, on a desktop); the exported game player (no default
// features) and the iPhone go without.
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../engine/bundle/src");
    println!("cargo:rerun-if-changed=../../engine/bundle/Cargo.toml");
    println!("cargo:rerun-if-changed=../../engine/bundle/Cargo.lock");
    println!("cargo:rerun-if-env-changed=RAEA_BUNDLE_SKIP");
    let name = match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("macos") => "libraea_bundle.dylib",
        Ok("linux") => "libraea_bundle.so",
        Ok("windows") => "raea_bundle.dll",
        _ => return,
    };
    if std::env::var_os("CARGO_FEATURE_MODULES").is_none() || std::env::var_os("RAEA_BUNDLE_SKIP").is_some() {
        return;
    }
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    // OUT_DIR is target/<profile>/build/raea-runtime-react-<hash>/out: the binaries are three folders up
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let bins = out.ancestors().nth(3).expect("target/<profile>").to_path_buf();
    // its own target folder inside the same target folder, so its build is kept between raea's
    let target = bins.parent().expect("target").join("raea-bundle");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    // a cross build (Windows from Linux) builds the library for the same target, under target/raea-bundle/<triple>
    let (triple, host) = (std::env::var("TARGET").unwrap(), std::env::var("HOST").unwrap());
    let cross = (triple != host).then_some(triple);
    let mut cmd = Command::new(cargo);
    cmd.args(["build", "--release", "--lib", "--manifest-path"]).arg(manifest.join("../../engine/bundle/Cargo.toml"));
    if let Some(t) = &cross {
        cmd.args(["--target", t]);
    }
    let status = cmd
        .arg("--target-dir")
        .arg(&target)
        // the outer build's target folder and profile settings aren't the library's
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .status()
        .expect("cargo, to build raea-bundle's library");
    assert!(status.success(), "raea-bundle's library didn't build (bundle/); RAEA_BUNDLE_SKIP=1 builds raea without it");
    let lib = cross.map_or(target.clone(), |t| target.join(t)).join("release").join(name);
    // copied beside and renamed into place: a copy over the old file keeps its inode, whose code signature macOS has
    // cached, so the next process to load it was killed (SIGKILL, Code Signature Invalid; the legacy app's panel thread, 10-21)
    let tmp = bins.join(format!("{name}.new"));
    std::fs::copy(&lib, &tmp).and_then(|_| std::fs::rename(&tmp, bins.join(name))).unwrap_or_else(|e| panic!("{} -> {}: {e}", lib.display(), bins.display()));
}
