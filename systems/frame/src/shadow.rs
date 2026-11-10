//! The sun's and spots' shadow map size when a <Shadows> doesn't give one: 4096 texels a side, a film's, unless the
//! GPU has little memory of its own. On Vulkan an AMD APU's own memory is the carve-out its firmware leaves it (often
//! 512 MB), and wgpu's allocator never puts what the GPU draws into anywhere else, so a sun's four 4096 cascades
//! (256 MB of depth) ran the first frame out of memory (#66: a Ryzen 7 4700U). Under 2 GB they're 2048, under 1 GB
//! 1024, said once; a <Shadows mapSize> still wins. DX12 and Metal draw in the system's shared memory and keep 4096.

use std::sync::{Once, OnceLock};

use bevy::prelude::*;

pub fn shadow_map_default(world: &World) -> usize {
    static OWN: OnceLock<Option<u64>> = OnceLock::new();
    let Some(own) = *OWN.get_or_init(|| own_memory(world)) else { return 4096 };
    let size = size_for(own);
    if size < 4096 {
        static SAID: Once = Once::new();
        SAID.call_once(|| eprintln!("raea: the GPU has {} MB of its own memory (Vulkan): shadow maps are {size} texels a side, not 4096 (<Shadows mapSize> sets them)", own >> 20));
    }
    size
}

/// 4096 a side with 2 GB of the GPU's own memory or more, 2048 under that, 1024 under 1 GB
fn size_for(own: u64) -> usize {
    match own >> 20 {
        m if m < 1024 => 1024,
        m if m < 2048 => 2048,
        _ => 4096,
    }
}

/// The largest heap Vulkan calls the GPU's own (device-local), when Bevy draws on Vulkan
#[cfg(any(windows, target_os = "linux"))]
fn own_memory(world: &World) -> Option<u64> {
    let adapter = world.get_resource::<bevy::render::renderer::RenderAdapter>()?;
    // SAFETY: reads the physical device's memory properties; nothing is created, kept or destroyed
    let hal = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    let props = unsafe { hal.shared_instance().raw_instance().get_physical_device_memory_properties(hal.raw_physical_device()) };
    // DEVICE_LOCAL is bit 0 of a heap's flags
    props.memory_heaps[..props.memory_heap_count as usize].iter().filter(|h| h.flags.as_raw() & 1 != 0).map(|h| h.size).max()
}

#[cfg(not(any(windows, target_os = "linux")))]
fn own_memory(_: &World) -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn small_gpu_memory_gets_smaller_shadow_maps() {
        let mb = |m: u64| m << 20;
        assert_eq!(super::size_for(mb(512)), 1024);
        assert_eq!(super::size_for(mb(1536)), 2048);
        assert_eq!(super::size_for(mb(2048)), 4096);
        assert_eq!(super::size_for(mb(12282)), 4096);
    }
}
