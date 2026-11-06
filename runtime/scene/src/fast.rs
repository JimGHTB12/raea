//! Live play's fast path: a frame's changes found without making a render's rows (LiveScene::frame_fast).

use super::*;

/// the most values a part keys in a frame: placement (9), animation (1, 8 mixed, 32 graph), morphs (64) and a few more
const WANT: usize = 128;

impl LiveScene {
    /// `diff(current(f))` for a frame whose parts are all known, without the rows: each known part's changing values worked out into a fixed array and held
    /// against what was sent in place, as diff holds a row (the same rules: recoloured meshes keyed, the emissive strength
    /// only where the first frame glows, opacity once seen-through, visible and scale back to 1, lights without visible),
    /// so a frame allocates only for what it changes. None, with nothing changed, when a part is new.
    pub(crate) fn frame_fast(&mut self, f: Shot) -> Option<(Vec<raea_core_program::eval::Tup>, Vec<(String, String)>)> {
        if self.rows.is_empty() {
            return None;
        }
        // rows of other kinds (components, entities, fires...): the full path, which diffs them (G15)
        if self.rows.values().any(|r| !LIVE_GROUPS.contains(&r.kind.as_str())) {
            return None;
        }
        // a lens, motion blur or a toon surface's keyed values: the full path, which keys them (camera_of, rows_into_of)
        if self.cam.keys().any(|k| k.starts_with("lens_") || k.starts_with("mb_")) || self.rows.values().any(|r| ["sh_r", "rim_k", "spec_k", "outline_w"].iter().any(|k| r.first.contains_key(*k))) {
            return None;
        }
        let its = items_of(f.scene);
        // pass 1: every part's id (as tracks_of names it on one frame), or None before anything changes
        let mut parts: Vec<(usize, &String)> = Vec::with_capacity(its.len());
        // each group's (kind, name) counts and the key, borrowed and written in place (ZeroHeap)
        let mut seen: Vec<(&str, &str, usize)> = Vec::new();
        let mut key = String::new();
        for kinds in [&["directional", "point", "spot", "rect"][..], &["mesh"][..], &["model"][..], &["panel"][..], &["decal"][..]] {
            seen.clear();
            for (i, it) in its.iter().enumerate().filter(|(_, it)| kinds.contains(&it.kind)) {
                let base = base_ident(it);
                let k = match seen.iter_mut().find(|s| s.0 == it.kind && s.1 == base) {
                    Some(s) => {
                        s.2 += 1;
                        s.2 - 1
                    }
                    None => {
                        seen.push((it.kind, base, 1));
                        0
                    }
                };
                key.clear();
                {
                    use std::fmt::Write as _;
                    let _ = write!(key, "{}|{base}#{k}", it.kind);
                }
                let id = self.ids.get(key.as_str())?;
                if it.kind == "mesh" && it.shape.is_none() {
                    continue;
                }
                if !self.rows.contains_key(id) {
                    return None;
                }
                parts.push((i, id));
            }
        }
        let mut out: Vec<raea_core_program::eval::Tup> = Vec::new();
        // the camera, as diff holds it
        let mut cam: [(&str, Option<f64>); 9] = [("x", None), ("y", None), ("z", None), ("look_x", None), ("look_y", None), ("look_z", None), ("fov", None), ("ambient", None), ("focus", None)];
        match its.iter().find(|i| i.kind == "camera") {
            Some(c) => {
                let pos = vec3(&c.props, "position", [0.0, 0.0, 5.0]);
                let look = vec3(&c.props, "lookAt", [0.0, 0.0, 0.0]);
                for k in 0..3 {
                    cam[k].1 = Some(r6(pos[k]));
                    cam[3 + k].1 = Some(r6(look[k]));
                }
                cam[6].1 = Some(r6(n(&c.props, "fov", 50.0)));
                if self.cam.contains_key("focus") {
                    cam[8].1 = Some(r6(focus_of(c)));
                }
            }
            None => {
                for (k, d) in [0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 50.0].into_iter().enumerate() {
                    cam[k].1 = Some(match self.cam.get(cam[k].0) {
                        Some(V::N(v)) => *v,
                        _ => d,
                    });
                }
            }
        }
        let amb = its.iter().filter(|i| i.kind == "ambient").map(|i| n(&i.props, "intensity", 1.0)).sum::<f64>() * AMBIENT;
        cam[7].1 = if amb > 0.0 { Some(r6(amb)) } else { self.cam.contains_key("ambient").then_some(0.0) };
        let mut vals = BTreeMap::new();
        for (k, v) in cam {
            if let Some(v) = v.map(V::N) {
                if self.cam.get(k) != Some(&v) {
                    self.cam.insert(k.into(), v.clone());
                    vals.insert(k.to_string(), v);
                }
            }
        }
        // the paint look's keyed values, as camera_of keys them
        if self.cam.contains_key("paint_bands") {
            if let Some(l) = its.iter().find(|i| i.kind == "framelook") {
                for (k, v) in paint_keyed(&l.props) {
                    let v = V::N(r6(v));
                    if self.cam.get(k) != Some(&v) {
                        self.cam.insert(k.into(), v.clone());
                        vals.insert(k.to_string(), v);
                    }
                }
            }
        }
        if !vals.is_empty() {
            out.push(raea_core_program::eval::Tup { kind: "camera".into(), id: "cam".into(), vals });
        }
        for r in self.rows.values_mut() {
            r.seen = false;
        }
        // pass 2: each part's values, then held against what was sent
        for (i, id) in parts {
            let it = &its[i];
            let mut want: [(&str, Option<f64>); WANT] = [("", None); WANT];
            let mut nw = 0usize;
            let mut put = |k: &'static str, v: f64| {
                want[nw] = (k, Some(r6(v)));
                nw += 1;
            };
            let mut color: Option<std::borrow::Cow<str>> = None;
            let place = |put: &mut dyn FnMut(&'static str, f64)| {
                let (s, q, t) = it.world.to_scale_rotation_translation();
                let (ry, rx, rz) = q.to_euler(EulerRot::YXZ);
                for (name, v) in [("x", t.x), ("y", t.y), ("z", t.z), ("rot_x", rx), ("rot_y", ry), ("rot_z", rz)] {
                    put(name, v as f64);
                }
                let sc = [s.x as f64, s.y as f64, s.z as f64];
                if sc.iter().any(|v| (v - 1.0).abs() > 1e-9) {
                    for (name, v) in [("scale_x", sc[0]), ("scale_y", sc[1]), ("scale_z", sc[2])] {
                        put(name, v);
                    }
                }
            };
            let kind: &'static str = match it.kind {
                "directional" | "point" | "spot" | "rect" => {
                    let dir = it.kind == "directional";
                    put(if dir { "lux" } else { "lumens" }, n(&it.props, "intensity", 1.0) * if dir { LUX } else { LUMENS } * 1.0);
                    let v = vec3(&it.props, "position", if dir { [0.0, 1.0, 0.0] } else { [0.0; 3] });
                    let w = it.world.transform_point3(Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32));
                    let pos = [w.x as f64, w.y as f64, w.z as f64];
                    let target = vec3(&it.props, "target", [0.0; 3]);
                    if !dir {
                        put("x", pos[0]);
                        put("y", pos[1]);
                        put("z", pos[2]);
                    }
                    if it.kind != "point" {
                        put("dir_x", target[0] - pos[0]);
                        put("dir_y", target[1] - pos[1]);
                        put("dir_z", target[2] - pos[2]);
                    }
                    "light"
                }
                "mesh" => {
                    static EMPTY: std::sync::LazyLock<Map<String, Value>> = std::sync::LazyLock::new(Map::new);
                    let m: &Map<String, Value> = it.material.as_ref().map(|m| &*m.props).unwrap_or(&EMPTY);
                    color = Some(m.get("color").and_then(Value::as_str).and_then(hex_cow).unwrap_or(std::borrow::Cow::Borrowed("#cccccc")));
                    put("rough", n(m, "roughness", 0.5));
                    put("metal", n(m, "metalness", 0.0));
                    if m.get("emissive").and_then(Value::as_str).and_then(hex_cow).is_some_and(|e| e != "#000000") {
                        put("emissive_k", n(m, "emissiveIntensity", 1.0));
                    }
                    if m.get("transparent") == Some(&Value::Bool(true)) || num(m, "opacity").is_some_and(|o| o < 1.0) {
                        put("opacity", n(m, "opacity", 1.0));
                    }
                    place(&mut put);
                    "mesh"
                }
                "model" => {
                    if it.props.contains_key("animation") {
                        put("anim_t", n(&it.props, "time", 0.0));
                    }
                    if let Some(Value::Array(a)) = it.props.get("animations") {
                        for (i, c) in a.iter().enumerate().take(4) {
                            let o = c.as_object().cloned().unwrap_or_default();
                            put(MIX_COLS[i].0, n(&o, "time", 0.0));
                            put(MIX_COLS[i].1, n(&o, "weight", 1.0));
                        }
                    }
                    if let Some(Value::Array(a)) = it.props.get("graph") {
                        for (i, c) in a.iter().enumerate().take(16) {
                            let o = c.as_object().cloned().unwrap_or_default();
                            put(GRAPH_COLS[i][0], n(&o, "weight", 1.0));
                            put(GRAPH_COLS[i][1], n(&o, "time", 0.0));
                        }
                    }
                    if let Some(Value::Array(a)) = it.props.get("morphs") {
                        for (i, c) in a.iter().enumerate().take(MORPH_COLS.len()) {
                            put(MORPH_COLS[i], c.get("weight").and_then(Value::as_f64).unwrap_or(0.0));
                        }
                    }
                    if it.props.contains_key("look") {
                        put("look", look_of(&it.props));
                    }
                    if it.props.contains_key("screenPower") {
                        put("screen_power", n(&it.props, "screenPower", 1.0));
                    }
                    if let Some(f) = it.props.get("cloth").and_then(|c| c.get("frame")).and_then(Value::as_f64) {
                        put("cloth_frame", f);
                    }
                    place(&mut put);
                    "model"
                }
                "decal" => {
                    let c = rgb(&it.props.get("color").and_then(Value::as_str).and_then(hex_cow).unwrap_or(std::borrow::Cow::Borrowed("#ffffff")));
                    for (k, name) in ["col_r", "col_g", "col_b"].into_iter().enumerate() {
                        put(name, c[k]);
                    }
                    put("opacity", n(&it.props, "opacity", 1.0));
                    put("rough", n(&it.props, "roughness", 0.6));
                    put("metal", n(&it.props, "metalness", 0.0));
                    place(&mut put);
                    "decal"
                }
                _ => {
                    if it.props.contains_key("screenPower") {
                        put("screen_power", n(&it.props, "screenPower", 1.0));
                    }
                    if let Some(b) = screen_bright(&it.props) {
                        put("screen_bright", b);
                    }
                    place(&mut put);
                    "panel"
                }
            };
            let r = self.rows.get_mut(id).expect("checked in pass 1");
            r.seen = true;
            r.missing = 0;
            let set = |want: &mut [(&str, Option<f64>); WANT], nw: &mut usize, k: &'static str, v: Option<f64>, only_if_absent: bool| {
                match want[..*nw].iter_mut().find(|w| w.0 == k) {
                    Some(w) => {
                        if !only_if_absent {
                            w.1 = v;
                        }
                    }
                    None => {
                        want[*nw] = (k, v);
                        *nw += 1;
                    }
                }
            };
            if kind == "mesh" {
                let c = color.as_deref().unwrap_or("#cccccc");
                if !matches!(r.first.get("color"), Some(V::S(s)) if s == c) {
                    r.recoloured = true;
                }
                if r.recoloured {
                    let c = rgb(c);
                    for (k, name) in ["col_r", "col_g", "col_b"].into_iter().enumerate() {
                        set(&mut want, &mut nw, name, Some(r6(c[k])), false);
                    }
                }
                if !r.first.contains_key("emissive") {
                    set(&mut want, &mut nw, "emissive_k", None, false);
                } else {
                    set(&mut want, &mut nw, "emissive_k", Some(1.0), true);
                }
                if r.sent.contains_key("opacity") {
                    set(&mut want, &mut nw, "opacity", Some(1.0), true);
                }
            }
            if r.sent.contains_key("visible") || kind != "light" {
                set(&mut want, &mut nw, "visible", Some(1.0), true);
            }
            for k in ["scale_x", "scale_y", "scale_z"] {
                if r.sent.contains_key(k) {
                    set(&mut want, &mut nw, k, Some(1.0), true);
                }
            }
            if kind == "light" {
                set(&mut want, &mut nw, "visible", None, false);
            }
            let mut changed: BTreeMap<String, V> = BTreeMap::new();
            for (k, v) in &want[..nw] {
                let Some(v) = v.map(V::N) else { continue };
                if r.sent.get(*k) != Some(&v) {
                    match r.sent.get_mut(*k) {
                        Some(slot) => *slot = v.clone(),
                        None => {
                            r.sent.insert(k.to_string(), v.clone());
                        }
                    }
                    changed.insert(k.to_string(), v);
                }
            }
            if !changed.is_empty() {
                out.push(raea_core_program::eval::Tup { kind: kind.to_string(), id: id.clone(), vals: changed });
            }
        }
        // missing this frame: hidden (a light dark); dropped once it's been gone long enough
        let mut gone = Vec::new();
        for (id, r) in self.rows.iter_mut() {
            if r.seen {
                continue;
            }
            r.missing += 1;
            if r.missing > LIVE_FORGET {
                gone.push((r.kind.clone(), id.clone()));
                continue;
            }
            let (k, v) = match r.kind.as_str() {
                "light" => (if r.first.contains_key("lux") { "lux" } else { "lumens" }, V::N(0.0)),
                _ => ("visible", V::N(0.0)),
            };
            if r.sent.get(k) != Some(&v) {
                r.sent.insert(k.to_string(), v.clone());
                let mut vals = BTreeMap::new();
                vals.insert(k.to_string(), v);
                out.push(raea_core_program::eval::Tup { kind: r.kind.clone(), id: id.clone(), vals });
            }
        }
        for (_, id) in &gone {
            self.rows.remove(id);
            self.ids.retain(|_, v| v != id);
        }
        Some((out, gone))
    }
}
