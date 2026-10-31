//! The compose passes in the render graph: the prepass shader, each view's passes synced from its rows, and their
//! pipelines.

use super::*;

/// Bevy's two prepass shaders patched once each has loaded (dof.rs's swap where Bevy loads them can't be used: Bevy
/// loads pbr_prepass.wesl as a library as its plugin builds, before raea-render's install runs). The shader assets are
/// changed in place, so the pipelines that use them are made again, as a hot reload would; it lands in the engine's
/// warm-up updates, before any frame is taken.
pub fn prepass_pass(assets: Res<AssetServer>, mut shaders: ResMut<Assets<Shader>>, mut handles: Local<Vec<(Handle<Shader>, bool)>>) {
    if handles.is_empty() {
        handles.push((assets.load("embedded://bevy_pbr/prepass/prepass.wesl"), false));
        handles.push((assets.load("embedded://bevy_pbr/render/pbr_prepass.wesl"), false));
    }
    for (i, (h, done)) in handles.iter_mut().enumerate() {
        if *done {
            continue;
        }
        let Some(src) = shaders.get(&*h).map(|s| s.source.as_str().to_string()) else { continue };
        *done = true;
        if src.contains("raea_pass_alpha") {
            continue;
        }
        let (fnv, patched) = if i == 0 { (PREPASS_FNV, patch_prepass(&src)) } else { (PBR_PREPASS_FNV, patch_pbr_prepass(&src)) };
        let found = fnv1a(src.as_bytes());
        assert!(found == fnv, "raea: Bevy's prepass shader changed (hash {found:#x}, compose.rs's patch was taken from {fnv:#x}); re-take it (docs/bevy-upgrade.md)");
        if let Some(mut s) = shaders.get_mut(&*h) {
            s.source = bevy::shader::Source::Wesl(patched.into());
        }
    }
}

pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3))
}

// ------------------------------------------------------------------------------------------------ the main world

/// Each frame: every row's pass on its meshes (MeshTag), and the cameras' passes (the mask, the prepasses, what bloom
/// and grade hold back). A frame whose camera row names no passes leaves every camera as it was.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_passes(
    mut commands: Commands,
    frame: Res<FrameRows>,
    edit: Res<Edit>,
    mut images: ResMut<Assets<Image>>,
    mut mask: ResMut<PassMask>,
    main: Query<(Entity, &RenderTarget, Option<&Projection>, Option<&bevy::post_process::dof::DepthOfField>, Option<&ComposeView>, Has<super::super::bevy_api::Untonemapped>, Has<DepthPrepass>, Has<NormalPrepass>), (With<Camera3d>, Without<LookCam>)>,
    composite: Query<(Entity, Option<&ComposeView>), (With<LookCam>, With<Camera3d>, Without<Mesh3d>)>,
    kids: Query<&Children>,
    tags: Query<Option<&MeshTag>, With<Mesh3d>>,
) {
    let Ok((cam, target, proj, dof, have, untm, dp, np)) = main.single() else { return };
    let Some(v) = frame.vals(cam) else { return };
    let ns = names(&v);
    if ns.is_empty() {
        if mask.on {
            commands.entity(cam).remove::<ComposeView>();
            for (c, h) in &composite {
                if h.is_some() {
                    commands.entity(c).remove::<ComposeView>();
                }
            }
            mask.on = false;
        }
        return;
    }
    // each row's pass on its meshes (the row's own, and a model's, which it spawns a few updates in)
    let pass_of = |r: usize| -> Option<f64> {
        match &frame.live {
            Some(rows) => rows.get(r).and_then(|v| v.get("pass")).map(V::num),
            None => frame.cols.num(r, "pass"),
        }
    };
    if frame.live.is_some() || frame.cols.cols.contains_key("pass") {
        for (&e, &r) in &frame.ix {
            let Some(p) = pass_of(r) else { continue };
            let want = (p.round().max(0.0) as u32).min(MOST as u32);
            let mut all = descendants(e, &kids);
            all.push(e);
            for d in all {
                if let Ok(t) = tags.get(d) {
                    if t.map_or(0, |t| t.value) != want {
                        commands.entity(d).insert(MeshTag::new(want));
                    }
                }
            }
        }
    }
    // the mask the size of whatever the camera draws into (the frame, or a look's cells)
    let size = match target {
        RenderTarget::Image(i) => images.get(&i.handle).map(|im| im.size()),
        _ => None,
    }
    .unwrap_or(UVec2::new(edit.prog.width, edit.prog.height));
    if mask.image.as_ref().is_none_or(|(_, s)| *s != size) {
        let mut img = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8Unorm, None);
        img.sampler = bevy::image::ImageSampler::linear();
        mask.image = Some((images.add(img), size));
    }
    mask.on = true;
    let m = mask.image.as_ref().map(|(h, _)| h.clone()).unwrap_or_default();
    // depth of field's lens, as Bevy works out its circle of confusion
    let (mut coc, mut near, mut far, mut bokeh) = (Vec4::ZERO, 0.1, f32::MAX, false);
    if let Some(Projection::Perspective(pp)) = proj {
        near = pp.near;
        if let Some(d) = dof {
            let f = bevy::post_process::dof::calculate_focal_length(d.sensor_height, pp.fov);
            coc = Vec4::new(d.focal_distance, f, f * f / (d.sensor_height * d.aperture_f_stops), d.max_circle_of_confusion_diameter);
            far = d.max_depth.min(f32::MAX);
            bokeh = d.mode == bevy::post_process::dof::DepthOfFieldMode::Bokeh;
        }
    }
    let grade = held(&v, "grade_passes");
    let want = ComposeView { mask: m.clone(), write: true, bloom: held(&v, "bloom_passes"), grade, late: untm, coc, bokeh, near, far };
    if have != Some(&want) {
        commands.entity(cam).insert(want);
    }
    if !dp || !np {
        commands.entity(cam).insert((DepthPrepass, NormalPrepass));
    }
    // the looks' composite (a frame look's vignette is drawn there): the grade's hold, reading the mask
    for (c, h) in &composite {
        let want = ComposeView { mask: m.clone(), write: false, bloom: Vec4::ZERO, grade, late: true, coc: Vec4::ZERO, bokeh: false, near, far };
        if h != Some(&want) {
            commands.entity(c).insert(want);
        }
    }
}

// ------------------------------------------------------------------------------------------------ the render world

#[derive(ShaderType, Clone, Copy)]
pub struct Params {
    pub hold: Vec4,
    pub coc: Vec4,
    pub z: Vec4,
}

#[derive(Resource)]
pub struct Pipelines {
    pub mask_layout: BindGroupLayoutDescriptor,
    pub post_layout: BindGroupLayoutDescriptor,
    pub shader: Handle<Shader>,
    pub fullscreen: VertexState,
    /// by (entry, format)
    pub ids: std::sync::Mutex<bevy::platform::collections::HashMap<(&'static str, TextureFormat), CachedRenderPipelineId>>,
}

impl Pipelines {
    pub fn get<'a>(&self, cache: &'a PipelineCache, entry: &'static str, format: TextureFormat) -> Option<&'a RenderPipeline> {
        let id = *self.ids.lock().unwrap().entry((entry, format)).or_insert_with(|| {
            cache.queue_render_pipeline(RenderPipelineDescriptor {
                label: Some(format!("raea_compose_{entry}").into()),
                layout: vec![if entry.starts_with("mask") { self.mask_layout.clone() } else { self.post_layout.clone() }],
                vertex: self.fullscreen.clone(),
                fragment: Some(FragmentState {
                    shader: self.shader.clone(),
                    entry_point: Some(entry.into()),
                    targets: vec![Some(ColorTargetState { format, blend: None, write_mask: ColorWrites::ALL })],
                    ..default()
                }),
                ..default()
            })
        });
        cache.get_render_pipeline(id)
    }
}

pub fn init(mut commands: Commands, assets: Res<AssetServer>, fullscreen: Res<FullscreenShader>) {
    let nf = || texture_2d(TextureSampleType::Float { filterable: false });
    commands.insert_resource(Pipelines {
        mask_layout: BindGroupLayoutDescriptor::new("raea_compose_mask", &BindGroupLayoutEntries::with_indices(ShaderStages::FRAGMENT, ((0, uniform_buffer::<Params>(false)), (1, nf()), (3, texture_depth_2d())))),
        post_layout: BindGroupLayoutDescriptor::new("raea_compose_post", &BindGroupLayoutEntries::with_indices(ShaderStages::FRAGMENT, ((0, uniform_buffer::<Params>(false)), (1, nf()), (2, nf()), (4, nf())))),
        shader: assets.load("embedded://raea_sys_compose/compose.wesl"),
        fullscreen: fullscreen.to_vertex_state(),
        ids: Default::default(),
    });
}

/// A view's own textures: the mask's half-blurred step and the colour an effect's hold keeps.
#[derive(Component)]
pub struct Kept {
    pub half: CachedTexture,
    pub kept: CachedTexture,
}

pub fn prepare(mut commands: Commands, mut cache: ResMut<TextureCache>, device: Res<RenderDevice>, views: Query<(Entity, &ViewTarget, &ComposeView)>) {
    for (e, target, _) in &views {
        let size = target.main_texture().size();
        let tex = |cache: &mut TextureCache, label: &'static str, format: TextureFormat, usage: TextureUsages| {
            cache.get(&device, TextureDescriptor { label: Some(label), size, mip_level_count: 1, sample_count: 1, dimension: TextureDimension::D2, format, usage, view_formats: &[] })
        };
        let half = tex(&mut cache, "raea_compose_half", TextureFormat::Rgba8Unorm, TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING);
        let kept = tex(&mut cache, "raea_compose_kept", target.main_texture_format(), TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING);
        commands.entity(e).insert(Kept { half, kept });
    }
}

pub fn params(device: &RenderDevice, queue: &RenderQueue, p: Params) -> UniformBuffer<Params> {
    let mut b = UniformBuffer::from(p);
    b.write_buffer(device, queue);
    b
}

pub fn draw(ctx: &mut RenderContext, label: &'static str, pipeline: &RenderPipeline, group: &BindGroup, to: &TextureView) {
    let mut pass = ctx.command_encoder().begin_render_pass(&RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(RenderPassColorAttachment { view: to, depth_slice: None, resolve_target: None, ops: Operations::default() })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, group, &[]);
    pass.draw(0..3, 0..1);
}
