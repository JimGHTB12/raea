//! T3d: where a source is, and an image by URL through raea's URL cache (a local server, so the test needs no
//! network): fetched once, then read from the cache with the server gone.

use std::io::{Read, Write};
use std::path::Path;

use raea_rune_media::images::{Images, Load};
use raea_rune_media::src::{Src, resolve};

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[test]
fn sources_resolve_as_a_browser_reads_them() {
    let base = Path::new("/app");
    assert_eq!(resolve("hero.png", Some(base)), Ok(Src::File("/app/hero.png".into())));
    assert_eq!(resolve("./media/a%20b.mp4?v=2", Some(base)), Ok(Src::File("/app/./media/a b.mp4".into())));
    assert_eq!(resolve("file:///tmp/x.png", None), Ok(Src::File("/tmp/x.png".into())));
    assert_eq!(resolve("https://a.example/x.png", None), Ok(Src::Url("https://a.example/x.png".into())));
    assert_eq!(resolve("data:text/plain,hi%21", None), Ok(Src::Bytes(b"hi!".to_vec())));
    assert!(resolve("blob:abc", Some(base)).is_err() && resolve("x.png", None).is_err() && resolve("", Some(base)).is_err());
}

#[test]
fn an_image_by_url_comes_through_the_cache() {
    let cache = std::env::temp_dir().join(format!("t3d-cache-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&cache);
    // SAFETY: set before anything in this test binary fetches
    unsafe { std::env::set_var("RAEA_CACHE", &cache) };
    let png = raea_io_net::data_url(&format!("data:image/png;base64,{PNG}")).unwrap().1;
    let server = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/dot.png", server.local_addr().unwrap());
    let serve = std::thread::spawn(move || {
        let (mut s, _) = server.accept().unwrap();
        let mut req = [0u8; 1024];
        let _ = s.read(&mut req);
        s.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", png.len()).as_bytes()).unwrap();
        s.write_all(&png).unwrap();
    });
    // headless: fetched where it's asked for, then decoded by Rune
    let images = Images::default();
    let Load::Image(i) = images.show(7, &url) else { panic!("the URL loads") };
    assert_eq!(i.wait().ok(), Some([1, 1]));
    serve.join().unwrap();
    assert_eq!(images.news_out(), [(7, serde_json::json!({"type": "load", "w": 1, "h": 1}))]);
    assert!(images.news_out().is_empty(), "told once");
    // a second page reads it from the cache: the server is gone, and offline refuses only what isn't cached
    unsafe { std::env::set_var("RAEA_OFFLINE", "1") };
    assert!(matches!(Images::default().get(&url), Load::Image(_)));
    assert!(matches!(Images::default().get("http://127.0.0.1:9/never.png"), Load::Failed(_)));
    let _ = std::fs::remove_dir_all(&cache);
}
