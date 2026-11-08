//! A row's source as a file: a staticFile path (/media/x) is found in the media folders, as a material's map and every
//! other media src is, not read as a file at the root (a three.js shader's texture once was: "/media/fonts/x.png").

use raea_sys_frame::resolve_src;

#[test]
fn a_static_file_is_found_in_media() {
    let dir = std::env::temp_dir().join(format!("raea-resolve-src-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("media/fonts")).unwrap();
    std::fs::write(dir.join("media/fonts/x.png"), "").unwrap();
    assert_eq!(resolve_src(&[], "/media/fonts/x.png", &dir).unwrap(), dir.join("media/fonts/x.png"));
    // a relative path is still the edit's own
    assert_eq!(resolve_src(&[], "media/fonts/x.png", &dir).unwrap(), dir.join("media/fonts/x.png"));
}
