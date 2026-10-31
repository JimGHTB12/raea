//! Pages in Bevy's picking (docs/blitz-bevy.feature B3 H3, H6): Blitz is a picking backend, and the pointer input
//! picking reads goes to the page whose element it picked. One route from input to a page: Bevy's pointers, every
//! backend's hits sorted by depth (a 3D object in front of a page takes the press, a page in front of one does), the
//! page's element first among them, then the page.
//!
//! - A page is shown somewhere: [`PageView`] on the entity showing it. On a UI node, the node's box is the page,
//!   scaled to it; on a mesh (a screen in the scene, a page on any surface), its UVs are: picking's ray for each
//!   camera and pointer (`RayMap`) is cast against the mesh's triangles here, and the hit's UV times the page's size
//!   is the point (H6), its depth the ray's distance, so a page and the 3D sort as Bevy sorts any hits.
//! - The backend ([`pick_pages`]) reports, for each pointer over a view, the twin of the element Blitz's hit test finds
//!   there (`elementFromPoint`), with the point in the page's CSS px as the hit's position. Bevy's events then fire on
//!   the twins and bubble up their hierarchy (the elements'): PointerPress, PointerClick, PointerDrag and the rest,
//!   PointerDragDrop with `dragged` when one twin is dropped on another, or on a mesh.
//! - The route ([`route`]) runs between picking's hover map and its events: each PointerInput this frame becomes the
//!   page's UiEvent at the picked point, so React's handlers (and the `dom` events) come before Bevy's pointer events.
//!   A press holds its pointer on its page until the last button is up (pointer capture: the drag's moves and its
//!   release go to the page it started on, at points mapped through its view, past its edges). A pointer that leaves
//!   a page sends it a pointerleave.
//! - 3D objects in front of a page: a mesh marked [`PickableMesh`] is picked here too (its triangles against the same
//!   rays), so it takes the pointer where it's nearer than the page behind it. (Bevy's mesh_picking does this for every
//!   mesh, but turning it on rebuilds Bevy for every session; any other backend's hits sort with the page's the same way.)

use std::collections::HashMap;

use bevy::camera::RenderTarget;
use bevy::ecs::message::MessageCursor;
use bevy::input::keyboard::KeyCode;
use bevy::mesh::{Indices, Mesh3d, VertexAttributeValues};
use bevy::picking::backend::ray::{RayId, RayMap};
use bevy::picking::backend::{HitData, PointerHits};
use bevy::picking::hover::HoverMap;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput, PointerLocation};
use bevy::picking::PickingSystems;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, ComputedUiTargetCamera, UiGlobalTransform};
use bevy::window::PrimaryWindow;
use blitz_traits::events::{BlitzPointerEvent, BlitzPointerId, BlitzWheelDelta, BlitzWheelEvent, MouseEventButton, MouseEventButtons, PointerCoords, PointerDetails, UiEvent};

use raea_runtime_twins::bridge::{PageElement, Twins};
use raea_sys_page::view3d::ElementImages;
use raea_sys_page::{Docs, Page};

mod route;
pub use route::*;

#[cfg(test)]
mod tests;

/// Where a page is shown, for picking: on this entity's UI node, its box the page.
#[derive(Component, Clone, Copy, Debug)]
pub struct PageView {
    pub page: Entity,
    /// a pointer over the page's background (its root or body element) isn't the page's: it passes to whatever is
    /// under it (a HUD over a game); otherwise the page takes it, as a browser's page does
    pub pass_through: bool,
    /// keys with nothing focused go to this page (an app whose UI is the page; keys.rs), not to the game
    pub keys_when_unfocused: bool,
}

/// A mesh this backend picks, as Bevy's mesh_picking would: it's hit where a ray meets its triangles, and blocks a
/// page behind it (B3 H11: a card passing behind a 3D object).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PickableMesh;

impl PageView {
    pub fn new(page: Entity) -> Self {
        PageView { page, pass_through: false, keys_when_unfocused: false }
    }
}

pub struct PickPlugin;

impl Plugin for PickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Routes>()
            .init_resource::<ViewRays>()
            .init_resource::<raea_sys_page::view3d::ElementImages>()
            .add_systems(PreUpdate, view_rays.after(PickingSystems::ProcessInput).before(PickingSystems::Backend))
            .add_systems(PreUpdate, pick_pages.in_set(PickingSystems::Backend))
            .add_systems(PreUpdate, route.in_set(PickingSystems::Hover).in_set(raea_sys_page::PageSystems::Events).after(bevy::picking::hover::generate_hovermap).before(bevy::picking::events::pointer_events));
    }
}

/// `location` on a UI view, in its page's CSS px; None when the view's camera doesn't draw to the pointer's target.
/// Not clipped to the view: a captured pointer's point past the page's edge is one too.
fn ui_point(location: &Location, page: &Page, node: &ComputedNode, transform: &UiGlobalTransform, camera: Option<Entity>, cameras: &Query<(&Camera, &RenderTarget)>, window: Option<Entity>) -> Option<(Entity, Vec2, bool)> {
    let cam = camera?;
    let (camera, target) = cameras.get(cam).ok()?;
    if target.normalize(window) != Some(location.target.clone()) || node.size() == Vec2::ZERO {
        return None;
    }
    let mut at = location.position * camera.target_scaling_factor().unwrap_or(1.0);
    if let Some(viewport) = camera.physical_viewport_rect() {
        at -= viewport.min.as_vec2();
    }
    let inside = node.contains_point(*transform, at);
    // (-0.5, -0.5) at the node's top left, (0.5, 0.5) at its bottom right, as bevy_ui's backend measures it
    let n = transform.inverse().transform_point2(at) / node.size();
    Some((cam, (n + 0.5) * page.size.as_vec2(), inside))
}

/// The twin of the element at `at` in `page` (its nearest element up with a twin), and whether it's the page's
/// background (the root or body element).
fn element_at(docs: &Docs, twins: &Twins, page: Entity, at: Vec2) -> Option<(Entity, bool)> {
    let doc = docs.0.get(&page)?;
    let mut node = doc.element_from_point(at.x, at.y)?;
    loop {
        if let Some(e) = twins.of(page, node) {
            let background = doc.get_node(node).and_then(|n| n.element_data()).is_some_and(|el| matches!(&*el.name.local, "html" | "body"));
            return Some((e, background));
        }
        node = doc.get_node(node)?.parent?;
    }
}

/// The rays [`view_rays`] added to RayMap this frame, each with its hits' order: a scene seen in an element sorts above
/// the page showing it.
#[derive(Resource, Default)]
pub struct ViewRays(HashMap<RayId, f32>);

/// Where `at` in `page` falls on an element showing a Bevy Image (ElementImages: a camera's target, H7): the image and
/// the point's UV in its content box (the placeholder's).
fn image_at(docs: &Docs, shows: &ElementImages, page: Entity, at: Vec2) -> Option<(Handle<Image>, Vec2)> {
    let doc = docs.0.get(&page)?;
    let mut id = doc.element_from_point(at.x, at.y)?;
    loop {
        let node = doc.get_node(id)?;
        if let Some(image) = shows.0.get(&(page, id)) {
            let l = node.final_layout();
            let abs = node.absolute_position(0.0, 0.0);
            let min = Vec2::new(abs.x, abs.y) + Vec2::new(l.border.left + l.padding.left, l.border.top + l.padding.top);
            let size = Vec2::new(l.content_box_width(), l.content_box_height());
            return (size.min_element() > 0.0).then(|| (image.clone(), (at - min) / size));
        }
        id = node.parent?;
    }
}

/// A ray through a camera's picture (H6, H7's views): each pointer over an element showing a camera's target gets
/// that camera's ray through the same point of its picture in RayMap, so [`pick_pages`] (and any other ray backend)
/// picks the scene in the element, above the page showing it. Pages on UI nodes and on meshes both; one level (a
/// scene's own pages showing cameras aren't followed).
#[allow(clippy::too_many_arguments)]
pub fn view_rays(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(&Camera, &RenderTarget)>,
    image_cameras: Query<(Entity, &Camera, &RenderTarget, &GlobalTransform)>,
    window: Query<Entity, With<PrimaryWindow>>,
    views: Query<(&PageView, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera, Option<&InheritedVisibility>)>,
    meshes_views: Query<(&PageView, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>)>,
    meshes: Option<Res<Assets<Mesh>>>,
    pages: Query<&Page>,
    docs: NonSend<Docs>,
    shows: Res<ElementImages>,
    mut rays: ResMut<RayMap>,
    mut orders: ResMut<ViewRays>,
) {
    orders.0.clear();
    if shows.0.is_empty() {
        return;
    }
    // (pointer, page, its point there, the order of the page's hits)
    let mut points = vec![];
    let window = window.single().ok();
    for (pointer, location) in &pointers {
        let Some(location) = location.location() else { continue };
        for (view, node, transform, camera, visible) in &views {
            let Ok(page) = pages.get(view.page) else { continue };
            if visible.is_some_and(|v| !v.get()) {
                continue;
            }
            let Some((cam, at, true)) = ui_point(location, page, node, transform, camera.get(), &cameras, window) else { continue };
            points.push((*pointer, view.page, at, cameras.get(cam).map(|(c, _)| c.order).unwrap_or_default() as f32 + 0.5));
        }
    }
    if let Some(meshes) = &meshes {
        for (id, ray) in rays.iter() {
            let Ok((camera, _)) = cameras.get(id.camera) else { continue };
            for (view, mesh, transform, visible) in &meshes_views {
                if !camera.is_active || visible.is_some_and(|v| !v.get()) {
                    continue;
                }
                let (Some(mesh), Ok(page)) = (meshes.get(&mesh.0), pages.get(view.page)) else { continue };
                let Some((_, uv)) = cast(mesh, transform, *ray) else { continue };
                points.push((id.pointer, view.page, uv * page.size.as_vec2(), camera.order as f32));
            }
        }
    }
    for (pointer, page, at, order) in points {
        let Some((image, uv)) = image_at(&docs, &shows, page, at) else { continue };
        for (cam, camera, target, transform) in &image_cameras {
            if !camera.is_active || !matches!(target, RenderTarget::Image(t) if t.handle == image) {
                continue;
            }
            // the picture's UV as NDC (y up), near plane to far, as Camera::viewport_to_world takes a point
            let ndc = Vec2::new(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
            let (Some(near), Some(far)) = (camera.ndc_to_world(transform, ndc.extend(1.0)), camera.ndc_to_world(transform, ndc.extend(f32::EPSILON))) else { continue };
            let Ok(dir) = Dir3::new(far - near) else { continue };
            let id = RayId::new(cam, pointer);
            rays.map.insert(id, Ray3d::new(near, dir));
            orders.0.insert(id, order + 0.25);
        }
    }
}

/// The backend: each pointer over a page's view hits the twin of the element under it.
#[allow(clippy::too_many_arguments)]
pub fn pick_pages(
    pointers: Query<(&PointerId, &PointerLocation)>,
    cameras: Query<(&Camera, &RenderTarget)>,
    window: Query<Entity, With<PrimaryWindow>>,
    views: Query<(&PageView, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera, Option<&InheritedVisibility>)>,
    meshes_views: Query<(&PageView, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>)>,
    solids: Query<(Entity, &Mesh3d, &GlobalTransform, Option<&InheritedVisibility>), (With<PickableMesh>, Without<PageView>)>,
    meshes: Option<Res<Assets<Mesh>>>,
    rays: Res<RayMap>,
    view_rays: Res<ViewRays>,
    pages: Query<&Page>,
    docs: NonSend<Docs>,
    twins: Res<Twins>,
    mut out: MessageWriter<PointerHits>,
) {
    // pages on meshes: each camera's ray for each pointer, cast against the views' triangles (an App without meshes
    // has none)
    for (id, ray) in meshes.is_some().then(|| rays.iter()).into_iter().flatten() {
        let meshes = meshes.as_ref().unwrap();
        let Ok((camera, _)) = cameras.get(id.camera) else { continue };
        if !camera.is_active {
            continue;
        }
        // a ray through a camera's picture in an element sorts above the page showing it
        let order = view_rays.0.get(id).copied().unwrap_or(camera.order as f32);
        for (view, mesh, transform, visible) in &meshes_views {
            if visible.is_some_and(|v| !v.get()) {
                continue;
            }
            let (Some(mesh), Ok(page)) = (meshes.get(&mesh.0), pages.get(view.page)) else { continue };
            let Some((depth, uv)) = cast(mesh, transform, *ray) else { continue };
            let at = uv * page.size.as_vec2();
            let Some((twin, background)) = element_at(&docs, &twins, view.page, at) else { continue };
            if background && view.pass_through {
                continue;
            }
            out.write(PointerHits::new(id.pointer, vec![(twin, HitData::new(id.camera, depth, Some(at.extend(0.0)), None))], order));
        }
        // 3D objects, sorted with the pages by depth
        for (e, mesh, transform, visible) in &solids {
            if visible.is_some_and(|v| !v.get()) {
                continue;
            }
            let Some(mesh) = meshes.get(&mesh.0) else { continue };
            let Some((depth, _)) = cast(mesh, transform, *ray) else { continue };
            out.write(PointerHits::new(id.pointer, vec![(e, HitData::new(id.camera, depth, Some(ray.get_point(depth)), None))], order));
        }
    }
    let window = window.single().ok();
    for (pointer, location) in &pointers {
        let Some(location) = location.location() else { continue };
        for (view, node, transform, camera, visible) in &views {
            if visible.is_some_and(|v| !v.get()) {
                continue;
            }
            let Ok(page) = pages.get(view.page) else { continue };
            let Some((cam, at, true)) = ui_point(location, page, node, transform, camera.get(), &cameras, window) else { continue };
            let Some((twin, background)) = element_at(&docs, &twins, view.page, at) else { continue };
            if background && view.pass_through {
                continue;
            }
            let order = cameras.get(cam).map(|(c, _)| c.order).unwrap_or_default() as f32 + 0.5;
            out.write(PointerHits::new(*pointer, vec![(twin, HitData::new(cam, 0.0, Some(at.extend(0.0)), None))], order));
        }
    }
}

/// `location` in `page`'s CSS px through its first UI view that draws to the pointer's target.
fn view_point(world: &mut World, page: Entity, location: &Location, pointer: Option<PointerId>) -> Option<Vec2> {
    let window = world.query_filtered::<Entity, With<PrimaryWindow>>().iter(world).next();
    let at = *world.get::<Page>(page)?;
    let mut state = bevy::ecs::system::SystemState::<(Query<(&PageView, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera)>, Query<(&Camera, &RenderTarget)>)>::new(world);
    let Ok((views, cameras)) = state.get(world) else { return None };
    if let Some(p) = views.iter().filter(|v| v.0.page == page).find_map(|(_, node, transform, camera)| ui_point(location, &at, node, transform, camera.get(), &cameras, window).map(|p| p.1)) {
        return Some(p);
    }
    // on a mesh: where this frame's ray meets it (a held pointer off the mesh keeps its last point)
    let pointer = pointer?;
    let mut state = bevy::ecs::system::SystemState::<(Query<(&PageView, &Mesh3d, &GlobalTransform)>, Option<Res<Assets<Mesh>>>, Res<RayMap>)>::new(world);
    let Ok((views, Some(meshes), rays)) = state.get(world) else { return None };
    rays.iter().filter(|(id, _)| id.pointer == pointer).find_map(|(_, ray)| {
        views.iter().filter(|v| v.0.page == page).find_map(|(_, mesh, transform)| cast(meshes.get(&mesh.0)?, transform, *ray).map(|(_, uv)| uv * at.size.as_vec2()))
    })
}

/// The nearest point where `ray` (world space) meets `mesh` (a triangle list, either face) placed at `transform`: its
/// distance along the ray and its UV.
pub fn cast(mesh: &Mesh, transform: &GlobalTransform, ray: Ray3d) -> Option<(f32, Vec2)> {
    let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return None };
    // (a mesh with no UVs is still hit: an object in front of a page; its UV is zero)
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uvs)) => Some(uvs),
        _ => None,
    };
    if mesh.primitive_topology() != bevy::mesh::PrimitiveTopology::TriangleList {
        return None;
    }
    // the ray in the mesh's space: distances along it stay the world's (an affine map keeps the parameter)
    let inv = transform.affine().inverse();
    let (o, d) = (inv.transform_point3(ray.origin), inv.transform_vector3(*ray.direction));
    let index: Vec<usize> = match mesh.indices() {
        Some(Indices::U16(i)) => i.iter().map(|i| *i as usize).collect(),
        Some(Indices::U32(i)) => i.iter().map(|i| *i as usize).collect(),
        None => (0..pos.len()).collect(),
    };
    let mut best: Option<(f32, Vec2)> = None;
    for tri in index.chunks_exact(3) {
        let [a, b, c] = [tri[0], tri[1], tri[2]].map(|i| Vec3::from(pos[i]));
        // Moller-Trumbore
        let (e1, e2) = (b - a, c - a);
        let p = d.cross(e2);
        let det = e1.dot(p);
        if det.abs() < 1e-9 {
            continue;
        }
        let t0 = o - a;
        let u = t0.dot(p) / det;
        let q = t0.cross(e1);
        let v = d.dot(q) / det;
        let t = e2.dot(q) / det;
        if u < 0.0 || v < 0.0 || u + v > 1.0 || t < 0.0 || best.is_some_and(|b| b.0 <= t) {
            continue;
        }
        let uv = uvs.map(|uvs| {
            let [ua, ub, uc] = [tri[0], tri[1], tri[2]].map(|i| Vec2::from(uvs[i]));
            ua * (1.0 - u - v) + ub * u + uc * v
        });
        best = Some((t, uv.unwrap_or_default()));
    }
    best
}

fn button(b: PointerButton) -> (MouseEventButton, MouseEventButtons) {
    match b {
        PointerButton::Primary => (MouseEventButton::Main, MouseEventButtons::Primary),
        PointerButton::Secondary => (MouseEventButton::Secondary, MouseEventButtons::Secondary),
        PointerButton::Middle => (MouseEventButton::Auxiliary, MouseEventButtons::Auxiliary),
    }
}

fn coords(at: Vec2) -> PointerCoords {
    PointerCoords { page_x: at.x, page_y: at.y, screen_x: at.x, screen_y: at.y, client_x: at.x, client_y: at.y }
}

fn pointer_event(pointer: PointerId, at: Vec2, button: MouseEventButton, buttons: MouseEventButtons, mods: keyboard_types::Modifiers) -> BlitzPointerEvent {
    let id = match pointer {
        PointerId::Touch(n) => BlitzPointerId::Finger(n),
        _ => BlitzPointerId::Mouse,
    };
    BlitzPointerEvent {
        id,
        is_primary: true,
        coords: coords(at),
        button,
        buttons,
        mods,
        details: PointerDetails { pressure: if buttons.is_empty() { 0.0 } else { 0.5 }, ..Default::default() },
        element: Default::default(),
        active_pointers: Default::default(),
    }
}

/// The modifier keys held now.
fn mods(world: &World) -> keyboard_types::Modifiers {
    use keyboard_types::Modifiers;
    let mut m = Modifiers::empty();
    if let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() {
        m.set(Modifiers::SHIFT, keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]));
        m.set(Modifiers::CONTROL, keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]));
        m.set(Modifiers::ALT, keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]));
        m.set(Modifiers::META, keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]));
    }
    m
}
