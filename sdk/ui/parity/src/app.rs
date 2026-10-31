//! `ui-parity app`: apps/app's review screens on the Blitz page against Chrome (docs/blitz-bevy.feature B5c),
//! compared as a corpus case's mount is (compare::diff: each data-testid's box, value and state, the frame's SSIM and
//! each box's). Chrome reads the app's own build (`vite build`, dist/ served here as Vite serves it); raea-page-host runs
//! its Blitz build (`vite build -c blitz/vite.config.ts`: React DOM swapped for @raea/ui's adapter) with the app's
//! stylesheet and RAEA_PAGE_ROOT at public/. The screens are apps/app/blitz/screens.tsv's, each a name and the hash
//! route it opens at, as e2e/screens.spec.ts shoots them; a screen of a state only the engine reaches (a game paused
//! with an element picked) runs a stand-in for the engine's calls on both sides and presses where the state needs it.

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use serde_json::json;

use crate::compare::{Snap, diff, diffs_json};
use crate::{Input, bevy, chrome};

pub struct Opts {
    pub out: PathBuf,
    pub chrome: PathBuf,
    pub bevy: PathBuf,
    pub engine: String,
    pub only: Vec<String>,
    pub app: PathBuf,
}

/// the review screens' viewport (playwright.config.ts)
const W: u32 = 1440;
const H: u32 = 900;
/// frames each side runs before it's read: the open transition and a video's first frame
const SETTLE: u32 = 60;
/// every CSS animation held at its start on both sides: Chrome's clock includes the page's load before its frames
/// and the page host's starts at its first frame, so a running one (the sidebar's agent spinner) was read at two phases.
/// Written into the Chrome build's index.html and appended to the page host's copy of the app's stylesheet, the same text
const PIN: &str = "*, ::before, ::after { animation-play-state: paused !important; }";
/// the JS clock as a count of frames on both sides, for the same reason: a game's clock adds up the time between
/// animation frames from its mount (play's 0:01.2 against Chrome's 0:00.9), and the two sides run different numbers of
/// frames before they're read. performance.now() and the frames' stamps advance 1/60 s a frame from the first frame
/// that runs the page's animation callbacks, held at 0.95 s; both sides run more than 57 frames before they're read,
/// so both are read at 0.95 s of their own time, after the open transition (which runs on the same clock). A game
/// mounts a frame or two later on the page host than in Chrome, and play's clock shows 0.1 s steps (7 frames): 0.95 s puts
/// both sides' 0.82 to 0.93 s inside one step. Chrome's are wrapped by CLOCK; the page host's by @raea/ui's dom.ts, told
/// by FRAME_CLOCK before the bundle.
const CLOCK: &str = "{ const t0 = performance.now(); let v = 0, started = false; const now = () => t0 + v; \
    Object.defineProperty(performance, 'now', { value: now, configurable: true, writable: true }); \
    const raf = window.requestAnimationFrame.bind(window); const tick = () => { if (started) v = Math.min(v + 1000 / 60, 950); raf(tick); }; raf(tick); \
    window.requestAnimationFrame = (cb) => raf(() => { started = true; cb(now()); }); }";
const FRAME_CLOCK: &str = "globalThis.__FRAME_CLOCK = { step: 1000 / 60, cap: 950 };";

/// A review screen (blitz/screens.tsv): its name and hash route, the boxes whose picture only the engine has, the
/// stand-in for the engine's stage calls it runs before the app (blitz/stages/NAME.js), and a press once it has settled:
/// a box's id and the point in it (fractions of its width and height).
struct Screen {
    name: String,
    hash: String,
    engine: Vec<String>,
    stage: Option<String>,
    press: Option<(String, f64, f64)>,
}

/// `ID@FX,FY`
fn press_of(p: &str) -> Option<(String, f64, f64)> {
    let (id, at) = p.split_once('@')?;
    let (fx, fy) = at.split_once(',')?;
    Some((id.to_string(), fx.parse().ok()?, fy.parse().ok()?))
}

/// The point a press lands at on a side, from that side's own box.
fn point(s: &Snap, (id, fx, fy): &(String, f64, f64)) -> Result<(f64, f64), String> {
    let r = s.nodes.get(id).map(|n| n.rect).ok_or_else(|| format!("press: no {id} on the screen"))?;
    Ok((r[0] + fx * r[2], r[1] + fy * r[3]))
}

fn vite(app: &PathBuf, args: &[&str]) -> Result<(), String> {
    vite_env(app, args, &[])
}

fn vite_env(app: &PathBuf, args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    let o = Command::new("npx").arg("vite").args(args).envs(env.iter().copied()).current_dir(app).output().map_err(|e| format!("vite: {e}"))?;
    if !o.status.success() {
        return Err(format!("vite {}: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).lines().rev().take(8).collect::<Vec<_>>().join(" | ")));
    }
    Ok(())
}

/// A box's pixels (viewport points, the screenshot's pixels) painted black.
fn black(png: &mut image::RgbaImage, [x, y, w, h]: [f64; 4]) {
    let (pw, ph) = png.dimensions();
    for yy in (y.floor().max(0.0) as u32)..((y + h).ceil().max(0.0) as u32).min(ph) {
        for xx in (x.floor().max(0.0) as u32)..((x + w).ceil().max(0.0) as u32).min(pw) {
            png.put_pixel(xx, yy, image::Rgba([0, 0, 0, 255]));
        }
    }
}

pub fn run(o: Opts) -> Result<i32, String> {
    let t0 = Instant::now();
    std::fs::create_dir_all(&o.out).map_err(|e| e.to_string())?;
    let list = std::fs::read_to_string(o.app.join("blitz/screens.tsv")).map_err(|e| format!("blitz/screens.tsv: {e}"))?;
    // NAME, ROUTE and, for a screen with a picture only the engine paints (the play item's live stage), `engine=ID,...`:
    // those boxes are laid out and checked as any other, their pixels blacked out on both sides; `stage=` and `press=`
    let screens: Vec<Screen> = list
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let mut f = l.split('\t').map(str::trim);
            let mut s = Screen { name: f.next()?.to_string(), hash: f.next()?.to_string(), engine: vec![], stage: None, press: None };
            for x in f {
                if let Some(ids) = x.strip_prefix("engine=") {
                    s.engine.extend(ids.split(',').map(String::from));
                } else if let Some(n) = x.strip_prefix("stage=") {
                    s.stage = Some(n.to_string());
                } else if let Some(p) = x.strip_prefix("press=") {
                    s.press = press_of(p);
                }
            }
            Some(s)
        })
        .filter(|s| crate::chosen(&o.only, &s.name))
        .collect();
    vite(&o.app, &["build", "--logLevel", "error"])?;
    vite(&o.app, &["build", "-c", "blitz/vite.config.ts", "--logLevel", "error"])?;
    // the artifact frame's page and bundle beside it (blitz/frame.tsx), what the app's <iframe> loads
    vite_env(&o.app, &["build", "-c", "blitz/vite.config.ts", "--logLevel", "error"], &[("BLITZ_ENTRY", "frame")])?;
    let index = o.app.join("dist/index.html");
    let html = std::fs::read_to_string(&index).map_err(|e| format!("dist/index.html: {e}"))?;
    let html = html.replacen("<head>", &format!("<head><script>{CLOCK}</script>"), 1);
    std::fs::write(&index, html.replacen("</head>", &format!("<style>{PIN}</style></head>"), 1)).map_err(|e| e.to_string())?;
    let base = crate::wpt::serve(o.app.join("dist").canonicalize().map_err(|e| e.to_string())?)?;
    let js = std::fs::read_to_string(o.app.join("blitz/out/app.js")).map_err(|e| format!("blitz/out/app.js: {e}"))?;
    let sheet = std::fs::read_to_string(o.app.join("blitz/out/app.css")).map_err(|e| format!("blitz/out/app.css: {e}"))?;
    std::fs::write(o.app.join("blitz/out/app.pinned.css"), format!("{sheet}\n{PIN}\n")).map_err(|e| e.to_string())?;
    let css = o.app.join("blitz/out/app.pinned.css").into_os_string();
    // the Blitz build's own files (the artifact frame's page and bundle), then the app's public/, both served at "/"
    let public = std::env::join_paths([o.app.join("blitz/out"), o.app.join("public")]).map_err(|e| e.to_string())?;
    let tmp = o.out.join("tmp");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let mut browser = chrome::Browser::launch(&o.chrome, &tmp)?;
    let mut rows = vec![];
    let mut fails = 0;
    for Screen { name, hash, engine, stage, press } in &screens {
        let t = Instant::now();
        let dir = o.out.join(name);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let r = (|| -> Result<(Snap, Snap), String> {
            let stand_in = match stage {
                Some(n) => std::fs::read_to_string(o.app.join(format!("blitz/stages/{n}.js"))).map_err(|e| format!("blitz/stages/{n}.js: {e}"))?,
                None => String::new(),
            };
            let mut p = browser.page(W, H, "chrome")?;
            if stage.is_some() {
                let session = p.session().to_string();
                browser.send("Page.addScriptToEvaluateOnNewDocument", json!({"source": stand_in}), Some(&session))?;
            }
            p.open(&mut browser, &format!("{base}/#{hash}"))?;
            let mut sa = p.read(&mut browser, SETTLE);
            if let (Some(pr), Ok(s)) = (press, &sa) {
                let (x, y) = point(s, pr)?;
                p.input(&mut browser, &Input::Down(x, y, 1, false))?;
                p.input(&mut browser, &Input::Up(x, y, 1, false))?;
                sa = p.read(&mut browser, 10);
            }
            p.close(&mut browser);
            // the route as the page's location before the app's first render reads it, at the http://localhost/ the
            // page is served from (the base a root-relative fetch, listen's "/media/…", resolves against)
            let bundle = dir.join("bevy.js");
            let at = json!({ "hash": format!("#{hash}"), "href": format!("http://localhost/#{hash}"), "origin": "http://localhost", "protocol": "http:", "host": "localhost", "hostname": "localhost", "port": "", "pathname": "/", "search": "" });
            std::fs::write(&bundle, format!("globalThis.location ??= {at};\n{FRAME_CLOCK}\n{stand_in};\n{js}"))
                .map_err(|e| e.to_string())?;
            let mut h = bevy::Host::launch(&o.bevy, &bundle, &dir, W, H, &o.engine, Some(css.clone()), &[("RAEA_PAGE_ROOT", public.clone())]).map_err(|e| format!("bevy: {e}"))?;
            let mut sb = h.read(SETTLE).map_err(|e| format!("bevy: {e}"))?;
            if let Some(pr) = press {
                let (x, y) = point(&sb, pr)?;
                h.input(&Input::Down(x, y, 1, false)).map_err(|e| format!("bevy: {e}"))?;
                h.input(&Input::Up(x, y, 1, false)).map_err(|e| format!("bevy: {e}"))?;
                sb = h.read(10).map_err(|e| format!("bevy: {e}"))?;
            }
            Ok((sa?, sb))
        })();
        let secs = t.elapsed().as_secs_f64();
        let row = match r {
            Ok((mut sa, mut sb)) => {
                // saved as each side drew it, the engine's picture in it
                let _ = sa.png.save(dir.join("chrome.png"));
                let _ = sb.png.save(dir.join("bevy.png"));
                for id in engine {
                    if let Some(r) = sa.nodes.get(id).map(|n| n.rect) {
                        black(&mut sa.png, r);
                        black(&mut sb.png, r);
                    }
                }
                let d = diff(&sa, &sb);
                sa.write_oracle(&dir);
                let pass = d.layout.is_empty() && d.pixels.is_empty();
                let first = d.layout.first().or(d.pixels.first()).cloned().unwrap_or_default();
                eprintln!(
                    "{} {:<22} {secs:>5.1}s  ssim {:.4}, lowest box {} {:.4}; layout {} off, pixels {} under  {}",
                    if pass { "PASS" } else { "FAIL" },
                    name,
                    d.ssim.unwrap_or(0.0),
                    d.ssim_box_min.as_ref().map(|b| b.0.as_str()).unwrap_or("-"),
                    d.ssim_box_min.as_ref().map(|b| b.1).unwrap_or(1.0),
                    d.layout.len(),
                    d.pixels.len(),
                    first.chars().take(120).collect::<String>()
                );
                fails += usize::from(!pass);
                json!({"screen": name, "hash": hash, "pass": pass, "secs": secs, "diffs": diffs_json("mount", &d)})
            }
            Err(e) => {
                eprintln!("FAIL {name:<22} {secs:>5.1}s  {e}");
                fails += 1;
                json!({"screen": name, "hash": hash, "pass": false, "secs": secs, "error": e})
            }
        };
        rows.push(row);
    }
    let lines: Vec<String> = rows.iter().map(|r| r.to_string()).collect();
    std::fs::write(o.out.join("results.jsonl"), lines.join("\n") + "\n").map_err(|e| e.to_string())?;
    eprintln!(
        "ui-parity app: {} of {} screens pass, in {:.0} s, the page host against {} (results: {})",
        screens.len() - fails,
        screens.len(),
        t0.elapsed().as_secs_f64(),
        browser.version,
        o.out.join("results.jsonl").display()
    );
    Ok(i32::from(fails > 0))
}
