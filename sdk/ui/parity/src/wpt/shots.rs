//! Each page shot in Chrome and in Bevy, diffed and judged.

use super::*;

/// Chrome's shots, kept in `dir` (a cache per checkout and Chrome: the pinned pair draws the same every time, so only
/// pages not drawn before are drawn).
pub(crate) fn chrome_shots(chrome: &Path, base: &str, pages: &[String], jobs: usize, tmp: &Path, dir: &Path) -> HashMap<String, Shot> {
    let mut have = HashMap::new();
    let mut todo = vec![];
    for p in pages {
        match image::open(dir.join(p.replace('/', "__") + ".png")) {
            Ok(i) => {
                have.insert(p.clone(), Ok(i.to_rgba8()));
            }
            Err(_) => todo.push(p.clone()),
        }
    }
    let queue = Arc::new(Mutex::new(todo));
    let out = Arc::new(Mutex::new(have));
    let mut hs = vec![];
    for _ in 0..jobs.max(1) {
        let (queue, out, chrome, base, tmp, dir) = (queue.clone(), out.clone(), chrome.to_path_buf(), base.to_string(), tmp.to_path_buf(), dir.to_path_buf());
        hs.push(std::thread::spawn(move || {
            let mut b: Option<Browser> = None;
            loop {
                let Some(p) = queue.lock().unwrap().pop() else { break };
                if b.is_none() {
                    b = Browser::launch(&chrome, &tmp).ok();
                }
                let Some(br) = b.as_mut() else {
                    out.lock().unwrap().insert(p, Err("chrome didn't launch".into()));
                    continue;
                };
                let got = shot_chrome(br, &format!("{base}/{p}"));
                if let Ok(i) = &got {
                    let _ = i.save(dir.join(p.replace('/', "__") + ".png"));
                }
                if got.is_err() {
                    b = None; // a fresh browser for the next page
                }
                out.lock().unwrap().insert(p, got);
            }
        }));
    }
    for h in hs {
        let _ = h.join();
    }
    Arc::try_unwrap(out).map(|m| m.into_inner().unwrap()).unwrap_or_default()
}

pub(crate) fn shot_chrome(b: &mut Browser, url: &str) -> Shot {
    let mut page = b.page(W, H, "chrome")?;
    let r = (|| -> Shot {
        page.open(b, url)?;
        let s = Some(page.session());
        b.send("Runtime.evaluate", json!({"expression": READY_JS, "awaitPromise": true, "returnByValue": true}), s)?;
        let shot = b.send("Page.captureScreenshot", json!({"format": "png", "fromSurface": true, "captureBeyondViewport": false}), s)?;
        let png = base64::engine::general_purpose::STANDARD.decode(shot["data"].as_str().unwrap_or("")).map_err(|e| e.to_string())?;
        Ok(image::load_from_memory(&png).map_err(|e| e.to_string())?.to_rgba8())
    })();
    page.close(b);
    r
}

/// The Bevy page's shots: `jobs` `raea shots` processes, each fed a share of the pages and restarted when one
/// dies (a page that crashes the process is that page's error, not the rest's).
pub(crate) fn bevy_shots(motion: &Path, page: &str, root: &Path, pages: &[String], jobs: usize, dir: &Path) -> HashMap<String, Shot> {
    let queue = Arc::new(Mutex::new(pages.to_vec()));
    let out = Arc::new(Mutex::new(HashMap::new()));
    let mut hs = vec![];
    for j in 0..jobs.max(1) {
        let (queue, out, motion, root, dir, page) = (queue.clone(), out.clone(), motion.to_path_buf(), root.to_path_buf(), dir.to_path_buf(), page.to_string());
        hs.push(std::thread::spawn(move || {
            let log = dir.join(format!("bevy-{j}.log"));
            loop {
                let Some(first) = queue.lock().unwrap().pop() else { break };
                let err = std::fs::OpenOptions::new().create(true).append(true).open(&log).map(Stdio::from).unwrap_or(Stdio::null());
                let child = Command::new(&motion)
                    .args(["shots", "--page", &page, "--size", &format!("{W}x{H}"), "--root", &root.to_string_lossy()])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(err)
                    .spawn();
                let Ok(mut child) = child else {
                    out.lock().unwrap().insert(first, Err(format!("{} didn't start", motion.display())));
                    continue;
                };
                let mut stdin = child.stdin.take().unwrap();
                let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
                let mut next = Some(first);
                while let Some(p) = next.take() {
                    let png = dir.join("bevy").join(p.replace('/', "__") + ".png");
                    let sent = writeln!(stdin, "{}\t{}", root.join(&p).display(), png.display()).and_then(|_| stdin.flush());
                    let said = if sent.is_ok() { lines.next().and_then(Result::ok) } else { None };
                    let got: Shot = match said {
                        Some(l) if l.starts_with("ok ") => image::open(&png).map(|i| i.to_rgba8()).map_err(|e| e.to_string()),
                        Some(l) => Err(l.split_once(": ").map(|(_, w)| w.to_string()).unwrap_or(l)),
                        None => {
                            out.lock().unwrap().insert(p, Err("raea shots died on this page".into()));
                            break;
                        }
                    };
                    out.lock().unwrap().insert(p, got);
                    next = queue.lock().unwrap().pop();
                }
                drop(stdin);
                let _ = child.kill();
                let _ = child.wait();
            }
        }));
    }
    for h in hs {
        let _ = h.join();
    }
    Arc::try_unwrap(out).map(|m| m.into_inner().unwrap()).unwrap_or_default()
}

// ---------------------------------------------------------------- judging

/// The most any channel differs, and how many pixels differ at all.
pub(crate) fn diff(a: &image::RgbaImage, b: &image::RgbaImage) -> (u8, u64) {
    if a.dimensions() != b.dimensions() {
        return (255, u64::MAX);
    }
    let (mut most, mut n) = (0u8, 0u64);
    for (p, q) in a.pixels().zip(b.pixels()) {
        let d = p.0.iter().zip(q.0.iter()).map(|(x, y)| x.abs_diff(*y)).max().unwrap_or(0);
        if d > 0 {
            n += 1;
            most = most.max(d);
        }
    }
    (most, n)
}

/// A test's verdict on one engine's shots: pass, fail (with the worst reference's difference) or error.
pub(crate) fn judge(t: &Test, shots: &HashMap<String, Shot>) -> (&'static str, Value) {
    let Some(Ok(page)) = shots.get(&t.path) else {
        let e = shots.get(&t.path).and_then(|s| s.as_ref().err().cloned()).unwrap_or_else(|| "no shot".into());
        return ("error", json!({"error": e}));
    };
    for (r, must_match) in &t.refs {
        let Some(Ok(rs)) = shots.get(r) else {
            let e = shots.get(r).and_then(|s| s.as_ref().err().cloned()).unwrap_or_else(|| "no shot".into());
            return ("error", json!({"error": format!("{r}: {e}")}));
        };
        let (most, n) = diff(page, rs);
        let same = n == 0 || (most <= t.fuzzy.0 && n <= t.fuzzy.1);
        if same != *must_match {
            return ("fail", json!({"ref": r, "match": must_match, "max_diff": most, "pixels": n}));
        }
    }
    ("pass", Value::Null)
}

/// A miss to look at: the Bevy page's test, its first reference, and Chrome's test, side by side in `dir`.
pub(crate) fn sheet(t: &Test, bevy: &HashMap<String, Shot>, chrome: &HashMap<String, Shot>, dir: &Path) {
    let blank = image::RgbaImage::from_pixel(W, H, image::Rgba([128, 128, 128, 255]));
    let pick = |m: &HashMap<String, Shot>, p: &str| m.get(p).and_then(|s| s.as_ref().ok()).filter(|i| i.dimensions() == (W, H)).cloned().unwrap_or_else(|| blank.clone());
    let mut out = image::RgbaImage::from_pixel(W * 3 + 8, H, image::Rgba([255, 0, 255, 255]));
    for (i, im) in [pick(bevy, &t.path), pick(bevy, &t.refs[0].0), pick(chrome, &t.path)].iter().enumerate() {
        image::imageops::replace(&mut out, im, (i as u32 * (W + 4)) as i64, 0);
    }
    let _ = std::fs::create_dir_all(dir);
    let _ = out.save(dir.join(t.path.replace('/', "__") + ".png"));
}
