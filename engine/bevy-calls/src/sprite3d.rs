//! bevy_sprite3d (the declaration's `"calls": "sprite3d"`): a sprite on a quad in the 3D scene. `Sprite3d` isn't
//! reflected and it reads Bevy's `Sprite` beside it, so raea's reflected `SpriteQuad` says both in one component: a
//! scene puts `<SpriteQuad value={{ image: "hero.png", pixels_per_metre: 32 }} />` on an entity (as hanabi's Particles
//! does), and this turns it into the Sprite (its image, and an atlas cell when `atlas` is given) and the Sprite3d,
//! whose quad the plugin builds once the image has loaded. A changed SpriteQuad rebuilds both (a new frame is a new
//! `atlas.index`).
//!
//!   sprite3d/state  {entity}  -> {loaded, size: [w, h] in metres (null until loaded), index}

use bevy::prelude::*;
use bevy_remote::BrpResult;
use bevy_sprite3d::Sprite3d;
use serde_json::{Value, json};

use crate::{add_method, entity, fail, params};

/// A sprite on a quad: Bevy's Sprite and bevy_sprite3d's Sprite3d from one reflected component
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component, Default)]
pub struct SpriteQuad {
    /// the image, as the bridge reads any Handle<Image>: a path (beside the composition, or in media), an inline
    /// {width, height, pixels} picture, or "render:NAME"
    pub image: Handle<Image>,
    pub pixels_per_metre: f32,
    /// None centres it; (0, 0) to (1, 1) across the sprite
    pub pivot: Option<Vec2>,
    pub unlit: bool,
    pub double_sided: bool,
    /// alpha blended, where the default cuts out at 0.5
    pub blend: bool,
    pub emissive: LinearRgba,
    /// a cell of a sprite sheet, when the image is one
    pub atlas: Option<SpriteSheet>,
}

/// A grid of equal cells, numbered row by row from the top left
#[derive(Reflect, Clone, Debug, Default)]
pub struct SpriteSheet {
    pub tile: UVec2,
    pub columns: u32,
    pub rows: u32,
    pub index: usize,
}

impl Default for SpriteQuad {
    fn default() -> Self {
        let d = Sprite3d::default();
        Self { image: Handle::default(), pixels_per_metre: d.pixels_per_metre, pivot: None, unlit: d.unlit, double_sided: d.double_sided, blend: false, emissive: d.emissive, atlas: None }
    }
}

pub fn calls(app: &mut App) {
    app.register_type::<SpriteQuad>().add_systems(Update, apply);
    add_method(app, "sprite3d/state", state);
}

fn apply(q: Query<(Entity, &SpriteQuad), Changed<SpriteQuad>>, mut layouts: ResMut<Assets<TextureAtlasLayout>>, mut commands: Commands) {
    for (e, s) in &q {
        let mut sprite = Sprite::from_image(s.image.clone());
        if let Some(a) = &s.atlas {
            let layout = layouts.add(TextureAtlasLayout::from_grid(a.tile, a.columns, a.rows, None, None));
            sprite.texture_atlas = Some(TextureAtlas { layout, index: a.index });
        }
        let quad = Sprite3d {
            pixels_per_metre: s.pixels_per_metre,
            pivot: s.pivot,
            unlit: s.unlit,
            double_sided: s.double_sided,
            alpha_mode: if s.blend { AlphaMode::Blend } else { Sprite3d::default().alpha_mode },
            emissive: s.emissive,
        };
        commands.entity(e).insert((sprite, quad));
    }
}

#[derive(serde::Deserialize)]
struct Of {
    entity: u64,
}

fn state(In(p): In<Option<Value>>, world: &mut World) -> BrpResult {
    let Of { entity: bits } = params(p)?;
    let e = entity(bits)?;
    let s = world.get::<SpriteQuad>(e).ok_or_else(|| fail(format!("entity {bits} has no SpriteQuad")))?.clone();
    let index = s.atlas.as_ref().map(|a| a.index);
    // the quad's extent in x and y, from its vertex positions (none until the image has loaded and the mesh is made)
    let positions = world.get::<Mesh3d>(e).and_then(|m| world.resource::<Assets<Mesh>>().get(&m.0)).and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)?.as_float3().map(<[_]>::to_vec));
    let size = positions.filter(|p| !p.is_empty()).map(|p| {
        let span = |i: usize| p.iter().map(|v| v[i]).fold(f32::MIN, f32::max) - p.iter().map(|v| v[i]).fold(f32::MAX, f32::min);
        [span(0), span(1)]
    });
    Ok(json!({ "loaded": size.is_some(), "size": size, "index": index }))
}
