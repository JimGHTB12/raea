use super::*;

/// A <BufferMesh>'s vertex colours (`colors`: r, g, b, a in linear light for each vertex, or "#rrggbb" sRGB each),
/// laid out per triangle corner as buffer_tris lays its positions; None without them. StandardMaterial multiplies its
/// base colour by them (Bevy's VERTEX_COLORS).
pub fn buffer_colors(json: &str) -> Result<Option<Vec<[f32; 4]>>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("a buffer mesh isn't JSON: {e}"))?;
    let Some(c) = v.get("colors").filter(|c| !c.is_null()) else { return Ok(None) };
    let a = c.as_array().ok_or("a buffer mesh's colors: an array")?;
    let n = v.get("positions").and_then(|p| p.as_array()).map(|p| p.len() / 3).unwrap_or(0);
    let per: Vec<[f32; 4]> = if a.iter().all(|x| x.is_string()) {
        a.iter().map(|x| {
            let s = x.as_str().unwrap();
            Srgba::hex(s.trim_start_matches('#')).map(|c| LinearRgba::from(c).to_f32_array()).map_err(|e| format!("a buffer mesh's colour {s:?}: {e}"))
        }).collect::<Result<_, _>>()?
    } else {
        let f: Vec<f32> = a.iter().map(|x| x.as_f64().map(|f| f as f32).ok_or_else(|| format!("a buffer mesh's colors: numbers or \"#rrggbb\", got {x}"))).collect::<Result<_, _>>()?;
        if f.len() != n * 4 {
            return Err(format!("a buffer mesh's colors: r, g, b, a for each of its {n} vertices, got {} numbers", f.len()));
        }
        f.chunks(4).map(|c| [c[0], c[1], c[2], c[3]]).collect()
    };
    if per.len() != n {
        return Err(format!("a buffer mesh's colors: one for each of its {n} vertices, got {}", per.len()));
    }
    let idx: Vec<usize> = match v.get("indices").and_then(|i| i.as_array()) {
        Some(i) => i.iter().map(|x| x.as_f64().unwrap_or(0.0) as usize).collect(),
        None => (0..n).collect(),
    };
    Ok(Some(idx.iter().map(|&i| per[i.min(n.saturating_sub(1))]).collect()))
}

/// One of Bevy's primitive shapes as a Mesh (Mesh::from), its reflected fields given over its default: {primitive:
/// "Torus", minor_radius: 0.2, major_radius: 1}. 3D: Sphere, Cuboid, Cylinder, Capsule3d, Cone, ConicalFrustum, Torus,
/// Plane3d, Tetrahedron, Triangle3d; 2D (flat, facing +Z): Circle, Rectangle, RegularPolygon, Ellipse, Annulus, Rhombus,
/// Capsule2d, Triangle2d.
pub fn primitive_mesh(o: &serde_json::Map<String, serde_json::Value>) -> Result<Mesh, String> {
    use bevy::shape::*;
    fn fill<T: Default + bevy::reflect::PartialReflect + Into<Mesh>>(name: &str, o: &serde_json::Map<String, serde_json::Value>) -> Result<Mesh, String> {
        let mut x = T::default();
        for (k, v) in o.iter().filter(|(k, _)| k.as_str() != "primitive") {
            let bevy::reflect::ReflectMut::Struct(st) = x.reflect_mut() else { return Err(format!("{name} has no fields")) };
            let f = st.field_mut(k).ok_or_else(|| format!("{name} has no field {k:?}"))?;
            apply_json(f, v).map_err(|e| format!("{name}.{k}: {e}"))?;
        }
        Ok(x.into())
    }
    let name = o.get("primitive").and_then(|p| p.as_str()).unwrap_or("");
    match name {
        "Sphere" => fill::<Sphere>(name, o),
        "Cuboid" => fill::<Cuboid>(name, o),
        "Cylinder" => fill::<Cylinder>(name, o),
        "Capsule3d" => fill::<Capsule3d>(name, o),
        "Cone" => fill::<Cone>(name, o),
        "ConicalFrustum" => fill::<ConicalFrustum>(name, o),
        "Torus" => fill::<Torus>(name, o),
        "Plane3d" => fill::<Plane3d>(name, o),
        "Tetrahedron" => fill::<Tetrahedron>(name, o),
        "Triangle3d" => fill::<Triangle3d>(name, o),
        "Circle" => fill::<Circle>(name, o),
        "Rectangle" => fill::<Rectangle>(name, o),
        "RegularPolygon" => fill::<RegularPolygon>(name, o),
        "Ellipse" => fill::<Ellipse>(name, o),
        "Annulus" => fill::<Annulus>(name, o),
        "Rhombus" => fill::<Rhombus>(name, o),
        "Capsule2d" => fill::<Capsule2d>(name, o),
        "Triangle2d" => fill::<Triangle2d>(name, o),
        _ => Err(format!("no primitive {name:?} (Sphere, Cuboid, Cylinder, Capsule3d, Cone, ConicalFrustum, Torus, Plane3d, Tetrahedron, Triangle3d, Circle, Rectangle, RegularPolygon, Ellipse, Annulus, Rhombus, Capsule2d, Triangle2d)")),
    }
}

/// A JSON value onto a reflected field: a number onto f32/f64/u32/usize/i32, [x, y(, z)] onto Vec2/Vec3/Dir2/Dir3,
/// an object onto a struct's fields (an Annulus's inner_circle).
pub fn apply_json(f: &mut dyn bevy::reflect::PartialReflect, v: &serde_json::Value) -> Result<(), String> {
    let any = f.try_as_reflect_mut().map(|r| r.as_any_mut());
    if let Some(any) = any {
        let n = v.as_f64();
        let arr: Option<Vec<f32>> = v.as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(f64::NAN) as f32).collect());
        if let (Some(x), Some(n)) = (any.downcast_mut::<f32>(), n) { *x = n as f32; return Ok(()); }
        if let (Some(x), Some(n)) = (any.downcast_mut::<f64>(), n) { *x = n; return Ok(()); }
        if let (Some(x), Some(n)) = (any.downcast_mut::<u32>(), n) { *x = n as u32; return Ok(()); }
        if let (Some(x), Some(n)) = (any.downcast_mut::<usize>(), n) { *x = n as usize; return Ok(()); }
        if let (Some(x), Some(n)) = (any.downcast_mut::<i32>(), n) { *x = n as i32; return Ok(()); }
        if let Some(a) = &arr {
            if let (Some(x), 2) = (any.downcast_mut::<Vec2>(), a.len()) { *x = Vec2::new(a[0], a[1]); return Ok(()); }
            if let (Some(x), 3) = (any.downcast_mut::<Vec3>(), a.len()) { *x = Vec3::new(a[0], a[1], a[2]); return Ok(()); }
            if let (Some(x), 3) = (any.downcast_mut::<Dir3>(), a.len()) { *x = Dir3::new(Vec3::new(a[0], a[1], a[2])).map_err(|e| e.to_string())?; return Ok(()); }
            if let (Some(x), 2) = (any.downcast_mut::<Dir2>(), a.len()) { *x = Dir2::new(Vec2::new(a[0], a[1])).map_err(|e| e.to_string())?; return Ok(()); }
            // a list of points (a Tetrahedron's vertices, a triangle's): each onto its element
        }
    }
    match (f.reflect_mut(), v) {
        (bevy::reflect::ReflectMut::Struct(st), serde_json::Value::Object(o)) => {
            for (k, x) in o {
                let g = st.field_mut(k).ok_or_else(|| format!("no field {k:?}"))?;
                apply_json(g, x).map_err(|e| format!("{k}: {e}"))?;
            }
            Ok(())
        }
        (bevy::reflect::ReflectMut::TupleStruct(ts), _) if ts.field_len() == 1 => apply_json(ts.field_mut(0).unwrap(), v),
        (bevy::reflect::ReflectMut::Array(ar), serde_json::Value::Array(a)) if ar.len() == a.len() => {
            for (i, x) in a.iter().enumerate() {
                apply_json(ar.get_mut(i).unwrap(), x).map_err(|e| format!("[{i}]: {e}"))?;
            }
            Ok(())
        }
        _ => Err(format!("can't set it from {v}")),
    }
}

/// A mesh from its own vertex buffers (<BufferMesh>, or a Mesh handle given as {positions, ...}): JSON
/// {positions: [x, y, z, ...], indices?: [i, ...], normals?: [x, y, z, ...], uvs?: [u, v, ...]}, a triangle list
/// (indexed, or each three positions a triangle), as flat triangles; without normals each face's own. Its texture
/// coordinates when it gave them.
pub fn buffer_tris(json: &str) -> Result<(raea_core_shape::Tris, Option<Vec<[f32; 2]>>), String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("a buffer mesh isn't JSON: {e}"))?;
    let nums = |k: &str| -> Result<Option<Vec<f32>>, String> {
        match v.get(k) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Array(a)) => a.iter().map(|x| x.as_f64().map(|f| f as f32).ok_or_else(|| format!("a buffer mesh's {k}: numbers, got {x}"))).collect::<Result<Vec<_>, _>>().map(Some),
            Some(x) => Err(format!("a buffer mesh's {k}: an array of numbers, got {x}")),
        }
    };
    let pos = nums("positions")?.ok_or("a buffer mesh needs positions: [x, y, z, ...]")?;
    if pos.len() % 3 != 0 || pos.is_empty() {
        return Err(format!("a buffer mesh's positions: x, y, z for each vertex, got {} numbers", pos.len()));
    }
    let n = pos.len() / 3;
    let idx: Vec<usize> = match nums("indices")? {
        Some(i) => i.iter().map(|x| *x as usize).collect(),
        None => (0..n).collect(),
    };
    if idx.len() % 3 != 0 || idx.iter().any(|&i| i >= n) {
        return Err(format!("a buffer mesh's indices: three a triangle, each under {n} (the vertices), got {}", idx.len()));
    }
    let normals = nums("normals")?;
    let uvs = nums("uvs")?;
    if normals.as_ref().is_some_and(|x| x.len() != pos.len()) || uvs.as_ref().is_some_and(|u| u.len() != n * 2) {
        return Err("a buffer mesh's normals are x, y, z and its uvs u, v for each vertex".into());
    }
    let p3 = |i: usize| [pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]];
    let mut t = raea_core_shape::Tris { positions: Vec::with_capacity(idx.len()), normals: Vec::with_capacity(idx.len()) };
    let mut uv = uvs.as_ref().map(|_| Vec::with_capacity(idx.len()));
    for tri in idx.chunks(3) {
        let (a, b, c) = (Vec3::from(p3(tri[0])), Vec3::from(p3(tri[1])), Vec3::from(p3(tri[2])));
        let face = (b - a).cross(c - a).normalize_or_zero().to_array();
        for &i in tri {
            t.positions.push(p3(i));
            t.normals.push(match &normals {
                Some(nr) => [nr[i * 3], nr[i * 3 + 1], nr[i * 3 + 2]],
                None => face,
            });
            if let (Some(out), Some(u)) = (uv.as_mut(), uvs.as_ref()) {
                out.push([u[i * 2], u[i * 2 + 1]]);
            }
        }
    }
    Ok((t, uv))
}

/// Texture coordinates for a Manifold mesh (it has none): each vertex projected along its face's main axis onto the
/// mesh's bounds, so a picture map covers each side of a box once, the right way round seen from outside.
pub fn box_uvs(pos: &[[f32; 3]], nor: &[[f32; 3]]) -> Vec<[f32; 2]> {
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for p in pos {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let span = |k: usize| (hi[k] - lo[k]).max(1e-6);
    let a = |p: &[f32; 3], k: usize| (p[k] - lo[k]) / span(k);
    let b = |p: &[f32; 3], k: usize| (hi[k] - p[k]) / span(k);
    pos.iter().zip(nor).map(|(p, n)| {
        let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
        if ax >= ay && ax >= az {
            if n[0] > 0.0 { [b(p, 2), b(p, 1)] } else { [a(p, 2), b(p, 1)] }
        } else if ay >= az {
            if n[1] > 0.0 { [a(p, 0), a(p, 2)] } else { [a(p, 0), b(p, 2)] }
        } else if n[2] > 0.0 {
            [a(p, 0), b(p, 1)]
        } else {
            [b(p, 0), b(p, 1)]
        }
    }).collect()
}

