//! A model measured before a scene uses it, and fixed on the way in (docs/handoff/solids.md; the 11-06 research on
//! generated props). Generators and downloads hand over models at any scale, facing any way, with their origin
//! anywhere, at a million triangles, with loose fragments and holes: every scene that took generated props had to fix
//! the same things by hand. `check` measures them and says each in scene lint's shape (`found[]` of `{check, level,
//! says, fix?}`, `errors` counted); `fix` writes a copy scaled to its real height with its origin at the bottom
//! centre (a parent node added over the file's roots: buffers, materials and textures copied untouched).
//!
//! Frames: glTF's, y up, +z the model's front (where a camera in front of it stands).

pub mod blender;
mod glb;
mod topo;

pub use glb::Glb;
use serde_json::{Value, json};

/// What the person says the model should be.
#[derive(Default, Clone, Debug)]
pub struct Want {
    /// its real height, metres (a chair 0.9, a mug 0.1)
    pub h: Option<f64>,
    /// its real width, depth, height (x, z, y), metres: boxy things, whose depth generators invent
    pub dims: Option<[f64; 3]>,
    /// the most triangles a scene should carry for it (150 000 by default: a hero seen close)
    pub faces: Option<usize>,
    /// degrees about y to turn it so its front faces +z (the generator's front was elsewhere)
    pub turn: f64,
    /// how far away it's seen, metres: the triangle budget when `faces` isn't given (blender::budget)
    pub seen: Option<f64>,
    /// the parts the brief says move on their own (a vise's jaw, a cabinet's door): a blob in fewer pieces is split
    pub parts: Vec<String>,
    /// the brief says it moves (bends, turns at joints): it needs a skeleton
    pub rig: bool,
    /// meant to be a closed solid (booleans, contact checks, 3D printing): open edges are a fault, not a note
    pub solid: bool,
    /// the file, for the fixes' command lines
    pub file: Option<String>,
}

fn r(x: f64) -> f64 {
    (x * 1e5).round() / 1e5
}

fn r3(p: [f64; 3]) -> Value {
    json!([r(p[0]), r(p[1]), r(p[2])])
}

/// The box of points, after turning by `turn` degrees about y and scaling by `s`.
fn bounds(verts: &[[f64; 3]], turn: f64, s: f64) -> ([f64; 3], [f64; 3]) {
    let (c, si) = (turn.to_radians().cos(), turn.to_radians().sin());
    let mut lo = [f64::MAX; 3];
    let mut hi = [f64::MIN; 3];
    for p in verts {
        let q = [s * (c * p[0] + si * p[2]), s * p[1], s * (-si * p[0] + c * p[2])];
        for k in 0..3 {
            lo[k] = lo[k].min(q[k]);
            hi[k] = hi[k].max(q[k]);
        }
    }
    (lo, hi)
}

/// `--parts jaw,body`: the parts a brief says move on their own, by name.
pub fn parts(list: &str) -> Vec<String> {
    list.split(',').map(str::trim).filter(|x| !x.is_empty()).map(String::from).collect()
}

/// The finding list, the measurements and the fix that `fix` would make, for a model.
pub fn check(g: &Glb, bytes: usize, want: &Want) -> Result<Value, String> {
    let mut found: Vec<Value> = Vec::new();
    let file = want.file.as_deref().unwrap_or("MODEL.glb");
    fn push(found: &mut Vec<Value>, check: &str, level: &str, says: String, extra: Value) {
        let mut v = json!({"check": check, "level": level, "says": says});
        if let (Some(o), Value::Object(e)) = (v.as_object_mut(), extra) {
            o.extend(e);
        }
        found.push(v);
    }
    macro_rules! say {
        ($($a:expr),*) => { push(&mut found, $($a),*) };
    }
    let required: Vec<String> = g.json["extensionsRequired"].as_array().into_iter().flatten().filter_map(|e| e.as_str().map(str::to_string)).collect();
    if let Some(e) = required.iter().find(|e| e.contains("draco") || e.contains("meshopt") || e.contains("quantization")) {
        let f = blender::fix("convert.py", &format!("{file} {}", blender::beside(file, "plain")));
        say!("compressed", "error", blender::says(&format!("the file's meshes are compressed ({e}): unpack it first"), &f), json!({"fix": f}));
        return Ok(report(found, json!({"bytes": bytes}), json!(null)));
    }
    let w = g.world()?;
    if w.tris.is_empty() {
        say!("empty", "error", "no triangles: the default scene draws no mesh".into(), json!({}));
        return Ok(report(found, json!({"bytes": bytes}), json!(null)));
    }
    let (lo, hi) = bounds(&w.verts, 0.0, 1.0);
    let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, (lo[2] + hi[2]) / 2.0];
    let topo = topo::measure(&w.verts, &w.tris, size.iter().cloned().fold(0.0, f64::max));

    // size: against the real height, else a sanity range (a model authored in centimetres is 100 times too big)
    let s = match want.h {
        Some(h) if size[1] > 1e-9 => {
            let s = h / size[1];
            if (s - 1.0).abs() > 0.01 {
                say!("size", "warning", format!("{:.3} m tall, wants {h} m: scale {:.4}", size[1], s), json!({"fix": {"scale": r(s)}}));
            } else {
                say!("size", "ok", format!("{:.3} m tall, as wanted", size[1]), json!({}));
            }
            s
        }
        _ => {
            let big = size.iter().cloned().fold(0.0, f64::max);
            if !(0.01..=50.0).contains(&big) {
                say!("size", "warning", format!("{:.3} x {:.3} x {:.3}: {} for most props; pass --h with its real height", size[0], size[1], size[2], if big > 50.0 { "too big (authored in cm or mm?)" } else { "tiny" }), json!({}));
            } else {
                say!("size", "note", format!("{:.3} x {:.3} x {:.3} (w x h x d); its real height isn't given (--h), so it's kept", size[0], size[1], size[2]), json!({}));
            }
            1.0
        }
    };
    let (flo, fhi) = bounds(&w.verts, want.turn, s);
    let fsize = [fhi[0] - flo[0], fhi[1] - flo[1], fhi[2] - flo[2]];
    // dims: generators invent the depth of boxy things
    if let Some([dw, dd, dh]) = want.dims {
        for (axis, got, wanted) in [("width (x)", fsize[0], dw), ("depth (z)", fsize[2], dd), ("height (y)", fsize[1], dh)] {
            if wanted > 0.0 && (got / wanted - 1.0).abs() > 0.15 {
                say!("dims", "warning", format!("its {axis} is {got:.3} m where the real one is {wanted} m ({:+.0}%): if it's the front-to-back depth, the generator invented it; turn it (--turn) if the axes are swapped, else rebuild it in code", (got / wanted - 1.0) * 100.0), json!({}));
            }
        }
    }
    // the pivot: raea stands a model at its origin, so the origin belongs at the bottom centre
    let off = [centre[0], lo[1], centre[2]];
    let span = size[0].max(size[2]).max(size[1]).max(1e-9);
    if off[1].abs() > 0.01 * size[1].max(1e-9) || (off[0].hypot(off[2])) > 0.05 * span {
        say!("pivot", "warning", format!("its origin is {:.3} m from its bottom centre (bottom at y {:.3}, footprint centred at x {:.3}, z {:.3}): placed at y 0 it floats, sinks or sits off its spot", (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt(), lo[1], centre[0], centre[2]), json!({"fix": {"origin": "bottom centre"}}));
    } else {
        say!("pivot", "ok", "its origin is at its bottom centre".into(), json!({}));
    }
    if want.turn != 0.0 {
        say!("facing", "note", format!("turned {}° about y so its front faces +z", want.turn), json!({"fix": {"turn": want.turn}}));
    } else {
        say!("facing", "note", "facing isn't measured: render it from the front (+z) and pass --turn 90|-90|180 if the front faces elsewhere (Tripo's default front is +x; raea asks for align_image)".into(), json!({}));
    }
    // weight
    let faces = want.faces.unwrap_or_else(|| want.seen.map_or(150_000, blender::budget));
    let n = w.tris.len();
    if n > faces {
        let seen = want.seen.map(|d| format!(" seen from {d} m")).unwrap_or_default();
        let f = blender::fix("decimate.py", &format!("{file} {} {faces}", blender::beside(file, &format!("{}k", faces / 1000))));
        say!("triangles", "warning", blender::says(&format!("{n} triangles, over the {faces} a scene should carry for it{seen} (or regenerate with a lower face_limit)"), &f), json!({"fix": f}));
    } else {
        say!("triangles", "ok", format!("{n} triangles"), json!({}));
    }
    if bytes > 20 << 20 {
        say!("file", "warning", format!("{:.1} MB, over the 20 MB a model should be (bevy/limits): shrink its textures or its mesh", bytes as f64 / 1048576.0), json!({}));
    }
    let imgs = g.images();
    if let Some((tw, th)) = imgs.iter().flatten().max_by_key(|(a, b)| a * b) {
        if *tw > 4096 || *th > 4096 {
            say!("textures", "warning", format!("a {tw}x{th} texture: 2048 is plenty for a prop, 4096 for a hero seen close"), json!({}));
        }
    }
    // shape health: the welded surface's open and over-shared edges, and loose bits
    if topo.pieces > 1 {
        let loose: Vec<Value> = topo.loose.iter().map(|(c, t)| json!({"center": r3(*c), "triangles": t})).collect();
        let level = if topo.loose.is_empty() { "note" } else { "warning" };
        let says = if topo.loose.is_empty() {
            format!("{} separate pieces (parts modelled apart: fine for a prop drawn as it is)", topo.pieces)
        } else {
            format!("{} separate pieces, {} of them loose bits under 0.5% of its triangles (generator debris: floating specks near the model)", topo.pieces, topo.loose.len())
        };
        if topo.loose.is_empty() {
            say!("pieces", level, says, json!({"loose": loose}));
        } else {
            let f = blender::fix("repair.py", &format!("{file} {}", blender::beside(file, "clean")));
            say!("pieces", level, blender::says(&says, &f), json!({"loose": loose, "fix": f}));
        }
    }
    if !want.parts.is_empty() && topo.pieces < want.parts.len() {
        let boxes: Vec<String> = want.parts.iter().map(|p| format!("{p}=X0,Y0,Z0,X1,Y1,Z1")).collect();
        let f = blender::fix("split.py", &format!("{file} {} {} --each", blender::beside(file, "parts"), boxes.join(" ")));
        say!("parts", "warning", blender::says(&format!("{} piece(s) where the brief has {} parts that move on their own ({}): give each part's box from this report's box", topo.pieces, want.parts.len(), want.parts.join(", ")), &f), json!({"fix": f}));
    }
    if topo.open_edges > 0 || topo.shared_edges > 0 {
        let words = format!("{} open edges and {} edges shared by 3+ faces: it draws fine, but it isn't a closed solid (`manifold.import` and contact checks treat it as triangles)", topo.open_edges, topo.shared_edges);
        if want.solid {
            let f = blender::fix("repair.py", &format!("{file} {}", blender::beside(file, "closed")));
            say!("surface", "warning", blender::says(&words, &f), json!({"fix": f}));
        } else {
            say!("surface", "note", words, json!({}));
        }
    }
    let skins = g.json["skins"].as_array().map_or(0, Vec::len);
    if want.rig && skins == 0 {
        let f = blender::fix("rig.py", &format!("{file} {} --bone NAME=HX,HY,HZ:TX,TY,TZ[:PARENT] ... --pose CLIP=BONE:RX,RY,RZ", blender::beside(file, "rigged")));
        say!("skeleton", "warning", blender::says("the brief says it moves and it has no skeleton: give it bones from this report's box, joint by joint", &f), json!({"fix": f}));
    } else if want.rig {
        say!("skeleton", "ok", format!("{skins} skin(s), clips: {}", g.json["animations"].as_array().map_or(0, Vec::len)), json!({}));
    }
    if topo.degenerate > 0 {
        say!("degenerate", "note", format!("{} triangles with no area", topo.degenerate), json!({}));
    }
    if w.skipped > 0 {
        say!("skipped", "note", format!("{} primitives drawn as lines or points, not measured", w.skipped), json!({}));
    }
    let measured = json!({
        "triangles": n, "vertices": w.verts.len(), "bytes": bytes,
        "size": r3(size), "box": {"min": r3(lo), "max": r3(hi)}, "center": r3(centre), "bottom": r(lo[1]),
        "pieces": topo.pieces, "openEdges": topo.open_edges, "sharedEdges": topo.shared_edges,
        "textures": imgs.iter().map(|i| i.map(|(a, b)| json!([a, b])).unwrap_or(json!(null))).collect::<Vec<_>>(),
        "materials": g.json["materials"].as_array().map(|m| m.len()).unwrap_or(0),
    });
    let t = [-(flo[0] + fhi[0]) / 2.0, -flo[1], -(flo[2] + fhi[2]) / 2.0];
    let fix = json!({"scale": r(s), "turn": want.turn, "translate": r3(t), "size": r3(fsize)});
    Ok(report(found, measured, fix))
}

fn report(found: Vec<Value>, measured: Value, fix: Value) -> Value {
    let errors = found.iter().filter(|f| f["level"] == "error").count();
    let warnings = found.iter().filter(|f| f["level"] == "warning").count();
    json!({"errors": errors, "warnings": warnings, "measured": measured, "fixed": fix, "found": found})
}

/// The file with a parent node over its scene's roots that scales it by `scale`, turns it `turn` degrees about y and
/// moves it by `translate` (applied in that order): its JSON rewritten, its buffers untouched.
pub fn fix(g: &Glb, scale: f64, turn: f64, translate: [f64; 3]) -> Result<Vec<u8>, String> {
    let mut j = g.json.clone();
    let scene = j["scene"].as_u64().unwrap_or(0) as usize;
    let roots = j["scenes"][scene]["nodes"].clone();
    let nodes = j["nodes"].as_array_mut().ok_or("no nodes")?;
    let half = turn.to_radians() / 2.0;
    nodes.push(json!({
        "name": "raea-fixed",
        "translation": translate, "rotation": [0.0, half.sin(), 0.0, half.cos()], "scale": [scale, scale, scale],
        "children": roots,
    }));
    let top = nodes.len() - 1;
    j["scenes"][scene]["nodes"] = json!([top]);
    j["asset"]["generator"] = json!(format!("{} + raea gen check", j["asset"]["generator"].as_str().unwrap_or("unknown")));
    g.with_json(&j)
}

#[cfg(test)]
mod tests;
