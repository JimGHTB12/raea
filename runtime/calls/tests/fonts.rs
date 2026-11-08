//! fonts.families (Bevy's ui/text/system_fonts): the system's families, sorted, each once.

#[test]
fn families_are_the_systems_sorted_and_each_once() {
    let v = raea_runtime_calls::call("fonts.families", &serde_json::json!({})).unwrap();
    let names: Vec<&str> = v.as_array().unwrap().iter().map(|n| n.as_str().unwrap()).collect();
    assert!(!names.is_empty(), "no system families");
    assert!(names.windows(2).all(|w| w[0] < w[1]), "not sorted and unique");
    #[cfg(target_vendor = "apple")]
    assert!(names.contains(&"Helvetica"), "no Helvetica among {} families", names.len());
}

/// fonts.atlases (Bevy's ui/text/font_atlas_debug): one 512 px page for a few glyphs, a glyph met twice packed once,
/// and a larger size filling more of it
#[test]
fn atlases_are_bevy_texts_pages() {
    let font = concat!(env!("CARGO_MANIFEST_DIR"), "/../../engine/rune/media/assets/Inter-Regular.ttf");
    let ink = |size: f64, text: &str| -> (usize, u64) {
        let v = raea_runtime_calls::call("fonts.atlases", &serde_json::json!({ "font": font, "size": size, "text": text })).unwrap();
        let pages = v.as_array().unwrap();
        let png = pages[0]["src"].as_str().unwrap().strip_prefix("data:image/png;base64,").unwrap();
        use base64::Engine as _;
        let img = image::load_from_memory(&base64::engine::general_purpose::STANDARD.decode(png).unwrap()).unwrap().to_rgba8();
        assert_eq!((pages[0]["width"].as_u64(), img.width()), (Some(512), 512));
        (pages.len(), img.pixels().map(|p| p.0[3] as u64).sum())
    };
    let (n, ab) = ink(50.0, "ab");
    assert_eq!(n, 1);
    assert!(ab > 0);
    assert_eq!(ink(50.0, "abab").1, ab, "a glyph met again is packed once");
    assert!(ink(80.0, "ab").1 > ab);
}
