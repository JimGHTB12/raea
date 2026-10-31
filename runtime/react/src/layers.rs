//! A page in the 3D world (docs/blitz-bevy.feature B3 H5, world and surface modes): [`PageLayer`] on an entity
//! draws a page's image on it, lit or unlit, and puts it in Bevy's picking (raea-page's PageView on a mesh: clicks,
//! hover and keys reach the page through its UVs).
//!
//! - World: a quad of the page's aspect in the scene, `width` wide in world units, either face, with depth (it sorts
//!   with the 3D and the 3D occludes it).
//! - Surface: the page on the entity's own mesh, through its UVs (a screen, a label on a box, a curved panel).
//!
//! The page's image is premultiplied (raea-page's premul pass), drawn as such. A page that's resized gets its new image.

use bevy::prelude::*;
use raea_runtime_input::pick::PageView;
use raea_sys_page::{Page, PageImage};

/// How a page is placed in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayerMode {
    /// a quad of the page's aspect, this wide
    World { width: f32 },
    /// on the entity's own Mesh3d
    Surface,
}

/// `page` drawn on this entity, in the world.
#[derive(Component, Clone, Copy, Debug)]
pub struct PageLayer {
    pub page: Entity,
    pub mode: LayerMode,
    /// shaded by the scene's lights (a printed poster), or not (a screen giving its own light)
    pub lit: bool,
}

impl PageLayer {
    pub fn world(page: Entity, width: f32) -> Self {
        PageLayer { page, mode: LayerMode::World { width }, lit: false }
    }
    pub fn surface(page: Entity) -> Self {
        PageLayer { page, mode: LayerMode::Surface, lit: false }
    }
}

pub struct PageLayersPlugin;

impl Plugin for PageLayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, place_layers.after(raea_sys_page::PageSystems::Commands));
    }
}

/// What a layer was given, to give it again only when its page's image or size changes.
#[derive(Component)]
struct Placed {
    image: Handle<Image>,
    size: UVec2,
    layer: (LayerMode, bool),
}

fn place_layers(
    mut commands: Commands,
    layers: Query<(Entity, &PageLayer, Option<&Placed>)>,
    pages: Query<(&Page, &PageImage)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (e, layer, placed) in &layers {
        let Ok((page, image)) = pages.get(layer.page) else { continue };
        if placed.is_some_and(|p| p.image == image.0 && p.size == page.size && p.layer == (layer.mode, layer.lit)) {
            continue;
        }
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(image.0.clone()),
            unlit: !layer.lit,
            alpha_mode: AlphaMode::Premultiplied,
            double_sided: true,
            cull_mode: None,
            ..default()
        });
        let mut entity = commands.entity(e);
        entity.insert((MeshMaterial3d(material), PageView::new(layer.page), Placed { image: image.0.clone(), size: page.size, layer: (layer.mode, layer.lit) }));
        if let LayerMode::World { width } = layer.mode {
            let height = width * page.size.y as f32 / page.size.x.max(1) as f32;
            entity.insert(Mesh3d(meshes.add(Rectangle::new(width, height))));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use raea_runtime_twins::{PageCommands, PageQuery};

    /// A layer's place in the world is the ECS's (B3 H9): the page's own transforms, React's through its ops and a
    /// system's through PageCommands, move what's drawn on the layer, never the layer's Transform.
    #[test]
    fn a_pages_transforms_never_move_its_layer() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default(), raea_page::PagePlugin, PageLayersPlugin))
            .init_asset::<Image>()
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>();
        let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), 200, 100, None);
        let image = app.world_mut().resource_mut::<Assets<Image>>().add(Image::default());
        app.world_mut().entity_mut(page).insert(PageImage(image));
        let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::json!([
            { "op": "create", "id": 1, "kind": "div", "props": { "id": "card", "style": { "width": "200px", "height": "100px", "transform": "translate(40px, 10px) rotate(30deg) scale(2)" } } },
            { "op": "append", "parent": 0, "child": 1 },
        ]))
        .unwrap();
        raea_sys_page::ops::apply(app.world_mut().non_send_mut::<raea_sys_page::Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
        let at = Transform::from_xyz(1.0, 2.0, -3.0).with_rotation(Quat::from_rotation_y(0.5)).with_scale(Vec3::splat(1.5));
        let layer = app.world_mut().spawn((PageLayer::world(page, 2.0), at)).id();
        for _ in 0..3 {
            app.update();
        }
        assert!(app.world().get::<Mesh3d>(layer).is_some(), "the layer wasn't placed");
        assert_eq!(*app.world().get::<Transform>(layer).unwrap(), at);
        let card = app.world_mut().run_system_once(move |q: PageQuery| q.by_id(page, "card")).unwrap().unwrap();
        app.world_mut().run_system_once(move |mut c: PageCommands| c.set_style(card, "translate", "0px 300px")).unwrap();
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(*app.world().get::<Transform>(layer).unwrap(), at);
    }
}
