use super::*;

pub(crate) struct Game {
    pub(crate) dir: PathBuf,
    pub(crate) name: String,
    pub(crate) w: u32,
    pub(crate) h: u32,
    pub(crate) responsive: bool,
    pub(crate) has_3d: bool,
    pub(crate) sound: bool,
    pub(crate) master: Option<(f64, f64)>,
    pub(crate) keys: String,
}

impl Game {
    pub(crate) fn load(dir: &Path) -> R<Game> {
        let text = std::fs::read_to_string(dir.join(MANIFEST)).map_err(|e| format!("{}: {e} (not an exported game's folder)", dir.join(MANIFEST).display()))?;
        let m: Value = serde_json::from_str(&text).map_err(|e| format!("{MANIFEST}: {e}"))?;
        let n = |k: &str, d: f64| m.get(k).and_then(Value::as_f64).unwrap_or(d);
        Ok(Game {
            dir: dir.to_path_buf(),
            name: m.get("name").and_then(Value::as_str).unwrap_or("game").to_string(),
            w: n("width", 1080.0) as u32,
            h: n("height", 1920.0) as u32,
            responsive: m.get("responsive").and_then(Value::as_bool).unwrap_or(false),
            has_3d: m.get("has_3d").and_then(Value::as_bool).unwrap_or(false),
            sound: m.get("sound").and_then(Value::as_bool).unwrap_or(false),
            master: m.get("master").and_then(|v| Some((v.get("gain_db")?.as_f64()?, v.get("ceiling_dbtp")?.as_f64()?))),
            keys: m.get("selftest_keys").and_then(Value::as_str).unwrap_or(DEFAULT_KEYS).to_string(),
        })
    }

    pub(crate) fn bundle(&self) -> PathBuf {
        self.dir.join("bundle.js")
    }

    pub(crate) fn open(&self, stage: (u32, u32)) -> R<Live> {
        let mut live = Live::open_bundle(&self.bundle(), &self.dir, stage, self.responsive)?;
        if let Ok(css) = std::fs::read_to_string(self.dir.join(FONTS)) {
            live.add_packaged_fonts_css(&fonts_in(&css, &self.dir));
        }
        live.set_game_gain(self.master);
        Ok(live)
    }
}

/// Where the package's game is: --game DIR, else beside the player (an app's Resources/game, or a folder's game/).
pub(crate) fn find_game(args: &[String]) -> R<PathBuf> {
    if let Some(d) = flag(args, "--game") {
        return Ok(PathBuf::from(d));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let exe = exe.canonicalize().unwrap_or(exe);
    let here = exe.parent().unwrap_or(Path::new("."));
    for c in [here.join("../Resources/game"), here.join("game")] {
        if c.join(MANIFEST).exists() {
            return Ok(c.canonicalize().unwrap_or(c));
        }
    }
    Err(format!("no game beside {} (an exported game's player looks in ../Resources/game or game/; --game DIR names one)", exe.display()))
}

pub(crate) static PANICS: AtomicU64 = AtomicU64::new(0);

/// `raea-game ...`: the player's main.
pub fn player_main(args: &[String]) -> i32 {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        PANICS.fetch_add(1, Ordering::Relaxed);
        prev(info);
    }));
    raea_runtime_react::native::register();
    let r = (|| -> R<String> {
        let game = Game::load(&find_game(args)?)?;
        if let Some(secs) = flag(args, "--selftest") {
            let secs: f64 = secs.trim_end_matches('s').parse().map_err(|_| "--selftest takes seconds, e.g. 5s")?;
            return selftest(&game, secs, args);
        }
        if args.iter().any(|a| a == "--info") {
            return Ok(std::fs::read_to_string(game.dir.join(MANIFEST)).unwrap_or_default());
        }
        windowed(game)
    })();
    match r {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
            0
        }
        Err(e) => {
            eprintln!("raea-game: {e}");
            println!("FAIL  {e}");
            1
        }
    }
}

/// The key sets the self-test cycles through: sets split by commas, keys in a set by "+", an empty set for none.
pub(crate) fn key_sets(spec: &str) -> Vec<Vec<String>> {
    spec.split(',').map(|set| set.split('+').filter(|k| !k.is_empty()).map(str::to_string).collect()).collect()
}

/// Peak and RMS (dBFS) of a 32-bit float or 16-bit WAV, and its seconds.
pub(crate) fn wav_levels(path: &Path) -> Option<(f64, f64, f64)> {
    let b = std::fs::read(path).ok()?;
    if b.len() < 44 || &b[0..4] != b"RIFF" {
        return None;
    }
    let (mut fmt, mut ch, mut rate, mut bits) = (0u16, 1u16, 48000u32, 32u16);
    let mut i = 12;
    while i + 8 <= b.len() {
        let id = &b[i..i + 4];
        let len = u32::from_le_bytes([b[i + 4], b[i + 5], b[i + 6], b[i + 7]]) as usize;
        let body = &b[i + 8..(i + 8 + len).min(b.len())];
        if id == b"fmt " && body.len() >= 16 {
            fmt = u16::from_le_bytes([body[0], body[1]]);
            ch = u16::from_le_bytes([body[2], body[3]]).max(1);
            rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
            bits = u16::from_le_bytes([body[14], body[15]]);
        } else if id == b"data" {
            let samples: Vec<f64> = match (fmt, bits) {
                (3, 32) => body.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64).collect(),
                (1, 16) => body.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f64 / 32768.0).collect(),
                _ => return None,
            };
            let peak = samples.iter().fold(0.0f64, |m, s| m.max(s.abs()));
            let rms = (samples.iter().map(|s| s * s).sum::<f64>() / samples.len().max(1) as f64).sqrt();
            let db = |x: f64| if x > 0.0 { (20.0 * x.log10() * 10.0).round() / 10.0 } else { -200.0 };
            return Some((db(peak), db(rms), samples.len() as f64 / ch as f64 / rate.max(1) as f64));
        }
        i += 8 + len + (len & 1);
    }
    None
}

/// Offscreen play with scripted input: what the package does, measured, with no window and no sound heard.
pub(crate) fn selftest(game: &Game, secs: f64, args: &[String]) -> R<String> {
    // the mixer's output kept (its tap) and silenced: the sound is mixed as it would be heard, and nobody hears it
    let tap = std::env::temp_dir().join(format!("raea-game-tap-{}.wav", std::process::id()));
    unsafe {
        std::env::set_var("RAEA_AUDIO_MUTE", "1");
        std::env::set_var("RAEA_AUDIO_TAP", &tap);
    }
    let (w, h) = (game.w, game.h);
    let snaps: Vec<(f64, String)> = args.windows(3).filter(|x| x[0] == "--snapshot").filter_map(|x| Some((x[1].parse().ok()?, x[2].clone()))).collect();
    let input_log = flag(args, "--input-log");
    let sets = key_sets(&flag(args, "--keys").unwrap_or_else(|| game.keys.clone()));
    // --no-pointer: keys alone (the pointer moving every frame is what a mouse-driven game needs; a keys-only game too)
    let no_pointer = args.iter().any(|a| a == "--no-pointer");
    let t_open = Instant::now();
    let mut stage = raea_gpu_device::present::offscreen(w, h)?;
    let mut live = game.open((w, h))?;
    if game.has_3d {
        live.scene_3d();
        live.wait_3d(120.0)?;
    }
    live.open_mixer();
    let open_ms = t_open.elapsed().as_secs_f64() * 1e3;
    let fps = live.fps;
    let start = Instant::now();
    // a hand on the keys: a change every 50-250 ms, cycling the sets (as play --input), stamped when it's made
    let shared = Arc::new(std::sync::Mutex::new((Vec::<String>::new(), Instant::now(), 0u64)));
    {
        let s2 = shared.clone();
        let until = start + Duration::from_secs_f64(secs);
        std::thread::spawn(move || {
            let mut x: u64 = 0x2545f4914f6cdd1d;
            let mut k = 0u64;
            while Instant::now() < until {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                std::thread::sleep(Duration::from_millis(50 + x % 200));
                k += 1;
                let keys = sets[(k % sets.len().max(1) as u64) as usize].clone();
                *s2.lock().unwrap() = (keys, Instant::now(), k);
            }
        });
    }
    let (mut seen, mut waiting) = (0u64, None::<Instant>);
    let mut latency: Vec<f64> = Vec::new();
    let mut logged = serde_json::Map::new();
    let (mut presented, mut stepped, mut keyed, mut pressed) = (0u64, 0u64, 0u64, 0u64);
    let mut gaps: Vec<f64> = Vec::new();
    let mut work: Vec<f64> = Vec::new();
    let mut last: Option<Instant> = None;
    let mut snapped = 0usize;
    let mut f: i64 = -1;
    let mut keys: Vec<String> = Vec::new();
    while start.elapsed().as_secs_f64() < secs {
        let t0 = Instant::now();
        let (k, at, seq) = shared.lock().unwrap().clone();
        keys = k;
        if seq != seen {
            seen = seq;
            waiting = Some(at);
        }
        f += 1;
        // a pointer too: across the page and back over four seconds, pressed for half of every second
        let ph = (f as f64 / fps / 4.0).fract();
        let px = (if ph < 0.5 { ph * 2.0 } else { 2.0 - ph * 2.0 }) * live.w as f64;
        let py = (0.5 + 0.3 * (f as f64 / fps * std::f64::consts::PI).sin()) * live.h as f64;
        let down = (f as f64 / fps).fract() < 0.5;
        let pointer = (!no_pointer).then_some((px, py, down));
        let lag = start.elapsed().as_secs_f64() - f as f64 / fps;
        live.step_at(f, &keys, pointer, lag)?;
        stepped += 1;
        if !keys.is_empty() {
            keyed += 1;
        }
        if pointer.is_some_and(|p| p.2) {
            pressed += 1;
        }
        if input_log.is_some() {
            let mut e = json!({"keys": keys, "live": true});
            if let Some((x, y, d)) = pointer {
                e["pointer"] = json!({"x": x, "y": y, "down": d});
            }
            // the session's seed and start with its first frame: a render of the log draws the same Math.random and Date
            if logged.is_empty() {
                e["session"] = live.session();
            }
            logged.insert(f.to_string(), e);
        }
        live.present(&mut stage, true)?;
        presented += 1;
        for (s, out) in snaps.iter().filter(|(s, _)| (*s * fps).round() as i64 == f) {
            live.save_snapshot(Path::new(out))?;
            eprintln!("snapshot at {s} s (frame {f}) -> {out}");
            snapped += 1;
        }
        let now = Instant::now();
        work.push((now - t0).as_secs_f64() * 1e3);
        if let Some(l) = last {
            gaps.push((now - l).as_secs_f64() * 1e3);
        }
        last = Some(now);
        if let Some(at) = waiting.take() {
            latency.push((now - at).as_secs_f64() * 1e3);
        }
        let next = start + Duration::from_secs_f64((f + 1) as f64 / fps);
        if let Some(d) = next.checked_duration_since(Instant::now()) {
            std::thread::sleep(d);
        }
    }
    let _ = keys;
    if let Some(p) = &input_log {
        std::fs::write(p, Value::Object(logged).to_string()).map_err(|e| format!("{p}: {e}"))?;
    }
    let (pw, ph) = (live.w, live.h);
    let drift: Vec<f64> = live.drift_ms().to_vec();
    // the mixer's tap is written as the mixer goes
    drop(live);
    let levels = wav_levels(&tap);
    let _ = std::fs::remove_file(&tap);
    let budget = 1000.0 / fps.max(1.0);
    let missed = gaps.iter().filter(|g| **g > budget * 1.5).count();
    let pct = |v: &[f64], p: f64| {
        if v.is_empty() {
            return 0.0;
        }
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        (s[((s.len() - 1) as f64 * p).round() as usize] * 100.0).round() / 100.0
    };
    let expected = (secs * fps).floor() as u64;
    let panics = PANICS.load(Ordering::Relaxed);
    let heard = levels.is_some_and(|(peak, _, _)| peak > -120.0);
    let mut why: Vec<String> = Vec::new();
    if presented < expected * 9 / 10 {
        why.push(format!("presented {presented} of {expected} frames"));
    }
    if missed as u64 > expected / 20 + 1 {
        why.push(format!("{missed} frames missed"));
    }
    if latency.is_empty() {
        why.push("no input change reached a frame".into());
    }
    if game.sound && !heard {
        why.push("the game has sound and none was mixed".into());
    }
    if snapped < snaps.len() {
        why.push(format!("{} of {} snapshots taken", snapped, snaps.len()));
    }
    if panics > 0 {
        why.push(format!("{panics} panics"));
    }
    let report = json!({
        "game": game.name,
        "seconds": secs,
        "fps": fps,
        "page": [pw, ph],
        "open_ms": (open_ms * 10.0).round() / 10.0,
        "frames": {"expected": expected, "stepped": stepped, "presented": presented, "missed": missed},
        "work_ms": {"p50": pct(&work, 0.5), "p99": pct(&work, 0.99), "max": pct(&work, 1.0)},
        "gap_ms": {"p50": pct(&gaps, 0.5), "p99": pct(&gaps, 0.99), "max": pct(&gaps, 1.0)},
        "input": {"changes_reached": latency.len(), "frames_with_keys": keyed, "frames_pointer_down": pressed, "to_present_ms": {"p50": pct(&latency, 0.5), "p99": pct(&latency, 0.99)}},
        "sound": {"in_game": game.sound, "mixed": heard, "peak_dbfs": levels.map(|l| l.0), "rms_dbfs": levels.map(|l| l.1), "seconds": levels.map(|l| (l.2 * 10.0).round() / 10.0), "master": game.master.map(|(g, c)| json!({"gain_db": g, "ceiling_dbtp": c})), "drift_ms_max": drift.iter().fold(0.0f64, |m, d| m.max(d.abs()))},
        "has_3d": game.has_3d,
        "snapshots": snapped,
        "panics": panics,
        "env": {"home": std::env::var("HOME").unwrap_or_default(), "raea_vars_given": std::env::vars().filter(|(k, _)| k.starts_with("RAEA_") && k != "RAEA_AUDIO_MUTE" && k != "RAEA_AUDIO_TAP").count()},
    });
    if let Some(p) = flag(args, "--json") {
        std::fs::write(&p, serde_json::to_string_pretty(&report).unwrap()).map_err(|e| format!("{p}: {e}"))?;
    }
    let verdict = if why.is_empty() { "PASS".to_string() } else { format!("FAIL  {}", why.join("; ")) };
    Ok(format!("{}\n{verdict}", serde_json::to_string_pretty(&report).unwrap()))
}

// ---------------------------------------------------------------------------------------------------------------------
// The window
