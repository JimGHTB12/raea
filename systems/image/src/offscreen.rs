//! Offscreen roots for renderers (docs/ts-native.feature P26): a root drawn into an image of a fixed size, a frame at a
//! time, and the frame read back as RGBA. A video renderer steps its own App: it applies the frame's ops, then asks for
//! the frame here, so a frame drawn alone is the frame drawn in sequence (nothing in the UI reads the clock).
//!
//! The image is `Rgba8UnormSrgb`, cleared to transparent unless the root says otherwise; the pixels come back as the
//! texture holds them (sRGB-encoded, premultiplied as bevy_ui blends over a transparent clear).

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

/// An image a root can draw into and the frame can be read back from.
pub fn target(images: &mut Assets<Image>, w: u32, h: u32) -> Handle<Image> {
    let mut image = Image::new_fill(Extent3d { width: w, height: h, ..default() }, TextureDimension::D2, &[0, 0, 0, 0], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT;
    images.add(image)
}

/// Readbacks as they land, by the entity that asked.
#[derive(Resource, Default)]
struct Landed(Vec<(Entity, Vec<u8>)>);

/// Updates a readback may take to land: its copy is submitted and its buffer's map asked for in the first, and it's
/// handed to the main world in the next one's extract, once the map's callback has run.
const MAX_UPDATES: usize = 32;

/// How long the GPU may take to finish a frame before the readback is given up as lost.
const GPU_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// Several images drawn by the same updates and read back together (a page and its screens), in the order asked.
pub fn frames(app: &mut App, images: &[(Handle<Image>, u32, u32)]) -> Result<Vec<Vec<u8>>, String> {
    app.world_mut().init_resource::<Landed>();
    let askers: Vec<Entity> = images
        .iter()
        .map(|(image, _, _)| app.world_mut().spawn(Readback::texture(image.clone())).observe(|ev: On<ReadbackComplete>, mut landed: ResMut<Landed>| landed.0.push((ev.entity, ev.data.clone()))).id())
        .collect();
    let mut got: Vec<Option<Vec<u8>>> = vec![None; images.len()];
    let despawn = |app: &mut App| {
        for a in &askers {
            app.world_mut().entity_mut(*a).despawn();
        }
    };
    for _ in 0..MAX_UPDATES {
        app.update();
        let mut landed = app.world_mut().resource_mut::<Landed>();
        for (k, a) in askers.iter().enumerate() {
            if got[k].is_none() {
                if let Some(i) = landed.0.iter().position(|(e, _)| e == a) {
                    got[k] = Some(landed.0.swap_remove(i).1);
                }
            }
        }
        if got.iter().all(Option::is_some) {
            landed.0.clear();
            break;
        }
        // Bevy never polls the device: a map's callback runs only when a later submit finds its copy done, so a busy
        // GPU (another process's work, a big frame) let the CPU's updates run past the copy and the frame came back
        // late or not within MAX_UPDATES. Waiting here for what was submitted makes it land on the next update.
        if let Err(e) = wait_for_gpu(app) {
            despawn(app);
            return Err(e);
        }
    }
    despawn(app);
    got.into_iter()
        .zip(images)
        .map(|(g, (_, w, h))| g.map(|d| unpad(d, *w, *h)).ok_or_else(|| format!("raea-runtime-js: the frame didn't come back in {MAX_UPDATES} updates")))
        .collect()
}

/// Blocks until the GPU has run everything submitted so far, and every map callback that work completes has run.
fn wait_for_gpu(app: &App) -> Result<(), String> {
    use bevy::render::render_resource::PollType;
    let Some(render) = app.get_sub_app(bevy::render::RenderApp) else { return Ok(()) };
    let Some(device) = render.world().get_resource::<bevy::render::renderer::RenderDevice>() else { return Ok(()) };
    device
        .poll(PollType::Wait { submission_index: None, timeout: Some(GPU_TIMEOUT) })
        .map(|_| ())
        .map_err(|e| format!("raea-runtime-js: the GPU didn't finish the frame in {}s: {e}", GPU_TIMEOUT.as_secs()))
}

/// Rows as the copy aligned them (each a multiple of 256 bytes), packed.
fn unpad(data: Vec<u8>, w: u32, h: u32) -> Vec<u8> {
    let row = w as usize * 4;
    let padded = row.div_ceil(256) * 256;
    if padded == row || data.len() < padded * (h as usize - 1) + row {
        return data;
    }
    let mut out = Vec::with_capacity(row * h as usize);
    for y in 0..h as usize {
        out.extend_from_slice(&data[y * padded..y * padded + row]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_unpadded() {
        // 3 px wide: 12 bytes a row, padded to 256
        let mut d = vec![0u8; 256 * 2];
        d[0] = 1;
        d[11] = 2;
        d[256] = 3;
        d[267] = 4;
        let out = unpad(d, 3, 2);
        assert_eq!(out.len(), 24);
        assert_eq!((out[0], out[11], out[12], out[23]), (1, 2, 3, 4));
        // 64 px: already aligned
        assert_eq!(unpad(vec![7; 256 * 2], 64, 2).len(), 512);
    }
}
