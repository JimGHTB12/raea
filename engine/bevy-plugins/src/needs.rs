//! What a plain headless App adds for a declared plugin (its declaration's "headless" list, sdk/raea/docs/bevy-plugins.md):
//! the pieces of DefaultPlugins a plugin reads, without a renderer or a window backend.

use bevy::asset::AssetApp;
use bevy::prelude::*;

pub(crate) fn add(app: &mut App, needs: &[&str]) {
    // big_space brings its own (it panics beside Bevy's)
    if !needs.contains(&"own-transform") {
        app.add_plugins(TransformPlugin);
    }
    // RAEA_BEVY_LOG=filter (as RUST_LOG): the Apps' tracing on stderr, for debugging a plugin (the first App sets it)
    if let Ok(filter) = std::env::var("RAEA_BEVY_LOG") {
        app.add_plugins(bevy::log::LogPlugin { filter, ..Default::default() });
    }
    if ["assets", "mesh", "shader", "image", "pbr"].iter().any(|n| needs.contains(n)) {
        // as the render Engine has it: a path anywhere, no .meta files
        app.add_plugins(bevy::asset::AssetPlugin { unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow, meta_check: bevy::asset::AssetMetaCheck::Never, ..Default::default() });
    }
    // meshes, and the skins whose bounds the camera's visibility and the gizmos read (MeshPlugin has both in a render)
    if needs.contains(&"mesh") {
        app.init_asset::<Mesh>().init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>();
    }
    // a plugin that adds its WGSL with load_internal_asset! (noisy_bevy) needs the asset store for it, not a renderer,
    // and one that keeps a storage buffer of its components (bevy_smud's GpuComponentArrayBufferPlugin) the buffers'
    if needs.contains(&"shader") {
        app.init_asset::<bevy::shader::Shader>().init_asset::<bevy::render::storage::ShaderBuffer>();
    }
    // images loaded from a path (PNG and the rest) and sprite-sheet layouts, as a sprite or a tile map reads them.
    // ImagePlugin only preregisters the loader; the renderer registers it once it knows the GPU's compressed formats,
    // so with no renderer it's registered here with none
    if needs.contains(&"image") {
        app.add_plugins(bevy::image::ImagePlugin::default()).init_asset::<TextureAtlasLayout>();
        app.register_asset_loader(bevy::image::ImageLoader::new(bevy::image::CompressedImageFormats::NONE));
    }
    // the standard material's store, for a plugin that makes materials (bevy_sprite3d), not the renderer that draws them
    if needs.contains(&"pbr") {
        app.init_asset::<StandardMaterial>();
    }
    // the world (scene) assets and their spawner, for a loader that makes one (bevy_vox_scene's VoxelScene)
    if needs.contains(&"scene") && !app.is_plugin_added::<bevy::world_serialization::WorldSerializationPlugin>() {
        app.add_plugins(bevy::world_serialization::WorldSerializationPlugin);
    }
    // picking headless: its pointers, hover and interaction, the mesh backend, and the cameras it casts rays from
    // (bevy_camera's CameraPlugin, with visibility and bounds, and bevy_render's, whose camera_system sizes a camera
    // from its window; with no RenderApp it extracts nothing); the mesh backend comes with the "bevy" builtin's
    // mesh_picking. raea/input's pointer moves the mouse pointer
    if needs.contains(&"picking") && !app.is_plugin_added::<bevy::picking::PickingPlugin>() {
        app.init_resource::<bevy::render::texture::ManualTextureViews>();
        app.add_plugins((bevy::camera::CameraPlugin, bevy::render::camera::CameraPlugin, bevy::picking::DefaultPickingPlugins));
        #[cfg(feature = "bevy")]
        app.add_plugins(bevy::picking::mesh_picking::MeshPickingPlugin);
    }
    if needs.contains(&"states") {
        app.add_plugins(bevy::state::app::StatesPlugin);
    }
    if needs.contains(&"input") {
        app.add_plugins(bevy::input::InputPlugin);
    }
    // the window messages (WindowEvent, CursorMoved, ...) a plugin's input systems read (bevy_egui), with no window;
    // "primary-window" also spawns the primary Window entity (no backend opens it), for a plugin that reads its size or
    // cursor (bevy_pancam). A Window can't be spawned from TypeScript: it reflects serde's Deserialize, which wants
    // every field
    if needs.contains(&"window") || needs.contains(&"primary-window") {
        let primary_window = needs.contains(&"primary-window").then(bevy::window::Window::default);
        app.add_plugins(bevy::window::WindowPlugin { primary_window, exit_condition: bevy::window::ExitCondition::DontExit, ..Default::default() });
    }
    // the gizmo stores a plugin's debug drawing writes into (vleue_navigator's debug-with-gizmos), with nothing
    // drawing them: GizmoPlugin's store, default group and GizmoAsset (bevy_gizmos_render, left out, draws them); its
    // skinned-mesh bounds read the skins "mesh" brings
    if needs.contains(&"gizmos") && !app.is_plugin_added::<bevy::gizmos::GizmoPlugin>() {
        app.add_plugins(bevy::gizmos::GizmoPlugin);
    }
}
