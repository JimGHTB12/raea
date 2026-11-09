//! A UiMaterial's own bindings (docs/control.feature FC2a, Bevy's ui/ui_material): its fragment shader is compiled as
//! written but for the numbers of its `@group(1)` bindings, which go to raea's UiMaterial's fixed slots in the order
//! the source declares them: uniforms at 0 to 7 (any type up to 256 bytes), textures at 8, 10, 12, 14 and samplers at
//! 9, 11, 13, 15 (the n-th sampler is the n-th texture's). A Rust UiMaterial's bind group is its own type's
//! (AsBindGroup's layout is the type's, not an instance's), so raea's one type takes every author's layout this way.

/// Uniform buffers a UiMaterial may bind, and textures (each with its sampler).
pub const UNIFORMS: usize = 8;
pub const TEXTURES: usize = 4;
/// A Material's textures of the other dimensions (`dims`), one of each, at their own slots: its first two samplers past
/// the 2D textures' at 17 and 19 (Bevy's view binds up to 8 of the fragment stage's 16).
pub const DIMS: [(&str, &str, usize); 5] = [("texture_1d<", "1d", 16), ("texture_2d_array<", "2d_array", 18), ("texture_cube<", "cube", 20), ("texture_cube_array<", "cube_array", 21), ("texture_3d<", "3d", 22)];
pub const SAMPLERS_WITH_DIMS: usize = 6;
/// A Material's read-only storage buffers (`dims`), at 23 and 24.
pub const STORAGE: [usize; 2] = [23, 24];
/// Where a Material's slots (`dims`) start: past a sprite's own bindings (0 to 2, 10 when bindless), whose group a
/// SpriteMaterial's share (shader/sprite_material). A UiMaterial's start at 0.
pub const OWN_BASE: usize = 30;

/// A UiMaterial source with its bindings renumbered: its uniforms (name, WGSL type) and its textures' names, each in
/// its slot's order, and how many samplers it binds.
#[derive(Debug, PartialEq)]
pub struct Rebound {
    pub src: String,
    pub uniforms: Vec<(String, String)>,
    pub textures: Vec<String>,
    /// its textures of other dimensions (`dims`): each name and its dimension, "1d", "2d_array", "cube", ...
    pub others: Vec<(String, &'static str)>,
    /// its read-only storage buffers (`dims`): each name and WGSL type, in slot order
    pub storages: Vec<(String, String)>,
    pub samplers: usize,
}

/// `src`'s `@group(1)` declarations renumbered into raea's slots (see the module's doc), a UiMaterial's.
pub fn rebind(src: &str) -> Result<Rebound, String> {
    rebind_in(src, &["1"], "a UiMaterial", false)
}

/// `src`'s declarations in any of `groups` (as the source writes the group: `1`, `constants::MATERIAL_BIND_GROUP`)
/// renumbered into raea's slots; bindings of other groups left as written. Refused: a binding that isn't a uniform, a
/// texture_2d or a sampler (with `dims`, also a texture of another dimension, DIMS), and more of each than raea has
/// slots for (`what` names the material in errors).
pub fn rebind_in(src: &str, groups: &[&str], what: &str, dims: bool) -> Result<Rebound, String> {
    rebind_with(src, groups, what, dims, &|slot| Ok(slot + if dims { OWN_BASE } else { 0 }))
}

/// As rebind_in, each slot then placed by `place` (an extension's, ext_bind.rs), which may refuse it.
pub fn rebind_with(src: &str, groups: &[&str], what: &str, dims: bool, place: &dyn Fn(usize) -> Result<usize, String>) -> Result<Rebound, String> {
    let mut others: Vec<(String, &'static str)> = Vec::new();
    let mut storages: Vec<(String, String)> = Vec::new();
    let max_samplers = if dims { SAMPLERS_WITH_DIMS } else { TEXTURES };
    let mut out = String::with_capacity(src.len());
    let (mut uniforms, mut textures, mut samplers) = (Vec::new(), Vec::new(), 0usize);
    let mut rest = src;
    let dead = bindless_spans(src);
    while let Some(i) = rest.find("@binding(") {
        let line_start = rest[..i].rfind('\n').map_or(0, |n| n + 1);
        // a binding named in a comment is left alone, and one under @if(BINDLESS) (bindless_spans)
        let at = src.len() - rest.len() + i;
        if rest[line_start..i].contains("//") || dead.iter().any(|s| s.contains(&at)) {
            out += &rest[..i + 1];
            rest = &rest[i + 1..];
            continue;
        }
        let stmt_start = rest[..i].rfind([';', '}']).map_or(0, |n| n + 1);
        let Some(end) = rest[i..].find(';').map(|n| i + n) else { return Err("a @binding declaration has no ';'".into()) };
        let stmt = &rest[stmt_start..end];
        let close = i + rest[i..].find(')').ok_or("@binding( has no ')'")?;
        if !group(stmt).is_some_and(|g| groups.contains(&g)) {
            out += &rest[..close + 1];
            rest = &rest[close + 1..];
            continue;
        }
        let decl = stmt.split_once("var").map(|(_, d)| d).ok_or_else(|| format!("{what}'s binding is a var, got {:?}", stmt.trim()))?;
        let d = decl.trim_start();
        let space: String = d.strip_prefix('<').and_then(|r| r.split_once('>')).map(|(s, _)| s.chars().filter(|c| !c.is_whitespace()).collect()).unwrap_or_default();
        let decl = if space.is_empty() { decl } else { &d[d.find('>').map_or(0, |i| i + 1)..] };
        let uniform = space == "uniform";
        if !space.is_empty() && !uniform && !(dims && (space == "storage" || space == "storage,read")) {
            return Err(format!("{what}'s binding is a var<uniform>, a var<storage, read>, a texture or a sampler, got var<{space}> (the rest are FC2a's to come)"));
        }
        let (name, ty) = decl.split_once(':').map(|(n, t)| (n.trim().to_string(), t.trim().to_string())).ok_or_else(|| format!("{what}'s binding is `var NAME: TYPE`, got {:?}", stmt.trim()))?;
        let slot = if !space.is_empty() && !uniform {
            storages.push((name.clone(), ty));
            *STORAGE.get(storages.len() - 1).ok_or_else(|| format!("{what} binds at most {} storage buffers, {name} is one more", STORAGE.len()))?
        } else if uniform {
            uniforms.push((name.clone(), ty));
            if uniforms.len() > UNIFORMS {
                return Err(format!("{what} binds at most {UNIFORMS} uniforms, {name} is one more"));
            }
            uniforms.len() - 1
        } else if ty.starts_with("texture_2d<") {
            textures.push(name.clone());
            if textures.len() > TEXTURES {
                return Err(format!("{what} binds at most {TEXTURES} textures, {name} is one more"));
            }
            UNIFORMS + 2 * (textures.len() - 1)
        } else if ty == "sampler" {
            samplers += 1;
            if samplers > max_samplers {
                return Err(format!("{what} binds at most {max_samplers} samplers, {name} is one more"));
            }
            UNIFORMS + 2 * (samplers - 1) + 1
        } else if let Some(&(_, d, slot)) = DIMS.iter().find(|(p, ..)| dims && ty.starts_with(p)) {
            if others.iter().any(|(_, o)| *o == d) {
                return Err(format!("{what} binds one texture_{d}, {name} is a second"));
            }
            others.push((name.clone(), d));
            slot
        } else {
            return Err(format!("{what}'s binding {name} is a var<uniform>, a texture_2d<f32> or a sampler, got {ty} (the rest are FC2a's to come)"));
        };
        out += &rest[..i];
        out += &format!("@binding({})", place(slot).map_err(|e| format!("{what}'s binding {name}: {e}"))?);
        rest = &rest[close + 1..];
    }
    out += rest;
    if samplers > textures.len() + others.len() {
        return Err(format!("{what}'s samplers are its textures', one each: {samplers} samplers for {} textures", textures.len() + others.len()));
    }
    Ok(Rebound { src: out, uniforms, textures, others, storages, samplers })
}

/// The spans of `src` under `@if(BINDLESS)`: the `{ ... }` block it opens, or the one statement it marks. raea's
/// materials aren't bindless (Bevy sets BINDLESS only for one that is), so these compile out, their bindings as written.
pub fn bindless_spans(src: &str) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    for (i, m) in src.match_indices("@if(BINDLESS)") {
        let after = i + m.len();
        let at = after + (src[after..].len() - src[after..].trim_start().len());
        let end = if src[at..].starts_with('{') {
            let mut depth = 0i32;
            src[at..].char_indices().find(|(_, c)| {
                depth += match c { '{' => 1, '}' => -1, _ => 0 };
                depth == 0
            }).map(|(k, _)| at + k)
        } else {
            src[at..].find(';').map(|k| at + k)
        };
        out.extend(end.map(|e| i..e));
    }
    out
}

/// The group a declaration's `@group(N)` names.
fn group(stmt: &str) -> Option<&str> {
    let g = stmt.find("@group(")? + "@group(".len();
    let close = stmt[g..].find(')')?;
    Some(stmt[g..g + close].trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    const UI: &str = "import bevy_ui_render::ui_vertex_output::UiVertexOutput;\n\n@group(1) @binding(0) var<uniform> color: vec4<f32>;\n@group(1) @binding(1) var<uniform> slider: vec4<f32>;\n@group(1) @binding(2) var material_color_texture: texture_2d<f32>;\n@group(1) @binding(3) var material_color_sampler: sampler;\n@group(1) @binding(4) var<uniform> border_color: vec4<f32>;\n\n@fragment\nfn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> { return color; }\n";

    #[test]
    fn bevy_ui_materials_bindings_go_to_raeas_slots_and_the_rest_stays_as_written() {
        let r = rebind(UI).unwrap();
        assert_eq!(r.uniforms, vec![("color".into(), "vec4<f32>".into()), ("slider".into(), "vec4<f32>".into()), ("border_color".into(), "vec4<f32>".into())]);
        assert_eq!(r.textures, vec!["material_color_texture".to_string()]);
        assert_eq!(r.samplers, 1);
        assert!(r.src.contains("@group(1) @binding(0) var<uniform> color"));
        assert!(r.src.contains("@group(1) @binding(1) var<uniform> slider"));
        assert!(r.src.contains("@group(1) @binding(8) var material_color_texture"));
        assert!(r.src.contains("@group(1) @binding(9) var material_color_sampler"));
        assert!(r.src.contains("@group(1) @binding(2) var<uniform> border_color"));
        assert!(r.src.ends_with("{ return color; }\n"));
        assert_eq!(r.src.len(), UI.len());
    }

    #[test]
    fn other_groups_comments_and_unsupported_bindings() {
        let other = "@group(0) @binding(7) var<uniform> view: View;\n// @binding(3) in a comment\n@binding(5) @group(1) var t: texture_2d<f32>;\n";
        let r = rebind(other).unwrap();
        assert!(r.src.contains("@group(0) @binding(7)"));
        assert!(r.src.contains("// @binding(3) in a comment"));
        assert!(r.src.contains("@binding(8) @group(1) var t"));
        assert!(rebind("@group(1) @binding(0) var<storage, read> s: array<f32>;").unwrap_err().contains("FC2a"));
        let nine = (0..9).map(|i| format!("@group(1) @binding({i}) var<uniform> u{i}: f32;\n")).collect::<String>();
        assert!(rebind(&nine).unwrap_err().contains("at most 8 uniforms"));
        assert!(rebind("@group(1) @binding(0) var s: sampler;\n@group(1) @binding(1) var s2: sampler;\n@group(1) @binding(2) var t: texture_2d<f32>;").unwrap_err().contains("one each"));
    }

    #[test]
    fn bindings_under_if_bindless_are_left_as_written() {
        // Bevy's bindless_material.wesl: its bindless set compiles out (raea's Material isn't bindless), its @else set is bound
        let src = "struct Color { base_color: vec4<f32> }
@if(BINDLESS) {
@group(3) @binding(0) var<storage> materials: array<u32>;
@group(3) @binding(10) var<storage> material_color: binding_array<Color>;
}
@else {
@group(3) @binding(0) var<uniform> material_color: Color;
@group(3) @binding(1) var t: texture_2d<f32>;
@group(3) @binding(2) var s: sampler;
}
fn f() { @if(BINDLESS) let x = 1; }
";
        let r = rebind_in(src, &["3"], "a Material", true).unwrap();
        assert_eq!((r.uniforms, r.textures, r.storages.len()), (vec![("material_color".to_string(), "Color".to_string())], vec!["t".to_string()], 0));
        assert!(r.src.contains("@binding(10) var<storage> material_color") && r.src.contains("@binding(30) var<uniform> material_color"));
    }

    #[test]
    fn a_materials_other_dimensions_go_to_their_slots_and_its_samplers_in_order() {
        // Bevy's fallback_image_test.wesl: a texture of each dimension, each with its sampler
        let src = ["1d", "2d", "2d_array", "cube", "cube_array", "3d"].iter().enumerate().map(|(i, d)| format!("@group(3) @binding({}) var t_{d}: texture_{d}<f32>;\n@group(3) @binding({}) var s_{d}: sampler;\n", 2 * i, 2 * i + 1)).collect::<String>();
        let r = rebind_in(&src, &["3"], "a Material", true).unwrap();
        assert_eq!(r.textures, vec!["t_2d".to_string()]);
        assert_eq!(r.others.iter().map(|o| o.1).collect::<Vec<_>>(), vec!["1d", "2d_array", "cube", "cube_array", "3d"]);
        for want in ["@binding(46) var t_1d", "@binding(39) var s_1d", "@binding(38) var t_2d", "@binding(41) var s_2d", "@binding(48) var t_2d_array", "@binding(50) var t_cube:", "@binding(51) var t_cube_array", "@binding(52) var t_3d", "@binding(49) var s_3d"] {
            assert!(r.src.contains(want), "{want} in {}", r.src);
        }
        // a UiMaterial takes none of them, and a Material one of each
        assert!(rebind_in(&src, &["3"], "a UiMaterial", false).unwrap_err().contains("texture_1d"));
        assert!(rebind_in("@group(3) @binding(0) var a: texture_3d<f32>;\n@group(3) @binding(1) var b: texture_3d<f32>;", &["3"], "a Material", true).unwrap_err().contains("a second"));
    }
}
