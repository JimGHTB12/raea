//! Looking back: every app's sessions, how each ended, and one summarized (`raea traces`).

use super::*;

// ---- looking back

/// Every app's sessions, newest first: (app, folder).
pub fn sessions() -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    if let Ok(apps) = std::fs::read_dir(root()) {
        for a in apps.flatten() {
            if let Ok(list) = std::fs::read_dir(a.path()) {
                for s in list.flatten() {
                    if s.path().join("session.json").exists() {
                        out.push((a.file_name().to_string_lossy().into_owned(), s.path()));
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| b.1.file_name().cmp(&a.1.file_name()));
    out
}

/// How a session ended: ok, panicked, crashed (a signal or macOS's report), unclean (killed), or running.
pub fn status(d: &Path) -> &'static str {
    if d.join("panic.txt").exists() {
        "panicked"
    } else if std::fs::metadata(d.join("crash.txt")).map(|m| m.len() > 0).unwrap_or(false) || d.join("crash.ips").exists() {
        "crashed"
    } else if d.join("end.json").exists() {
        "ok"
    } else if d.join("unclean.json").exists() {
        "unclean"
    } else {
        "running or unclean"
    }
}

/// A session told as text: how it ended and why, its glitches (the worst, and when), its errors, its memory.
pub fn summarize(d: &Path) -> String {
    let mut out = String::new();
    let read = |n: &str| std::fs::read_to_string(d.join(n)).unwrap_or_default();
    let session: Value = serde_json::from_str(&read("session.json")).unwrap_or(Value::Null);
    out += &format!("{} ({}), pid {}, started {}: {}\n", session["app"].as_str().unwrap_or("?"), d.display(), session["pid"], session["started"].as_str().unwrap_or("?"), status(d));
    if let Ok(e) = serde_json::from_str::<Value>(&read("end.json")) {
        out += &format!("ran {:.1} s\n", e["seconds"].as_f64().unwrap_or(0.0));
    }
    let panic = read("panic.txt");
    if !panic.is_empty() {
        out += "\n== panic\n";
        out += &panic.lines().take(40).collect::<Vec<_>>().join("\n");
        out += "\n";
    }
    let crash = read("crash.txt");
    if !crash.is_empty() {
        out += "\n== crash (signal)\n";
        out += &crash.lines().take(30).collect::<Vec<_>>().join("\n");
        out += "\n";
    }
    let ips = read("crash.ips");
    if !ips.is_empty() {
        // the report's body: the exception and the crashed thread's first frames
        out += "\n== macOS crash report (crash.ips)\n";
        if let Some(body) = ips.split_once('\n').and_then(|(_, b)| serde_json::from_str::<Value>(b).ok()) {
            out += &format!("exception: {}\n", body["exception"]);
            if let Some(term) = body.get("termination") {
                out += &format!("termination: {term}\n");
            }
            let images = body["usedImages"].as_array().cloned().unwrap_or_default();
            if let Some(threads) = body["threads"].as_array() {
                if let Some(t) = threads.iter().find(|t| t["triggered"].as_bool() == Some(true)) {
                    out += &format!("crashed thread: {}\n", t["name"].as_str().or(t["queue"].as_str()).unwrap_or("?"));
                    for f in t["frames"].as_array().into_iter().flatten().take(16) {
                        let img = f["imageIndex"].as_u64().and_then(|i| images.get(i as usize)).and_then(|im| im["name"].as_str()).unwrap_or("?");
                        out += &format!("  {img}  {}\n", f["symbol"].as_str().unwrap_or("?"));
                    }
                }
            }
        }
    }
    let events: Vec<Value> = read("events.jsonl").lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    let glitches: Vec<&Value> = events.iter().filter(|e| e["kind"] == "glitch").collect();
    out += &format!("\n== glitches: {}\n", glitches.len());
    let mut worst = glitches.clone();
    worst.sort_by(|a, b| b["work_ms"].as_f64().unwrap_or(0.0).total_cmp(&a["work_ms"].as_f64().unwrap_or(0.0)));
    for g in worst.iter().take(10) {
        out += &format!("  at {:>8.2} s  {:>7.2} ms{}  {}\n", g["t"].as_f64().unwrap_or(0.0), g["work_ms"].as_f64().unwrap_or(0.0), if g["missed"] == true { " missed" } else { "" }, g.as_object().map(|o| o.iter().filter(|(k, _)| !["t", "kind", "work_ms", "missed", "budget_ms"].contains(&k.as_str())).map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")).unwrap_or_default());
    }
    // the GPU and video, by the subsystem behind each (raea-gputrace classifies GPU errors by the labels they name)
    let gpu_kinds = ["gpu_error", "gpu_lost", "gpu_stall", "gpu_slow", "video_error", "video_slow"];
    let gpu_events: Vec<&Value> = events.iter().filter(|e| e["kind"].as_str().is_some_and(|k| gpu_kinds.contains(&k))).collect();
    out += &format!("\n== GPU and video: {} events\n", gpu_events.len());
    let mut by: std::collections::BTreeMap<(String, String), (usize, f64)> = Default::default();
    for e in &gpu_events {
        let k = (e["source"].as_str().unwrap_or("?").to_string(), e["kind"].as_str().unwrap_or("?").to_string());
        let ms = e["ms"].as_f64().unwrap_or(0.0);
        let x = by.entry(k).or_insert((0, 0.0));
        x.0 += 1;
        x.1 = x.1.max(ms);
    }
    for ((src, kind), (n, worst)) in &by {
        out += &format!("  {src:<8} {kind:<12} {n:>6}{}\n", if *worst > 0.0 { format!("   worst {worst:.1} ms") } else { String::new() });
    }
    for e in gpu_events.iter().filter(|e| e["kind"] != "gpu_slow" && e["kind"] != "video_slow").take(10) {
        out += &format!("  {e}\n");
    }
    // each subsystem's GPU latency over the run (the per-second summaries), and Bevy's slowest passes
    let mut lat: std::collections::BTreeMap<String, (u64, f64, f64)> = Default::default();
    for p in read("perf.jsonl").lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()) {
        if let Some(g) = p["gpu"].as_object() {
            for (src, v) in g {
                let x = lat.entry(src.clone()).or_insert((0, 0.0, 0.0));
                x.0 += v["n"].as_u64().unwrap_or(0);
                x.1 = x.1.max(v["p99"].as_f64().unwrap_or(0.0));
                x.2 = x.2.max(v["max"].as_f64().unwrap_or(0.0));
            }
        }
    }
    for (src, (n, p99, max)) in &lat {
        out += &format!("  {src:<8} {n} submissions, worst second's p99 {p99:.2} ms, max {max:.2} ms (submit to done)\n");
    }
    let mut passes: std::collections::BTreeMap<String, f64> = Default::default();
    for e in events.iter().filter(|e| e["kind"] == "bevy_passes") {
        for (k, v) in e["ms"].as_object().into_iter().flatten() {
            let x = passes.entry(k.clone()).or_insert(0.0);
            *x = x.max(v.as_f64().unwrap_or(0.0));
        }
    }
    let mut passes: Vec<_> = passes.into_iter().collect();
    passes.sort_by(|a, b| b.1.total_cmp(&a.1));
    if !passes.is_empty() {
        out += "  bevy's slowest passes (worst second): ";
        out += &passes.iter().take(6).map(|(k, v)| format!("{k} {v:.2} ms")).collect::<Vec<_>>().join(", ");
        out += "\n";
    }
    let errors: Vec<&Value> = events.iter().filter(|e| e["kind"] == "error" || e["level"] == "ERROR" || e["level"] == "WARN" || e["kind"] == "panic").collect();
    out += &format!("\n== errors and warnings: {}\n", errors.len());
    for e in errors.iter().rev().take(15).rev() {
        out += &format!("  {e}\n");
    }
    let perf: Vec<Value> = read("perf.jsonl").lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
    if !perf.is_empty() {
        let rss: Vec<f64> = perf.iter().filter_map(|p| p["rss_mb"].as_f64()).collect();
        let missed: u64 = perf.iter().filter_map(|p| p["missed"].as_u64()).sum();
        let frames: u64 = perf.iter().filter_map(|p| p["frames"].as_u64()).sum();
        out += &format!("\n== perf: {frames} frames, {missed} missed; memory {:.0} MB at start, {:.0} MB max, {:.0} MB at the end\n", rss.first().unwrap_or(&0.0), rss.iter().cloned().fold(0.0, f64::max), rss.last().unwrap_or(&0.0));
    }
    let stderr = read("stderr.log");
    let tail: Vec<&str> = stderr.lines().rev().take(12).collect();
    if !tail.is_empty() {
        out += "\n== the end of stderr.log\n";
        for l in tail.into_iter().rev() {
            out += &format!("  {l}\n");
        }
    }
    out
}
