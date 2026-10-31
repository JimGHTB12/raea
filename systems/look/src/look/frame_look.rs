//! A frame's own <Look>: its material's parameters set from the frame's look row each frame.

use super::*;

/// The whole frame's look (a <Look> in the scene: frame_look on the camera row): the scene's camera draws into a target
/// a cell per texel instead of the frame, its bloom, fog and depth of field with it, and the composite lays those cells
/// over the frame, opaque, under any model's own look. Read each frame, so a piece can change look from one scene to
/// the next (frame_looks, FrameRows::vals): a new look's parameters (and cells, if its cell size differs) on the
/// same quad, and on a frame without one the scene's camera back on the frame, drawn plain. The camera's <Outline>
/// follows its row the same way.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_frame_look(
    mut commands: Commands,
    frame: Res<FrameRows>,
    edit: Res<Edit>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<LookMaterial>>,
    mut main: Query<(Entity, &mut RenderTarget, &mut Camera, &Msaa, Has<bevy::camera::Hdr>, Option<&Tonemapping>, Option<&super::super::outline::InkOutline>, Has<super::super::bevy_api::Untonemapped>), (With<Camera3d>, Without<LookCam>)>,
    composite: Query<Entity, (With<LookCam>, With<Camera3d>, Without<Mesh3d>)>,
    mut vis: Query<&mut Visibility, (With<Mesh3d>, With<LookCam>)>,
    mut ui: Query<&mut bevy::ui::UiTargetCamera>,
    vignettes: Query<&bevy::post_process::effect_stack::Vignette>,
    pass_mask: Res<super::super::compose::PassMask>,
    mut st: Local<FrameLookState>,
) {
    let Ok((cam, mut target, mut camera, msaa, hdr, tm, ink, untm)) = main.single_mut() else { return };
    let Some(v) = frame.vals(cam) else { return };
    let v = &v;
    // the outline: on, changed or off with the row
    match v.get("outline") {
        Some(c) => {
            let line = colour(Some(c), [0.0, 0.0, 0.0, 1.0]).to_linear().to_f32_array();
            let want = super::super::outline::InkOutline { color: Vec4::from_array(line), depth: num(v, "outline_depth", 0.04), normal: num(v, "outline_normal", 0.3), width: num(v, "outline_width", 1.0).max(1.0), far: num(v, "outline_far", 0.0) / num(v, "near", 0.1), skip: super::super::compose::bits(v, "outline_skip") };
            if ink.is_none_or(|o| o.color != want.color || o.depth != want.depth || o.normal != want.normal || o.width != want.width || o.far != want.far || o.skip != want.skip) {
                commands.entity(cam).insert((want, bevy::core_pipeline::prepass::DepthPrepass, bevy::core_pipeline::prepass::NormalPrepass));
            }
        }
        None => {
            if ink.is_some() {
                commands.entity(cam).remove::<super::super::outline::InkOutline>();
            }
        }
    }
    let mode = match v.get("frame_look") {
        Some(V::S(m)) => Some(m.as_str()),
        _ => None,
    };
    let Some(mode) = mode else {
        // no look this frame: the camera back on the frame, as it was, and the quad hidden
        if st.on {
            if let Some(t) = st.frame_target.clone() {
                *target = t;
            }
            if let Some((t, c, u)) = st.own.clone() {
                camera.clear_color = c;
                match t {
                    Some(t) => commands.entity(cam).insert(t),
                    None => commands.entity(cam).remove::<Tonemapping>(),
                };
                if !u {
                    commands.entity(cam).remove::<super::super::bevy_api::Untonemapped>();
                }
            }
            if let Some(q) = st.quad {
                if let Ok(mut x) = vis.get_mut(q) {
                    *x = Visibility::Hidden;
                }
            }
            // the vignette back on the scene's camera (below)
            if let Ok(comp) = composite.single() {
                if let Ok(vg) = vignettes.get(comp) {
                    commands.entity(cam).insert(vg.clone());
                    commands.entity(comp).remove::<bevy::post_process::effect_stack::Vignette>();
                }
            }
            st.on = false;
        }
        return;
    };
    let paint = mode == "paint";
    let mut p = frame_params(v, mode);
    let px = num(v, "frame_look_px", 6.0).max(1.0);
    let (w, h) = (edit.prog.width, edit.prog.height);
    let (cw, ch) = (((w as f32) / px).ceil().max(1.0) as u32, ((h as f32) / px).ceil().max(1.0) as u32);
    p.size = Vec2::new(cw as f32, ch as f32);
    p.crisp = (px > 1.5) as u32;
    if st.frame_target.is_none() {
        st.frame_target = Some(target.clone());
    }
    if st.own.is_none() {
        st.own = Some((tm.copied(), camera.clear_color.clone(), untm));
    }
    // cells the look's size (made again when a look with another cell size comes on), the camera drawing into them
    let cells = match &st.cells {
        Some((c, a, b)) if (*a, *b) == (cw, ch) => c.clone(),
        _ => {
            let mut img = Image::new_target_texture(cw, ch, TextureFormat::Rgba8UnormSrgb, None);
            img.sampler = bevy::image::ImageSampler::nearest();
            let c = images.add(img);
            st.cells = Some((c.clone(), cw, ch));
            c
        }
    };
    // the composition passes' mask, while the look has passes of its own (else Bevy's default, never read)
    let mask = pass_mask.image().filter(|_| p.masked == 1).unwrap_or_default();
    if !matches!(&*target, RenderTarget::Image(i) if i.handle == cells) {
        *target = RenderTarget::Image(cells.clone().into());
    }
    // the paint's colours are chosen, not exposed: no tone curve between them and the frame (Untonemapped keeps it so
    // on Bevy 0.20, as 0.19's None did); and with a sky, the scene clears see-through so the composite knows where
    // nothing's drawn. Other looks: the camera's own.
    if let Some((own_tm, own_clear, own_untm)) = st.own.clone() {
        let want_tm = if paint { Some(Tonemapping::None) } else { own_tm };
        let want_untm = paint || own_untm;
        if untm != want_untm || (!want_untm && tm.copied() != want_tm) {
            match want_tm {
                Some(t) => commands.entity(cam).insert(t),
                None => commands.entity(cam).remove::<Tonemapping>(),
            };
            if want_untm {
                commands.entity(cam).insert(super::super::bevy_api::Untonemapped);
            } else {
                commands.entity(cam).remove::<super::super::bevy_api::Untonemapped>();
            }
        }
        let want_clear = if paint && p.sky_top.w > 0.0 { ClearColorConfig::Custom(Color::NONE) } else { own_clear };
        if !same_clear(&camera.clear_color, &want_clear) {
            camera.clear_color = want_clear;
        }
    }
    match &st.mat {
        Some(hm) => {
            if mats.get(hm).is_some_and(|m| m.p != p || m.cells != cells || m.mask != mask) {
                if let Some(mut m) = mats.get_mut(hm) {
                    m.p = p;
                    m.cells = cells;
                    m.mask = mask;
                }
            }
        }
        None => {
            let mat = mats.add(LookMaterial { p, cells, mask });
            st.mat = Some(mat.clone());
            st.quad = Some(commands.spawn((Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))), MeshMaterial3d(mat), RenderLayers::layer(COMPOSITE), Transform::default(), bevy::camera::visibility::NoFrustumCulling, LookCam)).id());
        }
    }
    if let Some(q) = st.quad {
        if let Ok(mut x) = vis.get_mut(q) {
            if *x != Visibility::Inherited {
                *x = Visibility::Inherited;
            }
        }
    }
    if !st.on && composite.is_empty() {
        let mut c = commands.spawn((
            Camera3d::default(),
            Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
            st.frame_target.clone().unwrap_or_default(),
            *msaa,
            RenderLayers::layer(COMPOSITE),
            Tonemapping::None,
            super::super::bevy_api::Untonemapped,
            DebandDither::Disabled,
            Transform::default(),
            LookCam,
        ));
        if hdr {
            c.insert(bevy::camera::Hdr);
        }
        // the page goes over the frame, not into its cells (and stays on the composite, which draws onto the frame,
        // when a later scene has no look)
        c.insert(bevy::ui::IsDefaultUiCamera);
        // and so does every page part made for the scene's camera (a render's clips and pages, drawn as UI on it):
        // left on a camera that now draws into the cells, they'd be laid out at the cells' size and drawn into them
        let c = c.id();
        commands.entity(cam).remove::<bevy::ui::IsDefaultUiCamera>();
        for mut t in &mut ui {
            if t.0 == cam {
                t.0 = c;
            }
        }
    }
    // a <Grade>'s vignette (scene3d.rs puts it on the scene's camera) over the looked frame, not under it: Bevy's
    // vignette writes every pixel opaque, so drawn into the cells it hid where nothing was drawn and the paint's sky
    // came out black (10-21, anything/kitchen/sky.tsx). Moved to the composite, it darkens the finished frame's edges.
    if let Ok(comp) = composite.single() {
        if let Ok(vg) = vignettes.get(cam) {
            commands.entity(comp).insert(vg.clone());
            commands.entity(cam).remove::<bevy::post_process::effect_stack::Vignette>();
        } else if v.get("vignette").is_none() && vignettes.get(comp).is_ok() {
            commands.entity(comp).remove::<bevy::post_process::effect_stack::Vignette>();
        }
    }
    st.on = true;
}
