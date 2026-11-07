//! T3d: an app's fonts gathered before Rune starts: a file in its fonts/ folder, an @font-face rule in its CSS and
//! one in its bundle, WOFF2 decoded; what doesn't load is noted, and a family with no face is missing.

use std::path::{Path, PathBuf};

use raea_rune_media::fonts;

fn core_font(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../core/fonts").join(name)
}

fn folder() -> PathBuf {
    let d = std::env::temp_dir().join(format!("t3d-fonts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("fonts")).unwrap();
    std::fs::create_dir_all(d.join("styles/faces")).unwrap();
    std::fs::copy(core_font("lato-400-normal.woff2"), d.join("fonts/lato.woff2")).unwrap();
    std::fs::copy(core_font("oswald-700-normal.woff2"), d.join("styles/faces/display.woff2")).unwrap();
    // a rule relative to its own file, a missing face, and a local() one the host can't read
    std::fs::write(
        d.join("styles/app.css"),
        "body { margin: 0 }\n@font-face { font-family: \"Display\"; src: url(./faces/display.woff2) format(\"woff2\"); font-weight: 700; }\n\
         @font-face { font-family: Gone; src: url(nowhere.woff2); }\n@font-face { font-family: Local; src: local(Helvetica); }",
    )
    .unwrap();
    d
}

#[test]
fn the_apps_faces_register_and_the_rest_is_said() {
    let d = folder();
    std::fs::copy(core_font("montserrat-400-normal.woff2"), d.join("brand.woff2")).unwrap();
    // a <style> in the TSX: CSS inside a JS string, escaped
    let bundle = r#"const s = "@font-face { font-family: \"Brand\"; src: url(\"brand.woff2\"); }";"#;
    let g = fonts::gather(&d, bundle);
    let names: Vec<Vec<String>> = g.faces.iter().map(|f| f.names.clone()).collect();
    assert_eq!(names, [vec!["Lato".to_string()], vec!["Display".into(), "Oswald".into()], vec!["Brand".into(), "Montserrat Thin".into()]], "{:?}", g.notes);
    assert!(g.faces.iter().all(|f| f.bytes.starts_with(&[0, 1, 0, 0]) || f.bytes.starts_with(b"OTTO")), "decoded to TrueType or OpenType");
    let notes: Vec<&str> = g.notes.iter().map(|(w, _)| w.as_str()).collect();
    assert_eq!(notes.len(), 2, "{:?}", g.notes);
    assert!(notes[0].starts_with("@font-face Gone") && notes[1].starts_with("@font-face Local"), "{notes:?}");

    fonts::register(g);
    // Inter first (Rune's sans), then the three faces
    assert_eq!(fonts::registered().len(), 6);
    assert_eq!(fonts::family("Display"), Some("Oswald"));
    // the face's own name as Rune's text system (fontique) knows it: the typographic family first
    assert_eq!(fonts::family("brand"), Some("Montserrat Thin"), "names match as CSS's do, ignoring case");
    assert_eq!(fonts::family("Lato"), Some("Lato"));
    assert!(fonts::missing("Nope").is_some() && fonts::missing("Gone").is_some());
    assert!(fonts::missing("sans-serif").is_none() && fonts::missing("Inter").is_none() && fonts::missing("Display").is_none());
    assert_eq!(fonts::notes().len(), 2);
    // a <style> later: a face that registered is fine, a new one is late (and @raea/ui's own template rule isn't one)
    assert_eq!(fonts::late("@font-face{font-family:Display;src:url(x)} @font-face{font-family:'Later';src:url(y)}"), ["Later"]);
    assert!(fonts::late("@font-face{font-family:${cssString(f.family)};src:${f._src}}").is_empty());
    let _ = std::fs::remove_dir_all(&d);
}
