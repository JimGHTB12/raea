//! T2b and T2d (docs/targets.feature): the native host answers net.* and the system calls as the page host does.
//! io/net/calls' recorded calls (a fetch, storage set and get, a socket's open, send and close) and io/os's (os.open,
//! the clipboard, files picked and saved, dialogs; headless, the answers queued) made through this host's pump get the
//! recorded answers; engine/page/host tests/net_calls.rs checks the web target against the same records.
use std::time::{Duration, Instant};

use raea_io_net_calls::serve;
use raea_io_os::recorded;
use raea_rune_host::js::App;

/// `script` run until it logs its record.
fn record(script: &str) -> (App, String) {
    let mut app = App::new(script, (320.0, 240.0)).unwrap();
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let found = app.logs.borrow().iter().find_map(|l| l.strip_prefix("0: recorded ").map(str::to_string));
        if let Some(r) = found {
            break (app, r);
        }
        assert!(Instant::now() < until, "no record after 20 s; the page logged {:?}", app.logs.borrow());
        // the answers come from the calls' threads: a frame pumps whatever has arrived
        std::thread::sleep(Duration::from_millis(5));
        app.frame().unwrap();
    }
}

// one test: the storage folder and headless mode are the process's
#[test]
fn the_recorded_calls_get_the_recorded_answers() {
    let dir = std::env::temp_dir().join(format!("raea-rune-calls-{}", std::process::id()));
    // SAFETY: this test binary's only test, set before any thread reads it
    unsafe { std::env::set_var("BEVY_TSX_STORAGE", dir.join("storage")) };
    let servers = serve::serve();
    let (_, net) = record(&servers.script());
    if let Err(e) = servers.check(&net) {
        panic!("net: {e}\nrecorded: {}", net.replace(&servers.http, "HTTP").replace(&servers.ws, "WS"));
    }
    let os = recorded::setup(dir.join("files"));
    let (app, got) = record(os.script());
    let checked = os.check(&got);
    let _ = std::fs::remove_dir_all(&dir);
    checked.unwrap_or_else(|e| panic!("os: {e}\nrecorded: {got}"));
    // the dialog no step answered is listed with the calls this host refused (T2c)
    assert!(app.refused.borrow().contains_key("dialog.message"), "{:?}", app.refused.borrow());
}
