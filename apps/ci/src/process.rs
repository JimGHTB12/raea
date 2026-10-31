use super::*;

/// The command after `--`
pub(crate) fn after_dashes(args: &[String]) -> Result<&[String], String> {
    match args.iter().position(|a| a == "--") {
        Some(i) if i + 1 < args.len() => Ok(&args[i + 1..]),
        _ => Err("no command after --".into()),
    }
}

pub(crate) fn unix_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64() * 1e3).unwrap_or(0.0)
}

pub(crate) fn time(args: &[String]) -> Result<i32, String> {
    let to = flag(args, "--to").ok_or("usage: raea ci time --to OUT.json -- CMD...")?;
    let cmd = after_dashes(args)?;
    let start = unix_ms();
    let t0 = Instant::now();
    let st = Command::new(&cmd[0]).args(&cmd[1..]).status().map_err(|e| format!("{}: {e}", cmd[0]))?;
    let wall = t0.elapsed().as_secs_f64();
    let code = st.code().unwrap_or(128 + std::os::unix::process::ExitStatusExt::signal(&st).unwrap_or(0));
    std::fs::write(to, json!({"start_unix_ms": start, "wall_s": wall, "code": code}).to_string()).map_err(|e| format!("{to}: {e}"))?;
    Ok(code)
}

#[cfg(target_vendor = "apple")]
unsafe extern "C" {
    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut std::ffi::c_void) -> i32;
}

/// The process's energy so far, in joules: ri_energy_nj of rusage_info_v6, the 41st 64-bit field after the 16-byte uuid
#[cfg(target_vendor = "apple")]
pub(crate) fn energy_j(pid: u32) -> Option<f64> {
    let mut buf = [0u64; 80];
    // SAFETY: rusage_info_v6 is under 80 u64s; the kernel writes at most that struct into buf
    let r = unsafe { proc_pid_rusage(pid as i32, 6, buf.as_mut_ptr().cast()) };
    (r == 0).then(|| buf[2 + 40] as f64 / 1e9)
}

/// No per-process energy off Apple (libproc's rusage_info is macOS's)
#[cfg(not(target_vendor = "apple"))]
pub(crate) fn energy_j(_pid: u32) -> Option<f64> {
    None
}

/// The Mac's input power now, W (the battery controller's SystemPowerIn, mW)
pub(crate) fn system_w() -> Option<f64> {
    let o = Command::new("ioreg").args(["-rn", "AppleSmartBattery"]).output().ok()?;
    let t = String::from_utf8_lossy(&o.stdout);
    let re = regex::Regex::new(r#""SystemPowerIn"=(\d+)"#).unwrap();
    re.captures(&t).and_then(|c| c[1].parse::<i64>().ok()).map(|mw| mw as f64 / 1000.0)
}

/// Process watts past the first `warm` seconds and the system's mean watts over the same span (raw; the harness rounds)
pub(crate) fn watch_energy(pid: u32, seconds: f64, warm: f64) -> Result<Value, String> {
    std::thread::sleep(Duration::from_secs_f64(warm));
    let e0 = energy_j(pid);
    let t0 = Instant::now();
    let mut sysw = Vec::new();
    while t0.elapsed().as_secs_f64() < seconds - warm {
        std::thread::sleep(Duration::from_secs(1));
        if let Some(w) = system_w() {
            sysw.push(w);
        }
    }
    let e1 = energy_j(pid);
    let span = t0.elapsed().as_secs_f64();
    let (Some(e0), Some(e1)) = (e0, e1) else { return Err("the process ended before its energy was read".into()) };
    let mean = (!sysw.is_empty()).then(|| sysw.iter().sum::<f64>() / sysw.len() as f64);
    Ok(json!({"process_w": (e1 - e0) / span, "system_w": mean, "span_s": span}))
}

/// A child with its output dropped
pub(crate) fn spawn_quiet(cmd: &[String]) -> Result<std::process::Child, String> {
    use std::process::Stdio;
    Command::new(&cmd[0]).args(&cmd[1..]).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| format!("{}: {e}", cmd[0]))
}

/// Wait up to `secs` for a child, then kill it
pub(crate) fn reap(mut c: std::process::Child, secs: f64) {
    let t0 = Instant::now();
    while t0.elapsed().as_secs_f64() < secs {
        if let Ok(Some(_)) = c.try_wait() {
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = c.kill();
    let _ = c.wait();
}

pub(crate) fn num_flag(args: &[String], name: &str, default: f64) -> Result<f64, String> {
    flag(args, name).map(|s| s.parse().map_err(|_| format!("{name} {s}: not a number"))).transpose().map(|v| v.unwrap_or(default))
}

pub(crate) fn energy(args: &[String]) -> Result<(), String> {
    let seconds = num_flag(args, "--seconds", 30.0)?;
    let warm = num_flag(args, "--warm", 5.0)?;
    let child = spawn_quiet(after_dashes(args)?)?;
    let r = watch_energy(child.id(), seconds, warm);
    reap(child, 60.0);
    println!("{}", r?);
    Ok(())
}

pub(crate) const UNITS: [(&str, f64); 4] = [("B", 1.0 / 1048576.0), ("KB", 1.0 / 1024.0), ("MB", 1.0), ("GB", 1024.0)];

pub(crate) fn unit_mb(u: &str) -> f64 {
    UNITS.iter().find(|(n, _)| *n == u).map(|(_, f)| *f).unwrap_or(0.0)
}

/// footprint(1)'s whole footprint (MB) and its "(graphics)" categories' dirty memory, each in the order listed (MB)
pub(crate) fn footprint(pid: u32) -> (Option<f64>, Vec<(String, f64)>) {
    let out = Command::new("footprint")
        .args(["-p", &pid.to_string()])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    let re = regex::Regex::new(
        r"(?m)^\s*([\d.]+) (B|KB|MB|GB)\s+[\d.]+ (?:B|KB|MB|GB)\s+[\d.]+ (?:B|KB|MB|GB)\s+\d+\s+(.*\(graphics\).*)$",
    )
    .unwrap();
    let parts = re
        .captures_iter(&out)
        .filter_map(|c| Some((c[3].trim().to_string(), c[1].parse::<f64>().ok()? * unit_mb(&c[2]))))
        .collect();
    let fp = regex::Regex::new(r"Footprint: ([\d.]+) (KB|MB|GB)").unwrap();
    let total = fp.captures(&out).and_then(|c| Some(c[1].parse::<f64>().ok()? * unit_mb(&c[2])));
    (total, parts)
}

pub(crate) fn graphics(args: &[String]) -> Result<(), String> {
    let after = num_flag(args, "--after", 5.0)?;
    let child = spawn_quiet(after_dashes(args)?)?;
    std::thread::sleep(Duration::from_secs_f64(after));
    let (fp, parts) = footprint(child.id());
    reap(child, 60.0);
    // parts as [name, MB] pairs in footprint's order (a JSON object here would come out sorted); the total summed in
    // that order, unrounded
    let total: f64 = parts.iter().map(|(_, mb)| mb).sum();
    println!("{}", json!({"graphics_mb": total, "parts": parts, "footprint_mb": fp}));
    Ok(())
}

pub(crate) fn patches(args: &[String]) -> Result<(), String> {
    let usage = "usage: raea ci patches RAW.rgb WIDTH HEIGHT COLS HEX...";
    let [raw, w, h, cols, hexes @ ..] = args else { return Err(usage.into()) };
    let p = |s: &str| s.parse::<usize>().map_err(|_| format!("{s}: not a number"));
    let (w, h, cols) = (p(w)?, p(h)?, p(cols)?);
    if cols == 0 || hexes.len() < cols {
        return Err(usage.into());
    }
    let px = std::fs::read(raw).map_err(|e| format!("{raw}: {e}"))?;
    let (pw, ph) = (w / cols, h / (hexes.len() / cols));
    let mut worst = 0i64;
    let mut out = Vec::new();
    for (i, hexv) in hexes.iter().enumerate() {
        let (cx, cy) = (((i % cols) * pw + pw / 2) as i64, ((i / cols) * ph + ph / 2) as i64);
        let (mut got, mut n) = ([0i64; 3], 0i64);
        for dy in (-8i64..9).step_by(4) {
            for dx in (-8i64..9).step_by(4) {
                let o = (((cy + dy) * w as i64 + cx + dx) * 3) as usize;
                for (c, g) in got.iter_mut().enumerate() {
                    *g += *px.get(o + c).ok_or(format!("{raw}: shorter than {w}x{h} RGB"))? as i64;
                }
                n += 1;
            }
        }
        // Python's round(g / n): half to even
        let got: Vec<i64> = got.iter().map(|&g| (g as f64 / n as f64).round_ties_even() as i64).collect();
        let want: Vec<i64> = (0..3)
            .map(|j| hexv.get(2 * j..2 * j + 2).and_then(|s| i64::from_str_radix(s, 16).ok()))
            .collect::<Option<_>>()
            .ok_or(format!("{hexv}: not a colour"))?;
        let e = got.iter().zip(&want).map(|(a, b)| (a - b).abs()).max().unwrap_or(0);
        worst = worst.max(e);
        out.push(json!([hexv, want, got, e]));
    }
    println!("{}", json!({"worst": worst, "patches": out}));
    Ok(())
}

pub(crate) fn rgba(path: &str) -> Result<Vec<u8>, String> {
    let o = Command::new("ffmpeg")
        .args(["-v", "error", "-i", path, "-f", "rawvideo", "-pix_fmt", "rgba", "-"])
        .output()
        .map_err(|e| format!("ffmpeg: {e}"))?;
    Ok(o.stdout)
}

pub(crate) fn rgba_diff(args: &[String]) -> Result<(), String> {
    let [a, b] = args else { return Err("usage: raea ci rgba-diff A.png B.png".into()) };
    let (p, q) = (rgba(a)?, rgba(b)?);
    if p.len() != q.len() {
        println!("{}", json!({"size_differs": [p.len(), q.len()]}));
        return Ok(());
    }
    let (mut n, mut worst) = (0u64, 0u8);
    for (x, y) in p.chunks_exact(4).zip(q.chunks_exact(4)) {
        let d = x.iter().zip(y).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0);
        if d > 0 {
            n += 1;
            worst = worst.max(d);
        }
    }
    println!("{}", json!({"pixels_differ": n, "of": p.len() / 4, "worst": worst}));
    Ok(())
}
