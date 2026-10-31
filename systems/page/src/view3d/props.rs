//! A Bevy 3D view's camera and scene from an element's props (docs/ts-native.feature P15): raea-page's Blitz page
//! makes `<view3d>` a box a camera renders into, and its `mesh3d` and `light3d` children the scene, with these.
//!
//! ```tsx
//! <view3d className="h-40 w-60" clearColor="#1e293b" projection="orthographic" viewHeight={3} eye={[0, 0, 10]} target={[0, 0, 0]}>
//!   <mesh3d shape="cuboid" size={[2, 2, 2]} color="#e11d48" unlit position={[0, 0, 0]} rotation={[0, 45, 0]} />
//!   <light3d kind="directional" color="#ffffff" illuminance={3000} position={[3, 5, 4]} target={[0, 0, 0]} />
//! </view3d>
//! ```
//!
//! - view3d: `clearColor` (any CSS colour), `projection` ("perspective", the default, with `fov` in degrees, or
//!   "orthographic" with `viewHeight` in world units), `eye` and `target` (where the camera is and looks), `camera`
//!   (an app camera's name in place of its own). Its camera has no tonemapping, so an unlit colour is drawn as given.
//! - mesh3d: `shape` (cuboid, sphere, cylinder, capsule, plane, torus), `size` (x, y, z: a cuboid's sides, a sphere's
//!   radius in x, a cylinder's radius and height, a plane's x and z), `color`, `unlit`, `metallic`, `roughness`,
//!   `position`, `rotation` (degrees about x, y, z), `scale`.
//! - light3d: `kind` (directional, point, ambient), `color`, `illuminance` / `intensity`, `position`, `target`.

use bevy::camera::{ScalingMode, visibility::RenderLayers};
use bevy::prelude::*;
use serde_json::{Map, Value};

fn vec3(v: Option<&Value>, or: Vec3) -> Vec3 {
    let Some(a) = v.and_then(Value::as_array) else { return or };
    let f = |i: usize, d: f32| a.get(i).and_then(Value::as_f64).map_or(d, |x| x as f32);
    Vec3::new(f(0, or.x), f(1, or.y), f(2, or.z))
}

fn f(p: &Map<String, Value>, k: &str) -> Option<f32> {
    p.get(k).and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse().ok())).map(|x| x as f32)
}

/// A view's clear colour from its props.
pub fn clear_color(p: &Map<String, Value>) -> Color {
    color(p, "clearColor", Color::NONE)
}

fn color(p: &Map<String, Value>, k: &str, or: Color) -> Color {
    let css = |s: &str| {
        let c = color::parse_color(s.trim()).ok()?.to_alpha_color::<color::Srgb>().components;
        Some(Color::srgba(c[0], c[1], c[2], c[3]))
    };
    p.get(k).and_then(Value::as_str).and_then(css).unwrap_or(or)
}

fn on(p: &Map<String, Value>, k: &str) -> bool {
    matches!(p.get(k), Some(Value::Bool(true))) || p.get(k).and_then(Value::as_str).is_some_and(|s| s != "false")
}

/// A view's projection from its props (raea-page's Blitz page builds its views with these too).
pub fn projection(p: &Map<String, Value>) -> Projection {
    if p.get("projection").and_then(Value::as_str) == Some("orthographic") {
        Projection::Orthographic(OrthographicProjection { scaling_mode: ScalingMode::FixedVertical { viewport_height: f(p, "viewHeight").unwrap_or(2.0) }, ..OrthographicProjection::default_3d() })
    } else {
        Projection::Perspective(PerspectiveProjection { fov: f(p, "fov").unwrap_or(45.0).to_radians(), ..default() })
    }
}

/// Where a view's camera is and looks, from its props.
pub fn eye(p: &Map<String, Value>) -> Transform {
    let at = vec3(p.get("eye"), Vec3::new(0.0, 0.0, 10.0));
    let target = vec3(p.get("target"), Vec3::ZERO);
    let up = if (target - at).normalize_or_zero().abs().y > 0.999 { Vec3::Z } else { Vec3::Y };
    Transform::from_translation(at).looking_at(target, up)
}

fn mesh_of(p: &Map<String, Value>) -> Mesh {
    let s = vec3(p.get("size"), Vec3::ONE);
    match p.get("shape").and_then(Value::as_str).unwrap_or("cuboid") {
        "sphere" => Sphere::new(s.x).mesh().uv(64, 32),
        "cylinder" => Cylinder::new(s.x, s.y).mesh().resolution(64).into(),
        "capsule" => Capsule3d::new(s.x, s.y).mesh().into(),
        "plane" => Plane3d::default().mesh().size(s.x, s.z).into(),
        "torus" => Torus::new(s.x, s.y).mesh().into(),
        _ => Cuboid::new(s.x, s.y, s.z).mesh().into(),
    }
}

/// A part's entity made from its props on render layer `layer` (`cam`: its view's camera, which an ambient light
/// goes on); None for a kind that isn't a part.
pub fn spawn_part(world: &mut World, kind: &str, p: &Map<String, Value>, layer: usize, cam: Option<Entity>) -> Option<Entity> {
    let rot = vec3(p.get("rotation"), Vec3::ZERO);
    let mut t = Transform::from_translation(vec3(p.get("position"), Vec3::ZERO))
        .with_rotation(Quat::from_euler(EulerRot::XYZ, rot.x.to_radians(), rot.y.to_radians(), rot.z.to_radians()))
        .with_scale(vec3(p.get("scale"), Vec3::ONE));
    let layers = RenderLayers::layer(layer);
    let made = match kind {
        "mesh3d" => {
            let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh_of(&p));
            let mat = StandardMaterial {
                base_color: color(&p, "color", Color::WHITE),
                unlit: on(&p, "unlit"),
                metallic: f(&p, "metallic").unwrap_or(0.0),
                perceptual_roughness: f(&p, "roughness").unwrap_or(0.5),
                alpha_mode: if color(&p, "color", Color::WHITE).alpha() < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque },
                ..default()
            };
            let mat = world.resource_mut::<Assets<StandardMaterial>>().add(mat);
            world.spawn((Mesh3d(mesh), MeshMaterial3d(mat), t, layers)).id()
        }
        "light3d" => {
            let c = color(&p, "color", Color::WHITE);
            if p.contains_key("target") {
                t.look_at(vec3(p.get("target"), Vec3::ZERO), Vec3::Y);
            }
            match p.get("kind").and_then(Value::as_str).unwrap_or("directional") {
                "point" => world.spawn((PointLight { color: c, intensity: f(&p, "intensity").unwrap_or(1_000_000.0), ..default() }, t, layers)).id(),
                "ambient" => {
                    if let Some(cam) = cam {
                        world.entity_mut(cam).insert(AmbientLight { color: c, brightness: f(&p, "intensity").unwrap_or(500.0), ..default() });
                    }
                    world.spawn((t, layers)).id()
                }
                _ => world.spawn((DirectionalLight { color: c, illuminance: f(&p, "illuminance").unwrap_or(3000.0), ..default() }, t, layers)).id(),
            }
        }
        _ => return None,
    };
    Some(made)
}
