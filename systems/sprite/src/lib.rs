//! 2D over the 3D (`<Sprite>` and `<Shape2D>` in raea/3d). A sprite is Bevy's Sprite, which bevy_sprite_render draws
//! as a quad with its sprite material, a sheet's cells cut by a TextureAtlasLayout; a shape is a Mesh2d with a
//! ColorMaterial. A Camera2d of the scene's own sees them, drawn after the 3D camera onto the same target without
//! clearing it, so 2D always lies over the 3D. It's made when the first 2D part is, with the 3D camera's target.
//!
//! Rows place a part in composition px from the top left, y down: the camera's viewport origin is its top left
//! (OrthographicProjection viewport_origin (0, 1)), so a part at (x, y) stands at (x, -y). Columns: x, y, rot (radians,
//! clockwise), scale_x, scale_y, z (draw order), col_r/g/b, opacity and visible, keyed; a sprite's w, h, sheet_cols,
//! sheet_rows, flip_x, flip_y and keyed sheet_frame (its picture comes decoded from the engine, which finds it as a
//! panel's: engine/render spawn.rs); a shape's shape (rect, circle, ellipse, triangle, polygon), w, h and sides.

use std::collections::BTreeMap;

use bevy::camera::{ClearColorConfig, OrthographicProjection, RenderTarget};
use bevy::image::{ImageSampler, TextureAtlas, TextureAtlasLayout};
use bevy::prelude::*;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use raea_core_program::eval::Tup;
use raea_core_program::ir::V;
use raea_sys_frame::{FrameRows, num};

/// A 2D part made from a row.
#[derive(Component)]
pub struct Part2d;

/// The scene's 2D camera.
#[derive(Component)]
pub struct Camera2dOver;

/// An entity a plugin draws in 2D (bevy_enoki's particles, bevy_smud's shapes, bevy_light_2d's lights): it brings up the
/// scene's 2D camera as a 2D part does, with no row placing it (`<BevyComponent type="raea_sys_sprite::Draws2d" />` on
/// its <Entity>, or required by a plugin adapter's component). The 2D camera's units are pixels from the frame's top
/// left, +y up, so a place on the frame is (x, -y).
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component, Default)]
pub struct Draws2d;

/// Draws2d in the TypeRegistry, so a <BevyComponent> names it.
pub fn install(app: &mut App) {
    app.register_type::<Draws2d>();
}

fn color_of(v: &BTreeMap<String, V>) -> Color {
    Color::srgba(num(v, "col_r", 1.0), num(v, "col_g", 1.0), num(v, "col_b", 1.0), num(v, "opacity", 1.0).clamp(0.0, 1.0))
}

fn place_of(v: &BTreeMap<String, V>) -> Transform {
    Transform::from_xyz(num(v, "x", 0.0), -num(v, "y", 0.0), num(v, "z", 0.0))
        .with_rotation(Quat::from_rotation_z(-num(v, "rot", 0.0)))
        .with_scale(Vec3::new(num(v, "scale_x", 1.0), num(v, "scale_y", 1.0), 1.0))
}

/// The sheet cell a row shows, wrapping.
fn cell(v: &BTreeMap<String, V>, cells: u32) -> usize {
    (num(v, "sheet_frame", 0.0).max(0.0) as u32 % cells.max(1)) as usize
}

/// A sprite's entity from its row and its picture (`size` in px).
pub fn spawn_sprite(world: &mut World, e: Entity, t: &Tup, image: Handle<Image>, size: UVec2) {
    let v = &t.vals;
    let (cols, rows) = (num(v, "sheet_cols", 1.0).max(1.0) as u32, num(v, "sheet_rows", 1.0).max(1.0) as u32);
    let (cw, ch) = ((size.x / cols).max(1), (size.y / rows).max(1));
    let texture_atlas = (cols > 1 || rows > 1).then(|| {
        // a sheet's cells sampled nearest: linear filtering at a cell's edge blends in the next cell (a column of it
        // showed down a scaled cell's side)
        if let Some(mut img) = world.resource_mut::<Assets<Image>>().get_mut(&image) {
            img.sampler = ImageSampler::nearest();
        }
        let layout = world.resource_mut::<Assets<TextureAtlasLayout>>().add(TextureAtlasLayout::from_grid(UVec2::new(cw, ch), cols, rows, None, None));
        TextureAtlas { layout, index: cell(v, cols * rows) }
    });
    // one size given, the other follows the cell's shape
    let aspect = ch as f32 / cw as f32;
    let custom_size = match (v.get("w").map(|x| x.num() as f32), v.get("h").map(|x| x.num() as f32)) {
        (Some(w), Some(h)) => Some(Vec2::new(w, h)),
        (Some(w), None) => Some(Vec2::new(w, w * aspect)),
        (None, Some(h)) => Some(Vec2::new(h / aspect, h)),
        (None, None) => None,
    };
    let sprite = Sprite {
        image,
        texture_atlas,
        custom_size,
        color: color_of(v),
        flip_x: num(v, "flip_x", 0.0) > 0.5,
        flip_y: num(v, "flip_y", 0.0) > 0.5,
        ..default()
    };
    world.entity_mut(e).insert((sprite, place_of(v), Visibility::default(), Part2d));
}

/// A flat shape's entity from its row.
pub fn spawn_shape(world: &mut World, e: Entity, t: &Tup) {
    let v = &t.vals;
    let (w, h) = (num(v, "w", 100.0), num(v, "h", 100.0));
    let mesh = match v.get("shape") {
        Some(V::S(s)) if s == "circle" => Mesh::from(Circle::new(w / 2.0)),
        Some(V::S(s)) if s == "ellipse" => Mesh::from(Ellipse::new(w / 2.0, h / 2.0)),
        Some(V::S(s)) if s == "triangle" => Mesh::from(Triangle2d::new(Vec2::new(0.0, h / 2.0), Vec2::new(-w / 2.0, -h / 2.0), Vec2::new(w / 2.0, -h / 2.0))),
        Some(V::S(s)) if s == "polygon" => Mesh::from(RegularPolygon::new(w / 2.0, num(v, "sides", 5.0).max(3.0) as u32)),
        _ => Mesh::from(Rectangle::new(w, h)),
    };
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    let mat = world.resource_mut::<Assets<ColorMaterial>>().add(ColorMaterial { color: color_of(v), alpha_mode: AlphaMode2d::Blend, ..default() });
    world.entity_mut(e).insert((Mesh2d(mesh), MeshMaterial2d(mat), place_of(v), Visibility::default(), Part2d));
}

/// The 2D camera, once the scene has a 2D part (or a plugin's Draws2d): over the 3D camera (one order after it), on its
/// target, not clearing.
pub fn camera_2d(mut commands: Commands, parts: Query<(), Or<(With<Part2d>, With<Draws2d>)>>, over: Query<(), With<Camera2dOver>>, three: Query<(&Camera, &RenderTarget), With<Camera3d>>) {
    if parts.is_empty() || !over.is_empty() {
        return;
    }
    let Some((cam, target)) = three.iter().max_by_key(|(c, _)| c.order) else { return };
    commands.spawn((
        Camera2d,
        Camera { order: cam.order + 1, clear_color: ClearColorConfig::None, ..default() },
        target.clone(),
        Projection::Orthographic(OrthographicProjection { viewport_origin: Vec2::new(0.0, 1.0), ..OrthographicProjection::default_2d() }),
        Msaa::Off,
        Camera2dOver,
    ));
}

/// Each 2D part at its row's place, cell, colour and opacity this frame.
pub fn sync_2d(frame: Res<FrameRows>, mut mats: ResMut<Assets<ColorMaterial>>, mut q: Query<(Entity, &mut Transform, &mut Visibility, Option<&mut Sprite>, Option<&MeshMaterial2d<ColorMaterial>>), With<Part2d>>) {
    for (e, mut t, mut vis, sprite, mat) in &mut q {
        let Some(v) = frame.vals(e) else { continue };
        let v = &v;
        let nt = place_of(v);
        if *t != nt {
            *t = nt;
        }
        let want = if v.get("visible").is_none_or(V::truthy) { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        let c = color_of(v);
        if let Some(mut s) = sprite {
            if s.color != c {
                s.color = c;
            }
            let cells = (num(v, "sheet_cols", 1.0).max(1.0) * num(v, "sheet_rows", 1.0).max(1.0)) as u32;
            let at = cell(v, cells);
            if let Some(a) = s.texture_atlas.as_mut().filter(|a| a.index != at) {
                a.index = at;
            }
        }
        // looked at first: get_mut marks the material changed whether or not it is
        if let Some(h) = mat.filter(|h| mats.get(&h.0).is_some_and(|m| m.color != c)) {
            if let Some(mut m) = mats.get_mut(&h.0) {
                m.color = c;
            }
        }
    }
}
