//! An Image read back to memory: for a check or a video's encoder, never for showing it (a layer is sampled where it
//! is). Waits for the GPU's work on it.

use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::GpuImage;

/// `image`'s pixels, rows tight (RGBA8 as its format stores them), or none before it's on the GPU.
pub fn read(app: &App, image: &Handle<Image>) -> Option<Vec<u8>> {
    let rw = app.sub_app(RenderApp).world();
    let gpu = rw.resource::<RenderAssets<GpuImage>>().get(image)?;
    let (device, queue) = (rw.resource::<RenderDevice>().wgpu_device().clone(), (**rw.resource::<RenderQueue>()).clone());
    let (w, h) = (gpu.texture.width(), gpu.texture.height());
    let row = (w * 4).div_ceil(256) * 256;
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("raea-rune-bevy read"), size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut enc = device.create_command_encoder(&Default::default());
    enc.copy_texture_to_buffer(
        gpu.texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit([enc.finish()]);
    buf.slice(..).map_async(wgpu::MapMode::Read, |r| r.expect("map the readback"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll the device");
    let data = buf.slice(..).get_mapped_range().expect("the mapped readback");
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h as usize {
        px.extend_from_slice(&data[y * row as usize..y * row as usize + (w * 4) as usize]);
    }
    Some(px)
}

/// The view of `image` on Bevy's device, for a host that draws its own Runtime into it (`draw_runtime`).
pub fn view(app: &App, image: &Handle<Image>) -> Option<wgpu::TextureView> {
    Some((*app.sub_app(RenderApp).world().resource::<RenderAssets<GpuImage>>().get(image)?.texture_view).clone())
}

/// Bevy's device and queue.
pub fn device(app: &App) -> (wgpu::Device, wgpu::Queue) {
    let rw = app.sub_app(RenderApp).world();
    (rw.resource::<RenderDevice>().wgpu_device().clone(), (**rw.resource::<RenderQueue>()).clone())
}
