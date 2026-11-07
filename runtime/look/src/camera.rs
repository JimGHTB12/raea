//! The camera row: where it is and looks (keyed), its lens and motion blur.

use super::*;

/// The camera row's 3D columns: where it is and looks (keyed), its lens, the ambient light, the post-processing.
pub fn camera(all: &[Vec<Item>], p: &mut Program, row: &mut BTreeMap<String, V>) {
    let cams: Vec<Option<Item>> = all.iter().map(|its| its.iter().find(|i| i.kind == "camera").cloned()).collect();
    let id = "cam";
    if cams.iter().any(Option::is_some) {
        let its = filled(&cams);
        let pos: Vec<[f64; 3]> = its.iter().map(|c| vec3(&c.props, "position", [0.0, 0.0, 5.0])).collect();
        let look: Vec<[f64; 3]> = its.iter().map(|c| vec3(&c.props, "lookAt", [0.0, 0.0, 0.0])).collect();
        for (k, name) in ["x", "y", "z"].iter().enumerate() {
            column(p, row, id, name, &pos.iter().map(|v| v[k]).collect::<Vec<_>>());
        }
        for (k, name) in ["look_x", "look_y", "look_z"].iter().enumerate() {
            column(p, row, id, name, &look.iter().map(|v| v[k]).collect::<Vec<_>>());
        }
        column(p, row, id, "fov", &its.iter().map(|c| n(&c.props, "fov", 50.0)).collect::<Vec<_>>());
        // an orthographic camera (<OrthographicCamera height>): the view's height in metres, keyed; 0 is perspective
        if its.iter().any(|c| c.props.contains_key("ortho")) {
            column(p, row, id, "ortho", &its.iter().map(|c| n(&c.props, "ortho", 0.0)).collect::<Vec<_>>());
        }
        // which way is up (`up`; +Y by default): a camera that rolls, as a solved one does (raea-render's up_x/y/z, Matchmove)
        if its.iter().any(|c| c.props.get("up").is_some()) {
            let up: Vec<[f64; 3]> = its.iter().map(|c| vec3(&c.props, "up", [0.0, 1.0, 0.0])).collect();
            for (k, name) in ["up_x", "up_y", "up_z"].iter().enumerate() {
                column(p, row, id, name, &up.iter().map(|v| v[k]).collect::<Vec<_>>());
            }
        }
        // a window on a larger view (`view`: its full size and this frame's offset, in frame pixels): a crop of
        // footage keeps its lens's centre (raea-render's camera view_*, Matchmove's 9:16 from 16:9)
        if its.iter().any(|c| c.props.get("view").is_some_and(Value::is_object)) {
            let view = |c: &Item, k: &str| c.props.get("view").and_then(|v| v.get(k)).and_then(Value::as_f64).unwrap_or(0.0);
            for (k, col) in [("width", "view_w"), ("height", "view_h"), ("x", "view_x"), ("y", "view_y")] {
                column(p, row, id, col, &its.iter().map(|c| view(c, k)).collect::<Vec<_>>());
            }
        }
        if all.iter().flatten().any(|i| i.kind == "dof") {
            column(p, row, id, "focus", &its.iter().map(focus_of).collect::<Vec<_>>());
        }
        let first = &its[0].props;
        for (k, col) in [("near", "near"), ("far", "far")] {
            if let Some(v) = num(first, k) {
                row.insert(col.into(), V::N(v));
            }
        }
        layers_column(row, first);
        // Bevy's own components on the camera (<BevyComponent> inside it)
        components(p, id, &its);
    } else {
        for (name, v) in [("x", 0.0), ("y", 0.0), ("z", 5.0), ("look_x", 0.0), ("look_y", 0.0), ("look_z", 0.0), ("fov", 50.0)] {
            row.insert(name.into(), V::N(v));
        }
    }
    // ambient: every AmbientLight's intensity summed, per frame; its colour the first one's
    let amb: Vec<f64> = all.iter().map(|its| its.iter().filter(|i| i.kind == "ambient").map(|i| light_power("ambient", &i.props)).sum::<f64>()).collect();
    if amb.iter().any(|a| *a > 0.0) {
        column(p, row, id, "ambient", &amb);
        if let Some(c) = all.iter().flatten().find(|i| i.kind == "ambient").and_then(|i| hex(i.props.get("color"))) {
            if c != "#ffffff" {
                row.insert("ambient_color".into(), V::S(c));
            }
        }
    }
    // an <Environment>: each frame's panorama (an index into env_srcs, -1 for none), its light, turn and sky, keyed, so
    // a cut from a hall to a street changes the light on that frame
    let envs: Vec<Option<&Item>> = all.iter().map(|its| its.iter().find(|i| i.kind == "environment")).collect();
    if envs.iter().any(Option::is_some) {
        let mut srcs: Vec<String> = Vec::new();
        let idx: Vec<f64> = envs.iter().map(|e| match e.and_then(|e| e.props.get("src")).and_then(Value::as_str) {
            Some(s) => {
                let s = src_path(s);
                (srcs.iter().position(|x| *x == s).unwrap_or_else(|| { srcs.push(s); srcs.len() - 1 })) as f64
            }
            None => -1.0,
        }).collect();
        row.insert("env_srcs".into(), V::S(serde_json::to_string(&srcs).unwrap_or_default()));
        column(p, row, id, "env_i", &idx);
        let k = |e: &Option<&Item>| e.map(|e| n(&e.props, "intensity", 1.0)).unwrap_or(0.0);
        column(p, row, id, "env_k", &envs.iter().map(|e| k(e) * ENV).collect::<Vec<_>>());
        column(p, row, id, "env_rot", &envs.iter().map(|e| e.map(|e| n(&e.props, "rotation", 0.0)).unwrap_or(0.0)).collect::<Vec<_>>());
        let sky: Vec<f64> = envs.iter().map(|e| match e {
            Some(e) if e.props.get("background") == Some(&Value::Bool(true)) => num(&e.props, "backgroundIntensity").unwrap_or(k(&Some(e))) * ENV,
            _ => 0.0,
        }).collect();
        if sky.iter().any(|s| *s > 0.0) {
            column(p, row, id, "sky_k", &sky);
        }
    }
    // a <Grade>'s colour fringes, keyed: each frame's own (0 where a frame has none), so a glitch's spike draws on its
    // frames and a frame is the same whichever others the render holds (it was the first frame's, for every frame)
    let ab: Vec<f64> = all.iter().map(|its| its.iter().find(|i| i.kind == "grade").and_then(|g| num(&g.props, "aberration")).unwrap_or(0.0)).collect();
    if all.iter().flatten().any(|i| i.kind == "grade" && num(&i.props, "aberration").is_some()) {
        column(p, row, id, "aberration", &ab);
    }
    // post-processing: as the first frame that has each asks
    let first =|kind: &str| all.iter().flatten().find(|i| i.kind == kind).map(|i| i.props.clone());
    // a procedural sky (<Atmosphere>): Earth's air, lighting the scene too when `light`
    if let Some(a) = first("atmosphere") {
        row.insert("atmosphere".into(), V::N(1.0));
        if a.get("light") == Some(&Value::Bool(true)) {
            row.insert("atmo_light".into(), V::N(1.0));
        }
        if let Some(x) = num(&a, "groundAlbedo") {
            row.insert("atmo_albedo".into(), V::N(x));
        }
    }
    if let Some(b) = first("bloom") {
        row.insert("bloom".into(), V::N(r6(n(&b, "intensity", 1.0) * BLOOM)));
        if let Some(t) = num(&b, "threshold") {
            row.insert("bloom_threshold".into(), V::N(t));
        }
        if let Some(s) = shares(&b) {
            row.insert("bloom_passes".into(), V::S(s));
        }
    }
    if let Some(f) = first("fog") {
        row.insert("fog_color".into(), V::S(hex(f.get("color")).unwrap_or("#808080".into())));
        match num(&f, "density") {
            Some(d) => {
                row.insert("fog_density".into(), V::N(d));
            }
            None => {
                row.insert("fog_start".into(), V::N(n(&f, "near", 5.0)));
                row.insert("fog_end".into(), V::N(n(&f, "far", 20.0)));
            }
        }
    }
    if let Some(d) = first("dof") {
        row.insert("dof_aperture".into(), V::N(n(&d, "aperture", 2.8)));
        row.insert("dof_max".into(), V::N(n(&d, "maxBlur", 64.0)));
        if d.get("mode").and_then(Value::as_str) == Some("gaussian") {
            row.insert("dof_mode".into(), V::S("gaussian".into()));
        }
    }
    frame_looks(all, p, row);
    // the composition passes the parts name (engine/compose.rs)
    let passes = pass_names(all);
    if !passes.is_empty() {
        row.insert("passes".into(), V::S(passes.join(",")));
    }
    if let Some(g) = first("grade") {
        if let Some(s) = shares(&g) {
            row.insert("grade_passes".into(), V::S(s));
        }
        for (k, col) in [("temperature", "grade_temperature"), ("tint", "grade_tint"), ("saturation", "grade_saturation")] {
            if let Some(x) = num(&g, k) {
                row.insert(col.into(), V::N(x));
            }
        }
        for (sec, pre) in [("shadows", "grade_s_"), ("midtones", "grade_m_"), ("highlights", "grade_h_")] {
            let Some(o) = g.get(sec).and_then(Value::as_object) else { continue };
            for k in ["saturation", "contrast", "gamma", "gain", "lift"] {
                if let Some(x) = o.get(k).and_then(Value::as_f64) {
                    row.insert(format!("{pre}{k}"), V::N(x));
                }
            }
        }
        match g.get("vignette") {
            Some(Value::Number(x)) => {
                row.insert("vignette".into(), V::N(x.as_f64().unwrap_or(0.0)));
            }
            Some(Value::Object(o)) => {
                row.insert("vignette".into(), V::N(o.get("intensity").and_then(Value::as_f64).unwrap_or(1.0)));
                for (k, col) in [("radius", "vignette_radius"), ("smoothness", "vignette_smooth")] {
                    if let Some(x) = o.get(k).and_then(Value::as_f64) {
                        row.insert(col.into(), V::N(x));
                    }
                }
                if let Some(c) = hex(o.get("color")) {
                    row.insert("vignette_color".into(), V::S(c));
                }
            }
            _ => {}
        }
    }
    // ambient occlusion (no multisampling with it: SMAA unless asked otherwise) and the anti-aliasing
    let ssao = first("ssao");
    if let Some(a) = &ssao {
        row.insert("ssao".into(), V::S(a.get("quality").and_then(Value::as_str).unwrap_or("medium").to_string()));
        if let Some(t) = num(a, "thickness") {
            row.insert("ssao_thickness".into(), V::N(t));
        }
    }
    let aa = first("aa").and_then(|a| a.get("mode").and_then(Value::as_str).map(str::to_string));
    match (aa.as_deref(), ssao.is_some()) {
        (Some("msaa") | None, true) => {
            row.insert("aa".into(), V::S("smaa".into()));
        }
        (Some(m), _) if m != "msaa" => {
            row.insert("aa".into(), V::S(m.to_string()));
        }
        _ => {}
    }
    // shadow quality: the map size and the filter on the camera; the cascades and biases go on each light's row
    if let Some(sh) = first("shadows") {
        if let Some(m) = num(&sh, "mapSize") {
            row.insert("shadow_map".into(), V::N(m));
        }
        if let Some(f) = sh.get("filter").and_then(Value::as_str) {
            row.insert("shadow_filter".into(), V::S(f.to_string()));
        }
    }
    if let Some(v) = first("volfog") {
        row.insert("vfog".into(), V::N(1.0));
        row.insert("vfog_steps".into(), V::N(n(&v, "steps", 64.0)));
        if v.get("dither").and_then(Value::as_bool) == Some(true) {
            row.insert("vfog_dither".into(), V::N(1.0));
        }
        if let Some(r) = num(&v, "resolution") {
            row.insert("vfog_resolution".into(), V::N(r));
        }
        if let Some(m) = v.get("mode").and_then(Value::as_str).filter(|m| matches!(*m, "froxel" | "march")) {
            row.insert("vfog_mode".into(), V::S(m.into()));
        }
        if v.get("hero").and_then(Value::as_bool) == Some(false) {
            row.insert("vfog_hero".into(), V::N(0.0));
        }
        for (prop, col) in [("slices", "vfog_slices"), ("tile", "vfog_tile"), ("samples", "vfog_samples")] {
            if let Some(x) = num(&v, prop) {
                row.insert(col.into(), V::N(x));
            }
        }
        row.insert("vfog_ambient".into(), V::N(n(&v, "ambient", 0.0)));
        if let Some(c) = hex(v.get("ambientColor")) {
            row.insert("vfog_ambient_color".into(), V::S(c));
        }
    }
    if let Some(t) = first("tonemapping") {
        if let Some(m) = t.get("mode").and_then(Value::as_str) {
            row.insert("tonemap".into(), V::S(m.to_lowercase()));
        }
        if let Some(e) = num(&t, "exposure") {
            row.insert("exposure".into(), V::N(r6(EV100 - e)));
        }
    }
    lens(all, p, row);
    motion_blur(all, p, row);
}

/// The camera's lens (Fisheye or LensDistortion; engine/render/src/engine/lens.rs): its kind and edges from the first frame that
/// has one, every value keyed per frame. A frame without it is no lens (strength 0, no k, zoom 1), so a lens inside a
/// Sequence bends only its frames. A barrel's zoom, when not given, is worked out here (enough to keep its corners
/// inside the picture), so it's keyed like the rest.
pub fn lens(all: &[Vec<Item>], p: &mut Program, row: &mut BTreeMap<String, V>) {
    let per: Vec<Option<&Item>> = all.iter().map(|its| its.iter().find(|i| i.kind == "fisheye" || i.kind == "distortion")).collect();
    let Some(first) = per.iter().flatten().next() else { return };
    let id = "cam";
    let fisheye = first.kind == "fisheye";
    let mode = if !fisheye { 3.0 } else if first.props.get("mode").and_then(Value::as_str) == Some("stereographic") { 2.0 } else { 1.0 };
    row.insert("lens_mode".into(), V::N(mode));
    if first.props.get("edges").and_then(Value::as_str) == Some("black") {
        row.insert("lens_black".into(), V::N(1.0));
    }
    let same = |i: &&Item| (i.kind == "fisheye") == fisheye;
    let at = |k: &str, d: f64| -> Vec<f64> { per.iter().map(|i| i.filter(same).map(|i| n(&i.props, k, d)).unwrap_or(0.0)).collect() };
    if fisheye {
        column(p, row, id, "lens_s", &at("strength", 1.0));
        if per.iter().flatten().any(|i| i.props.contains_key("fov")) {
            let cams: Vec<f64> = all.iter().map(|its| its.iter().find(|i| i.kind == "camera").map(|c| n(&c.props, "fov", 50.0)).unwrap_or(50.0)).collect();
            let fovs: Vec<f64> = per.iter().zip(&cams).map(|(i, c)| i.filter(same).and_then(|i| num(&i.props, "fov")).unwrap_or(*c)).collect();
            column(p, row, id, "lens_fov", &fovs);
        }
    } else {
        column(p, row, id, "lens_k1", &at("k1", 0.0));
        column(p, row, id, "lens_k2", &at("k2", 0.0));
    }
    let zoom: Vec<f64> = per.iter().map(|i| match i.filter(same) {
        Some(i) => num(&i.props, "zoom").unwrap_or(if fisheye { 1.0 } else { (1.0 + n(&i.props, "k1", 0.0) + n(&i.props, "k2", 0.0)).max(1.0) }),
        None => 1.0,
    }).collect();
    column(p, row, id, "lens_zoom", &zoom);
}

/// The camera's motion blur (engine/render/src/engine/motion_blur.rs): mb_on keyed per frame (0 where a frame has none), its
/// shutter keyed (a frame without it holds the nearest frame's, which an off frame doesn't draw), its samples the first
/// frame's.
pub fn motion_blur(all: &[Vec<Item>], p: &mut Program, row: &mut BTreeMap<String, V>) {
    let per: Vec<Option<Item>> = all.iter().map(|its| its.iter().find(|i| i.kind == "mblur").cloned()).collect();
    if !per.iter().any(Option::is_some) {
        return;
    }
    let id = "cam";
    column(p, row, id, "mb_on", &per.iter().map(|i| i.is_some() as u8 as f64).collect::<Vec<_>>());
    let its = filled(&per);
    column(p, row, id, "mb_shutter", &its.iter().map(|i| n(&i.props, "shutterAngle", 0.5)).collect::<Vec<_>>());
    if let Some(s) = num(&its[0].props, "samples") {
        row.insert("mb_samples".into(), V::N(s));
    }
}
