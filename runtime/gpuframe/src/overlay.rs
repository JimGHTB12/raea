//! The 2D overlay: rows drawn over the 3D as a picture (clips, props, mips), and the frame tag a readback checks.

use super::*;

/// A 2D row's id among the edit's 2D rows (rows2d.rs).
#[derive(Component, Clone)]
pub struct Row2d(pub String);

/// The edit's 2D rows, drawn by bevy_ui over the 3D (rows2d.rs).
pub struct Rows2d(pub raea_runtime_view::rows2d::Rows);

/// Every 2D row's frame values onto its element, the rows drawn by bevy_ui and their picture handed to the overlay.
pub fn overlay_rows(world: &mut World) {
    let f = world.resource::<Clock>().0;
    let mut q = world.query::<(Entity, &Base2d, &Row2d)>();
    let sets: Vec<(String, serde_json::Map<String, serde_json::Value>)> = {
        let frame = world.resource::<FrameRows>();
        q.iter(world).filter_map(|(e, base, id)| Some((id.0.clone(), props_2d(base, &*frame.vals(e)?)))).collect()
    };
    let Some(mut rows) = world.get_non_send_mut::<Rows2d>() else { return };
    for (id, p) in sets {
        rows.0.set(&id, p);
    }
    rows.0.draw(f);
    let rgba = match rows.0.rgba() {
        Ok(px) => px.to_vec(),
        Err(e) => {
            eprintln!("render: frame {f}: the 2D rows: {e}");
            return;
        }
    };
    // the GPU path takes the premultiplied pixels as they are (the composite blends premultiplied)
    if let Some(mut gf) = world.get_resource_mut::<GpuFrame>() {
        if gf.on {
            gf.overlay = Some(std::sync::Arc::new(rgba));
            return;
        }
    }
    let Some(overlay) = world.get_resource::<Overlay>().map(|o| o.0.clone()) else { return };
    let mut images = world.resource_mut::<Assets<Image>>();
    let Some(mut img) = images.get_mut(&overlay) else { return };
    let mut px = rgba;
    for p in px.chunks_mut(4) {
        let a = p[3] as u32;
        if a > 0 && a < 255 {
            for c in &mut p[..3] {
                *c = ((*c as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    img.data = Some(px);
}

/// Each clip row's picture at the clock: visible between its `from` and `to` frames, showing its source at
/// `src_in + (frame - from) / fps` seconds, cropped to cover its box (x, y, w, h; the whole frame by default) and
/// zoomed about the centre by `zoom`.
pub fn clips(
    clock: Res<Clock>,
    edit: Res<Edit>,
    mut media: NonSendMut<Media>,
    mut err: ResMut<Error>,
    mut st: ResMut<Stages>,
    mut images: ResMut<Assets<Image>>,
    frame: Res<FrameRows>,
    mut q: Query<(Entity, &RowKey, &mut Clip, &mut Node, &mut ImageNode, &mut Visibility, &mut GlobalZIndex)>,
) {
    let t_clips = std::time::Instant::now();
    let mut prep_ns = 0u64;
    let (f, fps) = (clock.0, edit.prog.fps);
    let (fw, fh) = (edit.prog.width as f32, edit.prog.height as f32);
    let mut rows: Vec<_> = q.iter_mut().collect();
    rows.sort_by(|a, b| a.1.id.cmp(&b.1.id));
    let mut slots: HashMap<String, usize> = HashMap::new();
    for (e, row, mut clip, mut node, mut inode, mut vis, mut z) in rows {
        let Some(v) = frame.vals(e) else { continue };
        let v = &v;
        let from = num(v, "from", 0.0) as i64;
        let to = v.get("to").map(|x| x.num() as i64).unwrap_or(i64::MAX);
        let opacity = num(v, "opacity", 1.0);
        if f < from || f >= to || opacity <= 0.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        let Some(V::S(src)) = v.get("src") else {
            err.0 = Some(format!("clip {}: no src", row.id));
            return;
        };
        let slot = {
            let s = slots.entry(src.clone()).or_insert(0);
            *s += 1;
            *s - 1
        };
        let t = num(v, "src_in", 0.0) as f64 + (f - from) as f64 / fps;
        let key = (src.clone(), slot, (t * 1e6).round() as i64);
        if clip.last.as_ref() != Some(&key) {
            let m = &mut *media;
            let dec = match m.decoders.entry((src.clone(), slot)) {
                std::collections::hash_map::Entry::Occupied(o) => o.into_mut(),
                std::collections::hash_map::Entry::Vacant(e) => match resolve_src(&edit.media, src, &m.dir).and_then(|p| m.backend.open_video(&p)) {
                    Ok(d) => e.insert(d),
                    Err(x) => {
                        err.0 = Some(format!("clip {}: {x}", row.id));
                        return;
                    }
                },
            };
            let frame = match dec.frame_at(t) {
                Ok(fr) => fr,
                Err(x) => {
                    err.0 = Some(format!("clip {} at {t:.3}s: {x}", row.id));
                    return;
                }
            };
            let t_prep = std::time::Instant::now();
            let Some(mut img) = images.get_mut(&clip.image) else { continue };
            if img.width() != frame.width || img.height() != frame.height {
                *img = Image::new(
                    bevy::render::render_resource::Extent3d { width: frame.width, height: frame.height, depth_or_array_layers: 1 },
                    bevy::render::render_resource::TextureDimension::D2,
                    frame.rgba,
                    TextureFormat::Rgba8UnormSrgb,
                    bevy::asset::RenderAssetUsages::default(),
                );
            } else {
                img.data = Some(frame.rgba);
            }
            prep_ns += t_prep.elapsed().as_nanos() as u64;
            clip.last = Some(key);
        }
        let Some(img) = images.get(&clip.image) else { continue };
        let (sw, sh) = (img.width() as f32, img.height() as f32);
        let (x, y, w, h) = (num(v, "x", 0.0), num(v, "y", 0.0), num(v, "w", fw), num(v, "h", fh));
        node.left = Val::Px(x);
        node.top = Val::Px(y);
        node.width = Val::Px(w);
        node.height = Val::Px(h);
        // cover: the largest centred source rect with the box's aspect, then zoomed
        let zoom = num(v, "zoom", 1.0).max(0.01);
        let (cx, cy, cw, ch) = cover(sw, sh, w, h, zoom, (num(v, "pan_x", 0.0), num(v, "pan_y", 0.0)));
        inode.rect = Some(Rect::new(cx - cw / 2.0, cy - ch / 2.0, cx + cw / 2.0, cy + ch / 2.0));
        inode.color = Color::srgba(1.0, 1.0, 1.0, opacity.min(1.0));
        *z = GlobalZIndex(num(v, "z", 0.0) as i32);
        *vis = Visibility::Inherited;
    }
    st.clips += t_clips.elapsed().as_nanos() as u64;
    st.clip_prep += prep_ns;
}

/// The updates `Engine::frame` holds out for two readbacks exactly the same before it takes two nearly the same.
pub const NEAR_AFTER: usize = 24;

/// Two readbacks of a frame the same but for GPU noise: at most one byte in 10,000 differs (300 of a 1080p NV12
/// frame), none by more than 16. A part still coming in (a light, a material, a model) changes far more than that.
pub fn nearly_same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let most = (a.len() / 10_000).max(16);
    let mut n = 0;
    for (x, y) in a.iter().zip(b) {
        if x != y {
            n += 1;
            if n > most || x.abs_diff(*y) > 16 {
                return false;
            }
        }
    }
    true
}
