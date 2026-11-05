use super::*;

/// A headless script bundled for the engine: imported where it lives (its own node_modules resolve), with the engine's
/// resolution on top (raea-bundle's script.rs: the SDK from its source, the test API, raea-runtime-react's node_modules as the
/// fallback); its default export, if a function, is called and awaited, then the run ends.
pub fn bundle_script(ts: &Path, out: &Path) -> R<()> {
    let entry = format!(
        "import * as M from {:?};\nconst call = (globalThis as any).__raea_call;\nPromise.resolve().then(() => (typeof (M as any).default === \"function\" ? (M as any).default() : undefined)).then(() => call(\"done\", JSON.stringify({{ code: 0 }})), (e: any) => {{ call(\"print\", JSON.stringify({{ text: \"error: \" + (e && e.stack ? e.stack : String(e)) }})); call(\"done\", JSON.stringify({{ code: 1 }})); }});\n",
        spec(ts)
    );
    bundle_entry(&entry, out, &[])
}

/// A path as an entry's import specifier. On Windows a canonical path is a verbatim one (\\?\C:\...), which Rolldown
/// can't resolve as an import (`raea run` on windows-latest, PT20); dunce drops the prefix where the plain path means the
/// same file, and changes nothing elsewhere.
fn spec(p: &Path) -> String {
    dunce::simplified(p).display().to_string()
}

/// A react-dom page's bundle (a surface): the module at `src` (its default
/// export a React component) rendered by react-dom into #root with the page's width and height, again on each resize,
/// bundled with its packages (the project's node_modules, raea-io-npm's) into `out`. The bundle's text. A legacy-app panel's
/// and a `<Screen app>`'s.
pub fn bundle_surface(src: &Path, out: &Path) -> R<String> {
    let entry = format!(
        r#"// raea writes this (motion.rs bundle_surface): {src} on react-dom, at the surface's size, again on each resize.
import {{ createElement }} from "react";
import {{ createRoot }} from "react-dom/client";
import Page from {src:?};

const root = createRoot(document.getElementById("root")!);
const draw = () => root.render(createElement(Page as any, {{ width: innerWidth, height: innerHeight }}));
addEventListener("resize", draw);
draw();
"#,
        src = spec(src)
    );
    bundle_entry(&entry, out, &[])?;
    std::fs::read_to_string(out).map_err(|e| format!("{}: {e}", out.display()))
}

/// `entry` (TypeScript source) bundled into `out` by raea-bundle's library (engine/bundle/src/script.rs's resolution); `extra`
/// are bundle.ts's flags (--define K=V, --alias SPEC=PATH, --node-stub a,b).
pub fn bundle_entry(entry: &str, out: &Path, extra: &[&str]) -> R<()> {
    let out = &std::path::absolute(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let work = std::env::temp_dir().join(format!("raea-cli-e{}-{:?}", std::process::id(), std::thread::current().id()));
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|e| e.to_string())?;
    std::fs::write(work.join("entry.ts"), entry).map_err(|e| e.to_string())?;
    let js = raea_io_home::home().join("sdk/react/js");
    std::fs::copy(js.join("tsconfig.json"), work.join("tsconfig.json")).map_err(|e| e.to_string())?;
    // in process by raea-bundle's library: the engine's resolution (engine/bundle/src/script.rs) and the flags
    {
        let (mut defines, mut aliases, mut stubs) = (serde_json::Map::new(), serde_json::Map::new(), Vec::new());
        let mut it = extra.iter();
        while let Some(flag) = it.next() {
            let Some(v) = it.next() else { break };
            match *flag {
                "--define" => {
                    if let Some((k, x)) = v.split_once('=') {
                        defines.insert(k.into(), x.into());
                    }
                }
                "--alias" => {
                    if let Some((k, p)) = v.split_once('=') {
                        aliases.insert(k.into(), std::path::absolute(p).unwrap_or_else(|_| PathBuf::from(p)).display().to_string().into());
                    }
                }
                "--node-stub" => stubs.extend(v.split(',').filter(|s| !s.is_empty()).map(|s| Value::from(s))),
                _ => {}
            }
        }
        let root = raea_io_home::home();
        let opts = json!({
            "script": {"js": js, "motion": root.join("sdk/raea/src"), "shell": root.join("sdk/shell/src/index.ts"),
                "aliases": aliases, "stubs": stubs},
            "defines": defines,
        });
        let r = inproc::bundle(&work.join("entry.ts"), opts);
        let _ = std::fs::remove_dir_all(&work);
        let (code, _css) = r.map_err(|e| format!("couldn't bundle it: {}", e.lines().filter(|l| !l.trim().is_empty()).take(8).collect::<Vec<_>>().join(" | ")))?;
        std::fs::write(out, code).map_err(|e| format!("{}: {e}", out.display()))
    }
}

/// Test files (bun:test style) bundled with the engine's test API and run on its JavaScriptCore: each case a PASS or FAIL line,
/// a summary, and the exit code 1 when any failed. `filter` keeps cases whose full name contains it.
pub fn bundle_tests(files: &[PathBuf], out: &Path, filter: Option<&str>, extra: &[&str]) -> R<()> {
    let mut entry = String::new();
    for f in files {
        entry += &format!("import {:?};\n", spec(f));
    }
    entry += &format!(
        "import {{ run }} from \"bun:test\";\nconst call = (globalThis as any).__raea_call;\nconst print = (text: string) => call(\"print\", JSON.stringify({{ text }}));\nrun(print, {}).then((r) => call(\"done\", JSON.stringify({{ code: r.fail ? 1 : 0 }})), (e: any) => {{ print(\"error: \" + (e && e.stack ? e.stack : String(e))); call(\"done\", JSON.stringify({{ code: 1 }})); }});\n",
        filter.map(|f| format!("{f:?}")).unwrap_or("undefined".into())
    );
    bundle_entry(&entry, out, extra)
}

/// raea-bundle (bundle/: Rolldown, lightningcss, the Z3 compile) as a library loaded at run time, never linked: its
/// crates' Cargo features would reach raea's own (f60e0b7). Found at RAEA_BUNDLE_LIB, beside this binary, or in
/// engine/bundle/target/release (`cargo build --release` in bundle/).
pub mod inproc {
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
}

pub fn bundle_fresh(tsx: &Path, out_js: &Path, o: &CompOpts, portable: Option<&Path>) -> R<()> {
    let t0 = std::time::Instant::now();
    let dir = tsx.parent().unwrap().to_path_buf();
    // one at a time: the work folder is the process's (void's stage re-bakes on a worker while another object opens)
    static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one = ONE.lock().unwrap_or_else(|e| e.into_inner());
    let work = std::env::temp_dir().join(format!("raea-cli-b{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    copy_dir(&dir, &work.join("src"), 0)?;
    let motion_src = raea_io_home::home().join("sdk/raea/src").canonicalize().map_err(|e| format!("raea/motion/src: {e}"))?;
    copy_dir(&motion_src, &work.join("motion"), 0)?;
    let sdks = sdks();
    for rel in sources(&work.join("motion"))? {
        let p = work.join("motion").join(&rel);
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        let pointed = point_sdks(&text, &"../".repeat(rel.components().count()), &sdks);
        if pointed != text {
            std::fs::write(&p, pointed).map_err(|e| e.to_string())?;
        }
    }
    for (_, d) in &sdks {
        let from = raea_io_home::home().join(d);
        let to = work.join(d.rsplit('/').nth(1).unwrap_or(d));
        copy_dir(&from, &to, 0)?;
        // a file in a folder of its package (animate/timelines.ts) is a level further from motion/ and the others
        for rel in sources(&to)? {
            let p = to.join(&rel);
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            let up = "../".repeat(rel.components().count());
            let rewritten = rewrite_into(&text, &from.join(rel.parent().unwrap_or(Path::new(""))), &motion_src, &format!("{up}motion"), portable)?;
            if rewritten != text {
                std::fs::write(&p, rewritten).map_err(|e| e.to_string())?;
            }
        }
    }
    // what the edit imports from outside its folder (an example of examples/), at ext/ under the work folder, its
    // imports pointed at the copies and its SDK's at motion/ as the folder's own files' are (outside.rs)
    let outs = outside(&dir)?;
    for o in &outs {
        let to = work.join(&o.copy);
        std::fs::create_dir_all(to.parent().unwrap()).map_err(|e| e.to_string())?;
        let from_dir = o.file.parent().unwrap();
        let at = o.copy.parent().unwrap();
        let text = std::fs::read_to_string(&o.file).map_err(|e| format!("{}: {e}", o.file.display()))?;
        let to_motion = format!("{}motion", "../".repeat(at.components().count()));
        let rewritten = rewrite_into(&point_outside(&text, from_dir, at, &dir, &outs), from_dir, &motion_src, &to_motion, portable).map_err(|e| format!("{}: {e}", o.file.display()))?;
        std::fs::write(&to, rewritten).map_err(|e| e.to_string())?;
    }
    let file = tsx.file_name().unwrap().to_string_lossy().to_string();
    let src = point_outside(&std::fs::read_to_string(tsx).map_err(|e| e.to_string())?, &dir, Path::new("src"), &dir, &outs);
    std::fs::write(work.join("src").join(&file), rewrite_into(&src, &dir, &motion_src, "../motion", portable)?).map_err(|e| e.to_string())?;
    // every other source in the edit's folder too (components/captions.tsx importing the SDK): one copy of the SDK,
    // so a component's hooks read the Clock the engine drives
    for rel in sources(&work.join("src"))? {
        let p = work.join("src").join(&rel);
        if p == work.join("src").join(&file) {
            continue;
        }
        let to_motion = format!("{}motion", "../".repeat(rel.components().count()));
        let from_dir = dir.join(rel.parent().unwrap_or(Path::new("")));
        let text = point_outside(&std::fs::read_to_string(&p).map_err(|e| e.to_string())?, &from_dir, &Path::new("src").join(rel.parent().unwrap_or(Path::new(""))), &dir, &outs);
        let rewritten = rewrite_into(&text, &from_dir, &motion_src, &to_motion, portable).map_err(|e| format!("{}: {e}", rel.display()))?;
        if rewritten != text {
            std::fs::write(&p, rewritten).map_err(|e| e.to_string())?;
        }
    }
    let js = raea_io_home::home().join("sdk/react/js");
    #[cfg(unix)]
    std::os::unix::fs::symlink(js.join("node_modules"), work.join("node_modules")).map_err(|e| format!("node_modules: {e}"))?;
    if o.lite {
        // the game's code and the SDK's resolve "react" to the live view runtime (tsconfig paths cover the files under
        // the work folder; packages in node_modules keep their own React)
        copy_dir(&js.join("lite"), &work.join("lite"), 0)?;
        let bridge = js.join("node_modules/bevy-react/src/bridge.ts");
        let index = std::fs::read_to_string(work.join("lite/index.ts")).map_err(|e| e.to_string())?.replace("\"../node_modules/bevy-react/src/bridge\"", &format!("{:?}", bridge.display().to_string()));
        std::fs::write(work.join("lite/index.ts"), index).map_err(|e| e.to_string())?;
        std::fs::write(work.join("tsconfig.json"), r#"{ "compilerOptions": { "jsx": "react-jsx", "jsxImportSource": "react", "strict": false, "module": "esnext", "target": "es2022", "moduleResolution": "bundler", "baseUrl": ".", "paths": { "react": ["./lite/index.ts"], "react/jsx-runtime": ["./lite/jsx-runtime.ts"], "react/jsx-dev-runtime": ["./lite/jsx-runtime.ts"] } } }"#).map_err(|e| e.to_string())?;
        std::fs::write(work.join("entry.tsx"), entry_lite(&file, o)).map_err(|e| e.to_string())?;
    } else {
        std::fs::copy(js.join("tsconfig.json"), work.join("tsconfig.json")).map_err(|e| e.to_string())?;
        std::fs::write(work.join("entry.tsx"), entry(&file, o)).map_err(|e| e.to_string())?;
    }
    let t_work = t0.elapsed();
    // in process by raea-bundle's library (Rolldown; a live bundle with its Z3 compile in Rust)
    let (js_text, css) = inproc::bundle(&work.join("entry.tsx"), serde_json::json!({"z3": o.lite, "edit": {"dir": dir, "js": js}})).map_err(|e| format!("couldn't bundle {file}: {}", e.lines().filter(|l| !l.trim().is_empty()).take(6).collect::<Vec<_>>().join(" | ")))?;
    // its CSS rides in the bundle's last line (bundle_css), so every copy and cache of it carries it
    let code = if css.is_empty() { js_text } else { format!("{js_text}\n//# raea-css={}\n", serde_json::to_string(&css).unwrap_or_default()) };
    if std::env::var_os("RAEA_BUNDLE_TRACE").is_some() {
        eprintln!("raea bundle: {} in {:.1} ms: the work folder {:.1} ms, Rolldown {:.1} ms", tsx.display(), t0.elapsed().as_secs_f64() * 1e3, t_work.as_secs_f64() * 1e3, (t0.elapsed() - t_work).as_secs_f64() * 1e3);
    }
    std::fs::write(out_js, code).map_err(|e| format!("{}: {e}", out_js.display()))?;
    let _ = std::fs::remove_dir_all(&work);
    Ok(())
}
