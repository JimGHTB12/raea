use super::*;

/// What live play hands a frame's 3D: its scene, and the colour clearing behind it.
#[derive(Clone, Copy)]
pub struct Shot<'a> {
    pub scene: &'a [Node3],
    pub clear: Option<&'a str>,
}

/// A live game's 3D (live.rs): its rows made frame by frame from that frame's own scene, as play computed it, instead
/// of from a scripted run stepped ahead (scene_program). Parts are matched across frames by the identity `tracks`
/// gives them (a name, else kind and shape counted in tree order), so a part keeps its row as a render's does. The
/// columns a render takes from a part's first frame (shape, surface, light colour, a model's file) are taken from the
/// frame it first shows here; the columns a render keys come from each frame. A part missing from a frame is hidden (a
/// light goes dark), as a render's `visible` column hides it. Each frame only what changed goes to the engine.
#[derive(Default, Clone)]
pub struct LiveScene {
    /// identity (as `tracks` keys it) -> row id
    pub(crate) ids: std::collections::HashMap<String, String>,
    /// per row id: its kind, its first values, what was last sent, whether its colour has changed (a render keys
    /// col_r/g/b on every frame once it does), and the frames it has been missing
    pub(crate) rows: std::collections::HashMap<String, LiveRow>,
    /// rows made per group (lights, meshes, models, panels, decals): the next one's number
    pub(crate) made: [usize; 5],
    pub(crate) cam: BTreeMap<String, V>,
    /// frame 0 was stepped when play opened (live.rs), and nothing since
    pub warm: bool,
}

#[derive(Clone)]
pub(crate) struct LiveRow {
    pub(crate) kind: String,
    pub(crate) first: BTreeMap<String, V>,
    pub(crate) sent: BTreeMap<String, V>,
    pub(crate) recoloured: bool,
    pub(crate) missing: u32,
    /// seen this frame (frame_fast's mark)
    pub(crate) seen: bool,
}

/// A row missing this many frames is dropped from the engine (its entity, mesh and surface freed) and forgotten: a game
/// that spawns entities with fresh names (`rock-${id}`) would otherwise keep every one it ever made, hidden. Coming back
/// later, it's a new row with the same values, so nothing drawn changes.
pub(crate) const LIVE_FORGET: u32 = 240;

/// The kinds a live frame keys column by column (live_dynamic); every other kind is diffed whole.
pub(crate) const LIVE_GROUPS: [&str; 5] = ["light", "mesh", "model", "panel", "decal"];

/// A <Model animations>'s keyed columns (its clips' times and weights) and a <Model morphs>'s (its targets' weights).
pub(crate) const MIX_COLS: [(&str, &str); 4] = [("mix0_t", "mix0_w"), ("mix1_t", "mix1_w"), ("mix2_t", "mix2_w"), ("mix3_t", "mix3_w")];
pub(crate) const GRAPH_COLS: [[&str; 2]; 16] = [["g0_w", "g0_t"], ["g1_w", "g1_t"], ["g2_w", "g2_t"], ["g3_w", "g3_t"], ["g4_w", "g4_t"], ["g5_w", "g5_t"], ["g6_w", "g6_t"], ["g7_w", "g7_t"], ["g8_w", "g8_t"], ["g9_w", "g9_t"], ["g10_w", "g10_t"], ["g11_w", "g11_t"], ["g12_w", "g12_t"], ["g13_w", "g13_t"], ["g14_w", "g14_t"], ["g15_w", "g15_t"]];
/// Up to 64 morph weights a model (Bevy takes 256; a MakeHuman face has 34 expression units).
pub(crate) const MORPH_COLS: [&str; 64] = ["morph0", "morph1", "morph2", "morph3", "morph4", "morph5", "morph6", "morph7", "morph8", "morph9", "morph10", "morph11", "morph12", "morph13", "morph14", "morph15", "morph16", "morph17", "morph18", "morph19", "morph20", "morph21", "morph22", "morph23", "morph24", "morph25", "morph26", "morph27", "morph28", "morph29", "morph30", "morph31", "morph32", "morph33", "morph34", "morph35", "morph36", "morph37", "morph38", "morph39", "morph40", "morph41", "morph42", "morph43", "morph44", "morph45", "morph46", "morph47", "morph48", "morph49", "morph50", "morph51", "morph52", "morph53", "morph54", "morph55", "morph56", "morph57", "morph58", "morph59", "morph60", "morph61", "morph62", "morph63"];

/// The columns a frame may change, per kind (what `rows_into` and `camera` key); everything else is the first frame's.
pub(crate) fn live_dynamic(kind: &str) -> &'static [&'static str] {
    match kind {
        "light" => &["lux", "lumens", "x", "y", "z", "dir_x", "dir_y", "dir_z"],
        "mesh" => &["x", "y", "z", "rot_x", "rot_y", "rot_z", "scale_x", "scale_y", "scale_z", "visible", "col_r", "col_g", "col_b", "rough", "metal", "emissive_k", "opacity", "sh_r", "sh_g", "sh_b", "rim_k", "spec_k", "outline_w", "pass"],
        "model" => &["x", "y", "z", "rot_x", "rot_y", "rot_z", "scale_x", "scale_y", "scale_z", "visible", "anim_t", "mix0_t", "mix0_w", "mix1_t", "mix1_w", "mix2_t", "mix2_w", "mix3_t", "mix3_w", "morph0", "morph1", "morph2", "morph3", "morph4", "morph5", "morph6", "morph7", "morph8", "morph9", "morph10", "morph11", "morph12", "morph13", "morph14", "morph15", "morph16", "morph17", "morph18", "morph19", "morph20", "morph21", "morph22", "morph23", "morph24", "morph25", "morph26", "morph27", "morph28", "morph29", "morph30", "morph31", "morph32", "morph33", "morph34", "morph35", "morph36", "morph37", "morph38", "morph39", "morph40", "morph41", "morph42", "morph43", "morph44", "morph45", "morph46", "morph47", "morph48", "morph49", "morph50", "morph51", "morph52", "morph53", "morph54", "morph55", "morph56", "morph57", "morph58", "morph59", "morph60", "morph61", "morph62", "morph63", "g0_w", "g0_t", "g1_w", "g1_t", "g2_w", "g2_t", "g3_w", "g3_t", "g4_w", "g4_t", "g5_w", "g5_t", "g6_w", "g6_t", "g7_w", "g7_t", "g8_w", "g8_t", "g9_w", "g9_t", "g10_w", "g10_t", "g11_w", "g11_t", "g12_w", "g12_t", "g13_w", "g13_t", "g14_w", "g14_t", "g15_w", "g15_t", "look", "glitch_explode", "glitch_scramble", "glitch_jitter", "glitch_snap", "glitch_smear_x", "glitch_smear_y", "glitch_smear_z", "glitch_seed", "cloth_frame", "screen_power", "pass"],
        "panel" => &["x", "y", "z", "rot_x", "rot_y", "rot_z", "scale_x", "scale_y", "scale_z", "visible", "screen_power", "screen_bright", "pass"],
        "decal" => &["x", "y", "z", "rot_x", "rot_y", "rot_z", "scale_x", "scale_y", "scale_z", "visible", "col_r", "col_g", "col_b", "opacity", "rough", "metal"],
        _ => &[],
    }
}

impl LiveScene {
    /// Start again (the engine was dropped: a new size).
    pub fn reset(&mut self) {
        *self = LiveScene::default();
    }

    /// The frame's rows as a render's program would hold them at this frame: the camera row and each part's full row,
    /// by identity, with the row ids this scene keeps.
    pub(crate) fn current(&mut self, f: Shot) -> (BTreeMap<String, V>, Vec<(String, String, BTreeMap<String, V>)>) {
        let all = vec![items_of(f.scene)];
        let mut p = Program::default();
        let mut cam = BTreeMap::new();
        cam.insert("id".into(), V::S("cam".into()));
        cam.insert("bg".into(), V::S(f.clear.and_then(hex_of).unwrap_or("#000000".into())));
        camera(&all, &mut p, &mut cam);
        if !all[0].iter().any(|i| i.kind == "camera") {
            // no camera this frame: a render holds the one before (filled)
            for k in ["x", "y", "z", "look_x", "look_y", "look_z", "fov", "focus"] {
                match self.cam.get(k) {
                    Some(v) => {
                        cam.insert(k.into(), v.clone());
                    }
                    None => {}
                }
            }
        }
        let tr = Tracks::of(&all);
        rows_into_of(&all, &tr, &mut p);
        // each local row (this frame's tracks, numbered as rows_into numbers them) by its identity
        let mut local: std::collections::HashMap<String, &String> = std::collections::HashMap::new();
        for (li, (key, track)) in tr.lights.iter().enumerate() {
            let kind = track[0].as_ref().map(|i| i.kind).unwrap_or("directional");
            local.insert(if kind == "directional" { format!("sun{li}") } else { format!("{kind}{li}") }, key);
        }
        for (prefix, group) in [("mesh", &tr.meshes), ("model", &tr.models), ("plane", &tr.panels), ("decal", &tr.decals)] {
            for (k, (key, _)) in group.iter().enumerate() {
                local.insert(format!("{prefix}{k}"), key);
            }
        }
        let mut out = Vec::new();
        // the other kinds (bare entities, Bevy components and resources, fires, fog volumes, splats, plates...): by
        // the ids a render gives them, a component's target as the live rows name it (G15)
        let mut others: Vec<(String, BTreeMap<String, V>)> = Vec::new();
        let mut live_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (kind, rows) in p.spawns {
            let group = match kind.as_str() {
                "light" => 0,
                "mesh" => 1,
                "model" => 2,
                "panel" => 3,
                "decal" => 4,
                "camera" => continue,
                _ => {
                    others.extend(rows.into_iter().map(|r| (kind.clone(), r)));
                    continue;
                }
            };
            for mut row in rows {
                let Some(V::S(lid)) = row.get("id").cloned() else { continue };
                let Some(key) = local.get(&lid) else { continue };
                let id = match self.ids.get(key.as_str()) {
                    Some(id) => id.clone(),
                    None => {
                        let prefix: String = lid.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();
                        let id = format!("{prefix}{}", self.made[group]);
                        self.made[group] += 1;
                        self.ids.insert((*key).clone(), id.clone());
                        id
                    }
                };
                live_id.insert(lid.clone(), id.clone());
                row.insert("id".into(), V::S(id.clone()));
                out.push((kind.clone(), id, row));
            }
        }
        // a model's references to other models (attach_to, bone_rows: a <Character>'s skin follows its joints) name
        // this frame's local ids; they become this scene's ids, or a skin mounted after the first (a keyed <Character>,
        // person after person) follows rows that are someone else's, or none
        for (_, _, row) in out.iter_mut() {
            for k in ["attach_to", "bone_rows"] {
                if let Some(V::S(refs)) = row.get(k) {
                    let ids: Vec<&str> = refs.split(',').map(|r| live_id.get(r).map_or(r, String::as_str)).collect();
                    row.insert(k.into(), V::S(ids.join(",")));
                }
            }
        }
        for (kind, mut row) in others {
            let Some(V::S(mut id)) = row.get("id").cloned() else { continue };
            if let Some(V::S(t)) = row.get("target").cloned() {
                let Some(to) = (if t.starts_with("ent") || t.starts_with("text") || t == "cam" { Some(t.clone()) } else { live_id.get(&t).cloned() }) else { continue };
                id = id.replacen(&t, &to, 1);
                row.insert("id".into(), V::S(id.clone()));
                row.insert("target".into(), V::S(to));
            }
            out.push((kind, id, row));
        }
        (cam, out)
    }

    /// The program the live engine opens with: this frame's camera and parts (a render's rows at this frame).
    pub fn open(&mut self, f: Shot, w: u32, h: u32, fps: f64) -> Program {
        self.reset();
        let (cam, rows) = self.current(f);
        let mut p = Program { fps, width: w, height: h, frames: 1, ..Default::default() };
        self.cam = cam.clone();
        p.spawns.push(("camera".into(), vec![cam]));
        for (kind, id, row) in rows {
            let first = row.clone();
            self.rows.insert(id, LiveRow { kind: kind.clone(), first, sent: row.clone(), recoloured: false, missing: 0, seen: false });
            p.spawns.push((kind, vec![row]));
        }
        p
    }

    /// What this frame changes: the rows to set (a new part's whole row; for one seen before, only the columns whose
    /// values changed) and the rows to drop (missing LIVE_FORGET frames).
    pub fn frame(&mut self, f: Shot) -> (Vec<raea_core_program::eval::Tup>, Vec<(String, String)>) {
        // RAEA_LIVE3D_CHECK=1: each frame also diffed from a render's full rows on a copy, and the two must agree
        let check = std::env::var_os("RAEA_LIVE3D_CHECK").is_some();
        let before = check.then(|| self.clone());
        let fast = std::env::var("RAEA_LIVE3D_FAST").as_deref() != Ok("0");
        let (out, used_fast) = match fast.then(|| self.frame_fast(f)).flatten() {
            Some(out) => (out, true),
            None => {
                let (cam, rows) = self.current(f);
                (self.diff(cam, rows), false)
            }
        };
        if let (Some(mut slow), true) = (before, used_fast) {
            let (cam, rows) = slow.current(f);
            let want = slow.diff(cam, rows);
            assert!(want == out, "RAEA_LIVE3D_CHECK: fast rows {:?} != full rows {:?}", out, want);
            assert!(slow.cam == self.cam, "RAEA_LIVE3D_CHECK: camera {:?} != {:?}", self.cam, slow.cam);
            assert!(slow.rows.len() == self.rows.len(), "RAEA_LIVE3D_CHECK: {} rows kept, {} on the full path", self.rows.len(), slow.rows.len());
            for (id, r) in &slow.rows {
                assert!(self.rows.get(id).is_some_and(|m| m.sent == r.sent && m.recoloured == r.recoloured && m.missing == r.missing), "RAEA_LIVE3D_CHECK: row {id}");
            }
        }
        out
    }

    /// What this frame changes, from its camera row and parts' rows (`current`'s)
    pub(crate) fn diff(&mut self, cam: BTreeMap<String, V>, rows: Vec<(String, String, BTreeMap<String, V>)>) -> (Vec<raea_core_program::eval::Tup>, Vec<(String, String)>) {
        let mut out: Vec<raea_core_program::eval::Tup> = Vec::new();
        // the camera: where it is and looks, its lens and the ambient light (0 once there's none)
        let mut vals = BTreeMap::new();
        // and the lens's and motion blur's, whatever of theirs the frame keys
        let keyed: Vec<String> = cam.keys().filter(|k| k.starts_with("lens_") || k.starts_with("mb_") || *k == "ortho").cloned().collect();
        for k in ["x", "y", "z", "look_x", "look_y", "look_z", "fov", "ambient", "focus", "paint_bands", "paint_grain", "paint_rag", "paint_seed"].into_iter().chain(keyed.iter().map(String::as_str)) {
            let v = cam.get(k).cloned().or_else(|| (k == "ambient" && self.cam.contains_key("ambient")).then_some(V::N(0.0)));
            if let Some(v) = v {
                if self.cam.get(k) != Some(&v) {
                    self.cam.insert(k.into(), v.clone());
                    vals.insert(k.to_string(), v);
                }
            }
        }
        if !vals.is_empty() {
            out.push(raea_core_program::eval::Tup { kind: "camera".into(), id: "cam".into(), vals });
        }
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (kind, id, row) in rows {
            seen.insert(id.clone());
            let Some(r) = self.rows.get_mut(&id) else {
                // first seen: its whole row, spawned with these values
                self.rows.insert(id.clone(), LiveRow { kind: kind.clone(), first: row.clone(), sent: row.clone(), recoloured: false, missing: 0, seen: false });
                out.push(raea_core_program::eval::Tup { kind, id, vals: row.into_iter().filter(|(k, _)| k != "id").collect() });
                continue;
            };
            r.missing = 0;
            let mut want: BTreeMap<String, V> = BTreeMap::new();
            if !LIVE_GROUPS.contains(&kind.as_str()) {
                // a kind the groups don't key column by column: every column it has, sent when it changes
                for (k, v) in row.iter().filter(|(k, _)| k.as_str() != "id") {
                    want.insert(k.clone(), v.clone());
                }
            }
            for k in live_dynamic(&kind) {
                if let Some(v) = row.get(*k) {
                    want.insert(k.to_string(), v.clone());
                }
            }
            if kind == "mesh" {
                // a colour other than its first: keyed from now on, as a render keys it once a part's colour changes
                if row.get("color") != r.first.get("color") {
                    r.recoloured = true;
                }
                if r.recoloured {
                    if let Some(V::S(c)) = row.get("color") {
                        let c = rgb(c);
                        for (k, name) in ["col_r", "col_g", "col_b"].iter().enumerate() {
                            want.insert(name.to_string(), V::N(r6(c[k])));
                        }
                    }
                }
                // the emissive strength only where the first frame glows (a render's column), opacity once it's been
                // see-through, scale back to 1: what a render's keyed column says on a frame this one leaves it out
                if !r.first.contains_key("emissive") {
                    want.remove("emissive_k");
                } else {
                    want.entry("emissive_k".into()).or_insert(V::N(1.0));
                }
                if r.sent.contains_key("opacity") {
                    want.entry("opacity".into()).or_insert(V::N(1.0));
                }
            }
            if (r.sent.contains_key("visible") || kind != "light") && !matches!(kind.as_str(), "component" | "resource") {
                want.entry("visible".into()).or_insert(V::N(1.0));
            }
            for k in ["scale_x", "scale_y", "scale_z"] {
                if r.sent.contains_key(k) {
                    want.entry(k.into()).or_insert(V::N(1.0));
                }
            }
            if kind == "light" {
                want.retain(|k, _| k != "visible");
            }
            let changed: BTreeMap<String, V> = want.into_iter().filter(|(k, v)| r.sent.get(k) != Some(v)).collect();
            if !changed.is_empty() {
                for (k, v) in &changed {
                    r.sent.insert(k.clone(), v.clone());
                }
                out.push(raea_core_program::eval::Tup { kind, id, vals: changed });
            }
        }
        // missing this frame: hidden (a light dark); dropped once it's been gone long enough
        let mut gone = Vec::new();
        for (id, r) in self.rows.iter_mut() {
            if seen.contains(id) {
                continue;
            }
            r.missing += 1;
            if r.missing > LIVE_FORGET {
                gone.push((r.kind.clone(), id.clone()));
                continue;
            }
            let off: Vec<(&str, V)> = match r.kind.as_str() {
                "light" => vec![(if r.first.contains_key("lux") { "lux" } else { "lumens" }, V::N(0.0))],
                // a component or resource row stays as last set (its part's own visibility hides it)
                "component" | "resource" => vec![],
                _ => vec![("visible", V::N(0.0))],
            };
            let mut vals = BTreeMap::new();
            for (k, v) in off {
                if r.sent.get(k) != Some(&v) {
                    r.sent.insert(k.to_string(), v.clone());
                    vals.insert(k.to_string(), v);
                }
            }
            if !vals.is_empty() {
                out.push(raea_core_program::eval::Tup { kind: r.kind.clone(), id: id.clone(), vals });
            }
        }
        for (_, id) in &gone {
            self.rows.remove(id);
            self.ids.retain(|_, v| v != id);
        }
        (out, gone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn model(props: Value) -> Node3 {
        Node3 { kind: "Model".into(), props: std::sync::Arc::new(props.as_object().unwrap().clone()), shape: None, kids: vec![] }
    }

    fn person(who: &str) -> Vec<Node3> {
        vec![model(json!({"name": format!("{who}-head"), "src": format!("/media/{who}/joint.glb")})), model(json!({"name": format!("{who}-body"), "src": format!("/media/{who}/body.glb"), "bones": {"head": format!("{who}-head")}}))]
    }

    /// A keyed <Character> swapping people: the second person's skin follows its own joints by this scene's ids, not
    /// the first frame's local ids (which are the first person's rows: "joint head follows model0", 10-27).
    #[test]
    fn a_later_skin_follows_its_own_joints() {
        let mut ls = LiveScene::default();
        let a = person("a");
        ls.open(Shot { scene: &a, clear: None }, 64, 64, 30.0);
        let b = person("b");
        let (set, _) = ls.frame(Shot { scene: &b, clear: None });
        let id_of = |name: &str| set.iter().find(|t| t.vals.get("name") == Some(&V::S(name.into()))).map(|t| t.id.clone()).unwrap();
        let body = set.iter().find(|t| t.vals.get("name") == Some(&V::S("b-body".into()))).unwrap();
        assert_eq!(body.vals.get("bone_rows"), Some(&V::S(id_of("b-head"))));
    }
}
