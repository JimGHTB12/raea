//! `raea app run|check|bundle APP.tsx [--target web|native]` (docs/targets.feature T0b): one set of verbs for both
//! targets, the target read from the app's own `export const app` (T0a) unless `--target` says otherwise.
//!
//!   run      a window of its own: Blitz on Bevy for web (raea-page-host's window build), Rune for native
//!   check    headless (R4b): the mount and each step of `--script` written as PNGs to `--png DIR`, with tree.json (each
//!            frame's elements by data-testid) and unmapped.json (what the target didn't draw)
//!   bundle   a Mac app, `-o App.app` (the file's name when left out)
//!
//! `raea rune ARGS` is `raea app ARGS --target native`, and its older spellings (`rune --headless APP`) still reach
//! raea-rune-host as they are.

use std::path::{Path, PathBuf};

use crate::{AppConfig, Target};

pub const USAGE: &str = "usage: raea app run APP.tsx [--target web|native] [--size WxH]\n       raea app check APP.tsx [--target web|native] [--script steps.json] [--png DIR] [--size WxH] [--settle N]\n       raea app bundle APP.tsx [--target web|native] [-o App.app] [--size WxH] [--identity NAME] [--notarize PROFILE]\n       raea rune ...   (raea app ... --target native)";

/// What a verb was asked for, after the target is known
#[derive(Debug)]
pub struct Ask {
    pub verb: Verb,
    pub app: PathBuf,
    pub config: AppConfig,
    /// `--size WxH`, over the app's own
    pub size: (u32, u32),
    pub title: String,
    pub script: Option<PathBuf>,
    pub png: PathBuf,
    pub settle: u32,
    pub out: Option<PathBuf>,
    /// the flags passed on to the native host as they are (--video, --audio)
    pub rest: Vec<String>,
    /// bundle: `--identity NAME` (or RAEA_SIGN_IDENTITY) and `--notarize PROFILE` (T1d)
    pub sign: crate::sign::Signing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verb {
    Run,
    Check,
    Bundle,
}

/// `raea app ...` or `raea rune ...`: the exit's message (empty on success) or the error.
pub fn command(args: &[String]) -> Result<String, String> {
    let (first, rest) = args.split_first().ok_or(USAGE)?;
    match first.as_str() {
        "app" => app(rest, None),
        // the R-era spelling: `rune --headless APP ...` goes to the host untouched
        "rune" if rest.first().is_some_and(|a| a.starts_with("--")) => crate::native::host(rest),
        "rune" => app(rest, Some(Target::Native)),
        _ => Err(USAGE.into()),
    }
}

fn app(args: &[String], forced: Option<Target>) -> Result<String, String> {
    let ask = parse(args, forced)?;
    match ask.config.target {
        Target::Native => crate::native::run(&ask),
        Target::Web => crate::web::run(&ask),
    }
}

pub fn parse(args: &[String], forced: Option<Target>) -> Result<Ask, String> {
    let mut it = args.iter();
    let verb = match it.next().map(String::as_str) {
        Some("run") => Verb::Run,
        Some("check") => Verb::Check,
        Some("bundle") => Verb::Bundle,
        Some("-h" | "--help" | "help") | None => return Err(USAGE.into()),
        Some(v) => return Err(format!("no verb {v:?}\n{USAGE}")),
    };
    let (mut app, mut target, mut size, mut script, mut png, mut settle, mut out, mut rest) = (None, forced, None, None, PathBuf::from("out"), 12, None, Vec::new());
    let (mut identity, mut notarize) = (None, None);
    while let Some(a) = it.next() {
        let mut val = || it.next().cloned().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--target" => {
                let t = Target::parse(&val()?)?;
                if forced.is_some_and(|f| f != t) {
                    return Err(format!("raea rune is the native target; --target {} is `raea app`'s", t.name()));
                }
                target = Some(t);
            }
            "--size" => size = Some(wxh(&val()?)?),
            "--script" => script = Some(PathBuf::from(val()?)),
            "--png" => png = PathBuf::from(val()?),
            "--settle" => settle = val()?.parse().map_err(|_| "--settle is a number of frames")?,
            "-o" | "--out" => out = Some(PathBuf::from(val()?)),
            "--video" | "--audio" => rest.extend([a.clone(), val()?]),
            "--identity" => identity = Some(val()?),
            "--notarize" => notarize = Some(val()?),
            s if s.starts_with('-') => return Err(format!("no option {s}\n{USAGE}")),
            s if app.is_none() => app = Some(PathBuf::from(s)),
            s => return Err(format!("one app at a time ({s:?} too)")),
        }
    }
    let app = app.ok_or(USAGE)?;
    let config = crate::read(&app, target)?;
    let size = size.or(config.size.map(|(w, h)| (w.round() as u32, h.round() as u32))).unwrap_or((800, 600));
    let title = config.title.clone().unwrap_or_else(|| stem(&app));
    if !rest.is_empty() && config.target == Target::Web {
        return Err(format!("{} is the native target's (raea-rune-host); this app is web", rest[0]));
    }
    if (identity.is_some() || notarize.is_some()) && verb != Verb::Bundle {
        return Err("--identity and --notarize sign a Mac app: they're `raea app bundle`'s".into());
    }
    let sign = crate::sign::Signing::new(identity, notarize);
    Ok(Ask { verb, app, config, size, title, script, png, settle, out, rest, sign })
}

pub fn wxh(v: &str) -> Result<(u32, u32), String> {
    let bad = || format!("--size {v}: WxH");
    let (w, h) = v.split_once('x').ok_or_else(bad)?;
    Ok((w.trim().parse().map_err(|_| bad())?, h.trim().parse().map_err(|_| bad())?))
}

pub fn stem(p: &Path) -> String {
    p.file_stem().map_or("app".into(), |s| s.to_string_lossy().into_owned())
}

/// A binary of raea's by `name`: beside this one, on PATH, or built in the checkout's `built` target folders.
pub fn find(name: &str, built: &[PathBuf]) -> Result<PathBuf, String> {
    // name.exe on Windows (T1f: the Windows archive's hosts)
    let exe = |p: PathBuf| {
        let mut s = p.into_os_string();
        s.push(std::env::consts::EXE_SUFFIX);
        PathBuf::from(s)
    };
    let beside = std::env::current_exe().ok().map(|e| e.canonicalize().unwrap_or(e)).and_then(|e| e.parent().map(|d| d.join(name)));
    let on_path = std::env::var_os("PATH").map(|p| std::env::split_paths(&p).map(|d| d.join(name)).collect::<Vec<_>>()).unwrap_or_default();
    beside.into_iter().chain(on_path).chain(built.iter().cloned()).map(exe).find(|p| p.is_file()).ok_or(format!("no {name}: put it beside raea or on PATH (in a checkout, build it)"))
}
