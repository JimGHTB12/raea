//! ui-parity: the ts-native differential harness (docs/ts-native.feature P30, the bar). Each case is a TSX file under
//! cases/<area>/ with an input script (`export const steps`); build.ts bundles it twice, for React DOM in headless
//! Chrome (the oracle: Chrome for Testing, pinned in chrome.sh) and for @raea/ui's React adapter on raea-page-host
//! (headless, driven over BRP). Both are fed the same input step by step and compared after the mount and after every
//! step on
//!
//!   layout     every data-testid's getBoundingClientRect against its ComputedNode, within 0.5 pt
//!   pixels     SSIM of the frame (>= 0.98) and in every element's box (>= 0.95), with no caret on either side
//!   behaviour  the event trace (the probe's `trace`) and each element's value, focus, hover and scroll offset, and the
//!              focused field's selection (its caret)
//!
//! The caret is left out of the pixels (build.ts injects `:root { caret-color: transparent }` on both sides): headless
//! Chrome's blinks in real time, drawn or not at a screenshot by chance, where the page host's is steady. Where it stands is
//! checked instead as the focused field's selectionStart and selectionEnd against the page host's editor selection.
//!
//! and reported per case and per area (P10-P17).
//!
//!   ui-parity run [--only TEXT]... [--tier quick|full] [--jobs N] [--out DIR] [--self] [--bevy BIN] [--chrome BIN]
//!   ui-parity calibrate        the harness checks itself: every case Chrome against Chrome (must pass), and the
//!                              calibration cases (cases/calib, `// expect:`) both ways, each as it expects
//!   ui-parity list             the cases, their areas and steps
//!
//! A case's header: `// proves: id, id` (docs/ts-native/checklist.json ids), `// tier: quick` (the CI's quick tier),
//! and for calibration cases `// expect: pass` or `// expect: fail=layout pass=behaviour`. Results: OUT/results.jsonl
//! (each case's differences), OUT/summary.txt, the failing cases' frames under OUT/<case>/, and
//! docs/ts-native/results/latest.json (RAEA_TS_NATIVE_RESULTS) in the checklist's contract, from a full corpus run
//! only (a partial run writes OUT/latest.json).

mod app;
mod bevy;
mod wpt;

use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde_json::{Value, json};

use ui_parity_compare::compare::{Diffs, Snap, diff, diffs_json};
use ui_parity_compare::{chrome, compare};
pub use ui_parity_compare::Input;

mod outcome;
use outcome::*;
mod report;
use report::*;

/// Whether `--only` lets `name` run: no filters, or one it contains.
pub fn chosen(only: &[String], name: &str) -> bool {
    only.is_empty() || only.iter().any(|f| name.contains(f.as_str()))
}

const W: u32 = 800;
const H: u32 = 600;
/// frames each side settles at least, after the mount and after each step
const SETTLE: u32 = 3;

fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn repo() -> PathBuf {
    here().join("../../..").canonicalize().unwrap_or(here().join("../../.."))
}

struct Opts {
    cmd: String,
    only: Vec<String>,
    /// `wpt`: the areas run (css/<area>), wpt::AREAS when none is named
    areas: Vec<String>,
    tier: String,
    jobs: usize,
    out: PathBuf,
    self_check: bool,
    bevy: PathBuf,
    chrome: Option<PathBuf>,
    /// raea-runtime-js's JS engine for the Bevy side: jsc or quickjs
    engine: String,
    /// wpt: the raea whose `shots` draws the Bevy side, and the WPT checkout (else bench/wpt/fetch.sh's)
    motion: PathBuf,
    wpt: Option<PathBuf>,
    /// wpt: the page `shots` draws on (blitz, the only one since docs/blitz-bevy.feature B5d)
    page: String,
}

fn sh(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd).args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn parse() -> Result<Opts, String> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let stamp = sh("date", &["+%Y%m%d-%H%M%S"]);
    let home = std::env::var("HOME").unwrap_or_default();
    let mut o = Opts {
        cmd: a.first().cloned().unwrap_or_else(|| "run".into()),
        only: vec![],
        areas: vec![],
        tier: "full".into(),
        jobs: 2,
        out: PathBuf::from(format!("{home}/Library/Caches/raea/ui-parity/runs/{stamp}")),
        self_check: false,
        bevy: repo().join("target/release/raea-page-host"),
        chrome: None,
        engine: "jsc".into(),
        motion: repo().join("target/release/raea"),
        wpt: None,
        page: "blitz".into(),
    };
    let mut i = 1;
    while i < a.len() {
        let next = |i: &mut usize| -> Result<String, String> {
            *i += 1;
            a.get(*i).cloned().ok_or_else(|| format!("{} needs a value", a[*i - 1]))
        };
        match a[i].as_str() {
            "--only" => o.only.push(next(&mut i)?),
            "--area" => o.areas.push(next(&mut i)?),
            "--tier" => o.tier = next(&mut i)?,
            "--jobs" => o.jobs = next(&mut i)?.parse().map_err(|_| "--jobs N")?,
            "--out" => o.out = PathBuf::from(next(&mut i)?),
            "--self" => o.self_check = true,
            "--bevy" => o.bevy = PathBuf::from(next(&mut i)?),
            "--chrome" => o.chrome = Some(PathBuf::from(next(&mut i)?)),
            "--engine" => o.engine = next(&mut i)?,
            "--motion" => o.motion = PathBuf::from(next(&mut i)?),
            "--page" => o.page = next(&mut i)?,
            "--wpt" => o.wpt = Some(PathBuf::from(next(&mut i)?)),
            x => return Err(format!("unknown argument {x}\n{}", usage())),
        }
        i += 1;
    }
    Ok(o)
}

fn usage() -> &'static str {
    "usage: ui-parity run|calibrate|list [--only TEXT]... [--tier quick|full] [--jobs N] [--out DIR] [--self] [--bevy BIN] [--chrome BIN] [--engine jsc|quickjs]\n       ui-parity wpt [--area NAME]... [--only TEXT]... [--tier quick|full] [--jobs N] [--out DIR] [--chrome BIN] [--motion BIN] [--page blitz] [--wpt DIR]   (src/wpt.rs)\n       ui-parity app [--only TEXT]... [--out DIR] [--bevy BIN] [--chrome BIN] [--engine jsc|quickjs]   (src/app.rs: apps/app's screens; RAEA_PARITY_APP=DIR for a copy of it)"
}

#[derive(Clone, Debug)]
struct Case {
    case: String,
    name: String,
    area: String,
    proves: Vec<String>,
    steps: Vec<Value>,
    error: Option<String>,
    quick: bool,
    /// the accessibility tree is compared too (P16's cases, or `// check: a11y`)
    a11y: bool,
    /// calibration: (checks that must fail, checks that must pass)
    expect: Option<(Vec<String>, Vec<String>)>,
    /// it has an <iframe> page (CASE.frame.ts, build.ts's frameOf), which the page host serves from its bevy-root/
    frame: bool,
}

fn expect_of(src: &str) -> Option<(Vec<String>, Vec<String>)> {
    let line = src.lines().find_map(|l| l.trim().strip_prefix("// expect:"))?;
    let (mut fail, mut pass) = (vec![], vec![]);
    for t in line.split_whitespace() {
        if t == "pass" {
            pass.extend(["layout", "pixels", "behaviour"].map(String::from));
        } else if let Some(v) = t.strip_prefix("fail=") {
            fail.extend(v.split(',').map(String::from));
        } else if let Some(v) = t.strip_prefix("pass=") {
            pass.extend(v.split(',').map(String::from));
        }
    }
    Some((fail, pass))
}

/// The cases' bundles and manifest (build.ts), into OUT/build.
fn build(o: &Opts) -> Result<Vec<Case>, String> {
    let dir = o.out.join("build");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if !here().join("node_modules/react-dom").is_dir() {
        let st = Command::new("bun").args(["install", "--frozen-lockfile"]).current_dir(here()).status().map_err(|e| format!("bun install: {e}"))?;
        if !st.success() {
            return Err("bun install failed".into());
        }
    }
    let t = Instant::now();
    let r = Command::new("bun").arg(here().join("build.ts")).arg(&dir).current_dir(here()).output().map_err(|e| format!("bun: {e}"))?;
    if !r.status.success() {
        return Err(format!("build.ts: {}{}", String::from_utf8_lossy(&r.stdout), String::from_utf8_lossy(&r.stderr)));
    }
    eprintln!("ui-parity: bundled in {} ms", t.elapsed().as_millis());
    let m: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut cases = vec![];
    for c in m.as_array().cloned().unwrap_or_default() {
        let case = c["case"].as_str().unwrap_or("").to_string();
        let src = std::fs::read_to_string(repo().join(&case)).unwrap_or_default();
        cases.push(Case {
            name: c["name"].as_str().unwrap_or("").to_string(),
            area: c["area"].as_str().unwrap_or("").to_string(),
            proves: c["proves"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default(),
            steps: c["steps"].as_array().cloned().unwrap_or_default(),
            error: c["error"].as_str().map(String::from),
            quick: src.lines().any(|l| l.trim() == "// tier: quick"),
            a11y: c["area"].as_str() == Some("P16") || src.lines().any(|l| l.trim() == "// check: a11y"),
            expect: expect_of(&src),
            frame: c["frame"].as_bool().unwrap_or(false),
            case,
        });
    }
    Ok(cases.into_iter().filter(|c| chosen(&o.only, &c.name) && (o.tier != "quick" || c.quick || c.area == "CALIB")).collect())
}

/// The pinned Chrome for Testing (chrome.sh fetches it once and prints its path).
fn chrome_bin(o: &Opts) -> Result<PathBuf, String> {
    if let Some(c) = &o.chrome {
        return Ok(c.clone());
    }
    let r = Command::new("bash").arg(here().join("chrome.sh")).output().map_err(|e| format!("chrome.sh: {e}"))?;
    if !r.status.success() {
        return Err(format!("chrome.sh: {}", String::from_utf8_lossy(&r.stderr)));
    }
    Ok(PathBuf::from(String::from_utf8_lossy(&r.stdout).trim().lines().last().unwrap_or("")))
}

/// A side of the comparison.
enum Side {
    Chrome(chrome::Page),
    Bevy(bevy::Host),
}

impl Side {
    fn input(&mut self, b: &mut chrome::Browser, i: &Input) -> Result<(), String> {
        match self {
            Side::Chrome(p) => p.input(b, i),
            Side::Bevy(h) => h.input(i),
        }
    }
    fn read(&mut self, b: &mut chrome::Browser) -> Result<Snap, String> {
        match self {
            Side::Chrome(p) => p.read(b, SETTLE),
            Side::Bevy(h) => h.read(SETTLE),
        }
    }
}

/// How long Chrome's page ran up to its read (since it loaded, or since its previous read), for Bevy's read to match.
fn catch_up(side: &mut Side, ms: f64) {
    if let Side::Bevy(h) = side {
        h.catch_up = ms.max(0.0);
    }
}

/// A step's input events, aimed by the oracle's latest layout.
fn inputs(step: &Value, at: &Snap) -> Result<Vec<Input>, String> {
    let centre = |id: &str| at.centre(id).ok_or_else(|| format!("no element {id:?} to aim at (chrome)"));
    let point = |v: &Value| -> Result<(f64, f64), String> {
        match v {
            Value::String(id) => {
                // "at": [fx, fy], a point inside the box as fractions of it
                if let (Some(r), Some(f)) = (at.rect(id), step.get("at").and_then(Value::as_array)) {
                    let fx = f.first().and_then(Value::as_f64).unwrap_or(0.5);
                    let fy = f.get(1).and_then(Value::as_f64).unwrap_or(0.5);
                    return Ok((r[0] + r[2] * fx, r[1] + r[3] * fy));
                }
                centre(id)
            }
            Value::Array(a) => Ok((a.first().and_then(Value::as_f64).unwrap_or(0.0), a.get(1).and_then(Value::as_f64).unwrap_or(0.0))),
            _ => Err(format!("can't aim at {v}")),
        }
    };
    let right = step["button"] == json!("right");
    let clicks = step["clicks"].as_u64().unwrap_or(1) as u32;
    let mut out = vec![];
    if let Some(t) = step.get("click") {
        let (x, y) = point(t)?;
        out.push(Input::Move(x, y));
        for n in 1..=clicks {
            out.push(Input::Down(x, y, n, right));
            out.push(Input::Up(x, y, n, right));
        }
    } else if let Some(t) = step.get("move") {
        let (x, y) = point(t)?;
        out.push(Input::Move(x, y));
    } else if let Some(t) = step.get("down") {
        let (x, y) = point(t)?;
        out.push(Input::Move(x, y));
        out.push(Input::Down(x, y, 1, right));
    } else if let Some(t) = step.get("up") {
        let (x, y) = point(t)?;
        out.push(Input::Move(x, y));
        out.push(Input::Up(x, y, 1, right));
    } else if step.get("leave").is_some() {
        out.push(Input::Leave);
    } else if let Some(t) = step.get("wheel") {
        let (x, y) = point(t)?;
        out.push(Input::Move(x, y));
        out.push(Input::Wheel(x, y, step["dx"].as_f64().unwrap_or(0.0), step["dy"].as_f64().unwrap_or(0.0)));
    } else if let Some(k) = step.get("key").and_then(Value::as_str) {
        let mods = step["mods"].as_array().map(|a| a.iter().filter_map(|m| m.as_str().map(String::from)).collect()).unwrap_or_default();
        out.push(Input::Key(k.into(), mods));
    } else if let Some(t) = step.get("text").and_then(Value::as_str) {
        out.push(Input::Text(t.into()));
    } else if let Some(r) = step.get("resize").and_then(Value::as_array) {
        out.push(Input::Resize(r[0].as_f64().unwrap_or(W as f64), r[1].as_f64().unwrap_or(H as f64)));
    } else if step.get("wait").is_none() {
        return Err(format!("unknown step {step}"));
    }
    if let Some(n) = step.get("wait").and_then(Value::as_u64) {
        out.push(Input::Wait(n as u32));
    }
    Ok(out)
}

/// One case: both sides launched, the mount and each step compared.
fn run_case(c: &Case, o: &Opts, browser: &mut chrome::Browser, dir: &Path) -> Outcome {
    let mut out = Outcome { layout: true, pixels: true, behaviour: true, a11y: true, ssim_min: 1.0, ..Default::default() };
    if let Some(e) = &c.error {
        out.error = Some(format!("build: {e}"));
        return out;
    }
    let bundle = o.out.join("build").join(&c.name);
    let url = format!("file://{}", bundle.join("index.html").display());
    let mut sides: Vec<Side> = vec![];
    let r = (|| -> Result<(), String> {
        let mut p = browser.page(W, H, "chrome")?;
        p.a11y = c.a11y;
        p.open(browser, &url)?;
        sides.push(Side::Chrome(p));
        if o.self_check {
            let mut p = browser.page(W, H, "b")?;
            p.a11y = c.a11y;
            p.open(browser, &url)?;
            sides.push(Side::Chrome(p));
        } else {
            // a frame page's files, at "/" for the page's <iframe> loads (raea-page's FrameNet)
            let env = if c.frame { vec![("RAEA_PAGE_ROOT", bundle.join("bevy-root").into_os_string())] } else { vec![] };
            let mut h = bevy::Host::launch(&o.bevy, &bundle.join("bevy.js"), dir, W, H, &o.engine, None, &env).map_err(|e| format!("bevy: {e}"))?;
            h.a11y = c.a11y;
            sides.push(Side::Bevy(h));
        }
        let [a, b] = &mut sides[..] else { unreachable!() };
        let mut sa = a.read(browser)?;
        catch_up(b, sa.now);
        let sb = b.read(browser).map_err(|e| format!("bevy: {e}"))?;
        // the engine asked for is the one that ran (a build without QuickJS would quietly run JavaScriptCore)
        if let Side::Bevy(h) = b && !h.engine.is_empty() && h.engine != o.engine {
            return Err(format!("harness: asked for the {} engine but the bundle ran on {} (build raea-page-host with --features portable)", o.engine, h.engine));
        }
        out.note("mount", &sa, &sb, &diff(&sa, &sb));
        for (i, step) in c.steps.iter().enumerate() {
            let evs = inputs(step, &sa)?;
            for e in &evs {
                a.input(browser, e)?;
            }
            for e in &evs {
                b.input(browser, e).map_err(|e| format!("bevy: {e}"))?;
            }
            let prev = sa.now;
            sa = a.read(browser)?;
            catch_up(b, sa.now - prev);
            let sb = b.read(browser).map_err(|e| format!("bevy: {e}"))?;
            out.note(&format!("step {} {}", i + 1, step), &sa, &sb, &diff(&sa, &sb));
        }
        sa.write_oracle(&dir);
        Ok(())
    })();
    for s in sides {
        if let Side::Chrome(p) = s {
            p.close(browser);
        }
    }
    if let Err(e) = r {
        out.error = Some(e);
    }
    out
}

fn worker(o: Arc<Opts>, chrome: PathBuf, queue: Arc<Mutex<VecDeque<Case>>>, results: Arc<Mutex<Vec<(Case, Outcome, f64)>>>, version: Arc<Mutex<String>>) {
    let tmp = o.out.join("tmp");
    let _ = std::fs::create_dir_all(&tmp);
    let mut browser = match chrome::Browser::launch(&chrome, &tmp) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("ui-parity: chrome: {e}");
            while let Some(c) = queue.lock().unwrap().pop_front() {
                let out = Outcome { error: Some(format!("chrome: {e}")), ..Default::default() };
                results.lock().unwrap().push((c, out, 0.0));
            }
            return;
        }
    };
    *version.lock().unwrap() = browser.version.clone();
    loop {
        let Some(c) = queue.lock().unwrap().pop_front() else { break };
        let t = Instant::now();
        let dir = o.out.join(&c.name);
        let _ = std::fs::create_dir_all(&dir);
        let out = run_case(&c, &o, &mut browser, &dir);
        let secs = t.elapsed().as_secs_f64();
        if let Some((pa, pb, label)) = &out.frames {
            let _ = pa.save(dir.join("chrome.png"));
            let _ = pb.save(dir.join(if o.self_check { "b.png" } else { "bevy.png" }));
            let _ = std::fs::write(dir.join("step.txt"), label);
        }
        let tag = if out.pass() { "PASS" } else { "FAIL" };
        eprintln!("{tag} {:<44} {:>5.1}s  {}", c.name, secs, out.error.clone().or(out.reason.clone()).unwrap_or_default().chars().take(150).collect::<String>());
        results.lock().unwrap().push((c, out, secs));
    }
}

fn run(o: Opts) -> Result<i32, String> {
    let cases = build(&o)?;
    if o.cmd == "list" {
        for c in &cases {
            println!("{:<6} {:<44} {} steps{}  proves {}", c.area, c.name, c.steps.len(), if c.quick { " (quick)" } else { "" }, c.proves.join(" "));
        }
        return Ok(0);
    }
    let chrome = chrome_bin(&o)?;
    if !o.self_check && !o.bevy.is_file() {
        return Err(format!("no page host at {} (build it: lane build -- cargo build --release -p raea-page-host -j 6)", o.bevy.display()));
    }
    let o = Arc::new(o);
    let queue = Arc::new(Mutex::new(cases.iter().cloned().collect::<VecDeque<_>>()));
    let results = Arc::new(Mutex::new(vec![]));
    let version = Arc::new(Mutex::new(String::new()));
    let t = Instant::now();
    let hs: Vec<_> = (0..o.jobs.max(1))
        .map(|_| {
            let (o, chrome, q, r, v) = (o.clone(), chrome.clone(), queue.clone(), results.clone(), version.clone());
            std::thread::spawn(move || worker(o, chrome, q, r, v))
        })
        .collect();
    for h in hs {
        let _ = h.join();
    }
    let mut results = std::mem::take(&mut *results.lock().unwrap());
    results.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    let chrome_version = version.lock().unwrap().clone();
    report(&o, &results, &chrome_version, t.elapsed().as_secs_f64())
}

fn main() {
    let o = match parse() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let code = match o.cmd.as_str() {
        "run" | "list" => run(o),
        "app" => chrome_bin(&o).and_then(|chrome| app::run(app::Opts { out: o.out.clone(), chrome, bevy: o.bevy.clone(), engine: o.engine.clone(), only: o.only.clone(), app: std::env::var_os("RAEA_PARITY_APP").map(PathBuf::from).unwrap_or_else(|| repo().join("apps/app")) })),
        "wpt" => chrome_bin(&o).and_then(|chrome| wpt::run(wpt::Opts { tier: o.tier, jobs: o.jobs, out: o.out, chrome, motion: o.motion, wpt: o.wpt, only: o.only, areas: o.areas, page: o.page })),
        "calibrate" => {
            // every case Chrome against Chrome (identical by construction: each must pass, the calibration cases as
            // they expect), then the calibration cases against the page host, as they expect
            let base = o.out.clone();
            let cross = Opts { cmd: "run".into(), only: vec!["calib/".into()], areas: vec![], tier: "full".into(), jobs: o.jobs, out: base.join("cross"), self_check: false, bevy: o.bevy.clone(), chrome: o.chrome.clone(), engine: o.engine.clone(), motion: o.motion.clone(), wpt: None, page: o.page.clone() };
            let r1 = run(Opts { cmd: "run".into(), self_check: true, out: base.join("self"), ..o });
            let r2 = run(cross);
            match (r1, r2) {
                (Ok(0), Ok(c)) if c != 2 => Ok(0),
                (Ok(_), Ok(_)) => Ok(2),
                (Err(e), _) | (_, Err(e)) => Err(e),
            }
        }
        _ => Err(usage().into()),
    };
    match code {
        Ok(c) => std::process::exit(c),
        Err(e) => {
            eprintln!("ui-parity: {e}");
            std::process::exit(2);
        }
    }
}
