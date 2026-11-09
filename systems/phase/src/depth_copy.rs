//! A camera's depth copied into an image (docs/control.feature FC2d), as Bevy's shader_advanced/render_depth_to_texture
//! copies its depth-only camera's: a camera with a DepthCopy has its depth buffer (ViewDepthStencilTexture) copied after
//! its prepass and before its main pass into `image`, a "render:NAME:WxH:depth" (raea-runtime-plugins' fields), which a
//! Material samples as a texture_depth_2d. The copy is the image's size from the depth buffer's corner; a depth buffer
//! smaller than the image isn't copied.

use bevy::core_pipeline::{Core3d, Core3dSystems};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::renderer::{RenderContext, ViewQuery};
use bevy::render::texture::GpuImage;
use bevy::render::view::ViewDepthStencilTexture;
use bevy::render::RenderApp;

/// The image a camera's depth is copied into each frame.
#[derive(Component, Clone, Debug, Default, Reflect, ExtractComponent)]
#[extract_app(RenderApp)]
#[reflect(Component, Default)]
#[type_path = "raea_sys_phase"]
pub struct DepthCopy {
    pub image: Handle<Image>,
}

pub fn install(app: &mut App) {
    app.register_type::<DepthCopy>();
    app.add_plugins(ExtractComponentPlugin::<DepthCopy>::default());
    let Some(r) = app.get_sub_app_mut(RenderApp) else { return };
    r.add_systems(Core3d, copy.after(Core3dSystems::Prepass).before(Core3dSystems::MainPass));
}

/// The view's depth into its DepthCopy's image.
fn copy(view: ViewQuery<(&ViewDepthStencilTexture, &DepthCopy)>, images: Res<RenderAssets<GpuImage>>, mut ctx: RenderContext) {
    let (depth, c) = view.into_inner();
    let Some(img) = images.get(&c.image) else { return };
    let (from, to) = (depth.texture().size(), img.texture.size());
    if from.width < to.width || from.height < to.height {
        return;
    }
    ctx.command_encoder().copy_texture_to_texture(depth.texture().as_image_copy(), img.texture.as_image_copy(), to);
}
