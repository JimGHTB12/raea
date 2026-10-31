//! Where a screen's picture lands on a model: the screen mesh's UV rectangle, found on a glTF's named node or a mesh,
//! and a box's UVs.

use super::*;

/// The affine map from a mesh's texture coordinates to the picture's (x right, y down, 0..1 over the extent of the
/// mesh's face), the mesh faced along its mean normal with +Y up: least squares over the face's vertices. The face is
/// the vertices whose normals are within 30 degrees of the face's mean normal (a flat panel's are all of them).
pub fn uv_map(mesh: &Mesh) -> Option<(Vec4, Vec4)> {
    use bevy::mesh::VertexAttributeValues as A;
    let Some(A::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None };
    let Some(A::Float32x2(uv)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else { return None };
    let normals: Vec<Vec3> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(A::Float32x3(ns)) if ns.len() == pos.len() => ns.iter().map(|v| Vec3::from(*v)).collect(),
        _ => vec![Vec3::Z; pos.len()],
    };
    // the mean normal, then again over the face it picks out (the skirt's normals lean the first one off)
    let mut n = normals.iter().sum::<Vec3>().normalize_or(Vec3::Z);
    let near = |n: Vec3| -> Vec<bool> { normals.iter().map(|v| v.dot(n) > 30f32.to_radians().cos()).collect() };
    let mut face = near(n);
    n = normals.iter().zip(&face).filter(|(_, f)| **f).map(|(v, _)| *v).sum::<Vec3>().normalize_or(n);
    face = near(n);
    if face.iter().filter(|f| **f).count() < 3 {
        face = vec![true; pos.len()];
    }
    let mut up = Vec3::Y - n * n.y;
    if up.length() < 1e-3 {
        up = Vec3::NEG_Z - n * -n.z;
    }
    let up = up.normalize();
    let right = up.cross(n).normalize();
    let st: Vec<(f32, f32)> = pos.iter().map(|p| (Vec3::from(*p).dot(right), Vec3::from(*p).dot(up))).collect();
    let on = || st.iter().zip(&face).filter(|(_, f)| **f).map(|(v, _)| *v);
    let (s0, s1) = on().fold((f32::MAX, f32::MIN), |a, v| (a.0.min(v.0), a.1.max(v.0)));
    let (t0, t1) = on().fold((f32::MAX, f32::MIN), |a, v| (a.0.min(v.1), a.1.max(v.1)));
    if s1 - s0 < 1e-6 || t1 - t0 < 1e-6 || uv.len() != pos.len() {
        return None;
    }
    // normal equations for [u v 1] -> x and -> y
    let mut m = Mat3::ZERO;
    let (mut bx, mut by) = (Vec3::ZERO, Vec3::ZERO);
    for ((q, (s, t)), _) in uv.iter().zip(&st).zip(&face).filter(|(_, f)| **f) {
        let r = Vec3::new(q[0], q[1], 1.0);
        m += Mat3::from_cols(r * r.x, r * r.y, r * r.z);
        let (x, y) = ((s - s0) / (s1 - s0), (t1 - t) / (t1 - t0));
        bx += r * x;
        by += r * y;
    }
    if m.determinant().abs() < 1e-12 {
        return None;
    }
    let inv = m.inverse();
    let (cx, cy) = (inv * bx, inv * by);
    Some((Vec4::new(cx.x, cx.y, cy.x, cy.y), Vec4::new(cx.z, cy.z, 0.0, 0.0)))
}

/// Each model's screens, once it's loaded: its meshes with a named material drawn as that screen.
pub fn model_screens(
    mut commands: Commands,
    frame: Res<FrameRows>,
    models: Query<Entity, (With<ModelReady>, Without<ScreensDone>)>,
    kids: Query<&Children>,
    named: Query<(&bevy::gltf::GltfMaterialName, &Mesh3d)>,
    meshes: Res<Assets<Mesh>>,
    mut mats: ResMut<Assets<CrtMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut screen_images: ResMut<ScreenImages>,
    mut err: ResMut<Error>,
) {
    for e in &models {
        commands.entity(e).insert(ScreensDone);
        let Some(v) = frame.vals(e) else { continue };
        let Some(serde_json::Value::Array(list)) = json(v.get("screens")) else { continue };
        let power = num(&v, "screen_power", 1.0);
        let all = descendants(e, &kids);
        for spec in &list {
            let material = spec.get("material").and_then(serde_json::Value::as_str).unwrap_or("");
            let name = spec.get("name").and_then(serde_json::Value::as_str).unwrap_or(material).to_string();
            let (w, h) = spec_size(spec);
            let img = match screen_images.0.get(&name) {
                Some((hd, iw, ih)) if (*iw, *ih) == (w, h) => hd.clone(),
                _ => {
                    let mut im = Image::new_fill(
                        bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                        bevy::render::render_resource::TextureDimension::D2,
                        &[0, 0, 0, 255],
                        TextureFormat::Rgba8UnormSrgb,
                        bevy::asset::RenderAssetUsages::RENDER_WORLD,
                    );
                    im.texture_descriptor.usage |= TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
                    let hd = images.add(im);
                    screen_images.0.insert(name.clone(), (hd.clone(), w, h));
                    hd
                }
            };
            let mut found = 0;
            for d in &all {
                let Ok((mn, mesh)) = named.get(*d) else { continue };
                if mn.0 != material {
                    continue;
                }
                let (a, b) = meshes.get(&mesh.0).and_then(uv_map).unwrap_or((Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::ZERO));
                let mut p = params(spec, a, b);
                p.res.w = power;
                let mat = mats.add(CrtMaterial { p, image: img.clone(), blend: see_through(spec) });
                commands.entity(*d).remove::<MeshMaterial3d<StandardMaterial>>().insert((MeshMaterial3d(mat.clone()), NotShadowCaster, Screen { mat, row: e }));
                found += 1;
            }
            if found == 0 {
                err.0 = Some(format!("model {}: no mesh has the material {material:?} for screen {name:?}", v.get("src").map(|s| format!("{s:?}")).unwrap_or_default()));
            }
        }
    }
}

/// Each mesh with a page on it (`surface`, sdk/render's layer="surface", S3a): its material made the screen, over the
/// mesh's whole UVs (B3 H5's surface mode). Checked once, when its row is first there.
pub fn mesh_screens(
    mut commands: Commands,
    frame: Res<FrameRows>,
    meshes: Query<(Entity, &Mesh3d), (With<MeshMaterial3d<StandardMaterial>>, Without<ScreensDone>)>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<CrtMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut screen_images: ResMut<ScreenImages>,
) {
    for (e, mesh) in &meshes {
        let Some(v) = frame.vals(e) else { continue };
        commands.entity(e).insert(ScreensDone);
        let Some(spec) = json(v.get("surface")) else { continue };
        let name = spec.get("name").and_then(serde_json::Value::as_str).unwrap_or("surface").to_string();
        let (w, h) = spec_size(&spec);
        let img = match screen_images.0.get(&name) {
            Some((hd, iw, ih)) if (*iw, *ih) == (w, h) => hd.clone(),
            _ => {
                let mut im = Image::new_fill(
                    bevy::render::render_resource::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                    bevy::render::render_resource::TextureDimension::D2,
                    &[0, 0, 0, 255],
                    TextureFormat::Rgba8UnormSrgb,
                    bevy::asset::RenderAssetUsages::RENDER_WORLD,
                );
                im.texture_descriptor.usage |= TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;
                let hd = images.add(im);
                screen_images.0.insert(name.clone(), (hd.clone(), w, h));
                hd
            }
        };
        // the whole UV square is the picture; a mesh without UVs (a Manifold shape) gets them from its faces
        if mesh_assets.get(&mesh.0).is_some_and(|m| m.attribute(Mesh::ATTRIBUTE_UV_0).is_none()) {
            if let Some(mut m) = mesh_assets.get_mut(&mesh.0) {
                box_uvs(&mut m);
            }
        }
        let mut p = params(&spec, Vec4::new(1.0, 0.0, 0.0, 1.0), Vec4::ZERO);
        p.res.w = num(&v, "screen_power", 1.0);
        let mat = mats.add(CrtMaterial { p, image: img, blend: see_through(&spec) });
        commands.entity(e).remove::<MeshMaterial3d<StandardMaterial>>().insert((MeshMaterial3d(mat.clone()), Screen { mat, row: e }));
    }
}

/// UVs for a mesh that has none: each vertex projected on the plane its normal faces most and scaled to the mesh's
/// bounds there, so each face of a box shows the whole picture, upright on the sides (v down from the top) and on the
/// top and bottom as seen from +Y.
pub fn box_uvs(m: &mut Mesh) {
    use bevy::mesh::VertexAttributeValues as A;
    let (Some(A::Float32x3(pos)), Some(A::Float32x3(nor))) = (m.attribute(Mesh::ATTRIBUTE_POSITION), m.attribute(Mesh::ATTRIBUTE_NORMAL)) else { return };
    let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for p in pos {
        lo = lo.min(Vec3::from(*p));
        hi = hi.max(Vec3::from(*p));
    }
    let size = (hi - lo).max(Vec3::splat(1e-6));
    let uv: Vec<[f32; 2]> = pos
        .iter()
        .zip(nor)
        .map(|(p, n)| {
            let t = (Vec3::from(*p) - lo) / size;
            let n = Vec3::from(*n);
            let a = n.abs();
            if a.x >= a.y && a.x >= a.z {
                // +X looks at it from +X: u along -Z
                [if n.x > 0.0 { 1.0 - t.z } else { t.z }, 1.0 - t.y]
            } else if a.z >= a.y {
                [if n.z > 0.0 { t.x } else { 1.0 - t.x }, 1.0 - t.y]
            } else {
                [t.x, if n.y > 0.0 { t.z } else { 1.0 - t.z }]
            }
        })
        .collect();
    m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
}
