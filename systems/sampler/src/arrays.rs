//! Texture arrays (docs/shaders.feature SH9): a shader's texture given as `{ layers: [src, ...] }` (one picture per
//! layer, all one size and format) or `{ src, layers: N }` (one picture of N layers stacked top to bottom, Bevy's own
//! way to make an array), which the shader reads as `texture_2d_array<f32> tex_NAME`, sampled by layer. The layers
//! load as any texture does; the array is made once they all have, under a handle reserved when the spec is read, so
//! the material that names it waits for it as it waits for a file.

use super::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension};

/// Arrays waiting on their layers: the array's handle, its layers' handles (one, for a stack), its layer count, and
/// who asked (for the report).
#[derive(Resource, Default)]
pub struct PendingArrays {
    pending: Vec<(Handle<Image>, Vec<Handle<Image>>, u32, String)>,
}

/// A texture spec that's an array: it has `layers`.
pub fn is_array(v: &Value) -> bool {
    v.as_object().is_some_and(|o| o.contains_key("layers"))
}

/// An array texture's handle, made once its layers load; `{sampler}` is set on the array as on any texture.
pub fn array(world: &mut World, v: &Value, who: &str, image: &mut impl FnMut(&mut World, &Value) -> Result<Handle<Image>, String>) -> Result<Handle<Image>, String> {
    let o = v.as_object().ok_or_else(|| format!("{who}: an array is {{layers: [picture, ...]}} or {{src, layers: N}}"))?;
    let (layers, n) = match (o.get("layers"), o.get("src")) {
        (Some(Value::Array(a)), None) if a.len() >= 2 => (a.iter().map(|x| image(world, x)).collect::<Result<Vec<_>, _>>()?, a.len() as u32),
        (Some(Value::Number(n)), Some(src)) => match n.as_u64().filter(|n| (2..=2048).contains(n)) {
            Some(n) => (vec![image(world, src)?], n as u32),
            None => return Err(format!("{who}: layers is a whole number from 2 to 2048, got {n}")),
        },
        _ => return Err(format!("{who}: an array is {{layers: [picture, ...]}} (2 or more) or {{src, layers: N}} (a picture of N layers stacked top to bottom), got {v}")),
    };
    let h = world.resource::<Assets<Image>>().reserve_handle();
    world.get_resource_or_insert_with(PendingArrays::default).pending.push((h.clone(), layers, n, who.to_string()));
    if let Some(s) = o.get("sampler") {
        PendingSamplers::give(world, h.clone(), Sampling::of(s)?, who.to_string());
    }
    Ok(h)
}

/// Each array whose layers have all loaded, made: the layers stacked into one picture and reinterpreted as `n`
/// layers. One that can't be made (sizes or formats differ, a stack whose height `n` doesn't divide) is reported and
/// dropped, so its material draws nothing.
pub fn sync_arrays(p: Option<ResMut<PendingArrays>>, mut images: ResMut<Assets<Image>>) {
    let Some(mut p) = p else { return };
    if p.pending.is_empty() {
        return;
    }
    for (h, layers, n, who) in std::mem::take(&mut p.pending) {
        if layers.iter().any(|l| images.get(l).is_none()) {
            p.pending.push((h, layers, n, who));
            continue;
        }
        match stack(&layers.iter().map(|l| images.get(l).unwrap()).collect::<Vec<_>>(), n) {
            Ok(img) => {
                let _ = images.insert(&h, img);
            }
            Err(e) => eprintln!("raea: {who}: {e}"),
        }
    }
}

fn stack(layers: &[&Image], n: u32) -> Result<Image, String> {
    let first = layers[0];
    let mut img = if layers.len() == 1 {
        first.clone()
    } else {
        let (size, format) = (first.texture_descriptor.size, first.texture_descriptor.format);
        let mut data = Vec::new();
        for (i, l) in layers.iter().enumerate() {
            if l.texture_descriptor.size != size || l.texture_descriptor.format != format {
                return Err(format!("layer {i} is {}x{} {:?}, layer 0 {}x{} {:?}: an array's layers are one size and format", l.width(), l.height(), l.texture_descriptor.format, size.width, size.height, format));
            }
            if l.texture_descriptor.mip_level_count != 1 || size.depth_or_array_layers != 1 {
                return Err(format!("layer {i} has mips or layers of its own: an array's layers are plain pictures"));
            }
            data.extend_from_slice(l.data.as_deref().ok_or_else(|| format!("layer {i} has no pixels on the CPU"))?);
        }
        let mut img = Image::new(Extent3d { width: size.width, height: size.height * n, depth_or_array_layers: 1 }, TextureDimension::D2, data, format, RenderAssetUsages::default());
        img.sampler = first.sampler.clone();
        img
    };
    img.reinterpret_stacked_2d_as_array(n).map_err(|e| format!("its {} layers: {e:?}", n))?;
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::render_resource::TextureFormat;

    fn pic(px: [u8; 4]) -> Image {
        Image::new_fill(Extent3d { width: 2, height: 2, depth_or_array_layers: 1 }, TextureDimension::D2, &px, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::all())
    }

    #[test]
    fn layers_stack_into_an_array_and_mismatches_are_refused() {
        let (a, b) = (pic([255, 0, 0, 255]), pic([0, 0, 255, 255]));
        let img = stack(&[&a, &b], 2).unwrap();
        assert_eq!((img.width(), img.height(), img.texture_descriptor.size.depth_or_array_layers), (2, 2, 2));
        assert_eq!(&img.data.as_ref().unwrap()[16..20], &[0, 0, 255, 255]);
        let tall = Image::new_fill(Extent3d { width: 2, height: 6, depth_or_array_layers: 1 }, TextureDimension::D2, &[9, 9, 9, 255], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::all());
        assert_eq!(stack(&[&tall], 3).unwrap().texture_descriptor.size.depth_or_array_layers, 3);
        assert!(stack(&[&tall], 4).is_err());
        let big = Image::new_fill(Extent3d { width: 4, height: 2, depth_or_array_layers: 1 }, TextureDimension::D2, &[0, 0, 0, 255], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::all());
        assert!(stack(&[&a, &big], 2).is_err());
    }
}
