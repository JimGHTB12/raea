//! An extension's own uniform struct (docs/control.feature FC2b): a source that declares `@binding(100)
//! var<uniform> NAME: TYPE` itself is compiled as written, and its uniforms are laid out in TYPE's fields, in WGSL's
//! uniform layout, across the ShaderExt's 16 vec4 slots (256 bytes), so the author's struct reads them where it
//! declares them (Bevy's extended_material.wesl: `quantize_steps: u32` at offset 0).

use bevy::prelude::Vec4;
use bevy::shader::ShaderDefVal;

/// The fields of the struct `src` binds at `@binding(100)` as a uniform, in order, each `(name, type)`: a field under
/// `@if(DEF)` only when DEF is set true in `defs`. None when the source declares no such binding (raea's own layout).
pub fn own_struct(src: &str, defs: &[ShaderDefVal]) -> Option<Result<Vec<(String, String)>, String>> {
    // the first not under @if(BINDLESS), which compiles out (ui_bind.rs bindless_spans)
    let dead = crate::ui_bind::bindless_spans(src);
    let at = src.match_indices("@binding(100)").map(|(i, _)| i).find(|i| !dead.iter().any(|s| s.contains(i)))?;
    let rest = &src[at..];
    let decl = &rest[..rest.find(';')?];
    let ty = match decl.find("var<uniform>").and_then(|v| decl[v..].split_once(':')) {
        Some((_, t)) => t.trim().to_string(),
        None => return Some(Err("@binding(100) is a var<uniform> NAME: TYPE (the extension's uniforms)".into())),
    };
    Some(struct_fields(src, &ty, defs).map_err(|e| format!("@binding(100)'s type: {e}")))
}

/// The fields of `struct TY` as `src` declares it, in order, each `(name, type)`: a field under `@if(DEF)` only when DEF
/// is set true in `defs`.
pub fn struct_fields(src: &str, ty: &str, defs: &[ShaderDefVal]) -> Result<Vec<(String, String)>, String> {
    let Some(open) = find_struct(src, ty) else { return Err(format!("{ty} isn't a struct declared in the source")) };
    let Some(close) = src[open..].find('}') else { return Err(format!("struct {ty} has no closing brace")) };
    let body = &src[open..open + close];
    let set = |d: &str| defs.iter().any(|v| matches!(v, ShaderDefVal::Bool(k, true) if k == d) || matches!(v, ShaderDefVal::Int(k, n) if k == d && *n != 0));
    let mut out = Vec::new();
    // an @if on its own line holds for the field after it (as Bevy's extended_material.wesl writes it)
    let mut pending: Option<&str> = None;
    for line in body.lines().map(|l| l.split("//").next().unwrap_or("").trim()).filter(|l| !l.is_empty()) {
        let (cond, field) = match line.strip_prefix("@if(") {
            Some(r) => match r.split_once(')') {
                Some((d, f)) => (Some(d.trim()), f.trim()),
                None => return Err(format!("struct {ty}: {line:?}")),
            },
            None => (pending.take(), line),
        };
        if field.is_empty() {
            pending = cond;
            continue;
        }
        if cond.is_some_and(|d| !set(d)) {
            continue;
        }
        // fields on one line too (`struct S { a: f32, b: vec2<f32> }`): split at the commas outside a type's <>
        let mut depth = 0i32;
        for part in field.split(|c: char| {
            depth += (c == '<') as i32 - (c == '>') as i32;
            c == ',' && depth == 0
        }).map(str::trim).filter(|p| !p.is_empty()) {
            let Some((n, t)) = part.split_once(':') else { return Err(format!("struct {ty}: {line:?} isn't NAME: TYPE")) };
            out.push((n.trim().to_string(), t.trim().to_string()));
        }
    }
    Ok(out)
}

/// Where `struct NAME {`'s body starts
fn find_struct(src: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = src[from..].find("struct") {
        let s = from + i + "struct".len();
        let after = src[s..].trim_start();
        if after.starts_with(name) && after[name.len()..].trim_start().starts_with('{') {
            return Some(s + src[s..].find('{')? + 1);
        }
        from = s;
    }
    None
}

/// A field's WGSL type as (scalar kind, components): f32, u32 or i32, alone or in a vec2/3/4
pub fn kind(t: &str) -> Option<(char, usize)> {
    let t = t.replace(' ', "");
    let scalar = |s: &str| match s {
        "f32" => Some('f'),
        "u32" => Some('u'),
        "i32" => Some('i'),
        _ => None,
    };
    if let Some(k) = scalar(&t) {
        return Some((k, 1));
    }
    for n in 2..=4 {
        if let Some(inner) = t.strip_prefix(&format!("vec{n}<")).and_then(|r| r.strip_suffix('>')) {
            return Some((scalar(inner)?, n));
        }
        if let Some(c) = t.strip_prefix(&format!("vec{n}")).filter(|c| c.len() == 1).and_then(|c| c.chars().next()) {
            return "fui".contains(c).then_some((c, n));
        }
    }
    None
}

/// One component's 32 bits as its kind ('f', 'u' or 'i', kind's) holds it: a u32 or i32 rounded, a u32 at least 0.
pub fn word(k: char, x: f32) -> u32 {
    match k {
        'f' => x.to_bits(),
        'u' => x.round().max(0.0) as u32,
        _ => (x.round() as i32) as u32,
    }
}

/// The uniforms laid out in `fields` (WGSL's uniform layout: a scalar aligned to 4, a vec2 to 8, a vec3 or vec4 to 16)
/// in 256 bytes. Each field takes the uniform of its name; every field needs one, and every uniform a field.
pub fn pack(fields: &[(String, String)], uniforms: &[(String, Vec4)]) -> Result<[Vec4; 16], String> {
    let mut words = [0u32; 64];
    let mut at = 0usize;
    for (name, ty) in fields {
        let (k, n) = kind(ty).ok_or_else(|| format!("uniform {name}: its type {ty} isn't f32, u32, i32 or a vec2/3/4 of one (the rest are FC2b's to come)"))?;
        let align = match n {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        at = at.div_ceil(align) * align;
        if at + n > words.len() {
            return Err(format!("uniform {name}: the struct passes 256 bytes"));
        }
        let v = uniforms.iter().find(|(u, _)| u == name).map(|(_, v)| *v).ok_or_else(|| format!("the shader's uniform struct has {name}, which the uniforms don't give"))?;
        for c in 0..n {
            words[at + c] = word(k, v[c]);
        }
        at += n;
    }
    if let Some((u, _)) = uniforms.iter().find(|(u, _)| !fields.iter().any(|(f, _)| f == u)) {
        return Err(format!("uniform {u} isn't a field of the shader's @binding(100) struct"));
    }
    let mut out = [Vec4::ZERO; 16];
    for (i, s) in out.iter_mut().enumerate() {
        *s = Vec4::from_array(std::array::from_fn(|c| f32::from_bits(words[i * 4 + c])));
    }
    Ok(out)
}

/// A material's uniforms (`given`, {name: value}) in its own bindings' slots: `declared` is each binding's (name, WGSL
/// type) in slot order (ui_bind.rs's), and each value is packed in its type's layout, a scalar or vector from a number,
/// an array or a colour and a struct from {field: value}. A value for a uniform the source doesn't declare is refused,
/// as is a declared one without a value.
/// The first name given that isn't declared.
pub fn undeclared<'a>(given: &'a serde_json::Map<String, serde_json::Value>, declared: &[(String, String)]) -> Option<&'a String> {
    given.keys().find(|k| !declared.iter().any(|(n, _)| n == *k))
}

pub fn slots(src: &str, declared: &[(String, String)], given: &serde_json::Map<String, serde_json::Value>, defs: &[ShaderDefVal]) -> Result<[[Vec4; 16]; 8], String> {
    use raea_sys_material::shader_material::uniform;
    if let Some(k) = undeclared(given, declared) {
        return Err(format!("no uniform {k}"));
    }
    let mut out = [[Vec4::ZERO; 16]; 8];
    for (i, (name, ty)) in declared.iter().enumerate() {
        let v = given.get(name).ok_or_else(|| format!("its uniform {name} has no value"))?;
        out[i] = match v.as_object() {
            Some(o) => {
                let fields = struct_fields(src, ty, defs).map_err(|e| format!("uniform {name}: {e}"))?;
                let vals = o.iter().map(|(k, x)| uniform(k, x).map(|u| (u.name, u.value))).collect::<Result<Vec<_>, _>>()?;
                pack(&fields, &vals)?
            }
            None => pack(&[(name.clone(), ty.clone())], &[(name.clone(), uniform(name, v)?.value)])?,
        };
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "struct MyExtendedMaterial {\n    quantize_steps: u32,\n    // padding\n    @if(SIXTEEN_BYTE_ALIGNMENT)\n    _pad: u32,\n    tint: vec3<f32>,\n}\n@group(constants::MATERIAL_BIND_GROUP) @binding(100)\nvar<uniform> my_extended_material: MyExtendedMaterial;\n";

    #[test]
    fn an_own_struct_is_read_with_its_conditions_and_packed_in_wgsl_layout() {
        let f = own_struct(SRC, &[]).unwrap().unwrap();
        assert_eq!(f, vec![("quantize_steps".into(), "u32".into()), ("tint".into(), "vec3<f32>".into())]);
        let with = own_struct(SRC, &[ShaderDefVal::Bool("SIXTEEN_BYTE_ALIGNMENT".into(), true)]).unwrap().unwrap();
        assert_eq!(with.len(), 3);
        let s = pack(&f, &[("quantize_steps".into(), Vec4::new(3.0, 0.0, 0.0, 0.0)), ("tint".into(), Vec4::new(0.5, 0.25, 1.0, 0.0))]).unwrap();
        // a u32 3 at offset 0, the vec3 aligned to 16 bytes: the next slot
        assert_eq!(s[0].x.to_bits(), 3);
        assert_eq!(s[1].truncate(), bevy::prelude::Vec3::new(0.5, 0.25, 1.0));
        assert!(own_struct("fn lit() {}", &[]).is_none());
        assert!(pack(&f, &[("quantize_steps".into(), Vec4::ZERO)]).unwrap_err().contains("tint"));
        assert!(pack(&f[..1], &[("quantize_steps".into(), Vec4::ZERO), ("tint".into(), Vec4::ZERO)]).unwrap_err().contains("isn't a field"));
    }
}
