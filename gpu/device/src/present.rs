//! Windows a picture is presented on (live play: `raea play`, a game's player, the phone, void's stage layer): a
//! surface on the process's shared device (shared.rs) where a texture the shared device holds (the bevy_ui page's) is
//! drawn fitted whole and centred, one draw, no readback. What engine/html's quads stage did for its pages, for any
//! texture.

use std::sync::Mutex;

/// When a window's frame last got its drawable (`Present::show`): what it draws is recorded from then on.
pub static LAST_ACQUIRE: Mutex<Option<std::time::Instant>> = Mutex::new(None);
/// When a window's frame was last presented: its drawable handed to the display.
pub static LAST_PRESENT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// The next frame presented on any window is copied out for `take_present` (a check of what a layer was given).
static WANT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static TAKEN: Mutex<Option<(u32, u32, Vec<u8>)>> = Mutex::new(None);

/// The next frame presented is copied out (RGBA as the layer got it, premultiplied, the surface's size).
pub fn read_next_present() {
    WANT.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// The frame `read_next_present` asked for, once presented.
pub fn take_present() -> Option<(u32, u32, Vec<u8>)> {
    TAKEN.lock().unwrap_or_else(|e| e.into_inner()).take()
}

/// A window's surface on the shared device, or none (offscreen: nothing shown, as a harness without a window runs).
pub struct Present {
    /// a drawable taken ahead (`hold_next`): declared before the surface so it's dropped first (a drawable dropped after
    /// its surface had wgpu discard it on a surface gone)
    held: Mutex<Option<wgpu::SurfaceTexture>>,
    surface: Option<wgpu::Surface<'static>>,
    format: wgpu::TextureFormat,
    pub w: u32,
    pub h: u32,
    pub vsync: bool,
    /// see-through where the picture has nothing: the layer composites what's under it there
    pub transparent: bool,
    blit: Option<Blit>,
}

/// The draw: the picture sampled into the fitted rectangle, premultiplied over the clear.
struct Blit {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uv: wgpu::Buffer,
}

const SHADER: &str = r#"
@group(0) @binding(0) var pic: texture_2d<f32>;
@group(0) @binding(1) var smp: sampler;
@group(0) @binding(2) var<uniform> uv_max: vec4<f32>;
struct V { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) i: u32) -> V {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: V;
    o.p = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    o.uv = uv * uv_max.xy;
    return o;
}
@fragment fn fs(v: V) -> @location(0) vec4<f32> {
    return textureSample(pic, smp, v.uv);
}
"#;

fn handles() -> Result<crate::Handles, String> {
    crate::device()
}

/// A window's presenter (anything wgpu takes as a surface target: a winit window).
pub fn window(target: impl Into<wgpu::SurfaceTarget<'static>>, w: u32, h: u32, vsync: bool) -> Result<Present, String> {
    let (instance, ..) = handles()?;
    let surface = instance.create_surface(target).map_err(|e| format!("surface: {e}"))?;
    Present::over(Some(surface), w, h, vsync)
}

/// A presenter over a raw layer (void's CAMetalLayer in its window).
///
/// # Safety
/// The target must outlive the presenter.
pub unsafe fn raw(target: wgpu::SurfaceTargetUnsafe, w: u32, h: u32, vsync: bool) -> Result<Present, String> {
    let (instance, ..) = handles()?;
    let surface = unsafe { instance.create_surface_unsafe(target) }.map_err(|e| format!("surface: {e}"))?;
    Present::over(Some(surface), w, h, vsync)
}

/// A presenter with no window: the picture is drawn and nothing is shown.
pub fn offscreen(w: u32, h: u32) -> Result<Present, String> {
    handles()?;
    Ok(Present { held: Mutex::new(None), surface: None, format: wgpu::TextureFormat::Rgba8UnormSrgb, w, h, vsync: false, transparent: false, blit: None })
}

/// A presenter over a CAMetalLayer of its own, in no window: presents as the legacy app's stage does (a drawable a frame), for
/// a harness to check what a layer is given without taking anyone's screen.
#[cfg(target_vendor = "apple")]
pub fn layer(w: u32, h: u32) -> Result<Present, String> {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    unsafe {
        let layer: *mut AnyObject = msg_send![class!(CAMetalLayer), layer];
        let _: *mut AnyObject = msg_send![layer, retain];
        let _: () = msg_send![layer, setOpaque: true];
        raw(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer as *mut std::ffi::c_void), w, h, true)
    }
}

/// A window's own background colour, set natively (no shader, nothing compiled): what shows in it before anything is
/// presented, as a game's player shows while its game opens. sRGB 0-1. A no-op off macOS.
pub fn window_background(window: &impl wgpu::rwh::HasWindowHandle, rgb: [f64; 3]) {
    #[cfg(target_vendor = "apple")]
    {
        use objc2::runtime::AnyObject;
        use objc2::{class, msg_send};
        let Ok(h) = window.window_handle() else { return };
        let wgpu::rwh::RawWindowHandle::AppKit(a) = h.as_raw() else { return };
        unsafe {
            let view = a.ns_view.as_ptr() as *mut AnyObject;
            let win: *mut AnyObject = msg_send![view, window];
            if win.is_null() {
                return;
            }
            let c: *mut AnyObject = msg_send![class!(NSColor), colorWithSRGBRed: rgb[0], green: rgb[1], blue: rgb[2], alpha: 1.0f64];
            let _: () = msg_send![win, setBackgroundColor: c];
        }
    }
    #[cfg(not(target_vendor = "apple"))]
    let _ = (window, rgb);
}

impl Present {
    fn over(surface: Option<wgpu::Surface<'static>>, w: u32, h: u32, vsync: bool) -> Result<Present, String> {
        let (_, adapter, ..) = handles()?;
        let format = match &surface {
            Some(s) => {
                let caps = s.get_capabilities(&adapter);
                caps.formats.iter().copied().find(|f| f.is_srgb()).or_else(|| caps.formats.first().copied()).ok_or("the surface takes no format")?
            }
            None => wgpu::TextureFormat::Rgba8UnormSrgb,
        };
        let mut p = Present { held: Mutex::new(None), surface, format, w, h, vsync, transparent: false, blit: None };
        p.configure()?;
        Ok(p)
    }

    fn configure(&mut self) -> Result<(), String> {
        let Some(surface) = &self.surface else { return Ok(()) };
        let (_, adapter, device, _) = handles()?;
        let caps = surface.get_capabilities(&adapter);
        let alpha = if self.transparent {
            [wgpu::CompositeAlphaMode::PreMultiplied, wgpu::CompositeAlphaMode::PostMultiplied].into_iter().find(|m| caps.alpha_modes.contains(m)).unwrap_or(caps.alpha_modes[0])
        } else if caps.alpha_modes.contains(&wgpu::CompositeAlphaMode::Opaque) {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            caps.alpha_modes[0]
        };
        let copyable = caps.usages.contains(wgpu::TextureUsages::COPY_SRC);
        surface.configure(&device, &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | if copyable { wgpu::TextureUsages::COPY_SRC } else { wgpu::TextureUsages::empty() },
            format: self.format,
            width: self.w.max(1),
            height: self.h.max(1),
            present_mode: if self.vsync { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::AutoNoVsync },
            desired_maximum_frame_latency: 1,
            alpha_mode: alpha,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        });
        Ok(())
    }

    /// See-through where the picture has nothing (a window's chrome over a stage under it).
    /// Whether frames go to a surface (a window or a layer); an offscreen present shows nothing.
    pub fn has_surface(&self) -> bool {
        self.surface.is_some()
    }

    pub fn set_transparent(&mut self, on: bool) -> Result<(), String> {
        self.transparent = on;
        self.held.lock().unwrap_or_else(|e| e.into_inner()).take();
        self.configure()
    }

    /// The next drawable taken now, for the next frame (a low-latency window): taking one waits until the display frees
    /// one, so it's done after a present, when the wait costs nothing. Nothing when one is held already, or the window
    /// gives none (hidden).
    pub fn hold_next(&self) {
        let Some(surface) = &self.surface else { return };
        let mut held = self.held.lock().unwrap_or_else(|e| e.into_inner());
        if held.is_some() {
            return;
        }
        if let wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) = surface.get_current_texture() {
            *held = Some(t);
        }
    }

    /// The window's new size in pixels.
    pub fn resize(&mut self, w: u32, h: u32) -> Result<(), String> {
        self.held.lock().unwrap_or_else(|e| e.into_inner()).take();
        (self.w, self.h) = (w, h);
        self.configure()
    }

    fn blit(&mut self) -> Result<&Blit, String> {
        if self.blit.is_none() {
            let (_, _, device, _) = handles()?;
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("present"), source: wgpu::ShaderSource::Wgsl(SHADER.into()) });
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("present"),
                entries: &[
                    wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false }, count: None },
                    wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                    wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                ],
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("present"), bind_group_layouts: &[Some(&layout)], immediate_size: 0 });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("present"),
                layout: Some(&pl),
                vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), buffers: &[], compilation_options: Default::default() },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    // the picture is premultiplied (bevy_ui blends over a transparent clear)
                    targets: &[Some(wgpu::ColorTargetState { format: self.format, blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor { label: Some("present"), mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default() });
            let uv = device.create_buffer(&wgpu::BufferDescriptor { label: Some("present uv"), size: 16, usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            self.blit = Some(Blit { pipeline, layout, sampler, uv });
        }
        Ok(self.blit.as_ref().unwrap())
    }

    /// The top left `w` x `h` of `pic` (a texture on the shared device, premultiplied) drawn fitted whole and centred
    /// over black (or clear, when transparent) and presented. True when a window's layer took the frame (false
    /// offscreen, or when the window gave no drawable: hidden or covered).
    pub fn show(&mut self, pic: &wgpu::Texture, w: u32, h: u32) -> Result<bool, String> {
        if self.surface.is_none() {
            return Ok(false);
        }
        let ta = std::time::Instant::now();
        let held = self.held.lock().unwrap_or_else(|e| e.into_inner()).take();
        let frame = match held {
            Some(t) => t,
            None => match self.surface.as_ref().unwrap().get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
                wgpu::CurrentSurfaceTexture::Occluded | wgpu::CurrentSurfaceTexture::Timeout => return Ok(false),
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    self.configure()?;
                    return Ok(false);
                }
                other => return Err(format!("the surface gave no frame: {other:?}")),
            },
        };
        *LAST_ACQUIRE.lock().unwrap_or_else(|e| e.into_inner()) = Some(ta);
        let (sw, sh) = (frame.texture.width(), frame.texture.height());
        let (_, _, device, queue) = handles()?;
        let transparent = self.transparent;
        let b = self.blit()?;
        queue.write_buffer(&b.uv, 0, bytemuck::cast_slice(&[w as f32 / pic.width().max(1) as f32, h as f32 / pic.height().max(1) as f32, 0.0, 0.0]));
        // sampled as sRGB, so the blend is in linear light and the surface encodes it again
        let view = pic.create_view(&wgpu::TextureViewDescriptor { format: Some(pic.format().add_srgb_suffix()), ..Default::default() });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present"),
            layout: &b.layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&b.sampler) },
                wgpu::BindGroupEntry { binding: 2, resource: b.uv.as_entire_binding() },
            ],
        });
        let target = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let k = (sw as f32 / w.max(1) as f32).min(sh as f32 / h.max(1) as f32);
        let (fw, fh) = ((w as f32 * k).round().max(1.0), (h as f32 * k).round().max(1.0));
        let (fx, fy) = (((sw as f32 - fw) / 2.0).floor(), ((sh as f32 - fh) / 2.0).floor());
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("present") });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("present"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(if transparent { wgpu::Color::TRANSPARENT } else { wgpu::Color::BLACK }), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_viewport(fx, fy, fw, fh, 0.0, 1.0);
            pass.set_pipeline(&b.pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.draw(0..3, 0..1);
        }
        let want = WANT.swap(false, std::sync::atomic::Ordering::SeqCst) && frame.texture.usage().contains(wgpu::TextureUsages::COPY_SRC);
        let copy = want.then(|| {
            let row = (sw * 4).div_ceil(256) * 256;
            let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("present copy"), size: (row * sh) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo { texture: &frame.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(sh) } },
                wgpu::Extent3d { width: sw, height: sh, depth_or_array_layers: 1 },
            );
            (buf, row)
        });
        queue.submit([enc.finish()]);
        queue.present(frame);
        *LAST_PRESENT.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::time::Instant::now());
        if let Some((buf, row)) = copy {
            let (tx, rx) = std::sync::mpsc::channel();
            buf.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            let _ = device.poll(wgpu::PollType::Wait { submission_index: None, timeout: Some(std::time::Duration::from_secs(10)) });
            if let Ok(Ok(())) = rx.recv() {
                let Ok(data) = buf.slice(..).get_mapped_range() else { return Ok(true) };
                let bgra = matches!(self.format, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);
                let mut px = Vec::with_capacity((sw * sh * 4) as usize);
                for y in 0..sh as usize {
                    for p in data[y * row as usize..y * row as usize + sw as usize * 4].chunks_exact(4) {
                        px.extend_from_slice(&if bgra { [p[2], p[1], p[0], p[3]] } else { [p[0], p[1], p[2], p[3]] });
                    }
                }
                *TAKEN.lock().unwrap_or_else(|e| e.into_inner()) = Some((sw, sh, px));
            }
        }
        Ok(true)
    }
}
