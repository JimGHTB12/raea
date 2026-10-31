//! A plain App's page elements and world layers taking the pointer as a 3D part does (docs/sdk.feature S4a):
//!
//! - Drag and drop on a page element: Bevy picking's drag events, which fire on the elements' twins as they do on a
//!   mesh or a sprite (raea-runtime-input's pick.rs: the page is a picking backend), go to the React element's
//!   onDragStart, onDrag, onDragEnd, onDragEnter, onDragOver, onDragLeave and onDrop, in Bevy's order (the drop
//!   before the drag's end), each naming what's dragged (`dragged`: its element's id, when it's an element).
//! - A page layer in the world (`layer="world"`, a flat <Screen> panel): its mesh is a PageView of the screen's page,
//!   so the pointer reaches the page's elements through the panel's UVs, and their on* handlers run as the page's do.

use super::*;
use bevy::picking::pointer::PointerButton;
use serde_json::json;

pub(crate) fn install(app: &mut App) {
    app.add_systems(Update, panel_views);
    install_routes(app);
}

/// The drag events' observers
fn install_routes(app: &mut App) {
    macro_rules! route {
        ($($ty:ident => $prop:literal;)*) => {$(
            app.add_observer(|ev: On<$ty>, twins: Query<&raea_runtime_twins::PageElement>, react: Query<(&ReactNode, &ReactProps)>, pages: Res<ReactPages>, out: Option<Res<raea_runtime_react::ReactOutbox>>| {
                let mut m = Map::new();
                // the point in the page's CSS px where the backend hit it, else the pointer's
                let at = Hit::hit(&*ev).and_then(|h| h.position).map(|p| p.truncate()).unwrap_or(ev.pointer.position);
                m.insert("x".into(), json!(at.x));
                m.insert("y".into(), json!(at.y));
                m.insert("button".into(), json!(match ev.button { PointerButton::Primary => 0, PointerButton::Middle => 1, PointerButton::Secondary => 2 }));
                if let Some(op) = Hit::dragged(&*ev).and_then(|d| twins.get(d).ok()).and_then(|t| t.op) {
                    m.insert("dragged".into(), json!(op));
                }
                send(&twins, &react, &pages, out.as_deref(), ev.entity, $prop, m);
            });
        )*};
    }
    route! {
        PointerDragStart => "onDragStart";
        PointerDrag => "onDrag";
        PointerDragEnd => "onDragEnd";
        PointerDragEnter => "onDragEnter";
        PointerDragOver => "onDragOver";
        PointerDragLeave => "onDragLeave";
        PointerDragDrop => "onDrop";
    }
}

/// The hit a drag event has (a drag and its end have none of their own), and the entity it carries over another
trait Hit {
    fn hit(&self) -> Option<&bevy::picking::backend::HitData> {
        None
    }
    fn dragged(&self) -> Option<Entity> {
        None
    }
}
impl Hit for PointerDragStart {
    fn hit(&self) -> Option<&bevy::picking::backend::HitData> {
        Some(&self.hit)
    }
}
impl Hit for PointerDrag {}
impl Hit for PointerDragEnd {}
macro_rules! over {
    ($($ty:ident.$dragged:ident),*) => {$(
        impl Hit for $ty {
            fn hit(&self) -> Option<&bevy::picking::backend::HitData> {
                Some(&self.hit)
            }
            fn dragged(&self) -> Option<Entity> {
                Some(self.$dragged)
            }
        }
    )*};
}
over!(PointerDragEnter.dragged, PointerDragOver.dragged, PointerDragLeave.dragged, PointerDragDrop.dropped);

/// The twin's React element's handler `prop`, when it has one and its page is React's
fn send(twins: &Query<&raea_runtime_twins::PageElement>, react: &Query<(&ReactNode, &ReactProps)>, pages: &ReactPages, out: Option<&raea_runtime_react::ReactOutbox>, entity: Entity, prop: &str, mut m: Map<String, Value>) {
    let (Ok(el), Some(out)) = (twins.get(entity), out) else { return };
    let Some(op) = el.op.filter(|_| pages.0.contains(&el.page)) else { return };
    if !react.iter().any(|(n, p)| n.0 == op && p.0.get(prop) == Some(&Value::Bool(true))) {
        return;
    }
    m.insert("id".into(), json!(op));
    m.insert("kind".into(), json!(format!("{}{}", prop[2..3].to_lowercase(), &prop[3..])));
    out.send(json!({"t": "uiEvent", "event": m}));
}

/// Each flat screen panel a PageView of its screen's page, once both are there (and again if its page is remade)
fn panel_views(
    mut commands: Commands,
    wp: Option<Res<WorldPage>>,
    panels: Query<(Entity, &raea_sys_screen::Screen, Option<&raea_runtime_input::pick::PageView>), With<raea_sys_screen::ScreenPanel>>,
    mats: Res<Assets<raea_sys_screen::CrtMaterial>>,
    images: Option<Res<raea_sys_screen::ScreenImages>>,
) {
    let (Some(wp), Some(images)) = (wp, images) else { return };
    for (e, screen, view) in &panels {
        let Some(image) = mats.get(screen.material()).map(|m| m.image().id()) else { continue };
        // the screen whose image the material shows, and that screen's page
        let mut page = None;
        for (name, (hd, _, _)) in &images.0 {
            if hd.id() == image {
                page = wp.screens.get(name).map(|s| s.page);
            }
        }
        let Some(page) = page else { continue };
        if view.is_none_or(|v| v.page != page) {
            commands.entity(e).insert(raea_runtime_input::pick::PageView::new(page));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::{Location, PointerId};
    use std::sync::{mpsc::channel, Mutex};

    /// A card dropped on a bin, both page elements: the bin's onDrop hears it with the card as `dragged`, at the
    /// point in the page where the backend hit it
    #[test]
    fn a_drop_on_a_page_element_goes_to_its_on_drop() {
        let mut app = App::new();
        let (tx, rx) = channel();
        app.add_plugins(MinimalPlugins).insert_resource(raea_runtime_react::ReactOutbox(Mutex::new(tx)));
        let page = app.world_mut().spawn_empty().id();
        app.insert_resource(ReactPages(vec![page]));
        install_routes(&mut app);
        let el = |op| raea_runtime_twins::PageElement { page, node: raea_runtime_twins::bridge::dom::NodeId::from_u64(op), op: Some(op) };
        let props = |keys: &[&str]| ReactProps(keys.iter().map(|k| (k.to_string(), Value::Bool(true))).collect());
        let card = app.world_mut().spawn(el(7)).id();
        let bin = app.world_mut().spawn(el(9)).id();
        app.world_mut().spawn((ReactNode(7), props(&["onDragStart"])));
        app.world_mut().spawn((ReactNode(9), props(&["onDrop"])));
        app.update();
        let at = Location { target: NormalizedRenderTarget::None { width: 100, height: 100 }, position: Vec2::new(500.0, 500.0) };
        let hit = HitData::new(Entity::PLACEHOLDER, 1.0, Some(Vec3::new(40.0, 12.0, 0.0)), None);
        let pointer = bevy::picking::events::Pointer::new(PointerId::Mouse, at);
        app.world_mut().trigger(PointerDragDrop { entity: bin, pointer, button: PointerButton::Primary, dropped: card, hit });
        let got: Vec<Value> = rx.try_iter().map(|s| serde_json::from_str(&s).unwrap()).collect();
        assert_eq!(got, vec![json!({"t": "uiEvent", "event": {"id": 9, "kind": "drop", "x": 40.0, "y": 12.0, "button": 0, "dragged": 7}})]);
    }
}
