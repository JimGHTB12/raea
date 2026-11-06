//! What Blitz loads from the page's own origin (an @font-face url, an image, a stylesheet named http://localhost/...)
//! is the file under the page's roots, as page_fetch answers the page's fetch; nothing outside a root.

use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request};
use raea_sys_page_net::FileNet;
use std::sync::{Arc, Mutex};

struct Got(Arc<Mutex<Option<Bytes>>>);
impl NetHandler for Got {
    fn bytes(self: Box<Self>, _url: String, bytes: Bytes) {
        *self.0.lock().unwrap() = Some(bytes);
    }
}

fn load(net: &FileNet, url: &str) -> Vec<u8> {
    let got = Arc::new(Mutex::new(None));
    net.fetch(0, Request::get(url::Url::parse(url).unwrap()), Box::new(Got(got.clone())));
    got.lock().unwrap().take().expect("answered at once").to_vec()
}

#[test]
fn own_origin_reads_the_roots() {
    let base = std::env::temp_dir().join(format!("raea-own-origin-{}", std::process::id()));
    let root = base.join("public");
    std::fs::create_dir_all(root.join("fonts")).unwrap();
    std::fs::write(root.join("fonts/face.woff2"), "face").unwrap();
    std::fs::write(base.join("secret.txt"), "secret").unwrap();
    let net = FileNet { roots: vec![root] };

    assert_eq!(load(&net, "http://localhost/fonts/face.woff2"), b"face");
    // missing, or outside the root: empty, never a network request
    assert!(load(&net, "http://localhost/fonts/none.woff2").is_empty());
    assert!(load(&net, "http://localhost/../secret.txt").is_empty());

    std::fs::remove_dir_all(&base).unwrap();
}
