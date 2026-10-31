//! One focus for pages and the world (docs/blitz-bevy.feature B3 H3, H6): Bevy's `InputFocus` is the page's focus
//! when a twin has it, and keys go where it is.
//!
//! - Focus both ways: a page that moves its focus (a click into a field, Tab) sets `InputFocus` to the element's twin
//!   (or clears it, if a twin of that page had it); Bevy code that sets `InputFocus` to a twin focuses its element, and
//!   to anything else (a game's entity, or nothing) blurs every page. FocusGained and FocusLost then fire as Bevy's.
//! - Keys: the window's key and IME events go to the page whose twin has focus, as UiEvents (host.rs's WindowInput:
//!   the DOM's key names, the modifiers held). With nothing focused they go to a page whose view asks for them
//!   ([`crate::pick::PageView::keys_when_unfocused`]: an app whose UI is a page, as a browser's page gets them), and
//!   otherwise to no page: the game has them.
//! - IME: on while a text field has focus, its candidate window at the field's bottom left (through its view).

use std::collections::HashMap;

use bevy::ecs::message::MessageCursor;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::{PrimaryWindow, WindowEvent};
use blitz_dom::NodeId;

use raea_runtime_twins::bridge::{ElementBox, PageElement, Twins};

use raea_sys_page::PageSystems;
use raea_runtime_dom::host::WindowInput;
use crate::pick::PageView;
use raea_sys_page::{Docs, Page};

pub struct KeysPlugin;

impl Plugin for KeysPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>()
            .init_resource::<FocusSync>()
            .add_systems(PreUpdate, (keys, focus_from_pages).chain().after(bevy::picking::PickingSystems::Last).in_set(PageSystems::Events))
            .add_systems(PostUpdate, (focus_to_pages.in_set(PageSystems::Commands), ime.after(PageSystems::Twins)))
            .add_observer(page_takes_focus);
    }
}

/// A press on a twin asks Bevy for the focus (bevy_input_focus's PointerFocusPlugin triggers AcquireFocus, which
/// bubbles to the window and clears the focus there when no TabIndex takes it). A page decides its own focus (Blitz's
/// default action for the press, then focus_from_pages), so the request stops at the twin: without this, an app with
/// PointerFocusPlugin blurred every element a click had just focused (ui-parity's use-state-counter on void, 10-25).
fn page_takes_focus(mut ev: On<bevy::input_focus::AcquireFocus>, twins: Query<(), With<PageElement>>) {
    if twins.contains(ev.focused_entity) {
        ev.propagate(false);
    }
}

/// The focus both sides last agreed on: `InputFocus`, and each page's focused element.
#[derive(Resource, Default)]
pub struct FocusSync {
    bevy: Option<Entity>,
    pages: HashMap<Entity, Option<NodeId>>,
}

/// Where the keys go now: the page of the focused twin, or with nothing focused a view's page that takes them.
fn key_page(world: &mut World) -> Option<Entity> {
    // a window's focus is no element's: bevy_input_focus starts with the primary window focused, and focuses it again
    // for a click on nothing focusable
    match world.resource::<InputFocus>().get().filter(|e| world.get::<Window>(*e).is_none()) {
        Some(e) => world.get::<PageElement>(e).map(|el| el.page),
        None => world.query::<&PageView>().iter(world).find(|v| v.keys_when_unfocused).map(|v| v.page),
    }
}

/// The window's key and IME events this frame, in order, to the page that has the keys.
pub fn keys(world: &mut World, mut cursor: Local<MessageCursor<WindowEvent>>, mut at: Local<WindowInput>) {
    let evs: Vec<WindowEvent> = cursor.read(world.resource::<Messages<WindowEvent>>()).filter(|e| matches!(e, WindowEvent::KeyboardInput(_) | WindowEvent::Ime(_))).cloned().collect();
    for ev in evs {
        // the modifiers are kept whoever gets the key
        let Some(ui) = at.event(&ev) else { continue };
        if let Some(page) = key_page(world) {
            crate::pick::send(world, page, ui);
        }
    }
}

/// The element `doc` has focused, if one is: Blitz answers the root element when none is (as
/// `document.activeElement` is the body), which isn't a focus here.
pub fn focused(doc: &blitz_dom::BaseDocument) -> Option<NodeId> {
    doc.get_focussed_node_id().filter(|n| Some(*n) != doc.try_root_element().map(|e| e.id))
}

/// Each page's focused element now.
fn page_focus(docs: &Docs) -> HashMap<Entity, Option<NodeId>> {
    docs.0.iter().map(|(p, d)| (*p, focused(d))).collect()
}

/// A page that moved its focus this frame (its input: a press, Tab) moves `InputFocus` with it.
pub fn focus_from_pages(world: &mut World) {
    let now = page_focus(&world.non_send::<Docs>());
    let mut sync = std::mem::take(&mut *world.resource_mut::<FocusSync>());
    let pressed = world.get_resource::<ButtonInput<MouseButton>>().is_some_and(|b| b.just_pressed(MouseButton::Left));
    for (page, node) in &now {
        if sync.pages.get(page) == Some(node) {
            continue;
        }
        let twin = node.and_then(|n| world.resource::<Twins>().of(*page, n));
        // the page blurred: if one of its twins had the focus, nothing has it now
        let had = world.resource::<InputFocus>().get().is_some_and(|f| world.get::<PageElement>(f).is_some_and(|el| el.page == *page));
        let mut focus = world.resource_mut::<InputFocus>();
        match twin {
            Some(t) => focus.set(t, if pressed { FocusCause::Pressed } else { FocusCause::Navigated }),
            None if had => focus.clear(),
            None => {}
        }
        sync.bevy = world.resource::<InputFocus>().get();
    }
    sync.pages = now;
    *world.resource_mut::<FocusSync>() = sync;
}

/// `InputFocus` set by Bevy code since: its twin's element focused, every other page's blurred.
pub fn focus_to_pages(focus: Res<InputFocus>, mut sync: ResMut<FocusSync>, mut docs: NonSendMut<Docs>, twins: Query<&PageElement>) {
    let want = focus.get();
    if want == sync.bevy {
        return;
    }
    // the focused twin's element went (a closed dialog's field) and bevy_input_focus cleared the focus it no longer
    // finds: that's the page's own change, not Bevy code blurring the pages (a script may have focused another element
    // already, as Radix gives the focus back to a dialog's trigger)
    if want.is_none() && sync.bevy.is_some_and(|e| twins.get(e).is_err()) {
        sync.bevy = None;
        return;
    }
    let target = want.and_then(|e| twins.get(e).ok()).map(|el| (el.page, el.node));
    for (page, doc) in docs.0.iter_mut() {
        match target {
            Some((p, n)) if p == *page => {
                if focused(doc) != Some(n) {
                    doc.set_focus_to(n);
                }
            }
            _ => {
                if focused(doc).is_some() {
                    doc.clear_focus();
                }
            }
        }
    }
    sync.bevy = want;
    sync.pages = page_focus(&docs);
}

/// The window's IME on while a focused twin is a text field, its candidate window at the field's bottom left.
#[allow(clippy::type_complexity)]
pub fn ime(focus: Res<InputFocus>, docs: NonSend<Docs>, twins: Query<(&PageElement, &ElementBox)>, views: Query<(&PageView, &ComputedNode, &UiGlobalTransform)>, pages: Query<&Page>, mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let field = focus.get().and_then(|e| twins.get(e).ok()).filter(|(el, _)| {
        docs.0.get(&el.page).and_then(|d| d.get_node(el.node)).and_then(|n| n.element_data()).is_some_and(|e| e.text_input_data().is_some())
    });
    let Ok(mut window) = windows.single_mut() else { return };
    let on = field.is_some();
    if window.ime_enabled != on {
        window.ime_enabled = on;
    }
    let Some((el, b)) = field else { return };
    let (Some((_, node, transform)), Ok(page)) = (views.iter().find(|v| v.0.page == el.page), pages.get(el.page)) else { return };
    // the field's bottom left, page px to the view's node (physical px) to the window (logical px)
    let n = Vec2::new(b.0.min.x, b.0.max.y) / page.size.as_vec2() - 0.5;
    let at = transform.transform_point2(n * node.size()) / window.scale_factor();
    if window.ime_position != at {
        window.ime_position = at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raea_runtime_twins::bridge::{BridgePlugin, PageQuery};
    use crate::pick::{PageView, PickPlugin};
    use bevy::camera::{NormalizedRenderTarget, RenderTarget};
    use bevy::ecs::system::RunSystemOnce;
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
    use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerId, PointerInput};
    use bevy::reflect::structs::Struct;
    use bevy::ui::ComputedUiTargetCamera;
    use bevy::window::WindowRef;

    /// A page with a field shown on a UI box, picking with the page's backend, and keys by focus.
    fn app() -> (App, Entity, Entity, NormalizedRenderTarget) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::input::InputPlugin, bevy::picking::PickingPlugin, bevy::picking::InteractionPlugin, BridgePlugin::<()>::default(), PickPlugin, KeysPlugin))
            .add_message::<WindowEvent>()
            .init_non_send::<Docs>()
            .init_resource::<raea_sys_page::PageClock>()
            .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
        let window = app.world_mut().spawn((Window::default(), PrimaryWindow)).id();
        let cam = app.world_mut().spawn((Camera::default(), RenderTarget::Window(WindowRef::Primary))).id();
        let at = Page { size: UVec2::new(400, 300), scale: 1.0 };
        let page = app.world_mut().spawn(at).id();
        let html = r#"<body style="margin:0"><input id="f" style="display:block;width:200px;height:30px"></body>"#;
        app.world_mut().non_send_mut::<Docs>().0.insert(page, blitz_html::HtmlDocument::from_html(html, raea_sys_page::config(at)).into_inner());
        let mut target = ComputedUiTargetCamera::default();
        target.field_mut("camera").unwrap().apply(&Some(cam));
        app.world_mut().spawn((PageView::new(page), ComputedNode { size: Vec2::new(400.0, 300.0), ..default() }, UiGlobalTransform::from_translation(Vec2::new(200.0, 150.0)), target));
        app.world_mut().spawn(PointerId::Mouse);
        app.update();
        let to = NormalizedRenderTarget::Window(WindowRef::Primary.normalize(Some(window)).unwrap());
        (app, page, window, to)
    }

    fn key(app: &mut App, window: Entity, c: &str) {
        for state in [ButtonState::Pressed, ButtonState::Released] {
            let k = KeyboardInput { key_code: KeyCode::KeyA, logical_key: Key::Character(c.into()), state, text: (state == ButtonState::Pressed).then(|| c.into()), repeat: false, window };
            app.world_mut().write_message(WindowEvent::KeyboardInput(k));
        }
        app.update();
    }

    fn value(app: &App, page: Entity, f: Entity) -> String {
        let node = app.world().get::<PageElement>(f).unwrap().node;
        app.world().non_send::<Docs>().0[&page].get_node(node).unwrap().element_data().unwrap().text_input_data().unwrap().editor.raw_text().to_string()
    }

    #[test]
    fn focus_is_one_and_keys_follow_it() {
        let (mut app, page, window, to) = app();
        let f = app.world_mut().run_system_once(move |q: PageQuery| q.by_id(page, "f")).unwrap().unwrap();
        // nothing focused: keys go to no page (the view doesn't take unfocused keys), the game has them
        key(&mut app, window, "z");
        assert_eq!(value(&app, page, f), "");
        // a click into the field: the page focuses it, and InputFocus is its twin
        for action in [PointerAction::Move { delta: Vec2::ZERO }, PointerAction::Press(PointerButton::Primary), PointerAction::Release(PointerButton::Primary)] {
            app.world_mut().write_message(PointerInput::new(PointerId::Mouse, Location { target: to.clone(), position: Vec2::new(20.0, 10.0) }, action));
            app.update();
        }
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(f));
        assert!(app.world_mut().query::<&Window>().single(app.world()).unwrap().ime_enabled);
        // typing reaches the field
        key(&mut app, window, "a");
        key(&mut app, window, "b");
        assert_eq!(value(&app, page, f), "ab");
        // the game takes the focus: the page blurs, and keys stop reaching it
        let game = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<InputFocus>().set(game, FocusCause::Navigated);
        app.update();
        assert_eq!(focused(&app.world().non_send::<Docs>().0[&page]), None);
        key(&mut app, window, "c");
        assert_eq!(value(&app, page, f), "ab");
        // Bevy focuses the twin: the page focuses its element
        app.world_mut().resource_mut::<InputFocus>().set(f, FocusCause::Navigated);
        app.update();
        let node = app.world().get::<PageElement>(f).unwrap().node;
        assert_eq!(focused(&app.world().non_send::<Docs>().0[&page]), Some(node));
        key(&mut app, window, "d");
        assert_eq!(value(&app, page, f), "abd");
    }
}
