//! The page's same-origin fetch (`fetch("/media/x.wav")`, what decodeAudioData reads) answers only from files under
//! RAEA_PAGE_ROOT: a file inside is 200 with its bytes and type, and `..` out of the root (raw or percent-encoded), a
//! link inside it to a file outside, a write and a missing file are 404. Another origin isn't the page's (None, the
//! network's), and with no root there's no page fetch at all.

use raea_sys_page_net::page_fetch;
use serde_json::json;

#[test]
fn the_page_fetches_only_inside_the_root() {
    let base = std::env::temp_dir().join(format!("raea-page-fetch-{}", std::process::id()));
    let root = base.join("public");
    std::fs::create_dir_all(root.join("media")).unwrap();
    std::fs::write(root.join("media/in.wav"), "RIFF").unwrap();
    std::fs::write(base.join("secret.txt"), "secret").unwrap();
    std::os::unix::fs::symlink(base.join("secret.txt"), root.join("link.txt")).unwrap();
    let get = |url: &str| page_fetch(&json!({ "url": url, "method": "GET" }));
    let status = |url: &str| get(url).map(|v| v["status"].as_u64().unwrap());

    // no root: not the page's to answer
    unsafe { std::env::remove_var("RAEA_PAGE_ROOT") };
    assert_eq!(status("http://localhost/media/in.wav"), None);

    unsafe { std::env::set_var("RAEA_PAGE_ROOT", &root) };
    let ok = get("http://localhost/media/in.wav").unwrap();
    assert_eq!(ok["status"], 200);
    assert_eq!(ok["body"], "UklGRg==");
    assert_eq!(ok["headers"][0], json!(["content-type", "audio/wav"]));
    // outside the root, every way there is
    assert_eq!(status("http://localhost/../secret.txt"), Some(404));
    assert_eq!(status("http://localhost/media/%2e%2e/%2e%2e/secret.txt"), Some(404));
    assert_eq!(status("http://localhost/media/..%2f..%2fsecret.txt"), Some(404));
    assert_eq!(status(&format!("http://localhost{}", base.join("secret.txt").display())), Some(404));
    assert_eq!(status("http://localhost/link.txt"), Some(404));
    assert_eq!(status("http://localhost/media/none.wav"), Some(404));
    assert_eq!(page_fetch(&json!({ "url": "http://localhost/media/in.wav", "method": "POST" })).unwrap()["status"], 404);
    // other origins go to the network
    assert_eq!(status("http://localhost:8080/media/in.wav"), None);
    assert_eq!(status("https://example.com/media/in.wav"), None);

    std::fs::remove_dir_all(&base).unwrap();
}
