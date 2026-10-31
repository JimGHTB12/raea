//! What a scripted <iframe> costs (docs/blitz-bevy.feature B5c, iframes 5): budgeted at 20 MB footprint and 1 ms a
//! frame idle each. The frame is apps/app's artifact page as its Blitz build makes it (blitz/out/artifact.html and
//! artifact.js, from `BLITZ_ENTRY=frame vite build -c blitz/vite.config.ts`, which ui-parity app runs), on the
//! artifact screen's item (i-ask). Two pages in one process, one with no iframes and one with FRAMES, stepped in turn
//! so the machine's load falls on both alike: the time is the medians' difference, the footprint (footprint(1)) what
//! mounting the frames' page added. No paint: a frame's encode is its document's, and nothing when it isn't dirty.
//!
//!   cargo test -p raea-page --release --test frames_budget -- --ignored --nocapture
//!
//! Ignored by default: it needs the app's build, and its times want a release build.

use bevy::prelude::*;
use raea_runtime_dom::host::{self, HostStepPlugin};
use raea_sys_page::{Docs, PageClock, PageSystems};
use std::time::Instant;

const FRAMES: usize = 4;
const STEPS: usize = 300;

const WINDOW: &str = r#"
  globalThis.Event = class { constructor(type) { this.type = type; } };
  globalThis.dispatchEvent = () => {};
  globalThis.location = { href: "http://localhost/" };
"#;

/// this process's footprint in MB, as footprint(1) gives it
fn footprint_mb() -> f64 {
    let o = std::process::Command::new("footprint").arg(std::process::id().to_string()).output().expect("footprint(1)");
    let s = String::from_utf8_lossy(&o.stdout);
    let at = s.find("Footprint: ").expect("footprint's line") + "Footprint: ".len();
    let mut w = s[at..].split_whitespace();
    let n: f64 = w.next().unwrap().parse().unwrap();
    match w.next().unwrap() {
        "KB" => n / 1024.0,
        "MB" => n,
        "GB" => n * 1024.0,
        u => panic!("footprint's unit {u}"),
    }
}

fn app_with(frames: usize) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, HostStepPlugin))
        .init_non_send::<Docs>()
        .init_resource::<PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    let ops: Vec<String> = (0..frames)
        .map(|i| {
            let id = i + 1;
            format!(r#"{{ op: "create", id: {id}, kind: "iframe", props: {{ src: "/artifact.html?doc=pick-one&item=i-ask", sandbox: "allow-scripts", style: {{ width: "600px", height: "400px" }} }} }}, {{ op: "append", parent: 0, child: {id} }}"#)
        })
        .collect();
    let code = format!("{WINDOW}\n__ui_flush(JSON.stringify({{ v: 1, ops: [{}] }}));", ops.join(", "));
    let page = host::mount(app.world_mut(), 1280, 1800, &code, &[], None).unwrap();
    (app, page)
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

#[test]
#[ignore]
fn a_scripted_frame_keeps_to_its_budget() {
    let app_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/app");
    let out = app_dir.join("blitz/out");
    assert!(out.join("artifact.js").exists(), "build the artifact frame first: BLITZ_ENTRY=frame vite build -c blitz/vite.config.ts");
    let roots = std::env::join_paths([out, app_dir.join("public")]).unwrap();
    // SAFETY: this binary's one test, set before anything reads them
    unsafe {
        std::env::set_var("RAEA_PAGE_IFRAMES", "1");
        std::env::set_var("RAEA_PAGE_ROOT", &roots);
    }

    let (mut bare, _) = app_with(0);
    for _ in 0..30 {
        bare.update();
    }
    let before = footprint_mb();
    let (mut framed, page) = app_with(FRAMES);
    // loaded, mounted and settled: the artifact's first render and its effects
    for _ in 0..60 {
        framed.update();
    }
    let mounted = framed.world().non_send::<Docs>().0.len() - 1;
    assert_eq!(mounted, FRAMES, "each iframe has its frame root's document");
    // and the artifact rendered in each (its document under the iframe, between the frame's steps)
    let elements: Vec<usize> = {
        let docs = framed.world().non_send::<Docs>();
        let doc = &docs.0[&page];
        doc.sub_document_node_ids().iter().map(|n| doc.get_node(*n).and_then(|n| n.subdoc()).map_or(0, |d| d.inner().query_selector_all("*").map_or(0, |v| v.len()))).collect()
    };
    assert!(elements.iter().all(|n| *n > 20), "an artifact frame didn't render: {elements:?} elements");
    let after = footprint_mb();

    let (mut a, mut b) = (vec![], vec![]);
    for _ in 0..STEPS {
        let t = Instant::now();
        bare.update();
        a.push(t.elapsed().as_secs_f64() * 1e3);
        let t = Instant::now();
        framed.update();
        b.push(t.elapsed().as_secs_f64() * 1e3);
    }
    let (ma, mb) = (median(a), median(b));
    let per_ms = (mb - ma) / FRAMES as f64;
    let per_mb = (after - before) / FRAMES as f64;
    eprintln!(
        "{{\"frames\": {FRAMES}, \"elements\": {elements:?}, \"step_ms_bare\": {ma:.3}, \"step_ms_framed\": {mb:.3}, \"idle_ms_per_frame\": {per_ms:.3}, \"footprint_mb_before\": {before:.1}, \"footprint_mb_after\": {after:.1}, \"footprint_mb_per_frame\": {per_mb:.1}}}"
    );
    assert!(per_mb <= 20.0, "a scripted frame's footprint {per_mb:.1} MB is over its 20 MB");
    assert!(per_ms <= 1.0, "a scripted frame's idle {per_ms:.3} ms a frame is over its 1 ms");
}
