//! raea's window on the feed (docs/app.feature AP5/AP6): with RAEA_PAGE_HOST set, the page's /host/* goes to that host
//! with its method, path and query, and so does a /media/* that isn't on disk; a file under the root is still the root's,
//! and a host that doesn't answer is a 502, not a hang.

use std::io::{Read, Write};
use std::net::TcpListener;

use raea_sys_page_net::fetch as page_fetch;
use serde_json::json;

/// A host that answers each request with its own request line as the body.
fn host() -> String {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let at = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for s in l.incoming() {
            let mut s = s.unwrap();
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap();
            let line = String::from_utf8_lossy(&buf[..n]).lines().next().unwrap_or("").to_string();
            let _ = write!(s, "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{line}", line.len());
        }
    });
    at
}

fn body(v: &serde_json::Value) -> String {
    use base64::Engine as _;
    String::from_utf8(base64::engine::general_purpose::STANDARD.decode(v["body"].as_str().unwrap()).unwrap()).unwrap()
}

#[test]
fn the_page_reaches_its_host() {
    let root = std::env::temp_dir().join(format!("raea-page-host-{}", std::process::id()));
    std::fs::create_dir_all(root.join("media")).unwrap();
    std::fs::write(root.join("media/on-disk.wav"), "RIFF").unwrap();
    unsafe { std::env::set_var("RAEA_PAGE_ROOT", &root) };
    let fetch = |url: &str, method: &str| page_fetch(&json!({ "url": url, "method": method })).unwrap();

    // no host: /host/* is a missing file, as before
    unsafe { std::env::remove_var("RAEA_PAGE_HOST") };
    assert_eq!(fetch("http://localhost/host/acp/events?after=3", "GET")["status"], 404);

    unsafe { std::env::set_var("RAEA_PAGE_HOST", host()) };
    let r = fetch("http://localhost/host/acp/events?after=3", "GET");
    assert_eq!(r["status"], 200);
    assert_eq!(body(&r), "GET /host/acp/events?after=3 HTTP/1.1");
    assert_eq!(body(&fetch("http://localhost/host/acp/act", "POST")), "POST /host/acp/act HTTP/1.1");
    // a render made after the page loaded is the host's; one on disk is the root's
    assert_eq!(body(&fetch("http://localhost/media/feed/new.mp4", "GET")), "GET /media/feed/new.mp4 HTTP/1.1");
    assert_eq!(body(&fetch("http://localhost/media/on-disk.wav", "GET")), "RIFF");

    // a host that's gone
    let gone = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    unsafe { std::env::set_var("RAEA_PAGE_HOST", format!("http://{gone}")) };
    assert_eq!(fetch("http://localhost/host/acp/act", "POST")["status"], 502);

    std::fs::remove_dir_all(&root).unwrap();
}
