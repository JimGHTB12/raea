//! GPU preview (G8): the edit's frames from the GPU pool, without the Bevy frame loop. A frame is its rows (the
//! rules, on the CPU, cached), its clips as NV12 planes in the pool (decoded and uploaded once), its Blitz overlay
//! (rendered once per distinct overlay state, kept on the GPU by digest), and one composite.k dispatch into a resident
//! frame buffer. Scrubbing back to a visited frame is a single dispatch. Edits with 3D content preview through the
//! engine instead (the background here is the camera's clear colour).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use raea_core_program::eval::Tup;
use raea_core_program::ir::{Program, V};

mod composite;
mod render;

type PoolKey = (String, i64);

fn num(v: &std::collections::BTreeMap<String, V>, k: &str, d: f64) -> f64 {
    v.get(k).map(V::num).unwrap_or(d)
}

/// Timings of one preview frame (ms).
#[derive(Debug, Clone, Default)]
pub struct FrameTimes {
    pub rules: f64,
    pub overlay: f64,
    pub decode_upload: f64,
    pub composite_gpu: f64,
    pub total: f64,
    pub uploads: usize,
    pub overlay_cached: bool,
}

/// A frame in the pool: its NV12 planes copied into a buffer, or (zero-copy) the decoder's own buffer as two textures.
enum Pooled {
    Buf(wgpu::Buffer),
    #[cfg(target_vendor = "apple")]
    Surf(zc::Planes),
}

impl Pooled {
    fn bytes(&self) -> usize {
        match self {
            Pooled::Buf(b) => b.size() as usize,
            #[cfg(target_vendor = "apple")]
            Pooled::Surf(p) => p.bytes,
        }
    }
}

/// Zero-copy on Apple: a decoded CVPixelBuffer (NV12, IOSurface-backed) becomes two wgpu textures, its luma plane as
/// R8 and its chroma plane as RG8, through CoreVideo's Metal texture cache. Nothing is copied: the GPU reads the
/// decoder's memory.
#[cfg(target_vendor = "apple")]
mod zc {
    use std::ptr::NonNull;

    use objc2_core_foundation::CFRetained;
    use objc2_core_video::{CVMetalTexture, CVMetalTextureCache, CVMetalTextureGetTexture, CVPixelBuffer};
    use objc2_metal::{MTLPixelFormat, MTLTextureType};

    pub struct Planes {
        pub y: wgpu::TextureView,
        pub uv: wgpu::TextureView,
        pub bytes: usize,
        _textures: (wgpu::Texture, wgpu::Texture),
        _cv: (CFRetained<CVMetalTexture>, CFRetained<CVMetalTexture>),
        _buffer: CFRetained<CVPixelBuffer>,
    }

    pub struct Cache(CFRetained<CVMetalTextureCache>);

    impl Cache {
        /// Let the cache drop textures nobody holds (CVMetalTextureCacheFlush): what the pool evicted goes now.
        pub fn flush(&self) {
            self.0.flush(0);
        }

        pub fn new(device: &wgpu::Device) -> Result<Cache, String> {
            let hal = unsafe { device.as_hal::<wgpu::hal::api::Metal>() }.ok_or("zero-copy: the device isn't Metal")?;
            let mtl = hal.raw_device().clone();
            let mut c: *mut CVMetalTextureCache = std::ptr::null_mut();
            let r = unsafe { CVMetalTextureCache::create(None, None, &mtl, None, NonNull::new(&mut c).unwrap()) };
            if r != 0 || c.is_null() {
                return Err(format!("CVMetalTextureCacheCreate: {r}"));
            }
            Ok(Cache(unsafe { CFRetained::from_raw(NonNull::new(c).unwrap()) }))
        }

        #[allow(clippy::too_many_arguments)]
        fn plane(&self, device: &wgpu::Device, pb: &CVPixelBuffer, plane: usize, w: u32, h: u32, mtl: MTLPixelFormat, format: wgpu::TextureFormat) -> Result<(wgpu::Texture, wgpu::TextureView, CFRetained<CVMetalTexture>), String> {
            let mut t: *mut CVMetalTexture = std::ptr::null_mut();
            let r = unsafe { CVMetalTextureCache::create_texture_from_image(None, &self.0, pb, None, mtl, w as usize, h as usize, plane, NonNull::new(&mut t).unwrap()) };
            if r != 0 || t.is_null() {
                return Err(format!("CVMetalTextureCacheCreateTextureFromImage (plane {plane}): {r}"));
            }
            let cv = unsafe { CFRetained::from_raw(NonNull::new(t).unwrap()) };
            let raw = CVMetalTextureGetTexture(&cv).ok_or("CVMetalTextureGetTexture: none")?;
            let extent = wgpu::hal::CopyExtent { width: w, height: h, depth: 1 };
            let hal_tex = unsafe { wgpu::hal::metal::Device::texture_from_raw(raw, format, MTLTextureType::Type2D, 1, 1, extent, None) };
            let desc = wgpu::TextureDescriptor {
                label: Some("zero-copy plane"),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            };
            let tex = unsafe { device.create_texture_from_hal::<wgpu::hal::api::Metal>(hal_tex, &desc, wgpu::TextureUses::UNINITIALIZED) };
            // (wgpu 30 asks for the tracked state; UNINITIALIZED is what wgpu 29 always assumed)
            let view = tex.create_view(&Default::default());
            Ok((tex, view, cv))
        }

        pub fn wrap(&self, device: &wgpu::Device, buffer: CFRetained<CVPixelBuffer>, w: u32, h: u32) -> Result<Planes, String> {
            let (ty, yv, ycv) = self.plane(device, &buffer, 0, w, h, MTLPixelFormat::R8Unorm, wgpu::TextureFormat::R8Unorm)?;
            let (tu, uv, ucv) = self.plane(device, &buffer, 1, w / 2, h / 2, MTLPixelFormat::RG8Unorm, wgpu::TextureFormat::Rg8Unorm)?;
            Ok(Planes { y: yv, uv, bytes: (w * h * 3 / 2) as usize, _textures: (ty, tu), _cv: (ycv, ucv), _buffer: buffer })
        }
    }
}

pub struct GpuView {
    prog: Program,
    rows: Vec<Tup>,
    dir: PathBuf,
    pub gpu: raea_sys_k::Gpu,
    backend: Box<dyn raea_io_media::Backend>,
    decoders: HashMap<(String, usize), Box<dyn raea_io_media::VideoDecoder>>,
    pool: HashMap<PoolKey, Pooled>,
    pub pool_bytes: usize,
    states: HashMap<i64, Vec<Tup>>,
    /// every frame of the program, evaluated once (built on first use)
    tl: Option<std::sync::Arc<raea_core_program::timeline::Timeline>>,
    rw: crate::rows2d::Rows,
    overlays: HashMap<u64, wgpu::Buffer>,
    /// the next frame's overlay, painting on the GPU while this frame composites: its frame, digest and paint
    ahead: Option<(i64, u64, Vec<u8>)>,
    /// an HTML composition's page (html.rs): the overlay instead of the 2D rows
    page: Option<raea_page::html::HtmlPage>,
    /// the page the caller handed for the next frame, already on this device (render_nv12_fed): the overlay as it is
    fed: Option<wgpu::Buffer>,
    comp: raea_sys_k::Compiled,
    _bg: wgpu::Texture,
    bg_view: wgpu::TextureView,
    pub out: wgpu::Buffer,
    dummy: wgpu::Buffer,
    /// kgpu's mask M when no clip is masked: zeros, w x h words (composite.k reads it for every pixel)
    no_mask: wgpu::Buffer,
    /// the direct render's nv12buf.k and its output, made on first use
    nv12: Option<(raea_sys_k::Compiled, wgpu::Buffer)>,
    /// the direct render hands P010 (p010buf.k: 10-bit BT.709 limited, w x h x 3 bytes) instead of NV12, for a
    /// 10-bit encoder (RenderOpts.ten_bit)
    pub ten_bit: bool,
    /// RAEA_GPU_TIMES: timestamps around the direct render's GPU stages (8 per frame)
    ts: Option<wgpu::QuerySet>,
    /// uploads timed this frame (query pairs from 8)
    ts_uploads: u32,
    /// CPU ms spent handing uploads to the queue (the staging copy), summed
    pub upload_cpu_ms: f64,
    /// RAEA_ZEROCOPY (Apple): decoded frames stay in VideoToolbox's buffers, wrapped as textures; composite_tex.k
    #[cfg(target_vendor = "apple")]
    zc: Option<(zc::Cache, raea_sys_k::Compiled, wgpu::TextureView, wgpu::TextureView, (wgpu::Texture, wgpu::Texture))>,
    pub w: u32,
    pub h: u32,
    /// when each pooled frame was last used, for the budget
    pool_used: HashMap<PoolKey, u64>,
    tick: u64,
}

const COMPOSITE_K: &str = include_str!("../../../core/k/kernels/composite.k");
const NV12BUF_K: &str = include_str!("../../../core/k/kernels/nv12buf.k");
const P010BUF_K: &str = include_str!("../../../core/k/kernels/p010buf.k");
#[cfg(target_vendor = "apple")]
const COMPOSITE_TEX_K: &str = include_str!("../../../core/k/kernels/composite_tex.k");

/// The overlay key of a fed page (no 2D state's digest is all ones in practice; a page's key is rotated).
const FED: u64 = u64::MAX;

/// RAEA_FRAME_LOG (raea's fed render): each frame's wait for the GPU (ms), kept when the caller puts a Vec here
pub static FRAME_WAITS: std::sync::Mutex<Option<Vec<(i64, f64)>>> = std::sync::Mutex::new(None);

/// The process's shared device (raea_gpu_device) as a kernel device, so the direct render composites pages on the
/// device that painted them (One renderer: one device); None where RAEA_SHARED_DEVICE=0.
pub fn paint_device() -> Option<raea_sys_k::Gpu> {
    if !raea_gpu_device::enabled() {
        return None;
    }
    let (_, adapter, device, queue) = raea_gpu_device::device().ok()?;
    let info = adapter.get_info();
    Some(raea_sys_k::Gpu { device, queue, adapter: format!("{} ({:?})", info.name, info.backend) })
}

impl GpuView {
    pub fn new(prog: &Program, rows: &[Tup], dir: &Path) -> Result<GpuView, String> {
        let (w, h) = (prog.width, prog.height);
        let gpu = paint_device().map_or_else(raea_sys_k::Gpu::new, Ok)?;
        // the 2D rows, as the engine installs them
        let rows2d: Vec<serde_json::Value> = rows.iter().filter(|t| crate::rows::is_2d(&t.kind)).map(|t| crate::rows::row2d(t, rows).1).collect();
        let rw = crate::rows2d::Rows::new(&rows2d, w, h, prog.frames.max(1), dir)?;
        // the background: the camera's clear colour (the engine's camera clears to it)
        let bg = rows.iter().find(|t| t.kind == "camera").and_then(|t| match t.vals.get("bg") {
            Some(V::S(s)) => Some(s.clone()),
            _ => None,
        });
        let rgb = bg
            .and_then(|s| {
                let h = s.trim_start_matches('#').to_string();
                let b = |i: usize| u8::from_str_radix(h.get(i..i + 2)?, 16).ok();
                Some([b(0)?, b(2)?, b(4)?])
            })
            .unwrap_or([13, 10, 26]);
        let texels: Vec<u8> = (0..(w * (h + 1)) as usize).flat_map(|_| [rgb[0], rgb[1], rgb[2], 255]).collect();
        let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: w, height: h + 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            &texels,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(w * 4), rows_per_image: None },
            wgpu::Extent3d { width: w, height: h + 1, depth_or_array_layers: 1 },
        );
        let bg_view = tex.create_view(&wgpu::TextureViewDescriptor { format: Some(wgpu::TextureFormat::Rgba8Unorm), ..Default::default() });
        let k = raea_core_k::parse(COMPOSITE_K)?;
        let sc: HashMap<&str, i32> = [("w", w as i32), ("h", h as i32), ("T_w", w as i32)].into();
        let comp = gpu.compile_tex(&k, &["P0", "P1", "P2", "P3", "O", "C", "M"], &["T"], &sc, &["px"])?;
        let no_mask = gpu.buffer((w * h * 4) as usize, Some(&vec![0u8; (w * h * 4) as usize]));
        let out = gpu.buffer((w * h * 4) as usize, None);
        let dummy = gpu.buffer(24, Some(&[0u8; 24]));
        let backend = raea_io_media::backend(&raea_sys_frame::media_backend())?;
        let mut v = GpuView {
            prog: prog.clone(),
            rows: rows.to_vec(),
            dir: dir.to_path_buf(),
            gpu,
            backend,
            decoders: HashMap::new(),
            pool: HashMap::new(),
            pool_bytes: 0,
            states: HashMap::new(),
            tl: None,
            rw,
            overlays: HashMap::new(),
            ahead: None,
            page: None,
            fed: None,
            comp,
            _bg: tex,
            bg_view,
            out,
            dummy,
            no_mask,
            nv12: None,
            ten_bit: false,
            ts: None,
            ts_uploads: 0,
            upload_cpu_ms: 0.0,
            #[cfg(target_vendor = "apple")]
            zc: None,
            w,
            h,
            pool_used: HashMap::new(),
            tick: 0,
        };
        // RAEA_ZEROCOPY=1 (Apple): frames stay in VideoToolbox's buffers, read as textures by composite_tex.k
        #[cfg(target_vendor = "apple")]
        if std::env::var_os("RAEA_ZEROCOPY").is_some() {
            let cache = zc::Cache::new(&v.gpu.device)?;
            let k = raea_core_k::parse(COMPOSITE_TEX_K)?;
            let sc: HashMap<&str, i32> = [("w", v.w as i32), ("h", v.h as i32), ("T_w", v.w as i32)].into();
            let ck = v.gpu.compile_tex(&k, &["O", "C", "M"], &["T", "Y0", "Y1", "Y2", "Y3", "U0", "U1", "U2", "U3"], &sc, &["px"])?;
            let one = |format: wgpu::TextureFormat| {
                let t = v.gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let view = t.create_view(&Default::default());
                (t, view)
            };
            let (ty, dy) = one(wgpu::TextureFormat::R8Unorm);
            let (tuv, duv) = one(wgpu::TextureFormat::Rg8Unorm);
            v.zc = Some((cache, ck, dy, duv, (ty, tuv)));
        }
        Ok(v)
    }

    /// The shared timeline of the current program (built once per program).
    fn timeline(&mut self) -> Result<&raea_core_program::timeline::Timeline, String> {
        if self.tl.is_none() {
            self.tl = Some(std::sync::Arc::new(raea_core_program::timeline::Timeline::new(&self.prog, &self.rows)?));
        }
        Ok(self.tl.as_ref().unwrap())
    }

    /// Use a timeline already evaluated for this program (shared with the sound).
    pub fn share_timeline(&mut self, tl: std::sync::Arc<raea_core_program::timeline::Timeline>) {
        self.tl = Some(tl);
        self.states.clear();
    }

    pub fn state(&mut self, f: i64) -> Result<&Vec<Tup>, String> {
        if !self.states.contains_key(&f) {
            let st = self.timeline()?.tups(f);
            self.states.insert(f, st);
        }
        Ok(&self.states[&f])
    }

    /// The overlay's digest at frame f: every 2D row's values and whether it shows (as the engine's overlay memo).
    /// The page as the overlay (html.rs).
    pub fn set_page(&mut self, page: raea_page::html::HtmlPage) {
        self.page = Some(page);
    }

    /// What the overlay shows at frame f: the page's state when there's a page, else the 2D rows'.
    fn overlay_key(&mut self, st: &[Tup], f: i64) -> u64 {
        // a page's frames are drawn each (its scripts' state isn't a key)
        match self.page {
            Some(_) => (f as u64 ^ 0x9e37_79b9_7f4a_7c15).rotate_left(1),
            None => Self::overlay_digest(st, f),
        }
    }

    pub(crate) fn overlay_digest(st: &[Tup], f: i64) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut rows: Vec<&Tup> = st.iter().filter(|t| crate::rows::is_2d(&t.kind)).collect();
        rows.sort_by(|a, b| (&a.kind, &a.id).cmp(&(&b.kind, &b.id)));
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for t in rows {
            let v = &t.vals;
            let on = f >= num(v, "from", f64::MIN) as i64 && v.get("to").map(|x| f < x.num() as i64).unwrap_or(true);
            (&t.kind, &t.id, on).hash(&mut h);
            if on {
                format!("{v:?}").hash(&mut h);
            }
        }
        h.finish()
    }

    /// Frame f into `out` (stays on the GPU). Returns where the time went.
    /// Data up to a new GPU buffer. Timed (RAEA_GPU_TIMES), it goes through a mapped staging buffer and an explicit
    /// copy between two timestamps, which is what the queue's write does underneath.
    fn upload(&mut self, data: &[u8]) -> wgpu::Buffer {
        let Some(q) = self.ts.as_ref().filter(|_| self.ts_uploads < 4) else {
            let t = Instant::now();
            let b = self.gpu.buffer(data.len(), Some(data));
            self.upload_cpu_ms += t.elapsed().as_secs_f64() * 1e3;
            return b;
        };
        let size = (data.len().max(4) as u64).div_ceil(4) * 4;
        let b = self.gpu.buffer(size as usize, None);
        let staging = self.gpu.device.create_buffer(&wgpu::BufferDescriptor { label: None, size, usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: true });
        staging.slice(..).get_mapped_range_mut().expect("a mapped buffer").slice(..data.len()).copy_from_slice(data);
        staging.unmap();
        let i = 8 + 2 * self.ts_uploads;
        self.ts_uploads += 1;
        let mut enc = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("raea-render frame") });
        enc.write_timestamp(q, i);
        enc.copy_buffer_to_buffer(&staging, 0, &b, 0, size);
        enc.write_timestamp(q, i + 1);
        self.gpu.queue.submit([enc.finish()]);
        raea_gpu_device::submitted(&self.gpu.queue, "render", "raea-render frame");
        b
    }

}

/// The direct render's time, ms summed over its frames.
#[derive(Default, Debug)]
pub struct DirectTimes {
    pub frames: usize,
    pub rules: f64,
    pub overlay: f64,
    pub decode_upload: f64,
    pub uploads: usize,
    /// composite already submitted; nv12buf.k, the copy into the ring and the map, recorded and submitted
    pub submit: f64,
    /// waiting for the oldest frame in flight
    pub wait: f64,
    /// out of the mapped ring buffer
    pub copy: f64,
    /// the sink (the encoder, or hashing)
    pub sink: f64,
    /// the caller making each frame's page (render_nv12_fed: React, layout and the paint's recording)
    pub fed: f64,
    pub total: f64,
    /// RAEA_GPU_TIMES, GPU ms from timestamps: the uploads (clip planes, a new overlay: each a copy from a staging
    /// buffer), composite.k, nv12buf.k, the copy into the readback ring, and their sum
    pub gpu_upload: f64,
    pub gpu_composite: f64,
    pub gpu_nv12: f64,
    pub gpu_copy: f64,
    pub gpu_total: f64,
    pub gpu_timed: bool,
}

impl DirectTimes {
    pub fn json(&self) -> serde_json::Value {
        let n = self.frames.max(1) as f64;
        serde_json::json!({"frames": self.frames, "fps": n / (self.total / 1e3), "ms_per_frame": {"rules": self.rules / n, "overlay": self.overlay / n,
            "decode_upload": self.decode_upload / n, "submit": self.submit / n, "wait": self.wait / n, "copy": self.copy / n, "sink": self.sink / n,
            "fed": self.fed / n, "wall": self.total / n}, "uploads": self.uploads,
            "gpu_ms_per_frame": if self.gpu_timed { serde_json::json!({"upload": self.gpu_upload / n, "composite": self.gpu_composite / n, "nv12": self.gpu_nv12 / n, "copy": self.gpu_copy / n, "total": self.gpu_total / n}) } else { serde_json::Value::Null }})
    }
}

impl FrameTimes {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({"rules": self.rules, "overlay": self.overlay, "decode_upload": self.decode_upload, "composite_gpu": self.composite_gpu,
            "total": self.total, "uploads": self.uploads, "overlay_cached": self.overlay_cached})
    }
}
