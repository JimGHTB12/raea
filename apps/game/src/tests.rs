#![cfg(test)]
/// A game's plugins are its nearest declaring package.json's (raea.plugins and raea.bevy.plugins together); a folder
/// whose package.json declares none reads its parent's, and none at all is None (docs/sdk.feature S6b).
#[test]
fn declared_plugins_are_the_nearest_declarations() {
    let root = std::env::temp_dir().join(format!("raea-declared-{}", std::process::id()));
    let game = root.join("proj/games/pong");
    std::fs::create_dir_all(&game).unwrap();
    std::fs::write(root.join("proj/package.json"), r#"{"raea": {"plugins": {"audio": {"from": "@raea/audio"}}, "bevy": {"plugins": {"avian": {}}}}}"#).unwrap();
    std::fs::write(game.join("package.json"), r#"{"name": "pong"}"#).unwrap();
    let (pkg, mut names) = super::declared_plugins(&game).expect("the project's declarations");
    names.sort();
    assert_eq!((pkg, names), (root.join("proj/package.json"), vec!["audio".to_string(), "avian".to_string()]));
    std::fs::write(root.join("proj/package.json"), r#"{"name": "proj"}"#).unwrap();
    assert!(super::declared_plugins(&game).is_none_or(|(p, _)| !p.starts_with(&root)));
    std::fs::remove_dir_all(&root).unwrap();
}

/// A sheet written out to files and read back is the sheet, byte for byte, and a face under two families is one file.
#[test]
fn packaged_fonts_round_trip() {
    use base64::Engine as _;
    let b = |s: &[u8]| base64::engine::general_purpose::STANDARD.encode(s);
    let (a, c) = (b(b"wOF2 face a"), b(b"wOF2 face c"));
    let css = format!(
        "@font-face {{ font-family: \"Inter\"; src: url(data:font/woff2;base64,{a}) format(\"woff2\"); }}\n@font-face {{ font-family: \"Roboto\"; src: url(data:font/woff2;base64,{a}) format(\"woff2\"); }}\n@font-face {{ font-family: \"X\"; src: url(data:font/ttf;base64,{c}); }}\n@font-face {{ font-family: \"Y\"; src: url(\"data:font/woff2;base64,{a}\") format(\"woff2\"); }}\n@font-face {{ src: url(\"https://example.com/x.woff2\"); }}\n"
    );
    let dir = std::env::temp_dir().join(format!("raea-fonts-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let out = super::fonts_out(&css, &dir).unwrap();
    assert!(!out.contains("base64"), "{out}");
    assert_eq!(std::fs::read_dir(dir.join("fonts")).unwrap().count(), 2);
    assert_eq!(super::fonts_in(&out, &dir), css);
    let _ = std::fs::remove_dir_all(&dir);
}
