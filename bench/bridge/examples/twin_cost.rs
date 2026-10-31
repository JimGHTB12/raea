//! B3 H1's cost (docs/blitz-bevy.feature; bench/bridge/run.sh): what the twins cost a frame on a page of the legacy app's size.
//! A React page in a plain App (no GPU: restyled, laid out and twinned each update, as PagePlugin does) of feed cards,
//! each a card, its title, a row of two spans and a button, four of them with their text: 6 elements, 10 nodes. The
//! twins system (PageSystems::Twins) is timed and its allocations counted (raea-runtime-react's counting allocator) over
//! frames where nothing changed and frames where the first card moves (every box below it with it).
//!
//!   twin_cost [ELEMENTS...]   default 50 (the legacy app's empty workspace: 51 bevy_ui nodes), 500 and 2000
//!
//! One line per size and kind of frame, then `twin_cost: PASS` when the legacy app's size with a feed (500) stays within
//! BUDGET_US and BUDGET_ALLOCS a frame on frames where nothing changed.
use bevy::prelude::*;
use raea_sys_page::{Docs, PageSystems};
use raea_core_alloc as zeroheap;
use std::time::{Duration, Instant};

#[global_allocator]
static A: zeroheap::Counting = zeroheap::Counting;

/// a frame where nothing changed, at 500 elements
const BUDGET_US: f64 = 250.0;
const BUDGET_ALLOCS: f64 = 0.0;
const WARM: usize = 10;
const FRAMES: usize = 120;

#[derive(Resource, Default)]
struct Span {
    at: Option<(Instant, u64)>,
    us: Vec<f64>,
    allocs: Vec<u64>,
}

struct Feed {
    nodes: raea_sys_page::ops::Nodes,
    page: Entity,
    moving: bool,
    frame: u64,
}

fn main() {
    // counting on before the first allocation that matters (zeroheap::init reads it)
    unsafe { std::env::set_var("RAEA_ZEROHEAP", "1") };
    zeroheap::init();
    let sizes: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let sizes = if sizes.is_empty() { vec![50, 500, 2000] } else { sizes };
    let mut ok = true;
    for n in sizes {
        for moving in [false, true] {
            let (us, allocs) = run(n, moving);
            let kind = if moving { "the first card moving" } else { "nothing changed" };
            let gate = n == 500 && !moving;
            let pass = us <= BUDGET_US && allocs <= BUDGET_ALLOCS;
            if gate {
                ok &= pass;
            }
            println!(
                "twin_cost: {n} elements, {kind}: {us:.1} µs and {allocs:.1} allocations a frame (median of {FRAMES}){}",
                if !gate { String::new() } else if pass { format!("  (budget {BUDGET_US} µs, {BUDGET_ALLOCS} allocations)") } else { format!("  <- over the budget ({BUDGET_US} µs, {BUDGET_ALLOCS} allocations)") }
            );
        }
    }
    println!("twin_cost: {}", if ok { "PASS" } else { "FAIL" });
    std::process::exit(if ok { 0 } else { 1 });
}

/// The twins system's median µs and mean allocations a frame on a page of `n` elements.
fn run(n: usize, moving: bool) -> (f64, f64) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(raea_runtime_twins::bridge::BridgePlugin::<()>::default())
        .init_non_send::<Docs>()
        .init_resource::<raea_sys_page::PageClock>()
        .init_resource::<Span>();
    app.add_systems(Update, feed)
        .add_systems(PostUpdate, (restyle.in_set(PageSystems::Restyle), begin.after(PageSystems::Layout).before(PageSystems::Twins), end.after(PageSystems::Twins).before(PageSystems::Paint)));
    let fonts = raea_sys_fonts::context(&raea_io_home::home().join("core/fonts"));
    let (page, mut nodes) = raea_sys_page::open_doc(app.world_mut(), 1440, 900, Some(fonts));
    let mut ops = vec![serde_json::json!({ "op": "create", "id": 1, "kind": "div", "props": { "style": { "display": "flex", "flexDirection": "column", "gap": "8px", "padding": "16px" } } })];
    let mut id = 2u64;
    let mut el = |ops: &mut Vec<serde_json::Value>, parent: u64, kind: &str, style: serde_json::Value, text: Option<&str>| {
        let me = id;
        id += 1;
        ops.push(serde_json::json!({ "op": "create", "id": me, "kind": kind, "props": { "style": style } }));
        ops.push(serde_json::json!({ "op": "append", "parent": parent, "child": me }));
        if let Some(t) = text {
            let tid = id;
            id += 1;
            ops.push(serde_json::json!({ "op": "create", "id": tid, "kind": "text", "text": t }));
            ops.push(serde_json::json!({ "op": "append", "parent": me, "child": tid }));
        }
        me
    };
    for c in 0..n.div_ceil(6) {
        let card = el(&mut ops, 1, "div", serde_json::json!({ "padding": "12px", "borderRadius": "8px", "background": "rgb(30, 32, 44)" }), None);
        el(&mut ops, card, "div", serde_json::json!({ "fontSize": "16px", "color": "white" }), Some(&format!("Item {c}")));
        let row = el(&mut ops, card, "div", serde_json::json!({ "display": "flex", "gap": "6px" }), None);
        el(&mut ops, row, "span", serde_json::json!({ "fontSize": "12px" }), Some("video"));
        el(&mut ops, row, "span", serde_json::json!({ "fontSize": "12px" }), Some("0:30"));
        el(&mut ops, card, "button", serde_json::json!({ "fontSize": "12px" }), Some("Open"));
    }
    ops.push(serde_json::json!({ "op": "append", "parent": 0, "child": 1 }));
    let ops: Vec<raea_core_wire::Op> = serde_json::from_value(serde_json::Value::Array(ops)).unwrap();
    raea_sys_page::ops::apply(app.world_mut().non_send_mut::<Docs>().0.get_mut(&page).unwrap(), &mut nodes, ops);
    app.insert_non_send(Feed { nodes, page, moving, frame: 0 });
    app.finish();
    app.cleanup();
    // one thread, so the counts around the twins system are its own
    raea_runtime_js::one_thread(&mut app);
    for _ in 0..WARM {
        app.update();
    }
    let twins = app.world_mut().query::<&raea_runtime_twins::PageElement>().iter(app.world()).filter(|e| e.page == page).count();
    assert!(twins >= n, "{twins} twins for {n} elements");
    app.world_mut().resource_mut::<Span>().us.clear();
    app.world_mut().resource_mut::<Span>().allocs.clear();
    for _ in 0..FRAMES {
        app.update();
    }
    let span = app.world_mut().remove_resource::<Span>().unwrap();
    let mut us = span.us.clone();
    us.sort_by(f64::total_cmp);
    (us[us.len() / 2], span.allocs.iter().sum::<u64>() as f64 / span.allocs.len() as f64)
}

/// the first card's top margin, one px each frame back and forth: every card below it moves
fn feed(mut feed: NonSendMut<Feed>, mut docs: NonSendMut<Docs>) {
    if !feed.moving {
        return;
    }
    feed.frame += 1;
    let at = feed.frame % 2;
    let op: raea_core_wire::Op = serde_json::from_value(serde_json::json!({ "op": "update", "id": 2, "set": { "style": { "padding": "12px", "borderRadius": "8px", "background": "rgb(30, 32, 44)", "marginTop": format!("{at}px") } } })).unwrap();
    let page = feed.page;
    raea_sys_page::ops::apply(docs.0.get_mut(&page).unwrap(), &mut feed.nodes, vec![op]);
}

/// as raea-page's resolve: restyled and laid out
fn restyle(mut docs: NonSendMut<Docs>) {
    for doc in docs.0.values_mut() {
        doc.resolve(0.0);
    }
}

fn begin(mut s: ResMut<Span>) {
    s.at = Some((Instant::now(), zeroheap::here().0));
}

fn end(mut s: ResMut<Span>) {
    let (t, a) = s.at.take().unwrap();
    let (dt, da): (Duration, u64) = (t.elapsed(), zeroheap::here().0 - a);
    s.us.push(dt.as_secs_f64() * 1e6);
    s.allocs.push(da);
}
