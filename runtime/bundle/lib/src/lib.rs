//! raea-bundle (engine/bundle: Rolldown, lightningcss, the Z3 compile) as a library loaded at run time, never linked: its
//! crates' Cargo features would reach raea's own (f60e0b7). Found at RAEA_BUNDLE_LIB or beside this binary. A crate of its own
//! (libloading and serde_json, nothing of raea's) so a host that must not reach Bevy (the native target's tests,
//! docs/targets.feature T1c) loads it as runtime/bundle does.

/// An app's script, as both targets run it
pub mod app;

use std::ffi::{c_char, CStr, CString};
use std::path::{Path, PathBuf};

pub struct Lib {
    pub path: PathBuf,
    bundle: unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char,
    free: unsafe extern "C" fn(*mut c_char),
    _lib: libloading::Library,
}

pub fn lib() -> Result<&'static Lib, String> {
    static LIB: std::sync::OnceLock<Result<Lib, String>> = std::sync::OnceLock::new();
    LIB.get_or_init(|| {
        // libraea_bundle.dylib, .so on Linux, raea_bundle.dll on Windows
        let name = libloading::library_filename("raea_bundle");
        let mut at: Vec<PathBuf> = std::env::var_os("RAEA_BUNDLE_LIB").map(PathBuf::from).into_iter().collect();
        // beside the binary (runtime/react/build.rs puts it there), or a folder up (test binaries run from deps/); the binary
        // itself, not a link to it (an install links ~/.local/bin/raea to ~/.raea/bin)
        if let Some(dir) = std::env::current_exe().ok().map(|e| e.canonicalize().unwrap_or(e)).and_then(|e| e.parent().map(Path::to_path_buf)) {
            at.push(dir.join(&name));
            at.extend(dir.parent().map(|d| d.join(&name)));
        }
        // no bundler at all when it isn't there: raea doesn't fall back to bun
        let path = at.into_iter().find(|p| p.is_file()).ok_or(format!("raea can't bundle: no {} beside {} (runtime/react/build.rs builds it with raea-runtime-react's modules feature; RAEA_BUNDLE_LIB names another)", name.to_string_lossy(), std::env::current_exe().map(|e| e.display().to_string()).unwrap_or_default()))?;
        // SAFETY: raea-bundle's own library, whose C API (engine/bundle/src/lib.rs ffi) is checked by its version first
        unsafe {
            let lib = libloading::Library::new(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let api: libloading::Symbol<unsafe extern "C" fn() -> u32> = lib.get(b"raea_bundle_api").map_err(|e| format!("{}: {e}", path.display()))?;
            if api() != 1 {
                return Err(format!("{}: API {} (this raea wants 1): build bundle/ again", path.display(), api()));
            }
            let bundle = *lib.get(b"raea_bundle").map_err(|e| format!("{}: {e}", path.display()))?;
            let free = *lib.get(b"raea_bundle_free").map_err(|e| format!("{}: {e}", path.display()))?;
            Ok(Lib { path, bundle, free, _lib: lib })
        }
    })
    .as_ref()
    .map_err(|e| e.clone())
}

/// `entry` bundled with `opts` (raea-bundle's: {"z3"}, {"script", "defines"}): the script and the CSS its modules
/// imported
pub fn bundle(entry: &Path, opts: serde_json::Value) -> Result<(String, String), String> {
    let lib = lib()?;
    let e = CString::new(entry.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    let o = CString::new(opts.to_string()).map_err(|e| e.to_string())?;
    // SAFETY: NUL-terminated strings in; the reply is the library's, freed by it
    let reply = unsafe {
        let p = (lib.bundle)(e.as_ptr(), o.as_ptr());
        if p.is_null() {
            return Err("raea-bundle gave no reply".into());
        }
        let s = CStr::from_ptr(p).to_string_lossy().into_owned();
        (lib.free)(p);
        s
    };
    let v: serde_json::Value = serde_json::from_str(&reply).map_err(|e| format!("raea-bundle's reply: {e}"))?;
    if let Some(err) = v["error"].as_str() {
        return Err(err.to_string());
    }
    Ok((v["js"].as_str().unwrap_or_default().to_string(), v["css"].as_str().unwrap_or_default().to_string()))
}
