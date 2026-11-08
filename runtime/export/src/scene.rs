//! `raea scene lint|explain|light`: a composition's frames stepped and handed to raea-sys-contact (what each part rests
//! on, and parts that clip) and raea-sys-light-meter (the light on a subject); and the clipping check on a render's
//! frames.

use serde_json::{Value, json};

use super::{Frame, bundle_composition, comp_opts, flag, scene_frames_to};

/// A frame's 3D walked into its parts (raea-runtime-scene).
pub(crate) fn items(f: &Frame) -> Vec<raea_core_scene::Item> {
    raea_runtime_scene::items_of(&f.scene)
}

/// Each frame's items, walked once (a frame's tree was walked again for each kind of part: 11 times a live frame)
pub(crate) fn all_items(frames: &[(i64, Frame)]) -> Vec<Vec<raea_core_scene::Item>> {
    frames.iter().map(|(_, f)| items(f)).collect()
}

/// raea_sys_contact::told on a render's frames: the parts that clip where the camera sees them, said on stderr
/// (RAEA_CLIP_CHECK=0 skips it).
pub(crate) fn told(frames: &[(i64, Frame)], w: u32, h: u32, fps: f64) -> Vec<Value> {
    if std::env::var("RAEA_CLIP_CHECK").as_deref() == Ok("0") || frames.iter().all(|(_, f)| f.scene.is_empty()) {
        return Vec::new();
    }
    let nums: Vec<i64> = frames.iter().map(|(f, _)| *f).collect();
    raea_sys_contact::told(&all_items(frames), &nums, w, h, fps)
}

pub const USAGE: &str = "raea scene lint|explain|light EDIT.tsx [--composition ID] [--frame N] [--part NAME] [--json]";

/// `raea scene lint|explain EDIT.tsx [--composition ID] [--frame N] [--json]`: the frame's parts measured
/// (frame 0 by default). `lint` says the faults (errors fail it: floating, sunk, clipping; warnings: in the air,
/// tilted); `explain` says what every part rests on. --json prints the findings (each part's support, gap and the y
/// that fixes it) instead.
pub fn cli(args: &[String]) -> Result<String, String> {
    let usage = USAGE;
    let (Some(mode), Some(edit)) = (args.get(1).map(String::as_str), args.get(2)) else { return Err(usage.into()) };
    if mode == "light" {
        return light(args);
    }
    if mode != "lint" && mode != "explain" {
        return Err(usage.into());
    }
    let tsx = std::path::PathBuf::from(edit).canonicalize().map_err(|e| format!("{edit}: {e}"))?;
    let dir = tsx.parent().unwrap().to_path_buf();
    let frame = frame_of(args)?;
    let js = std::env::temp_dir().join(format!("raea-scene-{}.js", std::process::id()));
    bundle_composition(&tsx, &js, &comp_opts(args)?, None)?;
    // and the frame after it, to see what moves
    let got = scene_frames_to(&js, &dir, frame + 2);
    let _ = std::fs::remove_file(&js);
    let (frames, ..) = got?.ok_or("the composition has no 3D")?;
    let k = frames.iter().position(|(f, _)| *f == frame).ok_or_else(|| format!("no frame {frame}"))?;
    // the frame, and the next (or the one before, at the end) to see what moves
    let other = if k + 1 < frames.len() { k + 1 } else { k.saturating_sub(1) };
    let (mut found, errors) = raea_sys_contact::lint(&items(&frames[k].1), &items(&frames[other].1));
    // and the Blender steps its models call for (a format raea can't load, triangles over budget for the distance)
    let (more, unloadable) = raea_sys_contact::assets(&items(&frames[k].1));
    found.extend(more);
    let errors = errors + unloadable;
    let shown: Vec<&Value> = found.iter().filter(|v| mode == "explain" || !matches!(v["level"].as_str(), Some("ok") | Some("note"))).collect();
    let text = if args.iter().any(|a| a == "--json") {
        serde_json::to_string_pretty(&json!({"frame": frame, "errors": errors, "found": shown})).unwrap_or_default()
    } else {
        let mut t: Vec<String> = shown.iter().map(|v| format!("{:<8} {}", v["level"].as_str().unwrap_or(""), v["says"].as_str().unwrap_or(""))).collect();
        let warnings = found.iter().filter(|v| v["level"] == "warning").count();
        t.push(format!("frame {frame}: {} parts, {errors} errors, {warnings} warnings", found.iter().filter(|v| v["check"] != "clips" && v["check"] != "tilted" && v["check"] != "skipped").count()));
        t.join("\n")
    };
    // errors fail a lint, with nothing after the findings (--json stays one document)
    if errors > 0 && mode == "lint" {
        println!("{text}");
        std::process::exit(1);
    }
    Ok(text)
}

/// One composition's frames, stepped.
fn frames_of(args: &[String], tsx: &std::path::Path, comp: &str) -> Result<(Vec<(i64, Frame)>, f32), String> {
    let dir = tsx.parent().unwrap().to_path_buf();
    let mut opts = comp_opts(args)?;
    opts.composition = Some(comp.to_string()).filter(|c| !c.is_empty());
    let js = std::env::temp_dir().join(format!("raea-light-{}-{comp}.js", std::process::id()));
    bundle_composition(tsx, &js, &opts, None)?;
    let got = scene_frames_to(&js, &dir, 1);
    let _ = std::fs::remove_file(&js);
    let (frames, w, h, _) = got?.ok_or("the composition has no 3D")?;
    Ok((frames, w as f32 / h.max(1) as f32))
}

/// `raea scene light`: the light on a shot's subject, metered (raea-sys-light-meter; docs/scene.feature, "A light plan").
fn light(args: &[String]) -> Result<String, String> {
    let usage = "raea scene light EDIT.tsx [--composition ID[,ID...]] [--frame N|last] [--part NAME] [--json]";
    let Some(edit) = args.get(2) else { return Err(usage.into()) };
    let tsx = std::path::PathBuf::from(edit).canonicalize().map_err(|e| format!("{edit}: {e}"))?;
    let flag = |k: &str| flag(args, k);
    let comps: Vec<String> = flag("--composition").map(|c| c.split(',').map(str::to_string).collect()).unwrap_or_else(|| vec![String::new()]);
    let part = flag("--part");
    let json_out = args.iter().any(|a| a == "--json");
    let pick = |frames: &[(i64, Frame)], which: &str| -> Result<usize, String> {
        match which {
            "last" => Ok(frames.len() - 1),
            f => {
                let f: i64 = f.parse().map_err(|_| "--frame takes a frame number or last")?;
                frames.iter().position(|(n, _)| *n == f).ok_or_else(|| format!("no frame {f}"))
            }
        }
    };
    let mut shots: Vec<(String, Vec<(i64, Value)>)> = Vec::new();
    for c in &comps {
        let (frames, aspect) = frames_of(args, &tsx, c)?;
        let all = all_items(&frames);
        // one composition: the frame asked for; a sequence: each shot's first and last, for its cuts
        let at: Vec<usize> = if comps.len() == 1 { vec![pick(&frames, flag("--frame").as_deref().unwrap_or("0"))?] } else { vec![0, frames.len() - 1] };
        let mut m = Vec::new();
        for k in at {
            m.push((frames[k].0, raea_sys_light_meter::meter(&all[k], part.as_deref(), aspect).map_err(|e| format!("{c} frame {}: {e}", frames[k].0))?));
        }
        shots.push((c.clone(), m));
    }
    let cuts = raea_sys_light_meter::cuts(&shots);
    if json_out {
        let shots: Vec<Value> = shots.iter().map(|(c, m)| json!({"composition": c, "frames": m.iter().map(|(f, v)| json!({"frame": f, "meter": v})).collect::<Vec<_>>()})).collect();
        return Ok(serde_json::to_string_pretty(&json!({"shots": shots, "cuts": cuts})).unwrap_or_default());
    }
    let mut t = Vec::new();
    for (c, m) in &shots {
        for (f, v) in m {
            t.push(raea_sys_light_meter::say(if c.is_empty() { "the composition" } else { c }, *f, v));
        }
    }
    for c in &cuts {
        t.push(format!("cut {}", c["says"].as_str().unwrap_or("")));
    }
    Ok(t.join("\n"))
}

/// `--frame N`, 0 when it isn't given (scene lint and render --review).
pub(crate) fn frame_of(args: &[String]) -> Result<i64, String> {
    match flag(args, "--frame") {
        Some(f) => f.parse().map_err(|_| "--frame takes a frame number".to_string()),
        None => Ok(0),
    }
}
