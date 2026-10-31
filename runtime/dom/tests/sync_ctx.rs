//! A context host call (raea_runtime_js::calls::on_sync_ctx) answered from the calling root's state (raea-page's PageSync):
//! called from the bundle as it mounts and from a timer in a step, it has the root's page, its document and what the
//! JS flushed just before (wire.callSync flushes first), which it can commit then; called with nothing lent, it's an
//! error. Its own binary: the
//! handlers are the process's.

use std::sync::Mutex;

use bevy::prelude::*;
use raea_runtime_dom::host::{self, HostStepPlugin, PageSync};
use raea_sys_page::{Docs, PageClock, PageSystems};
use serde_json::{Value, json};

static SEEN: Mutex<Vec<Value>> = Mutex::new(vec![]);

fn seen(ctx: &mut dyn std::any::Any, args: &Value) -> Result<Value, String> {
    let p = ctx.downcast_mut::<PageSync>().ok_or("not a page's call")?;
    let flushed = p.out.borrow().flushed.len();
    let doc = p.doc().is_some();
    // committed now, as style.computed does before it resolves: the element is in the page
    p.commit();
    let made = p.nodes().by_id.contains_key(&1);
    let v = json!({ "at": args["at"], "doc": doc, "flushed": flushed, "made": made, "page": p.page.to_bits() });
    SEEN.lock().unwrap().push(v.clone());
    Ok(v)
}

#[test]
fn a_context_call_gets_its_page() {
    raea_runtime_js::calls::on_sync_ctx("test.seen", seen);
    let outside: Value = serde_json::from_str(&raea_runtime_js::calls::sync("test.seen", "{}")).unwrap();
    assert_eq!(outside["err"], "test.seen: called outside a step");

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, HostStepPlugin))
        .init_non_send::<Docs>()
        .init_resource::<PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let code = r#"
        __ui_flush(JSON.stringify({ v: 1, ops: [{ op: "create", id: 1, kind: "div", props: {} }, { op: "append", parent: 0, child: 1 }] }));
        __ui_sync("test.seen", JSON.stringify({ at: "mount" }));
        // (a timer set in the bundle runs as the mount's first render is taken; the one it sets a millisecond on runs in a step, as that run took its now first)
        setTimeout(() => setTimeout(() => __ui_sync("test.seen", JSON.stringify({ at: "timer" })), 1), 0);
    "#;
    let page = host::mount(app.world_mut(), 200, 100, code, &[], None).unwrap();
    for _ in 0..4 {
        app.update();
    }
    let seen = SEEN.lock().unwrap().clone();
    let at = |w: &str| seen.iter().find(|v| v["at"] == w).cloned().unwrap_or_else(|| panic!("no {w} call in {seen:?}"));
    // the mount's: inside the bundle, its create flushed and not yet committed, until the call committed it
    assert_eq!(at("mount"), json!({ "at": "mount", "doc": true, "flushed": 1, "made": true, "page": page.to_bits() }));
    // a step's timer: the mount's committed, nothing pending
    assert_eq!(at("timer"), json!({ "at": "timer", "doc": true, "flushed": 0, "made": true, "page": page.to_bits() }));
}
