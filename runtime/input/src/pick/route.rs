//! Each pointer routed to the page it's over or held on: presses counted, events sent through Blitz.

use super::*;

/// Each pointer's route: the page it's over, the page a press holds it on, its buttons, its last point there, and
/// its presses' count (a double click's 2).
#[derive(Resource, Default)]
pub struct Routes(pub(crate) HashMap<PointerId, Route>);

#[derive(Default)]
pub(crate) struct Route {
    pub(crate) over: Option<Entity>,
    pub(crate) held: Option<Entity>,
    pub(crate) buttons: MouseEventButtons,
    pub(crate) at: Vec2,
    pub(crate) last_press: Option<(f64, Vec2)>,
    pub(crate) clicks: u32,
    /// the page the last pointermove went to, at `at`: a move that changes neither sends none, as Chrome sends none
    pub(crate) moved_on: Option<Entity>,
}

/// The press count the app says (a double click's 2, as its platform counts it: void's injector, AppKit): when
/// present, a press's detail is this, not the route's own count.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PressCount(pub u32);

/// How far apart, in seconds and CSS px, two presses still count as one double click (macOS's default interval).
pub(crate) const DOUBLE: (f64, f32) = (0.5, 4.0);

/// The frame's pointer input, to the pages it's over or held on (see the module's notes).
pub fn route(world: &mut World, mut cursor: Local<MessageCursor<PointerInput>>) {
    let inputs: Vec<PointerInput> = cursor.read(world.resource::<Messages<PointerInput>>()).cloned().collect();
    // each pointer's picked page and point, from the hover map (a twin's hit's position is its page point)
    let mut over: HashMap<PointerId, (Entity, Vec2)> = HashMap::new();
    {
        let mut twins = world.query::<&PageElement>();
        let hover = world.resource::<HoverMap>();
        for (pointer, hits) in hover.iter() {
            for (e, hit) in hits {
                if let (Ok(el), Some(p)) = (twins.get(world, *e), hit.position) {
                    over.insert(*pointer, (el.page, p.truncate()));
                }
            }
        }
    }
    let mods = mods(world);
    let now = world.resource::<Time>().elapsed_secs_f64();
    let mut routes = std::mem::take(&mut world.resource_mut::<Routes>().0);
    // pointers that left a page with no input this frame (the page moved under them) leave it too
    for (pointer, r) in routes.iter_mut() {
        let here = over.get(pointer).map(|o| o.0);
        if r.held.is_none() && r.over.is_some() && r.over != here && !inputs.iter().any(|i| i.pointer_id == *pointer) {
            // still inside the page's view, over something that isn't a twin's hit this frame (an overlay a commit
            // just made): it hasn't left, and Blitz's own hover follows the page under it (refresh_hover)
            if here.is_none() && inside_view(world, r.over.unwrap(), *pointer) {
                continue;
            }
            let left = r.over.take().unwrap();
            send(world, left, UiEvent::PointerLeave(pointer_event(*pointer, r.at, MouseEventButton::Main, r.buttons, mods)));
        }
    }
    for input in inputs {
        let r = routes.entry(input.pointer_id).or_default();
        let here = over.get(&input.pointer_id).copied();
        // the page it's over now: a held pointer stays on its page
        if r.held.is_none() && r.over != here.map(|h| h.0) {
            if let Some(left) = r.over {
                send(world, left, UiEvent::PointerLeave(pointer_event(input.pointer_id, r.at, MouseEventButton::Main, r.buttons, mods)));
            }
            r.over = here.map(|h| h.0);
            r.moved_on = None;
        }
        let Some(page) = r.held.or(r.over) else { continue };
        let was = r.at;
        r.at = match here {
            Some((p, at)) if p == page => at,
            _ => view_point(world, page, &input.location, Some(input.pointer_id)).unwrap_or(r.at),
        };
        let ev = match input.action {
            // a move to where the pointer already is (an injected press's move first, a window's repeated cursor
            // event) isn't one
            PointerAction::Move { .. } if r.moved_on == Some(page) && was == r.at => continue,
            PointerAction::Move { .. } => {
                r.moved_on = Some(page);
                UiEvent::PointerMove(pointer_event(input.pointer_id, r.at, MouseEventButton::Main, r.buttons, mods))
            }
            PointerAction::Press(b) => {
                let (button, bit) = button(b);
                r.buttons.insert(bit);
                r.held = Some(page);
                r.clicks = match r.last_press {
                    Some((t, at)) if now - t <= DOUBLE.0 && at.distance(r.at) <= DOUBLE.1 => r.clicks + 1,
                    _ => 1,
                };
                r.last_press = Some((now, r.at));
                if let Some(n) = world.get_resource::<PressCount>() {
                    r.clicks = n.0;
                }
                raea_runtime_dom::host::set_clicks(world, page, r.clicks);
                UiEvent::PointerDown(pointer_event(input.pointer_id, r.at, button, r.buttons, mods))
            }
            PointerAction::Release(b) => {
                let (button, bit) = button(b);
                r.buttons.remove(bit);
                if r.buttons.is_empty() {
                    r.held = None;
                }
                UiEvent::PointerUp(pointer_event(input.pointer_id, r.at, button, r.buttons, mods))
            }
            PointerAction::Scroll { unit, x, y, .. } => UiEvent::Wheel(BlitzWheelEvent {
                delta: match unit {
                    bevy::input::mouse::MouseScrollUnit::Line => BlitzWheelDelta::Lines(x as f64, y as f64),
                    bevy::input::mouse::MouseScrollUnit::Pixel => BlitzWheelDelta::Pixels(x as f64, y as f64),
                },
                coords: coords(r.at),
                buttons: r.buttons,
                mods,
                element: Default::default(),
            }),
            PointerAction::Cancel => {
                r.held = None;
                r.buttons = MouseEventButtons::None;
                UiEvent::PointerCancel(pointer_event(input.pointer_id, r.at, MouseEventButton::Main, r.buttons, mods))
            }
        };
        send(world, page, ev);
        // a release over another page (or none) leaves the page it was held on; over no twin's hit but inside the
        // page's view (an overlay its commit just made), it stays
        if r.held.is_none() && r.over != here.map(|h| h.0) && !(here.is_none() && r.over.is_some_and(|p| inside_view(world, p, input.pointer_id))) {
            if let Some(left) = r.over {
                send(world, left, UiEvent::PointerLeave(pointer_event(input.pointer_id, r.at, MouseEventButton::Main, r.buttons, mods)));
            }
            r.over = here.map(|h| h.0);
        }
    }
    world.resource_mut::<Routes>().0 = routes;
}

/// `ev` to `page`: a React page's host takes it (its handlers, then the twins'); a page without one (an HTML page)
/// is dispatched here, with Blitz's default actions and the twins' events.
pub fn send(world: &mut World, page: Entity, ev: UiEvent) {
    if world.get_non_send::<raea_runtime_dom::host::Roots>().is_some_and(|r| r.0.iter().any(|r| r.page == page)) {
        return raea_runtime_dom::host::input(world, page, ev);
    }
    let Some(mut doc) = world.non_send_mut::<Docs>().0.remove(&page) else { return };
    let mut fired = vec![];
    blitz_dom::EventDriver::new(&mut doc, raea_runtime_twins::bridge::Forward { inner: Plain, fired: &mut fired }).handle_ui_event(ev);
    world.non_send_mut::<Docs>().0.insert(page, doc);
    raea_runtime_twins::bridge::deliver::<raea_runtime_dom::host::Roots>(world, page, fired);
}

/// A page with no script's handler: nothing but Blitz's default actions.
pub(crate) struct Plain;

impl blitz_dom::EventHandler for Plain {
    fn handle_event(&mut self, _: &[blitz_dom::NodeId], _: &mut blitz_traits::events::DomEvent, _: &mut dyn blitz_dom::Document, _: &mut blitz_traits::events::EventState) {}
}

/// Whether `pointer` is inside one of `page`'s UI views now.
pub(crate) fn inside_view(world: &mut World, page: Entity, pointer: PointerId) -> bool {
    let window = world.query_filtered::<Entity, With<PrimaryWindow>>().iter(world).next();
    let Some(at) = world.get::<Page>(page).copied() else { return false };
    let location = world.query::<(&PointerId, &PointerLocation)>().iter(world).find(|(p, _)| **p == pointer).and_then(|(_, l)| l.location().cloned());
    let Some(location) = location else { return false };
    let mut state = bevy::ecs::system::SystemState::<(Query<(&PageView, &ComputedNode, &UiGlobalTransform, &ComputedUiTargetCamera)>, Query<(&Camera, &RenderTarget)>)>::new(world);
    let Ok((views, cameras)) = state.get(world) else { return false };
    views.iter().filter(|v| v.0.page == page).any(|(_, node, transform, camera)| ui_point(&location, &at, node, transform, camera.get(), &cameras, window).is_some_and(|p| p.2))
}
