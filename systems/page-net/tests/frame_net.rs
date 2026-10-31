//! An <iframe>'s loads can't read a file outside RAEA_PAGE_ROOT (docs/blitz-bevy.feature B5c, iframes 6): FrameNet
//! reads a file inside the root, a root-absolute path under it, and nothing that resolves outside it, by `..`, by a
//! link inside the root, or by an absolute path.

use raea_sys_page_net::FrameNet;

#[test]
fn a_frame_reads_only_inside_the_root() {
    let base = std::env::temp_dir().join(format!("raea-frame-net-{}", std::process::id()));
    let root = base.join("public");
    std::fs::create_dir_all(root.join("media")).unwrap();
    std::fs::write(root.join("media/in.html"), "in").unwrap();
    std::fs::write(base.join("secret.txt"), "secret").unwrap();
    std::os::unix::fs::symlink(base.join("secret.txt"), root.join("link.txt")).unwrap();
    let net = FrameNet { roots: vec![root.clone()] };

    // inside, by its own path and root-absolute (the app's "/media/in.html")
    assert!(net.confined(&root.join("media/in.html")).is_some());
    assert!(net.confined(std::path::Path::new("/media/in.html")).is_some());
    // outside: `..` out of the root, a link inside it to a file outside, an absolute path elsewhere
    assert_eq!(net.confined(&root.join("media/../../secret.txt")), None);
    assert_eq!(net.confined(std::path::Path::new("/../secret.txt")), None);
    assert_eq!(net.confined(&root.join("link.txt")), None);
    assert_eq!(net.confined(&base.join("secret.txt")), None);
    // no root: no file at all
    assert_eq!(FrameNet { roots: vec![] }.confined(&root.join("media/in.html")), None);

    std::fs::remove_dir_all(&base).unwrap();
}
