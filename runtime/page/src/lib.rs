//! raea's 2D page on bevy_ui (docs/ts-native.feature P26): the composition's HTML layer (its divs, text,
//! Tailwind classes and styles, images and videos) becomes raea-runtime-js elements in a Bevy App of its own, laid out by
//! Taffy, its text Bevy's, drawn into an image a frame at a time and read back. No DOM and no Blitz.
//!
//! The composition still runs on raea's React (ReactPlugin, its own App): each frame, the page is made to match the
//! React tree as it stands after the frame's commits and its <Animate> samples (`sync`): new elements created, changed
//! props and text updated, gone ones removed, children put in React's order. Those are raea-runtime-js's ops, applied by its
//! host to a fed root (`UiRoot::fed`), so the page is exactly what @raea/ui's React draws. A frame depends only on the
//! tree, so a frame drawn alone is the frame drawn in sequence.
//!
//! As Blitz's page had it, the elements go in a body the page's size whose face is Inter (the page's default), the
//! space's brand kit first as a `<style>`; a composition's own `<style>`s are elements like any other (raea-runtime-js's
//! css.rs reads them). 3D isn't drawn here: its rows go to raea-render as before, and the elements holding it are made see
//! through (`sync`'s `holders`), so the page lies over it. It is raea's one page: renders read it back, live
//! play draws it on the process's shared device and shows its texture.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use raea_core_wire::tree::{VideoAt, VideoFit, Want};
use serde_json::{Map, Value};

use raea_runtime_react::{ReactKind, ReactNode, ReactProps, ReactText};

mod bevy_page;
mod ui_material;
pub use bevy_page::*;
mod walk;
pub use walk::*;
mod elements;
pub(crate) use elements::*;
mod world;
mod world_input;
pub use world::*;
// app screens (`<Screen app>`): a component on @raea/ui's react-dom in a raea-runtime-js root of its own
pub mod screen;
// a take's face track (<Video framing="face">)
pub(crate) mod faces;

type R<T> = Result<T, String>;

/// The id of an element's own text (an element whose only child is a string carries it on the wire; on the page it is
/// a text child of its own).
const TEXT_BIT: u64 = 1 << 62;

/// The page's default face, as Blitz's page and the Chrome oracle have it.
const DEFAULT_FAMILY: &str = "Inter, sans-serif";

/// The page's own elements take the ids below REACT_IDS and React's are moved up past them: raea-runtime-js's css.rs merges a
/// root's stylesheets in id order, so the page's sheet (BASE) and the brand kit (BRAND) come before a composition's own
/// (its Tailwind preflight then overrides them, as in a browser where they'd come first in the document).
const BASE: u64 = 1;
const BRAND: u64 = 2;
const BODY: u64 = 3;
/// live play's 3D under the whole page (`set_under`): the body's first element, under every other (z-index -1)
const UNDER: u64 = 4;
const REACT_IDS: u64 = 16;

/// The page's own sheet, first so any other (a composition's Tailwind preflight, the brand kit) overrides it: the
/// line height and box-sizing a page without preflight has (Blitz's, a browser's), not Tailwind's 1.5 and border-box.
/// In a layer declared before every other, as a browser's own sheet is: unlayered, it would beat preflight's
/// `@layer base` in a real cascade (stylo's, on the Blitz page), whatever the order; raea-runtime-js reads layers in order.
const BASE_CSS: &str = "@layer raea-ua { html { line-height: normal } *, ::before, ::after { box-sizing: content-box } }";

/// raea in any Bevy App with one add_plugins (docs/sdk.feature S8b): raea's Engine plugins (RaeaPlugins, Scene3dPlugin),
/// React running the composition bundle in this App's World (ReactPlugin), its <Canvas3D> tree made into the scene
/// (ScenePlugin) drawn to `target`, and its page drawn on Blitz over it (raea-page's PagePlugin, WorldPagePlugin) unless
/// `scene_only`. examples/plain_page is one, and bench/bridge checks it against raea's own render. The rest of its fields
/// are set as Bevy's plugins' are, by struct update:
///
///   app.add_plugins((DefaultPlugins, RaeaPlugin { frame: Some(30), ..RaeaPlugin::new("EDIT.js", target, (1280, 720)) }));
pub struct RaeaPlugin {
    pub bundle: PathBuf,
    pub target: bevy::camera::RenderTarget,
    pub size: (u32, u32),
    /// held at one frame (a still), or None to step one frame each Update from 0
    pub frame: Option<i64>,
    pub fps: f64,
    /// the 3D only, no page
    pub scene_only: bool,
}

impl RaeaPlugin {
    pub fn new(bundle: impl Into<PathBuf>, target: bevy::camera::RenderTarget, size: (u32, u32)) -> Self {
        RaeaPlugin { bundle: bundle.into(), target, size, frame: None, fps: 30.0, scene_only: false }
    }
}

impl Plugin for RaeaPlugin {
    fn build(&self, app: &mut App) {
        use raea_render::engine::raea_plugins::{RaeaPlugins, Scene3dPlugin};
        let dir = self.bundle.parent().map(std::path::Path::to_path_buf).unwrap_or_default();
        app.add_plugins((RaeaPlugins, Scene3dPlugin, raea_runtime_react::ReactPlugin::new(self.bundle.clone())));
        app.add_plugins(raea_runtime_react::scene::ScenePlugin { target: self.target.clone(), size: self.size, dir: dir.clone(), frame: self.frame });
        if !self.scene_only {
            app.add_plugins((raea_page::PagePlugin, WorldPagePlugin { size: self.size, dir, fps: self.fps }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use raea_runtime_twins::bridge::dom;
    use serde_json::json;
    use std::sync::mpsc::channel;
    use std::sync::Mutex;

    /// H8's handler step: a DOM event delivered to a React element's twin, and its parent's, reaches each one's handler
    /// on React's outbox; an element without the prop, or one on a page that isn't React's, sends nothing.
    #[test]
    fn a_dom_event_on_a_react_pages_element_goes_to_its_handler() {
        let mut app = App::new();
        let (tx, rx) = channel();
        app.add_plugins(MinimalPlugins).insert_resource(raea_runtime_react::ReactOutbox(Mutex::new(tx)));
        handlers(&mut app);
        let (page, other) = (app.world_mut().spawn_empty().id(), app.world_mut().spawn_empty().id());
        app.world_mut().resource_mut::<ReactPages>().0.push(page);
        let props = |keys: &[&str]| ReactProps(keys.iter().map(|k| (k.to_string(), Value::Bool(true))).collect());
        app.world_mut().spawn((ReactNode(7), props(&["onFocus"])));
        app.world_mut().spawn((ReactNode(8), props(&["onFocus", "onPointerDown"])));
        app.world_mut().spawn((ReactNode(9), props(&[])));
        let twin = |app: &mut App, page, op| app.world_mut().spawn(raea_runtime_twins::PageElement { page, node: dom::NodeId::from_u64(op), op: Some(op) }).id();
        let (parent, button, quiet, elsewhere) = (twin(&mut app, page, 7), twin(&mut app, page, 8), twin(&mut app, page, 9), twin(&mut app, other, 8));
        // (focus: the one DOM event whose data is plain to make here; every kind is routed by the same macro)
        let mut focus = |at| app.world_mut().trigger(dom::Focus { entity: at, target: button, data: dom::BlitzFocusEvent, default_prevented: false, propagation_stopped: false });
        for at in [button, parent, quiet, elsewhere] {
            focus(at);
        }
        let got: Vec<Value> = rx.try_iter().map(|s| serde_json::from_str(&s).unwrap()).collect();
        let ev = |id: u64| json!({"t": "uiEvent", "event": {"id": id, "kind": "focus"}});
        assert_eq!(got, vec![ev(8), ev(7)]);
    }
}
