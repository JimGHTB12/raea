//! The process's paint device as Bevy's renderer, and the GPU frame pool's budget.

/// Bevy's renderer on the process's shared device (raea_gpu_device), so the pages and the 3D share one GPU device
/// (One renderer: one device, one frame graph); its own device where RAEA_SHARED_DEVICE=0.
pub fn render_creation() -> bevy::render::settings::RenderCreation {
    use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue};
    match raea_gpu_device::enabled().then(raea_gpu_device::device) {
        Some(Ok((instance, adapter, device, queue))) => {
            let info = adapter.get_info();
            bevy::render::settings::RenderCreation::manual(
                RenderDevice::from(device),
                // Bevy 0.20: WgpuWrapper is gone; the wrappers take the wgpu types through new
                RenderQueue::new(queue),
                RenderAdapterInfo::new(info),
                RenderAdapter::new(adapter),
                RenderInstance::new(instance),
            )
        }
        Some(Err(e)) => {
            eprintln!("raea: the shared GPU device isn't there ({e}); Bevy makes its own");
            Default::default()
        }
        None => Default::default(),
    }
}

/// Set by a render that walks its frames in order (raea-render's render): the pool's budget drops to 32 MiB.
pub static LINEAR_RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The GPU frame pool's budget: RAEA_POOL_MB, else 32 MiB for a render that walks its frames in order (it reuses
/// almost nothing: the UGC edit uploads 929 frames at every budget), else 1 GiB on the Mac and 256 MiB on phones (iOS,
/// Android) for scrubbing, where it sits beside the app. Shared with gpuview.
pub fn pool_budget() -> usize {
    static ENV: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    if let Some(mb) = *ENV.get_or_init(|| std::env::var("RAEA_POOL_MB").ok().and_then(|v| v.parse::<usize>().ok())) {
        return mb << 20;
    }
    if LINEAR_RENDER.load(std::sync::atomic::Ordering::Relaxed) {
        return 32 << 20;
    }
    (if cfg!(any(target_os = "ios", target_os = "android")) { 256 } else { 1024 }) << 20
}
