//! The process's one shared GPU device (One renderer: one device, one frame graph): Bevy's renderer (raea-render's engine),
//! the kernels of the direct render (raea-render's gpuview) and the stage's present (present.rs) all run on it, so a
//! texture one of them draws is one the others can read. Made once, on first use, as Bevy's renderer asks for its own
//! device (its Functionality priority: every feature the adapter has, less mappable primary buffers on a
//! discrete GPU). RAEA_SHARED_DEVICE=0: callers make their
//! own devices instead (`enabled`).

use std::sync::OnceLock;

/// what a page's Vello scene uses of each bump buffer, counted on the CPU (RAEA_VELLO_BUMP_USE)
pub mod bump;
/// whether the GPU still runs our work, and a black frame checked before it's kept
pub mod check;
/// windows a texture on the shared device is presented on (live play)
pub mod present;
mod telemetry;

pub use telemetry::*;

/// What the shared device is: the instance and adapter it came from, the device and its queue.
pub type Handles = (wgpu::Instance, wgpu::Adapter, wgpu::Device, wgpu::Queue);

/// Whether callers share the device (on unless RAEA_SHARED_DEVICE=0).
pub fn enabled() -> bool {
    std::env::var("RAEA_SHARED_DEVICE").as_deref() != Ok("0")
}

/// The shared device, made on first call; every later call gets the same one (or the same error).
pub fn device() -> Result<Handles, String> {
    static SHARED: OnceLock<Result<Handles, String>> = OnceLock::new();
    SHARED.get_or_init(make).clone()
}

fn make() -> Result<Handles, String> {
    // wgpu's indirect-draw validation off in release (it stays in debug builds; RAEA_WGPU_INDIRECT_VALIDATION=1 puts it
    // back): it checks each indirect draw's arguments in compute passes before drawing, and the indirect draws are
    // Bevy's own, written by its mesh preprocessing on this device. On arena3d it was ~7 extra passes and their blits a
    // frame, ~0.07 ms of GPU (lightspeed F2); pixels are the same.
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    if !cfg!(debug_assertions) && std::env::var("RAEA_WGPU_INDIRECT_VALIDATION").as_deref() != Ok("1") {
        desc.flags.remove(wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL);
    }
    let instance = wgpu::Instance::new(desc);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() }))
        .map_err(|e| format!("no GPU adapter: {e}"))?;
    let mut features = adapter.features();
    if adapter.get_info().device_type == wgpu::DeviceType::DiscreteGpu {
        features.remove(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
    }
    let limits = adapter.limits();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("raea shared"),
        required_features: features,
        required_limits: limits,
        // as Bevy's own request does (bevy_render renderer::initialize_renderer)
        experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
        // RAEA_WGPU_TRACE=DIR: every call on the device written to DIR/trace.ron, for tools/readbacks.sh to count
        // the copies the CPU reads back (docs/blitz-bevy.feature B1d)
        trace: std::env::var_os("RAEA_WGPU_TRACE").map_or(wgpu::Trace::Off, |d| {
            let _ = std::fs::create_dir_all(&d);
            wgpu::Trace::Directory(d.into())
        }),
        ..Default::default()
    }))
    .map_err(|e| format!("GPU device: {e}"))?;
    // GPU errors nothing caught, and the device lost, in the run's trace, put to the subsystem whose labels they name
    // (quads, Bevy rendering on this device, raea-render, video; the driver when none): telemetry
    telemetry::watch(&device, "quads");
    Ok((instance, adapter, device, queue))
}
