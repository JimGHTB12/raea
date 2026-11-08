//! `raea render EDIT.tsx --review DIR [--frame N]`: the same fixed cameras every round, in colour and in clay. The
//! cameras come from the frame's parts (scene lint's boxes; ground slabs and other flat, wide parts left out): hero
//! (the scene's own camera), front (+z looking -z), right, top, iso, and low (0.3 m off the floor, for feet and
//! floaters); an interior (a ceiling over the plan) gets the same views from inside, under the ceiling. Each is rendered by a child raea with RAEA_CAMERA (raea-core-review, which also places the cameras) and, for clay, RAEA_CLAY
//! (runtime/scene3d's clay.rs). The tiles are stitched into DIR/colour.png and DIR/clay.png, and DIR/review.json gives
//! the cameras, each tile's place on the sheets and its exposure, and what the exposure found.

use super::*;
use crate::scene::items;
use raea_core_review::{TILE_W, VIEWS, boxes_of, cameras, ceiling, inside, parts_box, passed, whole};

pub fn review(args: &[String], dir_out: &Path) -> R<String> {
    let tsx = PathBuf::from(&args[1]).canonicalize().map_err(|e| format!("{}: {e}", args[1]))?;
    let dir = tsx.parent().unwrap().to_path_buf();
    let frame = crate::scene::frame_of(args)?;
    std::fs::create_dir_all(dir_out).map_err(|e| format!("{}: {e}", dir_out.display()))?;
    // the frame's parts, measured as scene lint measures them
    let js = std::env::temp_dir().join(format!("raea-review-{}.js", std::process::id()));
    bundle_composition(&tsx, &js, &comp_opts(args)?, None)?;
    let got = scene_frames_to(&js, &dir, frame + 1);
    let _ = std::fs::remove_file(&js);
    let (frames, w, h, _) = got?.ok_or("--review: the composition has no 3D")?;
    let k = frames.iter().position(|(f, _)| *f == frame).ok_or_else(|| format!("--review: no frame {frame}"))?;
    let its = items(&frames[k].1);
    let (found, _) = raea_sys_contact::lint(&its, &its);
    let (lo, hi) = parts_box(&found).ok_or("--review: no part with a size in the frame")?;
    let aspect = w as f64 / h.max(1) as f64;
    let mut cams: Vec<(&str, Option<([f64; 3], [f64; 3], f64)>)> = vec![("hero", None)];
    let all = boxes_of(&found);
    let (wlo, whi) = whole(&all).unwrap_or((lo, hi));
    let ceil = ceiling(&all, wlo, whi);
    let fixed = match ceil { Some(y) => inside(wlo, whi, y), None => cameras(lo, hi, aspect) };
    cams.extend(fixed.into_iter().map(|(v, e, t, f)| (v, Some((e, t, f)))));
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let rest = passed(args);
    let mut tiles = Vec::new();
    let mut said = Vec::new();
    for (view, cam) in &cams {
        for mode in ["colour", "clay"] {
            let out = dir_out.join(format!("{view}-{mode}.png"));
            let mut cmd = std::process::Command::new(&exe);
            cmd.arg("render").arg(&tsx).args(&rest).args(["--frame", &frame.to_string(), "-o"]).arg(&out);
            cmd.env_remove("RAEA_CAMERA").env_remove("RAEA_CLAY");
            if let Some((e, t, f)) = cam {
                cmd.env("RAEA_CAMERA", format!("{},{},{},{},{},{},{}", e[0], e[1], e[2], t[0], t[1], t[2], f));
            }
            if mode == "clay" {
                cmd.env("RAEA_CLAY", "1");
            }
            let run = cmd.output().map_err(|e| format!("--review: {e}"))?;
            if !run.status.success() || !out.is_file() {
                let text = format!("{}{}", String::from_utf8_lossy(&run.stdout), String::from_utf8_lossy(&run.stderr));
                return Err(format!("--review: the {view} {mode} render failed: {}", text.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no output")));
            }
        }
        let colour = dir_out.join(format!("{view}-colour.png"));
        let (_, ev) = raea_sys_light_meter::exposure::of_pngs(std::slice::from_ref(&colour))?;
        for f in ev["found"].as_array().into_iter().flatten() {
            said.push(json!({"check": "exposure", "level": f["level"], "says": format!("{view} tile: {}", f["says"].as_str().unwrap_or("").trim_start_matches("the frame: "))}));
        }
        tiles.push(json!({"view": view, "colour": format!("{view}-colour.png"), "clay": format!("{view}-clay.png"), "exposure": ev["exposure"],
            "camera": cam.map(|(e, t, f)| json!({"eye": e, "target": t, "fov": f}))}));
    }
    // the sheets: three across, two down, each tile TILE_W wide
    let th = (TILE_W as f64 / aspect).round().max(1.0) as u32;
    for mode in ["colour", "clay"] {
        let mut sheet = image::RgbaImage::from_pixel(TILE_W * 3, th * 2, image::Rgba([0, 0, 0, 255]));
        for (i, view) in VIEWS.iter().enumerate() {
            let img = image::open(dir_out.join(format!("{view}-{mode}.png"))).map_err(|e| format!("{view}-{mode}.png: {e}"))?.to_rgba8();
            let small = image::imageops::resize(&img, TILE_W, th, image::imageops::FilterType::Triangle);
            image::imageops::overlay(&mut sheet, &small, (i as u32 % 3 * TILE_W) as i64, (i as u32 / 3 * th) as i64);
        }
        sheet.save(dir_out.join(format!("{mode}.png"))).map_err(|e| format!("{mode}.png: {e}"))?;
    }
    for (i, t) in tiles.iter_mut().enumerate() {
        t["rect"] = json!([i as u32 % 3 * TILE_W, i as u32 / 3 * th, TILE_W, th]);
    }
    let cameras: Vec<Value> = tiles.iter().map(|t| json!({"view": t["view"], "camera": t["camera"]})).collect();
    std::fs::write(dir_out.join("cameras.json"), serde_json::to_string_pretty(&json!({"frame": frame, "box": {"min": lo, "max": hi}, "interior": ceil.is_some(), "cameras": cameras})).unwrap_or_default()).map_err(|e| e.to_string())?;
    let doc = json!({"frame": frame, "sheets": {"colour": "colour.png", "clay": "clay.png"}, "size": [TILE_W * 3, th * 2],
        "caption": if ceil.is_some() { "An interior, seen from inside: tiles left to right, top to bottom: hero, front, right, top (under the ceiling), iso, low (0.3 m). Clay: every standard surface light grey and matte." } else { "Tiles left to right, top to bottom: hero, front, right, top, iso, low (0.3 m). Clay: every standard surface light grey and matte." },
        "box": {"min": lo, "max": hi}, "tiles": tiles, "found": said});
    std::fs::write(dir_out.join("review.json"), serde_json::to_string_pretty(&doc).unwrap_or_default()).map_err(|e| e.to_string())?;
    let mut msg = format!("APPLIED  review of frame {frame}: {} and {} (hero, front, right, top, iso, low), cameras and exposure in {}", dir_out.join("colour.png").display(), dir_out.join("clay.png").display(), dir_out.join("review.json").display());
    for f in &said {
        msg.push_str(&format!("\n  {}: {}", f["level"].as_str().unwrap_or(""), f["says"].as_str().unwrap_or("")));
    }
    Ok(msg)
}
