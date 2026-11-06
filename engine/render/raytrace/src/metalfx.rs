//! RT4 (docs/raytracing.feature): MetalFX's temporal denoiser on Solari's lighting, on the Mac. No denoiser of Bevy's
//! runs on Metal (Solari's is DLSS Ray Reconstruction, NVIDIA's), so Solari draws its few rays a pixel as noise there;
//! MTLFXTemporalDenoisedScaler (macOS 26) takes that noisy colour and the frame's guide data and gives it back clean.
//!
//! Each frame, after Solari has lit the view and before the opaque pass draws the forward meshes, a compute pass writes the guide textures MetalFX reads from Bevy's G-buffer (diffuse and specular albedo,
//! world normal, roughness); then a command buffer of its own, recorded through wgpu's Metal backend, runs the
//! denoiser on the view's colour, depth and motion and copies its output back over the colour, before the forward
//! meshes and the tonemapping. The two command buffers go to wgpu's queue in that order, after Solari's.
//!
//! Where there's no MetalFX denoiser (not macOS 26, not Metal, a device it doesn't support) the lighting draws as it
//! is and raea says so once. RAEA_SOLARI_DENOISE=0 turns it off, to see the noise.

use super::*;
use bevy::core_pipeline::core_3d::main_opaque_pass_3d;
use bevy::ecs::schedule::Schedules;
use bevy::core_pipeline::prepass::ViewPrepassTextures;
use bevy::core_pipeline::schedule::{Core3d, Core3dSystems};
use bevy::render::render_resource::TextureFormat;
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::view::{ExtractedView, ViewTarget};
use bevy::solari::prelude::SolariLighting;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::Message;
use objc2_metal::{MTLBlitCommandEncoder, MTLCommandBuffer, MTLCommandEncoder, MTLPixelFormat, MTLTexture};
use objc2_metal_fx::{MTLFXTemporalDenoisedScaler, MTLFXTemporalDenoisedScalerBase, MTLFXTemporalDenoisedScalerDescriptor};
use wgpu::hal::api::Metal;

/// After app.finish(), when Solari's plugin has put its lighting system in Core3d: the denoiser right after that system
/// and before the opaque pass, as forward-drawn meshes (a <Character>'s skin, an extended material) come after it with
/// nothing in the G-buffer, and denoising them would smear them. Bevy doesn't export Solari's system, so it's found by
/// name and ordered against by its system type set.
pub(crate) fn add(app: &mut App) {
    if std::env::var("RAEA_SOLARI_DENOISE").is_ok_and(|v| v == "0") {
        return;
    }
    let Some(r) = app.get_sub_app_mut(bevy::render::RenderApp) else { return };
    let mut schedules = r.world_mut().resource_mut::<Schedules>();
    let Some(core) = schedules.get_mut(Core3d) else { return };
    let mut solari = None;
    for (_, s, _) in core.graph().systems.iter() {
        if format!("{}", s.name()).ends_with("solari_lighting") {
            solari = s.default_system_sets().into_iter().next();
            break;
        }
    }
    let Some(solari) = solari else {
        eprintln!("raea: <SolariLighting>: Solari's lighting isn't in the render schedule; no MetalFX denoiser");
        return;
    };
    core.add_systems(denoise.after(solari).before(main_opaque_pass_3d).in_set(Core3dSystems::MainPass));
}

// Bevy's G-buffer (bevy_pbr deferred/functions.wesl, types.wesl) to MetalFX's guides: r = base colour (sRGB-ish, pow
// 1/2.2) and perceptual roughness as unorm4x8; b = reflectance, metallic, occlusion, clearcoat as unorm4x8; a = a 24-bit
// octahedral normal (two 12-bit halves) and flags. Specular albedo is F0 (Bevy's 0.16 reflectance² for dielectrics,
// the base colour for metals)
const DECODE_WGSL: &str = r#"
@group(0) @binding(0) var gbuf: texture_2d<u32>;
@group(0) @binding(1) var diffuse: texture_storage_2d<rgba16float, write>;
@group(0) @binding(2) var specular: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var normal: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var rough: texture_storage_2d<r32float, write>;
fn unorm4x8(p: u32) -> vec4<f32> {
    return vec4<f32>(f32(p & 255u), f32((p >> 8u) & 255u), f32((p >> 16u) & 255u), f32(p >> 24u)) / 255.0;
}
fn octahedral(v: vec2<f32>) -> vec3<f32> {
    let f = v * 2.0 - 1.0;
    var n = vec3<f32>(f, 1.0 - abs(f.x) - abs(f.y));
    let t = saturate(-n.z);
    let w = select(vec2<f32>(t), vec2<f32>(-t), n.xy >= vec2<f32>(0.0));
    n = vec3<f32>(n.xy + w, n.z);
    return normalize(n);
}
@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(diffuse);
    if (id.x >= size.x || id.y >= size.y) { return; }
    let p = vec2<i32>(id.xy);
    let g = textureLoad(gbuf, p, 0);
    let a = unorm4x8(g.r);
    let props = unorm4x8(g.b);
    let base = pow(a.rgb, vec3<f32>(2.2));
    let metallic = props.y;
    let f0 = mix(vec3<f32>(0.16 * props.x * props.x), base, metallic);
    let n = octahedral(vec2<f32>(f32(g.a & 0xFFFu), f32((g.a >> 12u) & 0xFFFu)) / 4095.0);
    textureStore(diffuse, p, vec4<f32>(base * (1.0 - metallic), 1.0));
    textureStore(specular, p, vec4<f32>(f0, 1.0));
    textureStore(normal, p, vec4<f32>(n, 0.0));
    textureStore(rough, p, vec4<f32>(a.w, 0.0, 0.0, 0.0));
}
"#;

/// A view's denoiser and its textures, made again when the view's size changes.
struct View {
    size: UVec2,
    scaler: Retained<ProtocolObject<dyn MTLFXTemporalDenoisedScaler>>,
    diffuse: wgpu::Texture,
    specular: wgpu::Texture,
    normal: wgpu::Texture,
    rough: wgpu::Texture,
    out: wgpu::Texture,
    fresh: bool,
}

#[derive(Default)]
struct Kit {
    pipeline: Option<wgpu::ComputePipeline>,
    views: HashMap<Entity, View>,
    off: bool,
}
// the Metal objects are only touched on the render thread, in this system
unsafe impl Send for Kit {}
unsafe impl Sync for Kit {}

fn texture(dev: &wgpu::Device, size: UVec2, format: wgpu::TextureFormat, usage: wgpu::TextureUsages, label: &str) -> wgpu::Texture {
    dev.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size.x, height: size.y, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

/// A texture's MTLTexture, retained.
fn mtl(t: &wgpu::Texture) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
    unsafe { t.as_hal::<Metal>() }.map(|t| t.raw_handle().retain())
}

impl View {
    fn new(dev: &wgpu::Device, size: UVec2) -> Result<View, String> {
        let hal = unsafe { dev.as_hal::<Metal>() }.ok_or("the GPU isn't on Metal")?;
        let device = hal.raw_device();
        unsafe {
            if !MTLFXTemporalDenoisedScalerDescriptor::supportsDevice(device) {
                return Err("MetalFX has no denoiser for this GPU (it needs macOS 26 and Apple silicon)".into());
            }
            let d = MTLFXTemporalDenoisedScalerDescriptor::new();
            d.setColorTextureFormat(MTLPixelFormat::RGBA16Float);
            d.setDepthTextureFormat(MTLPixelFormat::Depth32Float);
            d.setMotionTextureFormat(MTLPixelFormat::RG16Float);
            d.setDiffuseAlbedoTextureFormat(MTLPixelFormat::RGBA16Float);
            d.setSpecularAlbedoTextureFormat(MTLPixelFormat::RGBA16Float);
            d.setNormalTextureFormat(MTLPixelFormat::RGBA16Float);
            d.setRoughnessTextureFormat(MTLPixelFormat::R32Float);
            d.setOutputTextureFormat(MTLPixelFormat::RGBA16Float);
            d.setInputWidth(size.x as _);
            d.setInputHeight(size.y as _);
            d.setOutputWidth(size.x as _);
            d.setOutputHeight(size.y as _);
            d.setRequiresSynchronousInitialization(true);
            let scaler = d.newTemporalDenoisedScalerWithDevice(device).ok_or("MetalFX made no denoiser")?;
            drop(hal);
            use wgpu::TextureUsages as U;
            let guide = U::STORAGE_BINDING | U::TEXTURE_BINDING;
            Ok(View {
                size,
                scaler,
                diffuse: texture(dev, size, wgpu::TextureFormat::Rgba16Float, guide, "metalfx diffuse albedo"),
                specular: texture(dev, size, wgpu::TextureFormat::Rgba16Float, guide, "metalfx specular albedo"),
                normal: texture(dev, size, wgpu::TextureFormat::Rgba16Float, guide, "metalfx normal"),
                rough: texture(dev, size, wgpu::TextureFormat::R32Float, guide, "metalfx roughness"),
                out: texture(dev, size, wgpu::TextureFormat::Rgba16Float, guide | U::RENDER_ATTACHMENT | U::COPY_SRC, "metalfx output"),
                fresh: true,
            })
        }
    }

    /// The denoiser on `cb`, then its output copied over the colour.
    fn encode(&mut self, cb: &ProtocolObject<dyn MTLCommandBuffer>, color: &wgpu::Texture, depth: &wgpu::Texture, motion: &wgpu::Texture, view: &ExtractedView, reset: bool) -> Result<(), &'static str> {
        let get = |t: &wgpu::Texture| mtl(t).ok_or("a texture isn't a Metal one");
        let (color, depth, motion) = (get(color)?, get(depth)?, get(motion)?);
        let (diffuse, specular, normal, rough, out) = (get(&self.diffuse)?, get(&self.specular)?, get(&self.normal)?, get(&self.rough)?, get(&self.out)?);
        let s = &*self.scaler;
        unsafe {
            s.setColorTexture(Some(&color));
            s.setDepthTexture(Some(&depth));
            s.setMotionTexture(Some(&motion));
            s.setDiffuseAlbedoTexture(Some(&diffuse));
            s.setSpecularAlbedoTexture(Some(&specular));
            s.setNormalTexture(Some(&normal));
            s.setRoughnessTexture(Some(&rough));
            s.setOutputTexture(Some(&out));
            // Bevy's motion is this frame's UV minus the last's; MetalFX's points from this pixel to where it was, in
            // pixels
            s.setMotionVectorScaleX(-(self.size.x as f32));
            s.setMotionVectorScaleY(-(self.size.y as f32));
            s.setJitterOffsetX(0.0);
            s.setJitterOffsetY(0.0);
            s.setDepthReversed(true);
            s.setShouldResetHistory(reset || self.fresh);
            matrices(s, view.world_from_view.to_matrix().inverse(), view.clip_from_view);
            s.encodeToCommandBuffer(cb);
            let blit = cb.blitCommandEncoder().ok_or("no blit encoder")?;
            blit.copyFromTexture_toTexture(&out, &color);
            blit.endEncoding();
        }
        self.fresh = false;
        Ok(())
    }
}

/// worldToViewMatrix and viewToClipMatrix, simd_float4x4 properties objc2-metal-fx leaves out (no simd types in
/// objc2): sent by hand, as four float32x4 columns (an aggregate of four vectors, passed as simd_float4x4 is). Only on
/// Apple silicon, and only where the scaler has them; objc2 checks a message's types in a debug build and doesn't know
/// simd's, so a debug build leaves them out.
#[allow(unused_variables)]
fn matrices(s: &ProtocolObject<dyn MTLFXTemporalDenoisedScaler>, world_to_view: Mat4, view_to_clip: Mat4) {
    #[cfg(all(target_arch = "aarch64", not(debug_assertions)))]
    unsafe {
        use objc2::{msg_send, sel};
        use std::arch::aarch64::float32x4_t;
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Float4x4([float32x4_t; 4]);
        unsafe impl objc2::Encode for Float4x4 {
            const ENCODING: objc2::Encoding = objc2::Encoding::Struct("?", &[]);
        }
        let m = |m: Mat4| Float4x4(m.to_cols_array_2d().map(|c| std::mem::transmute::<[f32; 4], float32x4_t>(c)));
        let has: bool = msg_send![s, respondsToSelector: sel!(setWorldToViewMatrix:)];
        if has {
            let _: () = msg_send![s, setWorldToViewMatrix: m(world_to_view)];
            let _: () = msg_send![s, setViewToClipMatrix: m(view_to_clip)];
        }
    }
}

fn denoise(view: ViewQuery<(&ViewTarget, &ViewPrepassTextures, &ExtractedView, &SolariLighting)>, device: Res<RenderDevice>, mut kit: Local<Kit>, mut ctx: RenderContext) {
    if kit.off {
        return;
    }
    let entity = view.entity();
    let (target, pre, ext, solari) = view.into_inner();
    let (Some(depth), Some(motion), Some(gbuf)) = (pre.depth.as_ref(), pre.motion_vectors.as_ref(), pre.deferred.as_ref()) else { return };
    if target.main_texture_format() != TextureFormat::Rgba16Float {
        return;
    }
    let color: &wgpu::Texture = target.main_texture();
    let size = UVec2::new(color.width(), color.height());
    let dev = device.wgpu_device().clone();
    if kit.views.get(&entity).map(|v| v.size) != Some(size) {
        match View::new(&dev, size) {
            Ok(v) => {
                kit.views.insert(entity, v);
            }
            Err(e) => {
                kit.off = true;
                eprintln!("raea: <SolariLighting>: no MetalFX denoiser ({e}); the lighting draws un-denoised (docs/raytracing.feature RT4)");
                return;
            }
        }
    }
    let pipeline = kit
        .pipeline
        .get_or_insert_with(|| {
            let module = dev.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("metalfx guides"), source: wgpu::ShaderSource::Wgsl(DECODE_WGSL.into()) });
            dev.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { label: Some("metalfx guides"), layout: None, module: &module, entry_point: Some("main"), compilation_options: Default::default(), cache: None })
        })
        .clone();
    let v = kit.views.get_mut(&entity).unwrap();
    let views = [&v.diffuse, &v.specular, &v.normal, &v.rough].map(|t| t.create_view(&Default::default()));
    let mut entries = vec![wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&*gbuf.texture.default_view) }];
    for (i, tv) in views.iter().enumerate() {
        entries.push(wgpu::BindGroupEntry { binding: i as u32 + 1, resource: wgpu::BindingResource::TextureView(tv) });
    }
    let bind = dev.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("metalfx guides"), layout: &pipeline.get_bind_group_layout(0), entries: &entries });
    {
        let mut pass = ctx.command_encoder().begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("metalfx guides"), timestamp_writes: None });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(size.x.div_ceil(8), size.y.div_ceil(8), 1);
    }
    // a command buffer of its own (wgpu won't mix its own commands and raw ones on one encoder), after the guides'
    let mut raw = dev.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("metalfx denoise") });
    let done = unsafe {
        raw.as_hal_mut::<Metal, _, _>(|e| match e.and_then(|e| e.raw_command_buffer()) {
            Some(cb) => v.encode(cb, color, &depth.texture.texture, &motion.texture.texture, ext, solari.reset),
            None => Err("wgpu gave no Metal command buffer"),
        })
    };
    ctx.add_command_buffer(raw.finish());
    if let Err(e) = done {
        kit.off = true;
        eprintln!("raea: <SolariLighting>: the MetalFX denoiser stopped ({e}); the lighting draws un-denoised");
    }
}
