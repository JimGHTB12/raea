//! raea-rune-host: an app's script drawn by Rune. The script is the one raea bundles (`raea app run|check|bundle APP.tsx
//! --target native` hands it here; runtime/bundle/lib app.rs), as the page host is handed its bundle: no bun (T1c).
//!
//!   raea-rune-host run app.js [--size 800x600]
//!   raea-rune-host --headless app.js [--script steps.json] [--png out/] [--size 800x600] [--settle 12] [--video OUT.mp4 [--audio FILE]]
//!
//! Headless (R4b), how an agent checks its work: the app bundled and mounted, then each step of the script played
//! through Rune's input. After the mount and after each step, `--settle` frames run on Rune's virtual clock and a PNG
//! of the frame is written to the `--png` folder (00-mount.png, 01-press-inc.png, ...), and tree.json gets the element
//! tree at each (keys, roles, labels, text, boxes). The same app and script give the same files every run.
//!
//! `--video OUT.mp4` (R8a) also films the session: every frame on the virtual clock at 60 fps, a cursor gliding to
//! each element before it is pressed, through `raea media encode` (raea's encoder, signed), `--audio FILE` under it.
//! video.json beside the PNGs has each frame's hash, so two runs can be compared frame for frame.
//!
//! `run` (R7a) opens the app in a native window through Rune's shell (window.rs) until it's closed: Rune's window, GPU
//! and input, in real time.

use std::path::{Path, PathBuf};

use raea_rune_host::script::{self, Step};
use raea_rune_host::{dump, js::App, shot::Shooter};
use serde_json::json;

struct Args {
    app: PathBuf,
    script: Option<PathBuf>,
    out: PathBuf,
    size: (u32, u32),
    settle: u32,
    mode: Mode,
    video: Option<PathBuf>,
    audio: Option<PathBuf>,
    /// the window's title and the app's name (the file's name when left out)
    title: Option<String>,
    /// the app's folder (its .tsx's), which relative media and fonts resolve against (T3d)
    base: Option<PathBuf>,
    /// the CSS the app's bundle imported, whose @font-face rules register (T3d)
    css: Option<PathBuf>,
}

enum Mode {
    Headless,
    Window,
    /// a .app at the path, signed ad hoc (R7b; raea signs it as its developer asks, T1d)
    Bundle(PathBuf),
}

fn args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let (mut app, mut script, mut out, mut size, mut settle) = (None, None, PathBuf::from("out"), (800, 600), 12);
    let (mut headless, mut bundle, mut app_out) = (false, false, None);
    let (mut video, mut audio, mut title, mut base, mut css) = (None, None, None, None, None);
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--headless" => (headless, app) = (true, Some(PathBuf::from(val()?))),
            "run" if app.is_none() => app = Some(PathBuf::from(val()?)),
            "bundle" if app.is_none() => (bundle, app) = (true, Some(PathBuf::from(val()?))),
            "-o" => app_out = Some(PathBuf::from(val()?)),
            "--script" => script = Some(PathBuf::from(val()?)),
            "--png" => out = PathBuf::from(val()?),
            "--video" => video = Some(PathBuf::from(val()?)),
            "--audio" => audio = Some(PathBuf::from(val()?)),
            "--size" => {
                let v = val()?;
                let (w, h) = v.split_once('x').ok_or("--size is WxH")?;
                size = (w.parse().map_err(|_| "--size is WxH")?, h.parse().map_err(|_| "--size is WxH")?);
            }
            "--title" => title = Some(val()?),
            "--base" => base = Some(PathBuf::from(val()?)),
            "--css" => css = Some(PathBuf::from(val()?)),
            "--settle" => settle = val()?.parse().map_err(|_| "--settle is a number of frames")?,
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("{other}?\n{USAGE}")),
        }
    }
    let app: PathBuf = app.ok_or(USAGE)?;
    let mode = if headless {
        // the system's clipboard, files, dialogs and browser stay untouched: the check's steps answer them (T2d)
        raea_io_os::set_headless();
        Mode::Headless
    } else if bundle {
        let name = app.file_stem().map_or("App".into(), |s| s.to_string_lossy().into_owned());
        Mode::Bundle(app_out.unwrap_or_else(|| PathBuf::from(format!("{name}.app"))))
    } else {
        Mode::Window
    };
    Ok(Args { app, script, out, size, settle, mode, video, audio, title, base, css })
}

const USAGE: &str = "usage: raea-rune-host run app.js [--size 800x600] [--title T]\n       raea-rune-host bundle app.js [-o App.app] [--size 800x600] [--title T]\n       raea-rune-host --headless app.js [--script steps.json] [--png out/] [--size 800x600] [--settle 12] [--video OUT.mp4 [--audio FILE]]";

/// The app's script: a bundle raea made. A source file is refused with what makes its script.
fn script_of(app: &Path) -> Result<String, String> {
    if app.extension().is_some_and(|x| x != "js") {
        return Err(format!("{}: the host runs an app's script (one .js); `raea app run|check|bundle {} --target native` bundles it", app.display(), app.display()));
    }
    std::fs::read_to_string(app).map_err(|e| format!("{}: {e}", app.display()))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("raea-rune-host: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    // inside a .app made by `bundle`: its app, in a window
    if std::env::args().len() == 1 && let Some(js) = raea_rune_host::macapp::bundled() {
        let src = std::fs::read_to_string(&js).map_err(|e| format!("{}: {e}", js.display()))?;
        let size = std::fs::read_to_string(js.with_file_name("size")).ok().and_then(|s| s.trim().split_once('x').and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))).unwrap_or((800.0, 600.0));
        let name = std::env::current_exe().ok().and_then(|e| e.file_name().map(|n| n.to_string_lossy().into_owned())).unwrap_or_default();
        // the .app's own faces (macapp.rs wrote them with fonts.css) and its folder for media
        let res = js.parent().unwrap_or(Path::new("."));
        raea_rune_media::set_base(res);
        raea_rune_media::fonts::register(raea_rune_media::fonts::gather(res, ""));
        return raea_rune_host::window::run(src, &name, size);
    }
    let a = args()?;
    let steps: Vec<Step> = match &a.script {
        Some(p) => script::parse(&std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?)?,
        None => Vec::new(),
    };
    let src = script_of(&a.app);
    // the app's folder and fonts, before Rune starts (T3d): Rune takes its faces only then
    let base = match (&a.base, a.app.parent()) {
        (Some(b), _) => b.clone(),
        (None, dir) => dir.unwrap_or(Path::new(".")).to_path_buf(),
    };
    raea_rune_media::set_base(&base);
    let css = match &a.css {
        Some(c) => std::fs::read_to_string(c).map_err(|e| format!("--css {}: {e}", c.display()))?,
        None => String::new(),
    };
    let fonts = raea_rune_media::fonts::gather(&base, &format!("{}\n{css}", src.as_deref().unwrap_or("")));
    let title = a.title.clone().unwrap_or_else(|| a.app.file_stem().map_or("app".into(), |s| s.to_string_lossy().into_owned()));
    match &a.mode {
        Mode::Window => {
            raea_rune_media::fonts::register(fonts);
            return raea_rune_host::window::run(src?, &title, (a.size.0 as f32, a.size.1 as f32));
        }
        Mode::Bundle(out) => {
            for line in raea_rune_host::macapp::make(&src?, &title, a.size, out, &fonts.faces)? {
                println!("{line}");
            }
            println!("{}", out.display());
            return Ok(());
        }
        Mode::Headless => {}
    }
    raea_rune_media::fonts::register(fonts);
    std::fs::create_dir_all(&a.out).map_err(|e| format!("{}: {e}", a.out.display()))?;
    let mut app = App::new(&src?, (a.size.0 as f32, a.size.1 as f32))?;
    let film = a.video.as_deref().map(|v| raea_rune_host::video::start(&mut app.session, v, a.size, a.audio.as_deref())).transpose()?;
    // one renderer per session: while filming, the stills are the film's own frames (video.rs, `last`)
    let mut shooter = film.is_none().then(|| Shooter::new(a.size.0, a.size.1));
    let mut trees = Vec::new();
    let film_ref = film.as_ref();
    let mut shoot = |app: &mut App, i: usize, name: &str| -> Result<(), String> {
        for _ in 0..a.settle.max(film_ref.map_or(0, |_| 1)) {
            app.frame()?;
        }
        let file = format!("{i:02}-{name}.png");
        match (shooter.as_mut(), film_ref) {
            (Some(s), _) => s.png(&mut app.session.rt, &a.out.join(&file))?,
            (None, Some(f)) => f.png(&a.out.join(&file))?,
            (None, None) => unreachable!("a shooter or a film"),
        }
        trees.push(json!({"step": i, "name": name, "png": file, "clock": app.session.clock(), "tree": dump::tree(&app.session)}));
        Ok(())
    };
    shoot(&mut app, 0, "mount")?;
    for (i, s) in steps.iter().enumerate() {
        let glide = film.as_ref().map(|f| f.at());
        if let Some(f) = &film {
            f.glide(&mut app, s).map_err(|e| format!("step {}: {e}", i + 1))?;
        }
        let act = film.as_ref().map(|f| f.at());
        script::play(&mut app, s).map_err(|e| format!("step {}: {e}", i + 1))?;
        if let Some(f) = &film {
            f.mark(json!({"step": i + 1, "name": name(s), "glide": glide, "act": act, "done": f.at()}));
        }
        shoot(&mut app, i + 1, &name(s))?;
    }
    let tree = a.out.join("tree.json");
    std::fs::write(&tree, serde_json::to_string_pretty(&trees).unwrap()).map_err(|e| format!("{}: {e}", tree.display()))?;
    // what the host couldn't map, never dropped quietly: said here and kept beside the frames
    let unmapped: Vec<_> = app.session.rt.state.report.borrow().all().map(|(prop, s)| json!({"prop": prop, "value": s.value, "path": s.path, "why": s.why})).collect();
    for u in &unmapped {
        eprintln!("raea-rune-host: unmapped {}: {} at {} ({})", u["prop"].as_str().unwrap(), u["value"].as_str().unwrap(), u["path"].as_str().unwrap(), u["why"].as_str().unwrap());
    }
    let report = a.out.join("unmapped.json");
    std::fs::write(&report, serde_json::to_string_pretty(&unmapped).unwrap()).map_err(|e| format!("{}: {e}", report.display()))?;
    // and every call the app made that this host didn't answer (T2c), the same way
    let unanswered: Vec<_> = app.refused.borrow().iter().map(|(name, (why, n))| json!({"call": name, "why": why, "times": n})).collect();
    for (name, (why, n)) in app.refused.borrow().iter() {
        eprintln!("raea-rune-host: unanswered {name} ({why}), {n} time{}", if *n == 1 { "" } else { "s" });
    }
    let report = a.out.join("unanswered.json");
    std::fs::write(&report, serde_json::to_string_pretty(&unanswered).unwrap()).map_err(|e| format!("{}: {e}", report.display()))?;
    println!("{} frames written to {}", trees.len(), shown(&a.out));
    if let Some(f) = film {
        let v = f.finish(&mut app.session)?;
        let file = a.out.join("video.json");
        std::fs::write(&file, serde_json::to_string_pretty(&v).unwrap()).map_err(|e| format!("{}: {e}", file.display()))?;
        // raea's encoder says whether it signed the file (T1e): the public engine's doesn't
        let credentials = v["encoded"]["credentials"].as_str().unwrap_or("its signing unreported");
        println!("{} frames ({}) filmed to {}, {credentials}", v["frames"], v["hash"].as_str().unwrap_or(""), a.video.as_deref().map_or(String::new(), shown));
    }
    Ok(())
}

/// A step's file name: what it did, by its element.
fn name(s: &Step) -> String {
    let n = match s {
        Step::Press { press } => format!("press-{press}"),
        Step::Type { into, .. } => format!("type-{into}"),
        Step::Wheel { wheel, .. } => format!("wheel-{wheel}"),
        Step::Key { key } => format!("key-{key}"),
        Step::Frames { frames } => format!("frames-{frames}"),
        Step::Pick { pick } => format!("pick-{}", pick.len()),
        Step::Save { .. } => "save".into(),
        Step::Dialog { dialog } => format!("dialog-{dialog}"),
        Step::Hover { hover } => format!("hover-{hover}"),
        Step::Dark { dark } => format!("dark-{dark}"),
        Step::Width { width } => format!("width-{width}"),
    };
    n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect()
}

fn shown(p: &Path) -> String {
    p.display().to_string()
}
