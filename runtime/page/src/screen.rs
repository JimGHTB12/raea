//! Screens (`<Screen>` in raea/3d): each frame, every screen's nested composition is laid out in a bevy_ui root
//! of its own at the screen's size (the bevy_ui page's `sync_sub`, src/page.rs), drawn into its own picture, and that
//! picture is copied into the screen's image in raea-render's engine (Engine::copy_screen on the GPU in live play, or handed
//! as RGBA in a render: Engine::set_screen_picture), which draws it as a CRT or a flat panel
//! (engine/render/src/engine/screen.rs). A render's screen equals its play's, and a game on a screen plays live with the
//! player's input (the nested game reads the same useInput).
//!
//! An app screen (`<Screen app="chat.tsx">`) shows a page of its own instead: the module's component on @raea/ui's
//! react-dom in a raea-runtime-js root of its own. In live play the pointer reaches it through the 3D (`route`: the camera's
//! ray to the screen, Engine::screen_at): a press on an app screen gives it the keys, which the game then doesn't get,
//! until a press anywhere else or Escape.

/// What an app's prebuilt bundle is called beside its module (`chat.tsx` -> `chat.tsx.surface.js`): an exported game's
/// (game.rs gather).
pub const BUNDLED: &str = ".surface.js";

/// The page's app screens: the input live play routed to them for their next step.
#[derive(Default)]
pub(crate) struct Apps {
    /// screens whose app didn't open, with the module (said once)
    pub(crate) failed: std::collections::HashMap<String, String>,
    /// the app screen the keys go to (the one last pressed on), and the keys held for it
    focus: Option<String>,
    keys: Vec<String>,
    /// where the pointer is on an app screen (its picture's pixels) and whether it's down
    pointer: Option<(String, f64, f64, bool)>,
    down: bool,
    /// the wheel's turns this frame (`Wheel:Y DY`), for the app screen under the pointer
    wheel: Vec<String>,
}

/// Live play's input split between the page and its app screens: the pointer where it lands on an app screen (the
/// camera's ray through it, Engine::screen_at; `size` is the frame's, the 3D filling it) is that screen's; a press on
/// one gives it the keys until a press anywhere else or Escape, and while it has them the page gets none. The keys
/// the page gets.
pub fn route(page: &mut crate::BevyPage, engine: &mut raea_render::engine::Engine, keys: &[String], pointer: Option<(f64, f64, bool)>, size: (u32, u32)) -> Vec<String> {
    let apps = page.app_screens();
    let a = page.apps();
    if apps.is_empty() {
        a.focus = None;
        a.pointer = None;
        return keys.to_vec();
    }
    let hit = pointer.and_then(|(x, y, _)| engine.screen_at(x / size.0.max(1) as f64, y / size.1.max(1) as f64));
    let on = hit.and_then(|(name, u, v)| apps.iter().find(|s| s.0 == name).map(|s| (name, u * s.1 as f64, v * s.2 as f64)));
    let down = pointer.is_some_and(|p| p.2);
    if down && !a.down {
        a.focus = on.as_ref().map(|o| o.0.clone());
    }
    a.down = down;
    a.pointer = on.map(|(name, x, y)| (name, x, y, down));
    // the wheel over an app screen scrolls it, and the page doesn't get it
    a.wheel.clear();
    let keys: Vec<String> = match &a.pointer {
        Some(_) => keys.iter().filter(|k| !(k.starts_with("Wheel:Y ") && { a.wheel.push(k.to_string()); true })).cloned().collect(),
        None => keys.to_vec(),
    };
    let keys = &keys[..];
    let (mine, page_keys): (Vec<String>, Vec<String>) = keys.iter().cloned().partition(|k| !(k.starts_with("feed:") || k.starts_with("action:") || k.starts_with("space:")));
    match &a.focus {
        Some(_) if mine.iter().any(|k| k == "Escape") => {
            a.focus = None;
            a.keys.clear();
            page_keys
        }
        Some(_) => {
            a.keys = mine;
            page_keys
        }
        None => {
            a.keys.clear();
            keys.to_vec()
        }
    }
}

/// The app screens that went let go of: their focus, pointer and failures forgotten, so a screen back again opens
/// fresh. `screens`: each app screen now, its name, size and module.
pub(crate) fn retire(a: &mut Apps, screens: &[(String, u32, u32, String)]) {
    let now = |name: &str, app: &str| screens.iter().any(|s| s.0 == name && s.3 == app);
    a.failed.retain(|name, app| now(name, app));
    let shown = |name: &str| screens.iter().any(|s| s.0 == name);
    if a.focus.as_deref().is_some_and(|f| !shown(f)) {
        a.focus = None;
        a.keys.clear();
    }
    if a.pointer.as_ref().is_some_and(|p| !shown(&p.0)) {
        a.pointer = None;
    }
}

