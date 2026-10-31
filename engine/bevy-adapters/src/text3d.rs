//! bevy_rich_text3d (text as 3D meshes). What had to be written by hand: it loads no font unless told (system fonts
//! are off, and would differ from machine to machine), and a text mesh only shows through a material sampling its
//! glyph atlas, which a <BevyComponent> can't name. So this loads bevy_text's Fira Mono subset (vendored) into it, and
//! `Label3d` is a reflected component of ours that becomes the Text3d, its Text3dStyle, the mesh and an unlit material
//! over the atlas when it's added.

use bevy::prelude::*;
use bevy_rich_text3d::{LoadFonts, Text3d, Text3dPlugin, Text3dStyle, TextAtlas};

/// One line of text drawn as a mesh in the 3D scene, built into bevy_rich_text3d's parts when added.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
pub struct Label3d {
    pub text: String,
    /// glyph size in atlas pixels
    pub size: f32,
    /// the em's height in metres (bevy_rich_text3d's world_scale, as raea's own text sets it)
    pub world_scale: f32,
    /// sRGB
    pub color: Vec4,
    /// a loaded font's family ("Fira Mono", the one always loaded)
    pub font: String,
}

impl Default for Label3d {
    fn default() -> Self {
        Label3d { text: String::new(), size: 64.0, world_scale: 0.5, color: Vec4::ONE, font: "Fira Mono".into() }
    }
}

pub(crate) fn install(app: &mut App) {
    // raea's own text (raea-sys-text3d) may have added it already, in the render Engine
    if !app.is_plugin_added::<Text3dPlugin>() {
        app.add_plugins(Text3dPlugin::default());
    }
    app.world_mut().resource_mut::<LoadFonts>().font_embedded.push(include_bytes!("../../../vendor/bevy_text/src/FiraMono-subset.ttf"));
    app.register_type::<Label3d>();
    app.add_systems(Update, build_labels);
}

fn build_labels(added: Query<(Entity, &Label3d), Added<Label3d>>, mut materials: ResMut<Assets<StandardMaterial>>, mut commands: Commands) {
    for (e, l) in &added {
        let material = materials.add(StandardMaterial { base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()), alpha_mode: AlphaMode::Mask(0.5), unlit: true, cull_mode: None, ..default() });
        let style = Text3dStyle { size: l.size, font: l.font.as_str().into(), color: Srgba::new(l.color.x, l.color.y, l.color.z, l.color.w), world_scale: Some(Vec2::splat(l.world_scale)), ..default() };
        commands.entity(e).insert((Text3d::new(l.text.clone()), style, Mesh3d::default(), MeshMaterial3d(material)));
    }
}

/// The adapter's registration (the declaration's `"adapter": "text3d"`).
pub fn register() {
    raea_runtime_plugins::register(raea_runtime_plugins::Registered { name: "text3d", install, group: None, on_demand: false });
}
