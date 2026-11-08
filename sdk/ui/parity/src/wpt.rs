//! `ui-parity wpt`: an independent benchmark, WPT's CSS reftests (bench/wpt/fetch.sh pins the checkout and its areas)
//! drawn on the Bevy page. A reftest is a page and one or more references (`<link rel=match|mismatch>`) that must draw
//! the same (or, for a mismatch, differently), within the test's `<meta name=fuzzy>`; each engine is judged against
//! itself, as WPT judges a browser. Chrome (the pinned oracle) runs every test first, and the Bevy page is scored on
//! the tests Chrome passes, per area (docs/ts-native.feature P31's target: 95% of them in each).
//!
//! Both sides load the pages from the checkout as a server rooted there serves them (`/fonts/ahem.css` is the
//! checkout's): Chrome over HTTP from a small file server here, the Bevy page through `raea shots --root`
//! (raea-runtime-js's HtmlPage served mode), which draws each page's first frame over white at 800x600, WPT's viewport (a
//! page with scripts run by @raea/ui's HTML runtime, once its reftest-wait is gone).
//!
//! Left out, with the reason in the run's skipped.jsonl: pages wptserve rewrites (`.sub.`, `.headers`), references
//! that are reftests themselves (chains), print, paged-media and SVG-file tests, and SMIL tests (raea doesn't run SMIL).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use base64::Engine as _;
use serde_json::{Value, json};

use crate::chrome::Browser;

mod shots;
pub(crate) use shots::*;

const W: u32 = 800;
const H: u32 = 600;
/// the areas run when `--area` names none (bench/wpt/fetch.sh also fetches css-fonts, css-masking and compositing)
pub const AREAS: &[&str] = &["css-flexbox", "css-grid", "css-text", "css-backgrounds", "css-borders", "css-transforms"];
/// the share of Chrome's passes the Bevy page must pass in each area
const TARGET: f64 = 0.95;

pub struct Opts {
    pub tier: String,
    pub jobs: usize,
    pub out: PathBuf,
    pub chrome: PathBuf,
    pub motion: PathBuf,
    pub wpt: Option<PathBuf>,
    pub only: Vec<String>,
    /// the areas run, AREAS when empty
    pub areas: Vec<String>,
    /// the page `shots` draws on: blitz
    pub page: String,
}

#[derive(Clone, Debug)]
struct Test {
    area: String,
    /// the page, relative to the checkout
    path: String,
    /// each reference: its path and whether it must match (false: must differ)
    refs: Vec<(String, bool)>,
    /// most channel difference and most differing pixels tolerated
    fuzzy: (u8, u64),
}

/// A shot's outcome: the picture, or why there is none.
type Shot = Result<image::RgbaImage, String>;

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let lower = tag.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        let before = lower[..at].chars().last();
        if !matches!(before, Some(c) if c.is_whitespace()) {
            continue;
        }
        let rest = tag[at + name.len()..].trim_start();
        let Some(rest) = rest.strip_prefix('=') else { continue };
        let rest = rest.trim_start();
        return Some(match rest.chars().next() {
            Some(q @ ('"' | '\'')) => rest[1..].split(q).next().unwrap_or(""),
            _ => rest.split(|c: char| c.is_whitespace() || c == '>' || c == '/').next().unwrap_or(""),
        });
    }
    None
}

/// The page's tags named `name` (`<link ...>`, `<meta ...>`), as their source text.
fn tags<'a>(src: &'a str, name: &str) -> Vec<&'a str> {
    let lower = src.to_ascii_lowercase();
    let open = format!("<{name}");
    let mut out = vec![];
    let mut from = 0;
    while let Some(i) = lower[from..].find(&open) {
        let at = from + i;
        let end = lower[at..].find('>').map(|e| at + e).unwrap_or(src.len());
        if lower[at + open.len()..].starts_with(|c: char| c.is_whitespace()) {
            out.push(&src[at..end]);
        }
        from = end.max(at + 1);
    }
    out
}

/// `href` from the page at `page` (both relative to the checkout), normalised.
fn resolve(page: &str, href: &str) -> String {
    let href = href.split(['?', '#']).next().unwrap_or(href);
    let mut parts: Vec<&str> = if href.starts_with('/') { vec![] } else { page.split('/').collect::<Vec<_>>()[..page.matches('/').count()].to_vec() };
    for seg in href.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// `<meta name=fuzzy content="[url:]maxDifference=0-2;totalPixels=0-100">` (or the short `0-2;0-100`): the upper
/// bounds, for the reference it names or all of them.
fn fuzzy(src: &str) -> (u8, u64) {
    let mut best = (0u8, 0u64);
    for m in tags(src, "meta") {
        if !attr(m, "name").is_some_and(|n| n.eq_ignore_ascii_case("fuzzy")) {
            continue;
        }
        let Some(c) = attr(m, "content") else { continue };
        let c = c.rsplit_once(':').map(|(_, v)| v).unwrap_or(c);
        let nums: Vec<&str> = c.split(';').collect();
        let upper = |s: &str| -> Option<u64> { s.rsplit(['=', '-']).next()?.trim().parse().ok() };
        let (mut d, mut p) = (None, None);
        for (i, part) in nums.iter().enumerate() {
            let part = part.trim();
            if part.starts_with("maxDifference") || (i == 0 && !part.contains('=')) {
                d = upper(part);
            } else if part.starts_with("totalPixels") || (i == 1 && !part.contains('=')) {
                p = upper(part);
            }
        }
        best = (best.0.max(d.unwrap_or(0).min(255) as u8), best.1.max(p.unwrap_or(0)));
    }
    best
}

fn is_page(p: &Path) -> bool {
    matches!(p.extension().and_then(|e| e.to_str()), Some("html" | "htm" | "xht" | "xhtml"))
}

/// Every reftest in the areas, and what was left out with why.
fn discover(root: &Path, areas: &[String], only: &[String]) -> (Vec<Test>, Vec<Value>) {
    let mut tests = vec![];
    let mut skipped = vec![];
    let areas: Vec<&str> = if areas.is_empty() { AREAS.to_vec() } else { areas.iter().map(String::as_str).collect() };
    for area in areas {
        let mut stack = vec![root.join("css").join(area)];
        let mut files = vec![];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().into_owned();
                if p.is_dir() {
                    if !matches!(name.as_str(), "reference" | "support" | "resources" | "tools") {
                        stack.push(p);
                    }
                } else if is_page(&p) && !name.contains("-ref.") && !name.contains("-notref.") && !name.starts_with("ref-") {
                    files.push(p);
                }
            }
        }
        files.sort();
        for p in files {
            let rel = p.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            if !only.is_empty() && !only.iter().any(|o| rel.contains(o.as_str())) {
                continue;
            }
            let Ok(src) = std::fs::read_to_string(&p) else { continue };
            let refs: Vec<(String, bool)> = tags(&src, "link")
                .into_iter()
                .filter_map(|l| {
                    let rel_ = attr(l, "rel")?.to_ascii_lowercase();
                    let m = match rel_.as_str() { "match" => true, "mismatch" => false, _ => return None };
                    Some((resolve(&rel, attr(l, "href")?), m))
                })
                .collect();
            if refs.is_empty() {
                continue;
            }
            let skip = |why: &str| json!({"test": rel, "why": why});
            let lower = rel.to_ascii_lowercase();
            if lower.contains(".sub.") || refs.iter().any(|r| r.0.contains(".sub.")) || p.with_extension(format!("{}.headers", p.extension().unwrap().to_string_lossy())).exists() {
                skipped.push(skip("wptserve rewrites it (.sub. or .headers)"));
                continue;
            }
            if lower.contains("print") || lower.contains("paged") || src.contains("class=\"reftest-print\"") {
                skipped.push(skip("print or paged media"));
                continue;
            }
            // raea doesn't run SMIL by design: such a test fails as expected, not as a regression
            if ["<animate", "<set "].iter().any(|t| src.contains(t)) {
                skipped.push(skip("SMIL animation (<animate>, <set>), which raea doesn't run"));
                continue;
            }
            if let Some(r) = refs.iter().find(|r| !is_page(Path::new(&r.0))) {
                skipped.push(skip(&format!("its reference {} isn't an HTML page", r.0)));
                continue;
            }
            if let Some(r) = refs.iter().find(|r| std::fs::read_to_string(root.join(&r.0)).map(|s| tags(&s, "link").iter().any(|l| attr(l, "rel").is_some_and(|v| v.eq_ignore_ascii_case("match") || v.eq_ignore_ascii_case("mismatch")))).unwrap_or(false)) {
                skipped.push(skip(&format!("its reference {} is a reftest itself (a chain)", r.0)));
                continue;
            }
            if let Some(r) = refs.iter().find(|r| !root.join(&r.0).is_file()) {
                skipped.push(skip(&format!("its reference {} isn't in the checkout", r.0)));
                continue;
            }
            tests.push(Test { area: area.to_string(), path: rel, refs, fuzzy: fuzzy(&src) });
        }
    }
    (tests, skipped)
}

/// The quick tier's sample: every 10th test of each area (by path), so a commit's run stays short.
fn sample(tests: Vec<Test>, tier: &str) -> Vec<Test> {
    if tier != "quick" {
        return tests;
    }
    let mut n: HashMap<String, usize> = HashMap::new();
    tests
        .into_iter()
        .filter(|t| {
            let i = n.entry(t.area.clone()).or_default();
            *i += 1;
            (*i - 1) % 10 == 0
        })
        .collect()
}

// ---------------------------------------------------------------- the file server Chrome loads from

fn mime(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase().as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "xht" | "xhtml" => "application/xhtml+xml",
        "css" => "text/css",
        "js" => "text/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

/// Serves `root` on 127.0.0.1 (a port of its own) for the life of the process; its address.
pub(crate) fn serve(root: PathBuf) -> Result<String, String> {
    let l = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let addr = format!("http://{}", l.local_addr().map_err(|e| e.to_string())?);
    std::thread::spawn(move || {
        for s in l.incoming().flatten() {
            let root = root.clone();
            std::thread::spawn(move || {
                let mut s = s;
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut line = String::new();
                if r.read_line(&mut line).is_err() {
                    return;
                }
                loop {
                    let mut h = String::new();
                    if r.read_line(&mut h).map(|n| n == 0).unwrap_or(true) || h == "\r\n" {
                        break;
                    }
                }
                let path = line.split_whitespace().nth(1).unwrap_or("/").split(['?', '#']).next().unwrap_or("/");
                let path = path.replace("%20", " ");
                let mut file = root.join(path.trim_start_matches('/'));
                // a directory's index.html (an app's "/": src/app.rs)
                if file.is_dir() {
                    file = file.join("index.html");
                }
                let ok = file.canonicalize().is_ok_and(|f| f.starts_with(&root) && f.is_file());
                let (status, body, ty) = if ok { ("200 OK", std::fs::read(&file).unwrap_or_default(), mime(&file)) } else { ("404 Not Found", vec![], "text/plain") };
                // CORS for any origin: an app's sandboxed frame (origin "null") loads its module scripts from here (src/app.rs)
                let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Type: {ty}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n", body.len());
                let _ = s.write_all(&body);
            });
        }
    });
    Ok(addr)
}

// ---------------------------------------------------------------- the two sides

/// Waits for the page's fonts, `reftest-wait`'s removal (5 s at most) and two frames.
const READY_JS: &str = r#"(async () => {
  const raf = () => new Promise((r) => requestAnimationFrame(() => r()));
  await document.fonts.ready;
  const t = performance.now();
  while (document.documentElement.classList.contains("reftest-wait") && performance.now() - t < 5000) await raf();
  await raf(); await raf();
  return true;
})()"#;

pub fn run(o: Opts) -> Result<i32, String> {
    let t0 = Instant::now();
    let root = match o.wpt.clone() {
        Some(p) => p,
        None => {
            let fetch = crate::repo().join("bench/wpt/fetch.sh");
            let r = Command::new("bash").arg(&fetch).stderr(Stdio::inherit()).output().map_err(|e| format!("{}: {e}", fetch.display()))?;
            if !r.status.success() {
                return Err("bench/wpt/fetch.sh failed".into());
            }
            PathBuf::from(String::from_utf8_lossy(&r.stdout).trim())
        }
    };
    let root = root.canonicalize().map_err(|e| format!("{}: {e}", root.display()))?;
    let commit = std::fs::read_to_string(root.join(".git/HEAD")).unwrap_or_default().trim().to_string();
    let (tests, skipped) = discover(&root, &o.areas, &o.only);
    let tests = sample(tests, &o.tier);
    std::fs::create_dir_all(o.out.join("bevy")).map_err(|e| e.to_string())?;
    let mut pages: Vec<String> = tests.iter().flat_map(|t| std::iter::once(t.path.clone()).chain(t.refs.iter().map(|r| r.0.clone()))).collect::<HashSet<_>>().into_iter().collect();
    pages.sort();
    pages.reverse(); // the queues pop from the end: alphabetical order
    eprintln!("wpt: {} reftests ({} pages, {} left out) at {}, tier {}", tests.len(), pages.len(), skipped.len(), root.display(), o.tier);

    let base = serve(root.clone())?;
    let tmp = std::env::temp_dir().join(format!("ui-parity-wpt-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let tc = Instant::now();
    let version = crate::sh(&o.chrome.to_string_lossy(), &["--version"]);
    let cache = std::env::var_os("RAEA_WPT_CHROME_CACHE").map(PathBuf::from).unwrap_or_else(|| {
        let name = format!("chrome-{}-{}", &commit[..commit.len().min(12)], version.split_whitespace().last().unwrap_or("x"));
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Library/Caches/raea/wpt").join(name)
    });
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    let chrome = chrome_shots(&o.chrome, &base, &pages, o.jobs, &tmp, &cache);
    eprintln!("wpt: chrome drew {} pages in {:.0} s", chrome.len(), tc.elapsed().as_secs_f64());
    let tb = Instant::now();
    let bevy = bevy_shots(&o.motion, &o.page, &root, &pages, o.jobs, &o.out);
    eprintln!("wpt: the Bevy page drew {} pages in {:.0} s", bevy.len(), tb.elapsed().as_secs_f64());
    let _ = std::fs::remove_dir_all(&tmp);

    let mut results = String::new();
    // per area: reftests, Chrome's passes, the Bevy page's passes among them, the Bevy page's passes overall
    let mut areas: BTreeMap<String, [u64; 4]> = BTreeMap::new();
    for t in &tests {
        let (c, cw) = judge(t, &chrome);
        let (b, bw) = judge(t, &bevy);
        let a = areas.entry(t.area.clone()).or_default();
        a[0] += 1;
        if c == "pass" {
            a[1] += 1;
            if b == "pass" {
                a[2] += 1;
            }
        }
        if b == "pass" {
            a[3] += 1;
        }
        if c == "pass" && b != "pass" {
            sheet(t, &bevy, &chrome, &o.out.join("misses"));
        }
        results += &format!("{}\n", json!({"area": t.area, "test": t.path, "chrome": c, "bevy": b, "chrome_why": cw, "bevy_why": bw}));
    }
    let w = |name: &str, s: &str| std::fs::write(o.out.join(name), s).map_err(|e| format!("{name}: {e}"));
    w("results.jsonl", &results)?;
    w("skipped.jsonl", &skipped.iter().map(|s| format!("{s}\n")).collect::<String>())?;
    let mut ok = true;
    let mut summary = serde_json::Map::new();
    println!("WPT reftests on the Bevy page (tier {}, wpt {}):", o.tier, &commit[..commit.len().min(12)]);
    println!("  {:<16} {:>6} {:>7} {:>11} {:>7}", "area", "tests", "chrome", "bevy/chrome", "rate");
    for (area, [n, c, b, all]) in &areas {
        let rate = if *c > 0 { *b as f64 / *c as f64 } else { 1.0 };
        ok &= rate >= TARGET;
        println!("  {area:<16} {n:>6} {c:>7} {b:>11} {:>6.1}%", rate * 100.0);
        summary.insert(area.clone(), json!({"tests": n, "chrome_pass": c, "bevy_pass_of_chrome": b, "bevy_pass": all, "rate": (rate * 10000.0).round() / 10000.0}));
    }
    let doc = json!({
        "run": {"tier": o.tier, "page": o.page, "wpt": commit, "chrome": version, "chrome_shots": cache, "secs": t0.elapsed().as_secs(), "target": TARGET, "skipped": skipped.len()},
        "areas": summary,
    });
    w("summary.json", &(serde_json::to_string_pretty(&doc).unwrap() + "\n"))?;
    eprintln!("wpt: {} (results.jsonl, summary.json)", o.out.display());
    Ok(if ok { 0 } else { 1 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_forms() {
        assert_eq!(fuzzy(r#"<meta name="fuzzy" content="maxDifference=0-2;totalPixels=0-100">"#), (2, 100));
        assert_eq!(fuzzy(r#"<meta name=fuzzy content="1-3;10-50">"#), (3, 50));
        assert_eq!(fuzzy(r#"<meta name="fuzzy" content="ref.html:maxDifference=5;totalPixels=7">"#), (5, 7));
        assert_eq!(fuzzy("<meta charset=utf-8>"), (0, 0));
    }

    #[test]
    fn hrefs_resolve() {
        assert_eq!(resolve("css/css-flexbox/a.html", "reference/a-ref.html"), "css/css-flexbox/reference/a-ref.html");
        assert_eq!(resolve("css/css-flexbox/x/a.html", "../b.html"), "css/css-flexbox/b.html");
        assert_eq!(resolve("css/css-grid/a.html", "/css/reference/ref-filled-green-100px-square.xht"), "css/reference/ref-filled-green-100px-square.xht");
    }

    #[test]
    fn links_found() {
        let src = r#"<link rel="match" href="r.html"><LINK REL=mismatch HREF='n.html'><link rel=stylesheet href=s.css>"#;
        let refs: Vec<_> = tags(src, "link").into_iter().filter_map(|l| Some((attr(l, "rel")?.to_ascii_lowercase(), attr(l, "href")?))).collect();
        assert_eq!(refs, vec![("match".into(), "r.html"), ("mismatch".into(), "n.html"), ("stylesheet".into(), "s.css")]);
    }
}
