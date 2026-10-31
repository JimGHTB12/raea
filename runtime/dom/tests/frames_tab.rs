//! Tab through a scripted <iframe> (docs/blitz-bevy.feature B5c, iframes 4): as in Chrome,
//! the iframe is no stop of its own. A Tab onto it focuses the frame's first element, Tab past the frame's last goes on
//! to the parent's next element, and Shift+Tab from there comes back to the frame's last. Its own binary: the env var
//! and blitz-dom's FRAME_NET are the process's.

use bevy::prelude::*;
use blitz_dom::local_name;
use blitz_traits::events::{BlitzKeyEvent, KeyState, UiEvent};
use keyboard_types::{Code, Key, Location, Modifiers, NamedKey};
use raea_runtime_dom::host::{self, HostStepPlugin};
use raea_sys_page::{Docs, PageClock, PageSystems};

/// a window's Event and location for the raw pages (frames.js and frame.js use them)
const WINDOW: &str = r#"
  globalThis.Event = class { constructor(type) { this.type = type; } };
  globalThis.dispatchEvent = () => {};
  globalThis.location = { href: "http://localhost/" };
"#;

fn tab(back: bool) -> UiEvent {
    UiEvent::KeyDown(BlitzKeyEvent {
        key: Key::Named(NamedKey::Tab),
        code: Code::Tab,
        modifiers: if back { Modifiers::SHIFT } else { Modifiers::empty() },
        location: Location::Standard,
        is_auto_repeating: false,
        is_composing: false,
        state: KeyState::Pressed,
        text: None,
    })
}

/// the id of what has the focus: the parent's element, or inside the iframe (the parent's focus on it) the frame's
fn focused(app: &App, page: Entity) -> Option<String> {
    let docs = app.world().non_send::<Docs>();
    let doc = docs.0.get(&page)?;
    let node = doc.get_node(doc.get_focussed_node_id()?)?;
    if let Some(sub) = node.subdoc() {
        let inner = sub.inner();
        let n = inner.get_node(inner.get_focussed_node_id()?)?;
        return n.element_data()?.attr(local_name!("id")).map(|s| format!("frame:{s}"));
    }
    node.element_data()?.attr(local_name!("id")).map(Into::into)
}

#[test]
fn tab_goes_into_a_frame_and_out_again() {
    let root = std::env::temp_dir().join(format!("raea-frames-tab-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("frame.html"), r#"<!doctype html><html><body><script src="/frame.js"></script></body></html>"#).unwrap();
    // the frame: two buttons
    std::fs::write(
        root.join("frame.js"),
        format!(
            r#"{WINDOW}
            __raea_frame.installed();
            __ui_flush(JSON.stringify({{ v: 1, ops: [{{ op: "create", id: 1, kind: "button", props: {{ id: "x" }} }}, {{ op: "append", parent: 0, child: 1 }}, {{ op: "create", id: 2, kind: "button", props: {{ id: "y" }} }}, {{ op: "append", parent: 0, child: 2 }}] }}));
            "#
        ),
    )
    .unwrap();
    // SAFETY: this binary's one test, set before anything reads them
    unsafe {
        std::env::set_var("RAEA_PAGE_IFRAMES", "1");
        std::env::set_var("RAEA_PAGE_ROOT", &root);
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, HostStepPlugin))
        .init_non_send::<Docs>()
        .init_resource::<PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    // the parent: a button, the frame, a button
    let parent = format!(
        r#"{WINDOW}
        __ui_flush(JSON.stringify({{ v: 1, ops: [{{ op: "create", id: 1, kind: "button", props: {{ id: "a" }} }}, {{ op: "append", parent: 0, child: 1 }}, {{ op: "create", id: 2, kind: "iframe", props: {{ src: "/frame.html", sandbox: "allow-scripts", style: {{ width: "200px", height: "100px" }} }} }}, {{ op: "append", parent: 0, child: 2 }}, {{ op: "create", id: 3, kind: "button", props: {{ id: "b" }} }}, {{ op: "append", parent: 0, child: 3 }}] }}));
        "#
    );
    let page = host::mount(app.world_mut(), 400, 300, &parent, &[], None).unwrap();
    for _ in 0..8 {
        app.update();
    }

    let mut seen = vec![];
    for back in [false, false, false, false, true, true] {
        host::input(app.world_mut(), page, tab(back));
        app.update();
        seen.push(focused(&app, page).unwrap_or_else(|| "none".into()));
    }
    assert_eq!(seen, ["a", "frame:x", "frame:y", "b", "frame:y", "frame:x"]);

    std::fs::remove_dir_all(&root).unwrap();
}
