//! A UiMaterial's own bindings (docs/control.feature FC2a, Bevy's ui/ui_material): its fragment shader is compiled as
//! written but for the numbers of its `@group(1)` bindings, which go to raea's UiMaterial's fixed slots in the order
//! the source declares them: uniforms at 0 to 7 (any type up to 256 bytes), textures at 8, 10, 12, 14 and samplers at
//! 9, 11, 13, 15 (the n-th sampler is the n-th texture's). A Rust UiMaterial's bind group is its own type's
//! (AsBindGroup's layout is the type's, not an instance's), so raea's one type takes every author's layout this way.

/// Uniform buffers a UiMaterial may bind, and textures (each with its sampler).
pub const UNIFORMS: usize = 8;
pub const TEXTURES: usize = 4;

/// A UiMaterial source with its bindings renumbered: its uniforms (name, WGSL type) and its textures' names, each in
/// its slot's order, and how many samplers it binds.
#[derive(Debug, PartialEq)]
pub struct Rebound {
    pub src: String,
    pub uniforms: Vec<(String, String)>,
    pub textures: Vec<String>,
    pub samplers: usize,
}

/// `src`'s `@group(1)` declarations renumbered into raea's slots (see the module's doc), a UiMaterial's.
pub fn rebind(src: &str) -> Result<Rebound, String> {
    rebind_in(src, &["1"], "a UiMaterial")
}

/// `src`'s declarations in any of `groups` (as the source writes the group: `1`, `constants::MATERIAL_BIND_GROUP`)
/// renumbered into raea's slots; bindings of other groups left as written. Refused: a binding that isn't a uniform, a
/// texture_2d or a sampler, and more of each than raea has slots for (`what` names the material in errors).
pub fn rebind_in(src: &str, groups: &[&str], what: &str) -> Result<Rebound, String> {
    let mut out = String::with_capacity(src.len());
    let (mut uniforms, mut textures, mut samplers) = (Vec::new(), Vec::new(), 0usize);
    let mut rest = src;
    while let Some(i) = rest.find("@binding(") {
        let line_start = rest[..i].rfind('\n').map_or(0, |n| n + 1);
        // a binding named in a comment is left alone
        if rest[line_start..i].contains("//") {
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
        let (uniform, decl) = match decl.trim_start().strip_prefix("<uniform>") {
            Some(d) => (true, d),
            None => (false, decl),
        };
        let (name, ty) = decl.split_once(':').map(|(n, t)| (n.trim().to_string(), t.trim().to_string())).ok_or_else(|| format!("{what}'s binding is `var NAME: TYPE`, got {:?}", stmt.trim()))?;
        let slot = if uniform {
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
            if samplers > TEXTURES {
                return Err(format!("{what} binds at most {TEXTURES} samplers, {name} is one more"));
            }
            UNIFORMS + 2 * (samplers - 1) + 1
        } else {
            return Err(format!("{what}'s binding {name} is a var<uniform>, a texture_2d<f32> or a sampler, got {ty} (the rest are FC2a's to come)"));
        };
        out += &rest[..i];
        out += &format!("@binding({slot})");
        rest = &rest[close + 1..];
    }
    out += rest;
    if samplers > textures.len() {
        return Err(format!("{what}'s samplers are its textures', one each: {samplers} samplers for {} textures", textures.len()));
    }
    Ok(Rebound { src: out, uniforms, textures, samplers })
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
}
