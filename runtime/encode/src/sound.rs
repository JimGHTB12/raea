//! An edit's sound: its timeline's audio mixed on the one mixer core, and the cues a render hands the mixer.

use super::*;

/// As sound, reading the gains off a timeline already evaluated.
pub fn sound_tl(prog: &Program, tl: &timeline::Timeline, rows: &[Tup], dir: &Path) -> Result<Option<raea_io_media::Audio>, String> {
    // through the one mixer core
    sound_core(prog, tl, rows, dir)
}

/// The edit's sound through the one mixer core: each frame's cues from the rows (`cues_tl`), bounced raw, mastered by
/// the master row as every render masters (raea_sys_mix::offline::master_mix, the ceiling held after AAC on Apple).
/// RAEA_AUDIO_BOUNCE=OUT.wav keeps the bounce.
pub fn sound_core(prog: &Program, tl: &timeline::Timeline, rows: &[Tup], dir: &Path) -> Result<Option<raea_io_media::Audio>, String> {
    let cues = cues_tl(prog, tl, rows, dir)?;
    if cues.iter().all(|c| c.sounds.is_empty() && c.tone.graph.is_none() && c.tone.evs.is_empty()) {
        return Ok(None);
    }
    let raw = raea_sys_mix::offline::bounce_raw(&cues, prog.fps)?;
    let mix = rows.iter().find(|t| t.kind == "master").map(|m| (m.vals.get("lufs").map(V::num).unwrap_or(-14.0), m.vals.get("ceiling").map(V::num).unwrap_or(-1.0)));
    #[cfg(target_vendor = "apple")]
    let enc: Option<&dyn Fn(&raea_io_media::Audio) -> Result<raea_io_media::Audio, String>> = Some(&raea_io_media::apple::aac_round_trip);
    #[cfg(not(target_vendor = "apple"))]
    let enc: Option<&dyn Fn(&raea_io_media::Audio) -> Result<raea_io_media::Audio, String>> = None;
    let (_, a) = raea_sys_mix::offline::master_mix(&raw, mix, enc);
    if let Some(p) = std::env::var_os("RAEA_AUDIO_BOUNCE") {
        raea_sys_mix::write_wav(&a, Path::new(&p))?;
    }
    Ok(Some(a))
}

/// Each frame's cues for the mixer core, from the rows: the sounds (the same rows raea-render's lanes sound: audio rows, clips
/// with a gain or a sound, a quiet clip unless it lies over one that sounds) with their source time, volume, rate,
/// fades and bus, and the edit's Tone (`tone` and `tone_ev` rows). Rows a TSX render wrote carry the element's key, its
/// exact volume, source time and rate (`snd_key`, `snd_vol`, `snd_at`, `snd_rate`, `snd_bus`), so the core plays them as the TSX played; hand-written rows play
/// from `src_in` and `gain`.
pub fn cues_tl(prog: &Program, tl: &timeline::Timeline, rows: &[Tup], dir: &Path) -> Result<Vec<raea_sys_mix::offline::Cue>, String> {
    let fps = prog.fps;
    let live = |t: &Tup, f: i64| (f as f64) >= t.vals.get("from").map(V::num).unwrap_or(0.0) && (f as f64) < t.vals.get("to").map(V::num).unwrap_or(f64::INFINITY);
    let explicit = |t: &Tup| t.kind == "clip" && (t.vals.contains_key("gain") || matches!(t.vals.get("sound"), Some(V::S(s)) if !s.is_empty()));
    let mut over: std::collections::HashSet<String> = Default::default();
    for f in 0..prog.frames {
        let tups = tl.tups(f);
        if tups.iter().any(|t| explicit(t) && live(t, f)) {
            for t in tups.iter().filter(|t| t.kind == "clip" && !explicit(t) && live(t, f)) {
                over.insert(t.id.clone());
            }
        }
    }
    let graph = rows.iter().find(|t| t.kind == "tone").and_then(|t| match t.vals.get("tone_json") {
        Some(V::S(g)) => Some(g.clone()),
        _ => None,
    });
    let mut paths: std::collections::HashMap<String, Option<PathBuf>> = Default::default();
    let mut out = Vec::with_capacity(prog.frames.max(0) as usize);
    for f in 0..prog.frames {
        let mut cue = raea_sys_mix::offline::Cue { tone: raea_sys_mix::ToneCue { graph: graph.clone(), evs: Vec::new(), ..Default::default() }, ..Default::default() };
        for t in tl.tups(f) {
            if t.kind == "tone_ev" {
                if live(&t, f)
                    && let Some(V::S(e)) = t.vals.get("tone_json")
                    && let Ok(v) = serde_json::from_str::<serde_json::Value>(e)
                {
                    cue.tone.evs.push(v);
                }
                continue;
            }
            let muted = matches!(t.vals.get("sound"), Some(V::S(s)) if s.is_empty());
            let quiet_clip = t.kind == "clip" && !explicit(&t);
            let sounds = !muted && (t.kind == "audio" || explicit(&t) || (quiet_clip && !over.contains(&t.id)));
            if !sounds || !live(&t, f) {
                continue;
            }
            let g = |k: &str, d: f64| t.vals.get(k).map(V::num).unwrap_or(d);
            let src = match t.vals.get("sound").filter(|v| !matches!(v, V::S(s) if s.is_empty())).or(t.vals.get("src")) {
                Some(V::S(s)) => s.clone(),
                _ => return Err(format!("{} {}: no src", t.kind, t.id)),
            };
            let path = paths.entry(src.clone()).or_insert_with(|| raea_sys_frame::resolve_src(rows, &src, dir).ok()).clone();
            let Some(file) = path else { continue };
            let from = g("from", 0.0);
            let rate = g("snd_rate", 1.0);
            let at0 = t.vals.get("snd_at").map(V::num).unwrap_or_else(|| g("src_in", 0.0));
            let at = if (f as f64 - from).abs() < 0.5 { at0 } else { at0 + (f as f64 - from) * rate / fps };
            let volume = t.vals.get("snd_vol").map(V::num).unwrap_or_else(|| 10f64.powf(g("gain", 0.0) / 20.0));
            let key = match t.vals.get("snd_key") {
                Some(V::S(k)) => k.clone(),
                _ => format!("{}:{}", t.kind, t.id),
            };
            let bus = match t.vals.get("snd_bus") {
                Some(V::S(b)) => Some(b.clone()),
                _ => None,
            };
            cue.sounds.push(raea_sys_mix::Sound { key: key.into(), file: file.into(), at, volume: volume as f32, rate, fade_in: g("fade_in", 0.0), fade_out: g("fade_out", 0.0), bus: bus.map(Into::into) });
        }
        out.push(cue);
    }
    Ok(out)
}
