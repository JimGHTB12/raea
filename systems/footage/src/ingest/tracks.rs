//! Per-clip tracks kept beside its facts: confirmed faces, the camera per shot, sound, matte, attention, light and
//! face mesh.

use super::*;

pub(crate) const CONFIRM_NOTE: &str = "a face is visible once seen twice in a row at confidence 0.8 or more, stays through gaps of up to 0.5 s, and goes after 0.5 s under 0.5; its box is the median of its last three sightings";

/// Faces confirmed over time (the founder, 2026-10-15: "faces are confirmed with confidence"; raea-9f's proposal): each
/// sample's faces matched to the tracks by overlap or by centre distance and size (best first); per track, per sample, the raw
/// confidence (0 where unseen), `visible` with hysteresis (on at 0.8 or more, off after 0.5 s under 0.5, gaps up to
/// 0.5 s held) and a smoothed box (the median of the last three sightings, held through gaps). Rules that read faces
/// read the confirmed track, so a one-sample detection or dropout doesn't move what follows a face.
pub fn confirm_faces(raw: &[(f64, Vec<([f64; 4], f64)>)], fps: f64) -> serde_json::Value {
    let iou = |a: &[f64; 4], b: &[f64; 4]| {
        let (x0, y0) = (a[0].max(b[0]), a[1].max(b[1]));
        let (x1, y1) = ((a[0] + a[2]).min(b[0] + b[2]), (a[1] + a[3]).min(b[1] + b[3]));
        let i = (x1 - x0).max(0.0) * (y1 - y0).max(0.0);
        let u = a[2] * a[3] + b[2] * b[3] - i;
        if u > 0.0 { i / u } else { 0.0 }
    };
    let off_after = ((0.5 * fps).round() as usize).max(1);
    struct Track { seen: Vec<Option<([f64; 4], f64)>>, last: [f64; 4] }
    let mut tracks: Vec<Track> = Vec::new();
    for (k, (_, faces)) in raw.iter().enumerate() {
        let mut taken = vec![false; faces.len()];
        let mut pairs: Vec<(f64, usize, usize)> = Vec::new();
        for (ti, tr) in tracks.iter().enumerate() {
            // a track unseen for over 2 s takes no more sightings (an old box stole detections from the live track,
            // splitting the take's second person between two tracks); stitching below joins pieces in order
            let last_seen = tr.seen.iter().rposition(|s| s.is_some()).unwrap_or(0);
            if k - last_seen > (2.0 * fps).round() as usize {
                continue;
            }
            for (fi, f) in faces.iter().enumerate() {
                // the same face: overlapping, or its centre within 0.6 of a face width of the track's last sighting and
                // its size within 0.6-1.6x (4 samples a second lose overlap on a moving head or a handheld frame: matched
                // by overlap alone, the take's 2 people came out as 11 tracks)
                let (a, b) = (&tr.last, &f.0);
                let d = ((a[0] + a[2] / 2.0 - b[0] - b[2] / 2.0).powi(2) + (a[1] + a[3] / 2.0 - b[1] - b[3] / 2.0).powi(2)).sqrt();
                let size = (b[2] * b[3]) / (a[2] * a[3]).max(1e-9);
                let near = d < 0.6 * a[2].max(b[2]) && (0.6..1.6).contains(&size);
                let o = iou(a, b);
                if o > 0.3 || near {
                    pairs.push((o.max(1.0 - d / a[2].max(b[2]).max(1e-9)), ti, fi));
                }
            }
        }
        pairs.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut matched_track = vec![false; tracks.len()];
        for (_, ti, fi) in pairs {
            if matched_track[ti] || taken[fi] {
                continue;
            }
            matched_track[ti] = true;
            taken[fi] = true;
            tracks[ti].seen.push(Some(faces[fi]));
            tracks[ti].last = faces[fi].0;
        }
        for (ti, tr) in tracks.iter_mut().enumerate() {
            if !matched_track[ti] {
                tr.seen.push(None);
            }
        }
        for (fi, f) in faces.iter().enumerate() {
            if !taken[fi] {
                let mut seen = vec![None; k];
                seen.push(Some(*f));
                tracks.push(Track { seen, last: f.0 });
            }
        }
    }
    // stitch: a track that starts after another ends (within 2 s), near where it was last seen (1.5 face widths) and
    // about its size, is the same face that moved too far between samples (the take's second person, swaying)
    let span = |tr: &Track| -> Option<(usize, usize)> {
        let a = tr.seen.iter().position(|s| s.is_some())?;
        let b = tr.seen.iter().rposition(|s| s.is_some())?;
        Some((a, b))
    };
    let gap = (2.0 * fps).round() as usize;
    loop {
        let mut joined = false;
        'outer: for i in 0..tracks.len() {
            for j in 0..tracks.len() {
                if i == j {
                    continue;
                }
                let (Some((_, ei)), Some((sj, _))) = (span(&tracks[i]), span(&tracks[j])) else { continue };
                if sj <= ei || sj - ei > gap {
                    continue;
                }
                let (Some((a, _)), Some((b, _))) = (tracks[i].seen[ei], tracks[j].seen[sj]) else { continue };
                let d = ((a[0] + a[2] / 2.0 - b[0] - b[2] / 2.0).powi(2) + (a[1] + a[3] / 2.0 - b[1] - b[3] / 2.0).powi(2)).sqrt();
                let size = (b[2] * b[3]) / (a[2] * a[3]).max(1e-9);
                if d < 1.5 * a[2].max(b[2]) && (0.5..2.0).contains(&size) {
                    let moved: Vec<Option<([f64; 4], f64)>> = tracks[j].seen.clone();
                    for (k, s) in moved.into_iter().enumerate() {
                        if s.is_some() {
                            tracks[i].seen[k] = s;
                        }
                    }
                    tracks.remove(j);
                    joined = true;
                    break 'outer;
                }
            }
        }
        if !joined {
            break;
        }
    }
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let out: Vec<serde_json::Value> = tracks.iter().enumerate().map(|(id, tr)| {
        let mut visible = false;
        let mut below = 0usize;
        let mut recent: Vec<[f64; 4]> = Vec::new();
        let samples: Vec<serde_json::Value> = tr.seen.iter().enumerate().map(|(k, s)| {
            let conf = s.map(|f| f.1).unwrap_or(0.0);
            if let Some((b, _)) = s {
                recent.push(*b);
                if recent.len() > 3 {
                    recent.remove(0);
                }
            }
            // on after two sightings in a row at 0.8 or more (half a second at 4 a second: Vision's confidence alone doesn't
            // tell a one-off false face from a real one, its median here is 0.875)
            let prev = if k > 0 { tr.seen[k - 1].map(|f| f.1).unwrap_or(0.0) } else { 0.0 };
            if conf >= 0.8 && (visible || prev >= 0.8) {
                visible = true;
                below = 0;
            } else if conf >= 0.5 {
                below = 0;
            } else if conf < 0.5 {
                below += 1;
                if below >= off_after {
                    visible = false;
                }
            }
            let smooth = (!recent.is_empty()).then(|| {
                let med = |i: usize| { let mut v: Vec<f64> = recent.iter().map(|b| b[i]).collect(); v.sort_by(f64::total_cmp); v[v.len() / 2] };
                [r3(med(0)), r3(med(1)), r3(med(2)), r3(med(3))]
            });
            serde_json::json!({"t": raw[k].0, "conf": r3(conf), "visible": visible, "box": if visible { serde_json::json!(smooth) } else { serde_json::Value::Null }})
        }).collect();
        serde_json::json!({"id": id, "samples": samples})
    }).collect();
    serde_json::Value::Array(out)
}

/// Each shot's camera (Perception.swift › camera): frames every 0.5 s (720 px at most), windows of four chained
/// through their shared frame, Depth Anything 3's camera package, the side from the biggest face, the room vector.
pub fn camera(path: &Path, shots: &[(f64, f64)], backend: &str, v: &dyn Vision) -> Result<Vec<Option<raea_sys_vision::camera::ShotCamera>>, String> {
    let m = raea_sys_vision::depth::camera_model()?;
    let mut d = raea_io_media::backend(backend)?.open_video(path)?;
    let person = |p: &[u8], w: usize, h: usize| -> Result<(Vec<u8>, usize, usize), String> { let pe = v.people(p, w, h, false)?; Ok((pe.mask, pe.mw, pe.mh)) };
    let mut out = Vec::new();
    for &(start, end) in shots {
        if end - start < 1.5 {
            out.push(None);
            continue;
        }
        let mut images = Vec::new();
        let mut t = start + 0.05;
        while t <= end - 0.05 && images.len() < 13 {
            images.push(fit(&d.frame_at(t)?, 720));
            t += 0.5;
        }
        if images.len() < 4 {
            out.push(None);
            continue;
        }
        let mut windows = Vec::new();
        let mut i = 0;
        while i + 3 < images.len() {
            windows.push(raea_sys_vision::depth::camera_window(&m, &images[i..i + 4], &person)?);
            i += 3;
        }
        let (first, fw, fh) = &images[0];
        let biggest = v.people(first, *fw, *fh, true)?.faces.iter().map(|f| f.w * f.h).fold(0.0, f64::max);
        let side = if biggest >= 0.08 { "front" } else { "back" };
        let room = v.room(first, *fw, *fh)?.map(|r| r.into_iter().map(|x| x as f64).collect());
        out.push(Some(raea_sys_vision::camera::summary(&windows, images.len(), *fw, *fh, side, room)));
    }
    Ok(out)
}

/// The sound steps of ingest (docs/the roadmap Track H), for any file with sound: `vad` (Silero's speech spans, and the
/// level of the talking and of the floor between it), `beats` (Beat This! on the whole sound), `stems` (HTDemucs:
/// each stem's level 20 times a second, and Beat This! on the music stem: drums, bass and other), and `denoise`
/// (DeepFilterNet3, only on speech whose floor between words is above -40 dB and flat like noise: it reshapes a clean
/// voice, and a music bed isn't noise; the denoised
/// sound kept as 16 kHz mono floats for transcription). Each writes its json beside the picture's tracks.
pub fn ingest_sound(path: &Path, dir: &Path, backend: &str, m: &mut Manifest) -> Result<(), String> {
    let x16 = raea_sys_vision::sound::pcm16k(path, backend);
    if x16.len() < 16_000 {
        return Ok(());
    }
    let mpath = dir.join("manifest.json");
    let save = |m: &Manifest| std::fs::write(&mpath, serde_json::to_string_pretty(m).unwrap()).map_err(|e| e.to_string());
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let db = |xs: &mut dyn Iterator<Item = f32>| -> Option<f64> {
        let (mut s, mut n) = (0.0f64, 0usize);
        for v in xs {
            s += (v as f64) * (v as f64);
            n += 1;
        }
        (n > 0).then(|| r2(10.0 * (s / n as f64 + 1e-12).log10()))
    };
    if !m.steps.contains_key("vad") {
        let t = std::time::Instant::now();
        let probs = raea_sys_song::vad::Vad::load()?.probs(&x16)?;
        let spans = raea_sys_song::vad::spans(&probs, x16.len());
        let mut speech = vec![false; x16.len()];
        for (a, b) in &spans {
            speech[*a..*b].iter_mut().for_each(|v| *v = true);
        }
        let talk = db(&mut x16.iter().zip(&speech).filter(|(_, s)| **s).map(|(v, _)| *v));
        let floor = db(&mut x16.iter().zip(&speech).filter(|(_, s)| !**s).map(|(v, _)| *v));
        let j = serde_json::json!({"model": "silero-vad v5", "spans": spans.iter().map(|(a, b)| [r3(*a as f64 / 16_000.0), r3(*b as f64 / 16_000.0)]).collect::<Vec<_>>(),
            "talk_db": talk, "floor_db": floor});
        std::fs::write(dir.join("vad.json"), j.to_string()).map_err(|e| e.to_string())?;
        m.steps.insert("vad".into(), t.elapsed().as_secs_f64());
        save(m)?;
    }
    if !m.steps.contains_key("beats") {
        let t = std::time::Instant::now();
        let j = raea_sys_song::beatthis::file(path, backend)?;
        std::fs::write(dir.join("beats.json"), j.to_string()).map_err(|e| e.to_string())?;
        m.steps.insert("beats".into(), t.elapsed().as_secs_f64());
        save(m)?;
    }
    if !m.steps.contains_key("stems") {
        let t = std::time::Instant::now();
        let a = raea_io_media::soft_resample(&raea_io_media::backend(backend)?.decode_audio(path)?, raea_sys_song::stems::SR);
        let l = a.channels.first().cloned().unwrap_or_default();
        let r = a.channels.get(1).cloned().unwrap_or_else(|| l.clone());
        let stems = raea_sys_song::stems::Demucs::load()?.separate(&[l, r])?;
        let hop = raea_sys_song::stems::SR as usize / 20;
        let level = |st: &[Vec<f32>; 2]| -> Vec<f64> {
            (0..st[0].len() / hop).map(|i| db(&mut (i * hop..(i + 1) * hop).flat_map(|k| [st[0][k], st[1][k]])).unwrap_or(-120.0)).collect()
        };
        let mut levels = serde_json::Map::new();
        for (name, st) in raea_sys_song::stems::SOURCES.iter().zip(&stems) {
            levels.insert(name.to_string(), serde_json::json!(level(st)));
        }
        // the music stem (all but the vocals), mono at 22 050 Hz, through Beat This!
        let n = stems[0][0].len();
        let music: Vec<f32> = (0..n).map(|i| stems.iter().take(3).map(|st| (st[0][i] + st[1][i]) / 2.0).sum()).collect();
        let music22 = raea_io_media::soft_resample(&raea_io_media::Audio { rate: raea_sys_song::stems::SR, channels: vec![music] }, 22_050).channels.remove(0);
        let (bl, dl) = raea_sys_song::beatthis::BeatThis::load()?.logits(&raea_sys_song::beatthis::spectrogram(&music22))?;
        let (mb, md) = raea_sys_song::beatthis::beats(&bl, &dl);
        let j = serde_json::json!({"model": "htdemucs", "levels_per_second": 20, "levels_db": levels,
            "music_beats": mb.into_iter().map(r3).collect::<Vec<_>>(), "music_downbeats": md.into_iter().map(r3).collect::<Vec<_>>()});
        std::fs::write(dir.join("stems.json"), j.to_string()).map_err(|e| e.to_string())?;
        m.steps.insert("stems".into(), t.elapsed().as_secs_f64());
        save(m)?;
    }
    if !m.steps.contains_key("denoise") {
        let t = std::time::Instant::now();
        let vad: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("vad.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let spans: Vec<(f64, f64)> = vad["spans"].as_array().map(|a| a.iter().filter_map(|s| Some((s[0].as_f64()?, s[1].as_f64()?))).collect()).unwrap_or_default();
        // the floor between speech, and whether it's noise: the spectral flatness of its 512-sample frames (geometric
        // over arithmetic mean of the power, averaged): broadband noise is flat (near 1), music and hum are tonal (a
        // music bed under the voice isn't noise to take out; HTDemucs's vocals stem can't tell, it sends white noise to
        // "other" and lets some music into the vocals)
        let floor = vad["floor_db"].as_f64();
        let between: Vec<usize> = (0..x16.len() / 512).filter(|i| { let t = (*i * 512 + 256) as f64 / 16_000.0; !spans.iter().any(|(a, b)| t >= *a && t < *b) }).collect();
        let flat = |i: usize| -> Option<f64> {
            let w = |k: usize| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * k as f64 / 512.0).cos();
            let mut re: Vec<f64> = (0..512).map(|k| x16[i * 512 + k] as f64 * w(k)).collect();
            let mut im = vec![0.0; 512];
            raea_sys_song::beatthis::fft(&mut re, &mut im);
            let p: Vec<f64> = (1..256).map(|k| re[k] * re[k] + im[k] * im[k] + 1e-20).collect();
            let am = p.iter().sum::<f64>() / p.len() as f64;
            (am > 1e-12).then(|| (p.iter().map(|v| v.ln()).sum::<f64>() / p.len() as f64).exp() / am)
        };
        let fl: Vec<f64> = between.iter().filter_map(|i| flat(*i)).collect();
        let flatness = (!fl.is_empty()).then(|| r3(fl.iter().sum::<f64>() / fl.len() as f64));
        let gate = !spans.is_empty() && floor.is_some_and(|f| f > -40.0) && flatness.is_some_and(|f| f > 0.3);
        #[cfg(feature = "denoise")]
        let ran = if gate {
            let y = raea_sys_song::denoise::denoise(&raea_sys_song::denoise::pcm48k(path, backend)?)?;
            let y16 = raea_io_media::soft_resample(&raea_io_media::Audio { rate: raea_sys_song::denoise::SR, channels: vec![y] }, 16_000).channels.remove(0);
            std::fs::write(dir.join("denoised.f32"), y16.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<u8>>()).map_err(|e| e.to_string())?;
            true
        } else {
            false
        };
        #[cfg(not(feature = "denoise"))]
        let ran = false;
        let j = serde_json::json!({"floor_db": floor, "floor_flatness": flatness, "speech_spans": spans.len(),
            "gate": "speech, a floor over -40 dB, flatness over 0.3", "needed": gate, "ran": ran,
            "note": if gate && !ran { "built without the denoise feature" } else { "" }});
        std::fs::write(dir.join("denoise.json"), j.to_string()).map_err(|e| e.to_string())?;
        m.steps.insert("denoise".into(), t.elapsed().as_secs_f64());
        save(m)?;
    }
    Ok(())
}

/// The person, the depth and what's in focus through a clip (docs/the roadmap G5's depth touches: text behind the head, a
/// focus blur), `fps` times a second on a grid of 96 rows (columns by the picture's shape): matte.bin, the person mask
/// (0-255), depth.bin, Depth Anything 3's inverse depth over the sample's largest (0-255, bigger = nearer), and
/// sharp.bin, the mean absolute Laplacian of the grey picture (640 px) in each cell (x4, at most 255), one grid per
/// sample in that order; matte.json holds {"fps", "w", "h", "frames": [{"t", "faces": [[x, y, w, h]]}]} (0..1 from
/// the top left).
pub fn matte_track(path: &Path, dir: &Path, backend: &str, v: &dyn Vision, fps: f64) -> Result<(), String> {
    let model = raea_sys_vision::depth::model()?;
    let mut d = raea_io_media::backend(backend)?.open_video(path)?;
    let info = d.info().clone();
    let gh = 96usize;
    let gw = ((gh as f64 * info.width as f64 / info.height.max(1) as f64).round() as usize).clamp(16, 256);
    let (mut matte, mut depth, mut sharp, mut frames) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let mut t = 0.0;
    while t < info.duration {
        let (rgba, w, h) = fit(&d.frame_at(t)?, 640);
        let pe = v.people(&rgba, w, h, false)?;
        let (dv, dw, dh) = raea_sys_vision::depth::inverse_depth(&model, &rgba, w, h)?;
        let dmax = dv.iter().cloned().fold(0f32, f32::max).max(1e-6);
        let grey: Vec<f32> = rgba.chunks(4).map(|p| 0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32).collect();
        let mut sh = vec![0f32; gw * gh];
        let mut n = vec![0f32; gw * gh];
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                let l = (4.0 * grey[i] - grey[i - 1] - grey[i + 1] - grey[i - w] - grey[i + w]).abs();
                let c = (y * gh / h) * gw + x * gw / w;
                sh[c] += l;
                n[c] += 1.0;
            }
        }
        for gy in 0..gh {
            for gx in 0..gw {
                let (fx, fy) = ((gx as f64 + 0.5) / gw as f64, (gy as f64 + 0.5) / gh as f64);
                matte.push(if pe.mw > 0 && pe.mh > 0 { pe.mask[((fy * pe.mh as f64) as usize).min(pe.mh - 1) * pe.mw + ((fx * pe.mw as f64) as usize).min(pe.mw - 1)] } else { 0 });
                depth.push((dv[((fy * dh as f64) as usize).min(dh - 1) * dw + ((fx * dw as f64) as usize).min(dw - 1)] / dmax * 255.0).round() as u8);
                let c = gy * gw + gx;
                sharp.push((sh[c] / n[c].max(1.0) * 4.0).min(255.0) as u8);
            }
        }
        frames.push(serde_json::json!({"t": r2(t), "faces": pe.faces.iter().map(|f| [r3(f.x), r3(f.y), r3(f.w), r3(f.h)]).collect::<Vec<_>>()}));
        t += 1.0 / fps;
    }
    for (name, b) in [("matte.bin", &matte), ("depth.bin", &depth), ("sharp.bin", &sharp)] {
        std::fs::write(dir.join(name), b).map_err(|e| e.to_string())?;
    }
    std::fs::write(dir.join("matte.json"), serde_json::json!({"fps": fps, "w": gw, "h": gh, "frames": frames}).to_string()).map_err(|e| e.to_string())
}

/// Where the eye goes through a clip (docs/the roadmap G1), `fps` times a second, as attention.json: {"fps", "frames":
/// [{"t", "centre": [x, y], "peak": [x, y], "focus"}]}: the saliency map's weighted centre and its peak (0..1 of the
/// picture, from the top left), and focus = 1 - the map's entropy over the largest it can be (1: one place, 0: spread);
/// "rival": the strongest place (the map's mass in a disc of 0.1) at least 0.25 of the picture away from the strongest
/// one, and "rival_ratio" its mass over that one's (mass above the map's median) (the catalogue's focal point: does something compete with it).
pub fn attention_track(path: &Path, dir: &Path, backend: &str, v: &dyn Vision, fps: f64) -> Result<(), String> {
    let mut d = raea_io_media::backend(backend)?.open_video(path)?;
    let info = d.info().clone();
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let mut frames = Vec::new();
    let mut t = 0.0;
    while t < info.duration {
        let (rgba, w, h) = fit(&d.frame_at(t)?, 640);
        if let Some((m, mw, mh)) = v.attention(&rgba, w, h)? {
            let total: f64 = m.iter().map(|x| x.max(0.0) as f64).sum();
            if total > 0.0 {
                let (mut cx, mut cy, mut ent, mut best) = (0.0, 0.0, 0.0, (0usize, f32::MIN));
                for (i, x) in m.iter().enumerate() {
                    let p = x.max(0.0) as f64 / total;
                    cx += p * ((i % mw) as f64 + 0.5) / mw as f64;
                    cy += p * ((i / mw) as f64 + 0.5) / mh as f64;
                    if p > 0.0 {
                        ent -= p * p.ln();
                    }
                    if *x > best.1 {
                        best = (i, *x);
                    }
                }
                let focus = 1.0 - ent / ((mw * mh) as f64).ln();
                let peak = [r3(((best.0 % mw) as f64 + 0.5) / mw as f64), r3(((best.0 / mw) as f64 + 0.5) / mh as f64)];
                let (rv, rr) = match rival(&m, mw, mh) {
                    Some((q, r)) => (serde_json::json!([r3(q[0]), r3(q[1])]), r3(r)),
                    None => (serde_json::Value::Null, 0.0),
                };
                frames.push(serde_json::json!({"t": r2(t), "centre": [r3(cx), r3(cy)], "peak": peak, "focus": r3(focus), "rival": rv, "rival_ratio": rr, "rival_floor": true}));
            }
        }
        t += 1.0 / fps;
    }
    std::fs::write(dir.join("attention.json"), serde_json::json!({"fps": fps, "frames": frames}).to_string()).map_err(|e| e.to_string())
}

/// The focal point's rival in a saliency map (docs/the roadmap G's catalogue): the map's mass above its median in a disc
/// of 0.1 of the picture around each cell; the strongest disc 0.25 or more from the strongest, as (its place, its mass
/// over the strongest's). Vision's maps are smooth blobs, so a single cell's maximum says little: mass does.
pub fn rival(m: &[f32], mw: usize, mh: usize) -> Option<([f64; 2], f64)> {
    let at = |i: usize| [((i % mw) as f64 + 0.5) / mw as f64, ((i / mw) as f64 + 0.5) / mh as f64];
    let floor = { let mut v = m.to_vec(); v.sort_by(f32::total_cmp); *v.get(v.len() / 2)? };
    let (rx, ry) = ((0.1 * mw as f64).ceil() as i64, (0.1 * mh as f64).ceil() as i64);
    let disc = |c: usize| -> f64 {
        let q = at(c);
        let (cx, cy) = ((c % mw) as i64, (c / mw) as i64);
        let mut s = 0.0;
        for y in (cy - ry).max(0)..=(cy + ry).min(mh as i64 - 1) {
            for x in (cx - rx).max(0)..=(cx + rx).min(mw as i64 - 1) {
                let r = at(y as usize * mw + x as usize);
                if ((r[0] - q[0]).powi(2) + (r[1] - q[1]).powi(2)).sqrt() <= 0.1 {
                    s += (m[y as usize * mw + x as usize] - floor).max(0.0) as f64;
                }
            }
        }
        s
    };
    let masses: Vec<f64> = (0..m.len()).map(disc).collect();
    let top = (0..m.len()).max_by(|a, b| masses[*a].total_cmp(&masses[*b]))?;
    if masses[top] <= 0.0 {
        return None;
    }
    let tq = at(top);
    let i = (0..m.len()).filter(|i| { let q = at(*i); ((q[0] - tq[0]).powi(2) + (q[1] - tq[1]).powi(2)).sqrt() >= 0.25 }).max_by(|a, b| masses[*a].total_cmp(&masses[*b]))?;
    Some((at(i), masses[i] / masses[top]))
}

/// The light through a clip (docs/the roadmap G4), `fps` times a second, as light.json: {"fps", "frames": [{"t", "luma",
/// "warmth", "light", "face_level", "backlit_stops", "key_fill_stops", "mixed"}]}: the frame's mean luma (0..1) and
/// warmth ((R - B) / (R + G + B)), and the framing coach's read of the light on the face (vision::coach) when there is one.
pub fn light_track(path: &Path, dir: &Path, backend: &str, v: &dyn Vision, fps: f64) -> Result<(), String> {
    let mut d = raea_io_media::backend(backend)?.open_video(path)?;
    let info = d.info().clone();
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let mut frames = Vec::new();
    let mut t = 0.0;
    while t < info.duration {
        let (rgba, w, h) = fit(&d.frame_at(t)?, 640);
        let (mut luma, mut warm, mut n) = (0.0f64, 0.0f64, 0.0f64);
        for p in rgba.chunks_exact(4).step_by(4) {
            let (r, g, b) = (p[0] as f64 / 255.0, p[1] as f64 / 255.0, p[2] as f64 / 255.0);
            luma += 0.2126 * r + 0.7152 * g + 0.0722 * b;
            warm += (r - b) / (r + g + b).max(1e-4);
            n += 1.0;
        }
        let c = raea_sys_vision::vision::coach(v, &rgba, w, h, 0.0, 0.0)?;
        frames.push(serde_json::json!({"t": r2(t), "luma": r3(luma / n), "warmth": r3(warm / n), "light": c.light,
            "face_level": c.face_level, "backlit_stops": c.backlit_stops, "key_fill_stops": c.key_fill_stops, "mixed": c.mixed}));
        t += 1.0 / fps;
    }
    std::fs::write(dir.join("light.json"), serde_json::json!({"fps": fps, "frames": frames}).to_string()).map_err(|e| e.to_string())
}

/// The face mesh's reduced read through a clip (the catalogue's orphaned mouth and look room), `fps` times a second, as
/// mesh.json: {"fps", "frames": [{"t", "mouth", "head_yaw", "gaze_yaw", "cx", "cy"} | {"t"} without a face]}: the lips'
/// gap over the eyes' width, the head's turn and the gaze in degrees (positive to the subject's left), the face's centre
/// (0..1 of the picture). The face is followed from its last points, as performance reads it.
pub fn mesh_track(path: &Path, dir: &Path, backend: &str, fps: f64) -> Result<(), String> {
    use raea_sys_vision::face;
    use crate::performance as perf;
    let models = face::models()?;
    let mut d = raea_io_media::backend(backend)?.open_video(path)?;
    let info = d.info().clone();
    let r3 = |x: f64| (x * 1000.0).round() / 1000.0;
    let (mut frames, mut crop, mut last): (Vec<serde_json::Value>, Option<face::Crop>, Option<Vec<f32>>) = (Vec::new(), None, None);
    let mut t = 0.0;
    while t < info.duration {
        let (pic, w, h) = fit(&d.frame_at(t)?, 1280);
        let mut found = match crop { Some(c) => face::mesh(&models, &pic, w, h, c)?, None => None };
        if found.is_none() {
            if let Some(c) = face::detect(&models, &pic, w, h, true)? {
                found = face::mesh(&models, &pic, w, h, c)?;
            }
        }
        crop = found.as_ref().map(|m| face::crop_around(&m.points));
        match found {
            Some(m) => {
                let pts: Vec<Vec<f64>> = m.points.iter().map(|p| p.to_vec()).collect();
                let l = perf::look(&pts, &m.blendshapes, t, last.as_deref());
                let (cx, cy) = (m.points.iter().map(|p| p[0]).sum::<f64>() / 478.0 / w as f64, m.points.iter().map(|p| p[1]).sum::<f64>() / 478.0 / h as f64);
                frames.push(serde_json::json!({"t": r2(t), "mouth": r3(l.mouth), "head_yaw": r3(l.head_yaw), "gaze_yaw": r3(l.yaw), "cx": r3(cx), "cy": r3(cy)}));
                last = Some(m.blendshapes);
            }
            None => {
                frames.push(serde_json::json!({"t": r2(t)}));
                last = None;
            }
        }
        t += 1.0 / fps;
    }
    std::fs::write(dir.join("mesh.json"), serde_json::json!({"fps": fps, "frames": frames}).to_string()).map_err(|e| e.to_string())
}
