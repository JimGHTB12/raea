//! A storage buffer's contents in WGSL's storage layout (docs/control.feature FC2a): any host-shareable type its
//! source declares (f32, u32, i32 and their vec2-4, matCxR<f32>, array<T, N>, array<T>, and the source's own structs,
//! a field under @if(DEF) as layout.rs's struct_fields reads it) from the value given: a number, [components],
//! [elements] or {field: value}. Bevy's examples write such buffers with encase, as gpu_component_array_buffer's
//! `var<storage> material_data: CustomMaterialDataArray` holds `data_array: array<CustomMaterialData>`.

use bevy::shader::ShaderDefVal;
use serde_json::Value;

use crate::layout::{kind, struct_fields, undeclared, word};

/// The source whose structs a type may name, and the defs its fields' @if read.
struct Cx<'a> {
    src: &'a str,
    defs: &'a [ShaderDefVal],
}

/// `matCxR<f32>` (or `matCxRf`): its columns and rows.
fn mat(ty: &str) -> Option<(usize, usize)> {
    let t = ty.replace(' ', "");
    let r = t.strip_prefix("mat")?;
    let (dims, scalar) = r.split_at(3.min(r.len()));
    if !(scalar == "<f32>" || scalar == "f") {
        return None;
    }
    let (c, rows) = dims.split_once('x')?;
    Some((c.parse().ok().filter(|n| (2..=4).contains(n))?, rows.parse().ok().filter(|n| (2..=4).contains(n))?))
}

/// `array<T, N>` or `array<T>`: T, and N when it has one.
fn array(ty: &str) -> Option<(&str, Option<usize>)> {
    let inner = ty.trim().strip_prefix("array<")?.strip_suffix('>')?;
    Some(match inner.rfind(',') {
        Some(i) if inner[i + 1..].trim().parse::<usize>().is_ok() => (inner[..i].trim(), inner[i + 1..].trim().parse().ok()),
        _ => (inner.trim(), None),
    })
}

/// A type's alignment and size in bytes; a runtime array's size is that of the elements `v` gives.
fn align_size(cx: &Cx, ty: &str, v: Option<&Value>) -> Result<(usize, usize), String> {
    if let Some((_, n)) = kind(ty) {
        return Ok(match n {
            1 => (4, 4),
            2 => (8, 8),
            3 => (16, 12),
            _ => (16, 16),
        });
    }
    if let Some((c, r)) = mat(ty) {
        let a = if r == 2 { 8 } else { 16 };
        return Ok((a, c * a));
    }
    if let Some((el, n)) = array(ty) {
        let (a, s) = align_size(cx, el, None)?;
        let count = n.unwrap_or_else(|| v.and_then(Value::as_array).map_or(0, Vec::len));
        return Ok((a, count * s.div_ceil(a) * a));
    }
    let (mut end, mut align) = (0usize, 4usize);
    for (name, fty) in struct_fields(cx.src, ty, cx.defs)? {
        let (a, s) = align_size(cx, &fty, v.and_then(|v| v.get(&name)))?;
        end = end.div_ceil(a) * a + s;
        align = align.max(a);
    }
    Ok((align, end.div_ceil(align) * align))
}

/// `v` written as `ty` into `out` at byte `at`.
fn put(cx: &Cx, ty: &str, v: &Value, what: &str, out: &mut [u8], at: usize) -> Result<(), String> {
    if let Some((k, n)) = kind(ty) {
        let comps: Vec<f64> = match v {
            Value::Array(a) => a.iter().filter_map(Value::as_f64).collect(),
            x => x.as_f64().into_iter().collect(),
        };
        if comps.len() != n || v.as_array().is_some_and(|a| a.len() != n) {
            return Err(format!("{what} is {n} numbers ({ty}), got {v}"));
        }
        for (c, x) in comps.iter().enumerate() {
            out[at + 4 * c..at + 4 * c + 4].copy_from_slice(&word(k, *x as f32).to_le_bytes());
        }
        return Ok(());
    }
    if let Some((c, r)) = mat(ty) {
        // its columns, [[...], ...], or its c x r numbers column by column
        let cols: Vec<Value> = match v.as_array() {
            Some(a) if a.len() == c => a.clone(),
            Some(a) if a.len() == c * r => a.chunks(r).map(|ch| Value::Array(ch.to_vec())).collect(),
            _ => return Err(format!("{what} is a {ty}: {c} columns of {r} numbers, got {v}")),
        };
        let stride = if r == 2 { 8 } else { 16 };
        for (i, col) in cols.iter().enumerate() {
            put(cx, &format!("vec{r}<f32>"), col, &format!("{what}'s column {i}"), out, at + i * stride)?;
        }
        return Ok(());
    }
    if let Some((el, n)) = array(ty) {
        let items = v.as_array().ok_or_else(|| format!("{what} is a list of its elements ({ty}), got {v}"))?;
        if let Some(n) = n.filter(|n| items.len() > *n) {
            return Err(format!("{what} holds {n} elements ({ty}), got {}", items.len()));
        }
        let (a, s) = align_size(cx, el, None)?;
        for (i, it) in items.iter().enumerate() {
            put(cx, el, it, &format!("{what}'s element {i}"), out, at + i * s.div_ceil(a) * a)?;
        }
        return Ok(());
    }
    let fields = struct_fields(cx.src, ty, cx.defs).map_err(|e| format!("{what}: {e}"))?;
    let o = v.as_object().ok_or_else(|| format!("{what} is a {ty}, {{field: value}}, got {v}"))?;
    if let Some(k) = undeclared(o, &fields) {
        return Err(format!("{what}: {ty} has no field {k}"));
    }
    let mut off = 0usize;
    for (name, fty) in &fields {
        let fv = o.get(name).ok_or_else(|| format!("{what}: its field {name} has no value"))?;
        let (a, s) = align_size(cx, fty, Some(fv))?;
        off = off.div_ceil(a) * a;
        put(cx, fty, fv, &format!("{what}.{name}"), out, at + off)?;
        off += s;
    }
    Ok(())
}

/// The buffer `name` of WGSL type `ty` (declared in `src`) holding `v`, as 32-bit words; at least 16 bytes, as a
/// binding can't be empty.
pub fn words(src: &str, defs: &[ShaderDefVal], name: &str, ty: &str, v: &Value) -> Result<Vec<u32>, String> {
    let cx = Cx { src, defs };
    let (_, size) = align_size(&cx, ty, Some(v)).map_err(|e| format!("storage buffer {name}: {e}"))?;
    let mut out = vec![0u8; size.max(16).div_ceil(4) * 4];
    put(&cx, ty, v, &format!("storage buffer {name}"), &mut out, 0)?;
    Ok(out.chunks(4).map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]])).collect())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn structs_arrays_and_matrices_are_laid_out_as_wgsl_stores_them() {
        let src = "struct D {\n    color: vec3<f32>,\n    pad: u32,\n}\nstruct A {\n    data_array: array<D>,\n}\nstruct S { a: f32, b: vec2<f32>, c: vec3<f32> }\n";
        let w = super::words(src, &[], "m", "A", &json!({"data_array": [{"color": [0.5, 1, 2], "pad": 7}, {"color": [3, 4, 5], "pad": 0}]})).unwrap();
        assert_eq!(w.len(), 8);
        assert_eq!((f32::from_bits(w[0]), f32::from_bits(w[2]), w[3], f32::from_bits(w[4])), (0.5, 2.0, 7, 3.0));
        // a at 0, b at 8 (vec2 aligned to 8), c at 16; the struct 32 bytes (aligned to 16)
        let s = super::words(src, &[], "s", "array<S, 2>", &json!([{"a": 1, "b": [2, 3], "c": [4, 5, 6]}])).unwrap();
        assert_eq!((s.len(), f32::from_bits(s[2]), f32::from_bits(s[4]), f32::from_bits(s[6])), (16, 2.0, 4.0, 6.0));
        let m = super::words(src, &[], "t", "mat3x3<f32>", &json!([1, 2, 3, 4, 5, 6, 7, 8, 9])).unwrap();
        assert_eq!((m.len(), f32::from_bits(m[4]), f32::from_bits(m[10])), (12, 4.0, 9.0));
        assert!(super::words(src, &[], "m", "A", &json!({"data_array": [{"color": [1, 2]}]})).unwrap_err().contains("color"));
        assert!(super::words(src, &[], "m", "A", &json!({"data": []})).unwrap_err().contains("no field data"));
    }
}
