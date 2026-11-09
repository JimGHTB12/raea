//! The pictures an own Material (own.rs) binds besides its four textures: a depth image (Bevy's render_depth_to_texture)
//! and a binding_array<texture_2d<f32>>'s list (Bevy's texture_binding_array, drawn by own_array.rs), and a picture file
//! decoded at once.

use bevy::prelude::*;
use bevy::render::render_resource::WgpuFeatures;
use bevy::render::renderer::RenderDevice;

use crate::ui_bind::ARRAY_LEN;

/// A binding_array<texture_2d<f32>>'s pictures, the spec's list (`bindings`) for the source's `others` binding_array:
/// in 3D only, on a device with texture binding arrays, at most ARRAY_LEN. A source that declares no texture_2d samples
/// with its first sampler, the first texture's, so `t0` becomes the list's first picture and that sampler its own.
pub(crate) fn array(world: &World, bindings: Option<&(String, Vec<Handle<Image>>)>, others: &[(String, &'static str)], no_textures: bool, uniforms: bool, t0: &mut Option<Handle<Image>>, two_d: bool, path: &str) -> Result<Vec<Handle<Image>>, String> {
    let declared = others.iter().find(|o| o.1 == "binding_array").map(|o| o.0.as_str());
    let Some((name, list)) = bindings else {
        return match declared {
            Some(n) => Err(format!("{path}'s binding_array {n} has no list of pictures")),
            None => Ok(vec![]),
        };
    };
    if declared != Some(name.as_str()) {
        return Err(format!("{path} has no binding_array<texture_2d<f32>> {name}"));
    }
    if uniforms {
        return Err(format!("{path}: a binding_array can't sit beside uniforms in one bind group (wgpu); give its values as a storage buffer"));
    }
    if two_d {
        return Err(format!("{path}: a binding_array is a 3D Material's, not a 2D one's"));
    }
    if list.is_empty() || list.len() > ARRAY_LEN {
        return Err(format!("{path}'s binding_array {name} takes 1 to {ARRAY_LEN} pictures, got {}", list.len()));
    }
    let need = WgpuFeatures::TEXTURE_BINDING_ARRAY | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING;
    if world.get_resource::<RenderDevice>().is_some_and(|d| !d.features().contains(need)) {
        return Err(format!("{path}'s binding_array {name}: this GPU has no texture binding arrays"));
    }
    if no_textures {
        *t0 = Some(list[0].clone());
    }
    Ok(list.clone())
}

/// A depth image a shader samples (texture_depth_2d with a sampler_comparison): Depth32Float, its sampler comparing
/// Always, as Bevy's render_depth_to_texture makes the image its depth-only camera's depth is copied into.
pub fn depth_image(w: u32, h: u32) -> Image {
    use bevy::image::{ImageCompareFunction, ImageSampler, ImageSamplerDescriptor};
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut img = Image::new_uninit(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, TextureFormat::Depth32Float, bevy::asset::RenderAssetUsages::default());
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor { compare: Some(ImageCompareFunction::Always), ..default() });
    img
}

/// The 1 x 1 depth image every Material that binds no depth texture binds (made once).
pub(crate) fn no_depth(world: &mut World) -> Handle<Image> {
    #[derive(Resource)]
    struct NoDepth(Handle<Image>);
    if let Some(h) = world.get_resource::<NoDepth>() {
        return h.0.clone();
    }
    let h = world.resource_mut::<Assets<Image>>().add(depth_image(1, 1));
    world.insert_resource(NoDepth(h.clone()));
    h
}

/// A picture file decoded now as an Image (sRGB, the default sampler, as asset_server.load reads a colour texture), so
/// the first frame that draws it has it.
pub fn picture(world: &mut World, path: &str) -> Result<Handle<Image>, String> {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("png");
    let image = Image::from_buffer(&bytes, ImageType::Extension(ext), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()).map_err(|e| format!("{path}: {e}"))?;
    Ok(world.resource_mut::<Assets<Image>>().add(image))
}
