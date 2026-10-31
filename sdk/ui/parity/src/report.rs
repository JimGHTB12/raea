//! The run reported: the table, the JSON and the latest results kept.

use super::*;

/// results.jsonl, summary.txt and the checklist's latest.json (a full run's; a partial run's goes to OUT); the exit code: 0 all pass, 1 a case failed, 2 the
/// calibration didn't hold (the harness itself is wrong).
pub(crate) fn report(o: &Opts, results: &[(Case, Outcome, f64)], chrome_version: &str, secs: f64) -> Result<i32, String> {
    let mut lines = String::new();
    for (c, out, secs) in results {
        let j = json!({
            "case": c.case, "name": c.name, "area": c.area, "proves": c.proves, "pass": out.pass(),
            "checks": {"layout": out.check("layout"), "pixels": out.check("pixels"), "behaviour": out.check("behaviour"), "a11y": out.check("a11y")},
            "error": out.error, "reason": out.error.clone().or(out.reason.clone()), "ssim_min": (out.ssim_min * 10000.0).round() / 10000.0,
            "secs": (secs * 10.0).round() / 10.0, "diffs": out.steps,
        });
        lines.push_str(&j.to_string());
        lines.push('\n');
    }
    std::fs::create_dir_all(&o.out).map_err(|e| e.to_string())?;
    std::fs::write(o.out.join("results.jsonl"), lines).map_err(|e| e.to_string())?;

    // per area: cases passing, and each check
    let mut areas: BTreeMap<String, [usize; 6]> = BTreeMap::new();
    let mut calib_bad = vec![];
    for (c, out, _) in results {
        if c.area == "CALIB" || (o.self_check && c.expect.is_some()) {
            let (fail, pass) = c.expect.clone().unwrap_or_default();
            let ok = fail.iter().all(|k| !out.check(k)) && pass.iter().all(|k| out.check(k)) && (out.error.is_none() || !fail.is_empty());
            if !ok {
                calib_bad.push(format!("{}: expected fail={} pass={}, got layout={} pixels={} behaviour={}{}", c.name, fail.join(","), pass.join(","), out.check("layout"), out.check("pixels"), out.check("behaviour"), out.error.as_ref().map(|e| format!(" (error: {e})")).unwrap_or_default()));
            }
            continue;
        }
        if o.self_check && !out.pass() {
            calib_bad.push(format!("{}: Chrome against Chrome differs: {}", c.name, out.error.clone().or(out.reason.clone()).unwrap_or_default()));
        }
        let a = areas.entry(c.area.clone()).or_default();
        a[0] += 1;
        a[1] += out.pass() as usize;
        a[2] += out.check("layout") as usize;
        a[3] += out.check("pixels") as usize;
        a[4] += out.check("behaviour") as usize;
        a[5] += out.check("a11y") as usize;
    }
    let mut s = format!(
        "ui-parity {} ({}, {}), {} cases in {:.0} s, {} against {}\n\n{:<6} {:>6} {:>7} {:>8} {:>8} {:>10} {:>6}\n",
        if o.self_check { "self-check" } else { "run" },
        o.tier,
        if o.self_check { "chrome" } else { o.engine.as_str() },
        results.len(),
        secs,
        if o.self_check { "Chrome" } else { "raea-page-host" },
        chrome_version,
        "area",
        "cases",
        "pass",
        "layout",
        "pixels",
        "behaviour",
        "a11y"
    );
    let (mut n, mut p) = (0, 0);
    for (area, a) in &areas {
        let pct = |x: usize| format!("{:.0}%", 100.0 * x as f64 / a[0].max(1) as f64);
        s += &format!("{area:<6} {:>6} {:>7} {:>8} {:>8} {:>10} {:>6}\n", a[0], format!("{} {}", a[1], pct(a[1])), pct(a[2]), pct(a[3]), pct(a[4]), pct(a[5]));
        n += a[0];
        p += a[1];
    }
    s += &format!("{:<6} {:>6} {:>7}\n", "all", n, format!("{} {:.0}%", p, 100.0 * p as f64 / n.max(1) as f64));
    if calib_bad.is_empty() {
        s += "\ncalibration: holds\n";
    } else {
        s += "\nCALIBRATION FAILED (the harness, not the host):\n";
        for b in &calib_bad {
            s += &format!("  {b}\n");
        }
    }
    s += &format!("\nresults: {}\n", o.out.join("results.jsonl").display());
    print!("{s}");
    std::fs::write(o.out.join("summary.txt"), &s).map_err(|e| e.to_string())?;

    if !o.self_check && results.iter().any(|(c, _, _)| c.area != "CALIB") {
        write_latest(o, results, chrome_version)?;
    }
    Ok(if !calib_bad.is_empty() { 2 } else if p < n { 1 } else { 0 })
}

/// The checklist's contract (docs/ts-native/results/latest.json): only a full run of the whole corpus (no --only)
/// writes the tracked file, replacing it. A partial run (--only, or the quick tier) writes its own cases in the same
/// contract to OUT/latest.json, so agents' partial runs never overwrite the tracked results. RAEA_TS_NATIVE_RESULTS
/// names the file for either.
pub(crate) fn write_latest(o: &Opts, results: &[(Case, Outcome, f64)], chrome_version: &str) -> Result<(), String> {
    let full = o.only.is_empty() && o.tier == "full";
    let path = std::env::var("RAEA_TS_NATIVE_RESULTS")
        .map(PathBuf::from)
        .unwrap_or_else(|_| if full { repo().join("docs/ts-native/results/latest.json") } else { o.out.join("latest.json") });
    let mut cases: BTreeMap<String, Value> = BTreeMap::new();
    for (c, out, _) in results {
        if c.area == "CALIB" {
            continue;
        }
        let mut e = json!({"case": c.case, "pass": out.pass()});
        if let Some(r) = out.error.clone().or(out.reason.clone()).filter(|_| !out.pass()) {
            e["reason"] = json!(r.chars().take(300).collect::<String>());
        }
        cases.insert(c.case.clone(), e);
    }
    let bevy_ver = std::fs::read_to_string(repo().join("engine/page/host/Cargo.toml"))
        .ok()
        .and_then(|t| t.lines().find(|l| l.starts_with("bevy = "))?.split("version = \"=").nth(1)?.split('"').next().map(String::from))
        .unwrap_or_default();
    let doc = json!({
        "run": {"at": sh("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]), "commit": sh("git", &["-C", &repo().display().to_string(), "rev-parse", "HEAD"]), "bevy": bevy_ver, "chrome": chrome_version, "engine": o.engine},
        "cases": cases.into_values().collect::<Vec<_>>(),
    });
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&doc).unwrap() + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    eprintln!("ui-parity: wrote {}", path.display());
    Ok(())
}
