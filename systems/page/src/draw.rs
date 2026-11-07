//! The render world's side: each page's Vello scene drawn into its storage texture with `render_to_texture` on Bevy's
//! device, filtered layers run through raea-sys-filters' passes, views of other textures (H7) copied in, and the result
//! premultiplied into the page's image.

use std::collections::HashMap;

use super::*;

/// The render world's copy of this frame's scenes.
#[derive(Resource, Default)]
pub struct Scenes {
    pub todo: Vec<Drawn>,
}

/// The one Vello renderer on Bevy's device (not `Sync`: its shader cache holds cells), made on first use; each page's
/// storage texture Vello draws into; and the pass that premultiplies it into the page's image.
#[derive(Default)]
pub struct Vello {
    pub renderer: Option<vello::Renderer>,
    pub scratch: HashMap<AssetId<Image>, (wgpu::Texture, UVec2)>,
    pub premul: Option<Premul>,
    /// the filter passes (filters.rs) and the textures they work in
    pub passes: Option<raea_sys_filters::Passes>,
    /// each page's print as last drawn: a page painted the same isn't drawn again
    pub prints: HashMap<AssetId<Image>, u64>,
    /// image overrides not yet given to the renderer (`view`)
    pub pending: Vec<(vello::peniko::ImageData, Option<Placed>)>,
    /// this frame's views of Bevy images (the host page's view3d boxes: view3d.rs), their textures given each frame
    pub image_views: Vec<(vello::peniko::ImageData, Handle<Image>, bool)>,
    /// a premultiplied view's straight copy, by its image: what Vello copies in its place
    pub straightened: HashMap<AssetId<Image>, wgpu::Texture>,
    /// a premultiplied placed texture's straight copy, by its placeholder's blob
    pub straight_placed: HashMap<u64, wgpu::Texture>,
}

/// H7's views (docs/blitz-bevy.feature): wherever a page draws `image`, Vello draws the top left of `texture` instead,
/// copied on the GPU into its atlas each frame it's drawn (a 3D scene, a game, live video); None ends it. `image` is
/// the placeholder an element holds (its size and blob are the key). Both live on the App's device.
pub fn view(app: &mut App, image: vello::peniko::ImageData, texture: Option<wgpu::Texture>) {
    view_at(app, image, texture.map(|t| Placed { texture: t, origin: (0, 0), premultiplied: false }));
}

/// A texture an element shows (`view_at`): its box from `origin` at the placeholder's size, and whether it's a
/// camera's picture, premultiplied in its stored values, made straight on the way as raea's render makes its layered
/// 3D (B5: as raea-runtime-js's page does, its `place_premultiplied`).
pub struct Placed {
    pub texture: wgpu::Texture,
    pub origin: (u32, u32),
    pub premultiplied: bool,
}

/// `view`, the element's picture a box of the texture, straight or made so.
pub fn view_at(app: &mut App, image: vello::peniko::ImageData, placed: Option<Placed>) {
    if let Some(r) = app.get_sub_app_mut(RenderApp) {
        r.world_mut().non_send_mut::<Vello>().pending.push((image, placed));
    }
}

/// Vello stores straight alpha in an `Rgba8Unorm` storage texture (sRGB-encoded values); a page's image is
/// `Rgba8UnormSrgb` and premultiplied. One fullscreen triangle reads the one and writes the other.
pub struct Premul {
    pub pipeline: wgpu::RenderPipeline,
    /// the same pass for a StraightAlpha page: the colour decoded, its alpha kept apart
    pub straight: wgpu::RenderPipeline,
    /// a premultiplied picture (a camera's) made straight for Vello, which takes an element's picture so
    pub unpremul: wgpu::RenderPipeline,
    pub layout: wgpu::BindGroupLayout,
}

pub const PREMUL_WGSL: &str = r"
@group(0) @binding(0) var src: texture_2d<f32>;
@vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = vec2f(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4f(p * 2.0 - 1.0, 0.0, 1.0);
}
fn linear(c: vec3f) -> vec3f {
    return select(pow((c + 0.055) / 1.055, vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}
// premultiplied in the encoded values (as bevy_ui's page holds them); the target's sRGB view encodes what's written
@fragment fn fs(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    return vec4f(linear(c.rgb * c.a), c.a);
}
@fragment fn fs_straight(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    return vec4f(linear(c.rgb), c.a);
}
fn encoded(c: vec3f) -> vec3f {
    return select(1.055 * pow(c, vec3f(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}
// a camera's picture (premultiplied in its stored, sRGB-encoded values) made straight as raea's render makes its
// layered 3D (sdk/react motion.rs scene_straight): its alpha the largest of its four values, so light left in the
// clear (a sky, a glow) keeps; read decoded through an sRGB view, written through one
@fragment fn fs_unpremul(@builtin(position) p: vec4f) -> @location(0) vec4f {
    let c = textureLoad(src, vec2i(p.xy), 0);
    let e = round(encoded(c.rgb) * 255.0);
    let a = max(max(e.r, e.g), max(e.b, round(c.a * 255.0)));
    if c.a >= 1.0 || a == 0.0 {
        return c;
    }
    return vec4f(linear(min(floor((e * 255.0 + floor(a / 2.0)) / a), vec3f(255.0)) / 255.0), a / 255.0);
}
";

impl Premul {
    pub fn new(device: &wgpu::Device) -> Premul {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("raea-page premul"), source: wgpu::ShaderSource::Wgsl(PREMUL_WGSL.into()) });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raea-page premul"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                count: None,
            }],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("raea-page premul"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
        let make = |entry: &str| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("raea-page premul"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), compilation_options: default(), buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(entry),
                compilation_options: default(),
                targets: &[Some(wgpu::ColorTargetState { format: TextureFormat::Rgba8UnormSrgb, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            multiview_mask: None,
            cache: None,
        });
        let (pipeline, straight, unpremul) = (make("fs"), make("fs_straight"), make("fs_unpremul"));
        Premul { pipeline, straight, unpremul, layout }
    }
}

pub fn extract(painted: Extract<Option<Res<Painted>>>, image_views: Extract<Option<Res<view3d::ImageViews>>>, mut scenes: ResMut<Scenes>, mut vello: NonSendMut<Vello>) {
    if let Some(p) = painted.as_ref() {
        scenes.todo = std::mem::take(&mut *p.0.lock().unwrap());
    }
    vello.image_views = image_views.as_ref().map(|v| v.0.clone()).unwrap_or_default();
}

/// A straight copy's texture: sRGB-encoded, as Vello's atlas takes a picture's texels.
pub fn straight_target(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("raea-page view made straight"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// Each scene drawn by Vello into its page's storage texture on Bevy's device, then premultiplied into the page's
/// image, before the frame's passes sample it. No readback: Vello's non-robust path (B1f).
pub fn draw(mut scenes: ResMut<Scenes>, mut vello: NonSendMut<Vello>, device: Res<RenderDevice>, queue: Res<RenderQueue>, images: Res<RenderAssets<GpuImage>>) {
    let todo = std::mem::take(&mut scenes.todo);
    if todo.is_empty() {
        return;
    }
    let device = device.wgpu_device();
    let Vello { renderer, scratch, premul, passes, prints, pending, image_views, straightened, straight_placed } = &mut *vello;
    let premul = premul.get_or_insert_with(|| Premul::new(device));
    // the host's 3D views: each image's texture drawn in place of its placeholder (it changes each frame); a
    // premultiplied one's straight copy, made first
    let mut straighten = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-page views made straight") });
    straightened.retain(|id, _| image_views.iter().any(|v| v.2 && v.1.id() == *id));
    // the premultiplied texture made straight into `out` (its whole size, so a box of it is cut as from the texture)
    let unpremul = |encoder: &mut wgpu::CommandEncoder, src: &wgpu::TextureView, out: &wgpu::Texture| {
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("raea-page view made straight"), layout: &premul.layout, entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(src) }] });
        let view = out.create_view(&default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("raea-page view made straight"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view: &view, depth_slice: None, resolve_target: None, ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store } })],
            ..default()
        });
        pass.set_pipeline(&premul.unpremul);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    };
    let sized = |cache: &mut wgpu::Texture, size: wgpu::Extent3d| {
        if cache.size() != size {
            *cache = straight_target(device, size);
        }
    };
    for (placeholder, handle, premultiplied) in image_views.drain(..) {
        let Some(gpu) = images.get(&handle) else { continue };
        if !premultiplied {
            pending.push((placeholder, Some(Placed { texture: (*gpu.texture).clone(), origin: (0, 0), premultiplied: false })));
            continue;
        }
        let size = gpu.texture.size();
        let out = straightened.entry(handle.id()).or_insert_with(|| straight_target(device, size));
        sized(out, size);
        unpremul(&mut straighten, &gpu.texture_view, out);
        pending.push((placeholder, Some(Placed { texture: out.clone(), origin: (0, 0), premultiplied: false })));
    }
    // placed textures (`view_at`) that are premultiplied: their straight copies, a placeholder gone with its own
    straight_placed.retain(|id, _| pending.iter().any(|(i, p)| i.data.id() == *id && p.as_ref().is_some_and(|p| p.premultiplied)));
    for (placeholder, placed) in pending.iter_mut() {
        let Some(p) = placed.as_mut().filter(|p| p.premultiplied) else { continue };
        let size = p.texture.size();
        let out = straight_placed.entry(placeholder.data.id()).or_insert_with(|| straight_target(device, size));
        sized(out, size);
        unpremul(&mut straighten, &p.texture.create_view(&default()), out);
        p.texture = out.clone();
        p.premultiplied = false;
    }
    queue.submit([straighten.finish()]);
    // a view's texture changes under the same paint: any view placed this frame draws every page again
    let views = !pending.is_empty();
    let renderer = renderer.get_or_insert_with(|| {
        // Vello's bump buffers sized for pages, ~100 MB less GPU memory than its defaults (paris-30k's): lines 1M,
        // tiles, seg_counts and segments 1M, ptcl 2M, blend spill 256K. A scene past one draws without some of its
        // geometry and no error, so P26's corpus is the check (every frame at the defaults' SSIM). RAEA_VELLO_BUMP=full
        // puts Vello's own sizes back; RAEA_VELLO_BUMP=N makes each N bytes, for diagnosis only: past its buffers,
        // non-robust Vello can hang the GPU in the driver (10-26: 4 KB on a 640x360 page left a render stuck exiting
        // until a reboot), so never on a shared machine
        if let Some(k) = std::env::var("RAEA_VELLO_BUMP").ok().and_then(|v| v.parse::<u32>().ok()) {
            vello::low_level::vello_encoding::set_bump_allocation_sizes(vello::low_level::vello_encoding::BumpAllocationSizes { lines: k, bin_data: k, tiles: k, seg_counts: k, segments: k, blend_spill: k, ptcl: k });
        } else if std::env::var("RAEA_VELLO_BUMP").as_deref() != Ok("full") {
            vello::low_level::vello_encoding::set_bump_allocation_sizes(vello::low_level::vello_encoding::BumpAllocationSizes {
                lines: 1 << 20,
                bin_data: 1 << 18,
                tiles: 1 << 20,
                seg_counts: 1 << 20,
                segments: 1 << 20,
                blend_spill: 1 << 18,
                ptcl: 1 << 21,
            });
        }
        // blitz-paint asks for hinted glyph runs, and each face is drawn through where CoreGraphics puts its o and H tops
        // (fonts.rs coregraphics_tops), as Chrome on macOS draws it; RAEA_PAGE_CG_YMAP=0 draws them unhinted
        #[cfg(target_vendor = "apple")]
        if std::env::var("RAEA_PAGE_CG_YMAP").as_deref() != Ok("0") {
            vello::low_level::vello_encoding::set_glyph_tops(raea_sys_fonts::coregraphics_tops);
        }
        vello::Renderer::new(
            device,
            vello::RendererOptions { antialiasing_support: vello::AaSupport::area_only(), num_init_threads: std::num::NonZeroUsize::new(1), ..default() },
        )
        .expect("vello on Bevy's device")
    });
    for (image, placed) in pending.drain(..) {
        let at = placed.map(|p| wgpu::TexelCopyTextureInfoBase { texture: p.texture, mip_level: 0, origin: wgpu::Origin3d { x: p.origin.0, y: p.origin.1, z: 0 }, aspect: wgpu::TextureAspect::All });
        renderer.override_image(&image, at);
    }
    let passes = passes.get_or_insert_with(|| raea_sys_filters::Passes::new(device));
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-page premul") });
    for &(ref handle, px, ref scene, ref layers, print, straight, linear) in &todo {
        let Some(gpu) = images.get(handle) else { continue };
        if !views && !REDRAW_FULL.load(std::sync::atomic::Ordering::Relaxed) && prints.get(&handle.id()) == Some(&print) {
            continue;
        }
        prints.insert(handle.id(), print);
        // a bevy_ui page's renders (its layers' and its own) blend in linear light (vendor/vello_shaders fine.wgsl)
        vello::low_level::vello_encoding::set_blend_linear(linear);
        // each filtered layer drawn by Vello, filtered, and handed to Vello as its placeholder's pixels, an inner
        // layer before the one that holds it (its own render then copies it in)
        for (i, layer) in layers.iter().enumerate() {
            for before in &layers[..i] {
                renderer.mark_override_image_dirty(&before.image);
            }
            let drawn = passes.texture(device, layer.size, TextureFormat::Rgba8Unorm, TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT);
            let mut passes_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-page filters") });
            match &layer.linear {
                // a pattern tile's gradient, drawn exactly (filters.rs Linear)
                Some(l) => passes.linear(device, &mut passes_encoder, l, &drawn),
                None => {
                    if std::env::var_os("RAEA_VELLO_BUMP_USE").is_some() {
                        bump_use("layer", &layer.scene, layer.size.0, layer.size.1);
                    }
                    let params = vello::RenderParams { base_color: vello::peniko::Color::TRANSPARENT, width: layer.size.0, height: layer.size.1, antialiasing_method: vello::AaConfig::Area };
                    if let Err(why) = bump_guard(device, layer.size.0, layer.size.1) {
                        error!("raea-page: {why}");
                        raea_gpu_device::check::refuse(why);
                    } else if let Err(err) = renderer.render_to_texture(device, &queue, &layer.scene, &drawn.create_view(&default()), &params) {
                        error!("raea-page: vello, a filtered layer: {err}");
                    }
                }
            }
            let out = passes.run(device, &mut passes_encoder, layer, &drawn);
            queue.submit([passes_encoder.finish()]);
            let at = wgpu::TexelCopyTextureInfoBase { texture: out, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All };
            renderer.override_image(&layer.image, Some(at));
        }
        let (tex, _) = scratch.entry(handle.id()).and_modify(|(t, at)| {
            if *at != px {
                *t = storage(device, px);
                *at = px;
            }
        }).or_insert_with(|| (storage(device, px), px));
        let view = tex.create_view(&default());
        let params = vello::RenderParams { base_color: vello::peniko::Color::TRANSPARENT, width: px.x, height: px.y, antialiasing_method: vello::AaConfig::Area };
        if std::env::var_os("RAEA_VELLO_BUMP_USE").is_some() {
            bump_use("page", scene, px.x, px.y);
        }
        if let Err(why) = bump_guard(device, px.x, px.y) {
            error!("raea-page: {why}");
            raea_gpu_device::check::refuse(why);
            for layer in layers {
                renderer.override_image(&layer.image, None);
            }
            continue;
        }
        // every layer's image copied in again: Vello's atlas keeps an image a layer's own render already took in
        // (a backdrop's region holds the layers before it) as clean, and the page then drew it blank (10-28)
        for layer in layers {
            renderer.mark_override_image_dirty(&layer.image);
        }
        let drew = renderer.render_to_texture(device, &queue, &scene, &view, &params);
        for layer in layers {
            renderer.override_image(&layer.image, None);
        }
        if std::env::var_os("RAEA_TRACE").is_some() {
            eprintln!("trace raea-page: page drawn {}x{}, full bump {}, {}", px.x, px.y, REDRAW_FULL.load(std::sync::atomic::Ordering::Relaxed), drew.as_ref().map_or_else(|e| e.to_string(), |_| "ok".into()));
        }
        if let Err(err) = drew {
            error!("raea-page: vello: {err}");
            continue;
        }
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("raea-page premul"),
            layout: &premul.layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }],
        });
        let target = gpu.texture.create_view(&wgpu::TextureViewDescriptor { format: Some(TextureFormat::Rgba8UnormSrgb), ..default() });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("raea-page premul"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(if straight { &premul.straight } else { &premul.pipeline });
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..3, 0..1);
    }
    queue.submit([encoder.finish()]);
    vello::low_level::vello_encoding::set_blend_linear(false);
    passes.recycle();
    // the scenes given back for paint to reuse (ZeroHeap)
    let mut spare = SPARE.lock().unwrap();
    for d in todo {
        if spare.len() < SPARE_MAX {
            spare.push(d.2);
        }
    }
}

/// The texture Vello draws a page into: Rgba8Unorm with storage binding, read by the premultiply pass.
pub fn storage(device: &wgpu::Device, px: UVec2) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("raea-page vello"),
        size: Extent3d { width: px.x, height: px.y, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

#[cfg(test)]
mod tests {
    /// The premultiply pass's WGSL parses: a Rust keyword in it (a `pub` from a visibility sweep, def55553) broke every
    /// render's page.
    #[test]
    fn premul_wgsl_parses() {
        wgpu::naga::front::wgsl::parse_str(super::PREMUL_WGSL).unwrap();
    }
}
