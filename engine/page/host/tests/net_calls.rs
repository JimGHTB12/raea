//! T2b and T2d (docs/targets.feature): the page host answers io/net/calls' recorded calls (a fetch, storage set and
//! get, a socket's open, send and close) and io/os's (os.open, the clipboard, files picked and saved, dialogs;
//! headless, the answers queued) with the recorded answers, as engine/rune's tests/net.rs checks the native host does.
use std::time::{Duration, Instant};

use bevy::prelude::*;
use raea_io_net_calls::serve;
use raea_io_os::recorded;
use raea_runtime_dom::host::{self, HostStepPlugin, PageAsks};
use raea_sys_page::{Docs, PageClock, PageSystems};

/// `script` mounted on a page of its own and stepped until it emits its record.
fn record(script: &str) -> String {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, HostStepPlugin))
        .init_non_send::<Docs>()
        .init_resource::<PageClock>()
        .add_systems(PostUpdate, raea_sys_page::resolve.in_set(PageSystems::Restyle));
    host::mount(app.world_mut(), 320, 240, script, &[], None).unwrap();
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        // an emit's payload is its JSON, read: the record itself
        if let Some((_, _, v)) = app.world().resource::<PageAsks>().0.iter().find(|(_, n, _)| n == "recorded") {
            break v.to_string();
        }
        assert!(Instant::now() < until, "no record after 20 s");
        // the answers come from the calls' threads: a step delivers whatever has arrived
        std::thread::sleep(Duration::from_millis(5));
        app.update();
    }
}

// one test: the storage folder and headless mode are the process's
#[test]
fn the_recorded_calls_get_the_recorded_answers() {
    let dir = std::env::temp_dir().join(format!("raea-page-calls-{}", std::process::id()));
    // SAFETY: this test binary's only test, set before any thread reads it
    unsafe { std::env::set_var("BEVY_TSX_STORAGE", dir.join("storage")) };
    let servers = serve::serve();
    let net = record(&servers.script());
    if let Err(e) = servers.check(&net) {
        panic!("net: {e}\nrecorded: {}", net.replace(&servers.http, "HTTP").replace(&servers.ws, "WS"));
    }
    let os = recorded::setup(dir.join("files"));
    let got = record(os.script());
    let checked = os.check(&got);
    let _ = std::fs::remove_dir_all(&dir);
    checked.unwrap_or_else(|e| panic!("os: {e}\nrecorded: {got}"));
}
