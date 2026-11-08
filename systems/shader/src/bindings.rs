//! A shader file's own `@group(0)` bindings as written (docs/control.feature FC2d): each `@group(0) @binding(N) var<SPACE>
//! NAME: TYPE;` outside a comment, read in order, for a compute or frame pass that binds what its file declares
//! (raea-sys-compute and raea-sys-framepass written.rs).

/// One binding: its number, its address space (`uniform`, `storage, read_write`; empty for a handle such as a
/// texture or sampler), its name and its type (spaces kept as written).
#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub binding: u32,
    pub space: String,
    pub name: String,
    pub ty: String,
}

/// The source's `@group(0)` bindings, in order.
pub fn group0(src: &str) -> Result<Vec<Binding>, String> {
    let mut out = Vec::new();
    for (i, _) in src.match_indices("@binding(") {
        let line = &src[src[..i].rfind('\n').map_or(0, |n| n + 1)..i];
        if line.contains("//") || !line.contains("@group(0)") {
            continue;
        }
        let stmt = &src[i..i + src[i..].find(';').ok_or("a @binding declaration has no ';'")?];
        let binding: u32 = stmt["@binding(".len()..].split(')').next().and_then(|n| n.trim().parse().ok()).ok_or_else(|| format!("a binding number: {stmt}"))?;
        let decl = stmt.split_once("var").map(|(_, d)| d.trim()).ok_or_else(|| format!("a binding is a var: {stmt}"))?;
        let (space, decl) = match decl.strip_prefix('<') {
            Some(r) => r.split_once('>').map(|(s, d)| (s.trim(), d.trim())).ok_or_else(|| format!("var<...>: {stmt}"))?,
            None => ("", decl),
        };
        let (name, ty) = decl.split_once(':').ok_or_else(|| format!("var NAME: TYPE: {stmt}"))?;
        out.push(Binding { binding, space: space.to_string(), name: name.trim().to_string(), ty: ty.trim().to_string() });
    }
    Ok(out)
}

/// The source's `var<uniform>`s as (name, type), in order: the fields a value packs into.
pub fn uniforms(src: &str) -> Result<Vec<(String, String)>, String> {
    Ok(group0(src)?.into_iter().filter(|b| b.space == "uniform").map(|b| (b.name, b.ty)).collect())
}

/// A bind group's entries: each buffer whole, then each texture view.
pub fn entries<'a>(buffers: &'a [(u32, bevy::render::render_resource::Buffer)], views: Vec<(u32, &'a bevy::render::render_resource::TextureView)>) -> Vec<bevy::render::render_resource::BindGroupEntry<'a>> {
    use bevy::render::render_resource::{BindGroupEntry, BindingResource};
    let mut out: Vec<BindGroupEntry> = buffers.iter().map(|(b, buf)| BindGroupEntry { binding: *b, resource: buf.as_entire_binding() }).collect();
    out.extend(views.into_iter().map(|(b, v)| BindGroupEntry { binding: b, resource: BindingResource::TextureView(v) }));
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_files_group0_bindings_are_read_in_order() {
        let src = "@group(0) @binding(0) var t: texture_2d<f32>;\n// @group(0) @binding(9) var x: f32;\n@group(1) @binding(0) var y: f32;\n@group(0) @binding(2) var<storage, read_write> b: array<f32>;\n";
        let b = super::group0(src).unwrap();
        assert_eq!(b.len(), 2);
        assert_eq!((b[0].space.as_str(), b[0].ty.as_str()), ("", "texture_2d<f32>"));
        assert_eq!((b[1].binding, b[1].space.as_str(), b[1].name.as_str()), (2, "storage, read_write", "b"));
    }
}
