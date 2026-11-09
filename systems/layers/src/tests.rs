use super::*;

fn layer(key: &str, kind: &'static str) -> Layer {
    Layer { key: key.into(), kind, name: kind.into(), parent: None, loc: key.into(), uses: vec![], index: None, src: None }
}

#[test]
fn a_key_is_the_stamp_its_place_and_its_uses() {
    assert_eq!(key_of(Some("a.tsx:3:5"), None, None).as_deref(), Some("a.tsx:3:5"));
    assert_eq!(key_of(Some("card.tsx:2:10"), Some("2"), Some("comp.tsx:6:27#2")).as_deref(), Some("card.tsx:2:10#2|comp.tsx:6:27#2"));
    assert_eq!(key_of(None, Some("2"), Some("x")), None);
}

#[test]
fn masks_sample_each_fourth_pixel_and_keys_keep_their_index_across_frames() {
    let mut r = Recorder::new("x.layers".into(), "x.mp4".into());
    // 8x4: left half layer a, right half b; the mask is 2x1, sampled at x = 2 and 6
    let px: Vec<u16> = (0..32).map(|i| if i % 8 < 4 { 1 } else { 2 }).collect();
    r.frame(0, 8, 4, &px, &[layer("a", "element"), layer("b", "text")]);
    // next frame b comes first in the list and a is gone: b keeps value 2
    let px2: Vec<u16> = (0..32).map(|i| if i % 8 < 4 { 0 } else { 1 }).collect();
    r.frame(1, 8, 4, &px2, &[layer("b", "text")]);
    r.frame(2, 8, 4, &px2, &[layer("b", "text")]);
    let b = r.bytes(30.0);
    assert_eq!(&b[..8], b"RAEALAYR");
    let n = u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize;
    let head: serde_json::Value = serde_json::from_slice(&b[12..12 + n]).unwrap();
    assert_eq!(head["mask"], serde_json::json!({"w": 2, "h": 1, "scale": 4}));
    assert_eq!(head["layers"][1]["frames"], serde_json::json!([0, 2]));
    assert_eq!(head["layers"][0]["frames"], serde_json::json!([0, 0]));
    assert!(head.get("at").is_none());
    let words: Vec<u16> = b[12 + n..].chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    // frame 0: 2 runs (1,1),(2,1); frame 1: (0,1),(2,1); frame 2: the same as frame 1
    assert_eq!(words, vec![2, 0, 1, 1, 2, 1, 2, 0, 0, 1, 2, 1, 0, 0]);
}

#[test]
fn a_folder_of_frames_gets_the_compositions_name() {
    assert_eq!(path_for(std::path::Path::new("o/film.mp4"), std::path::Path::new("src/promo.tsx")), PathBuf::from("o/film.layers"));
    assert_eq!(path_for(std::path::Path::new("o/frames"), std::path::Path::new("src/promo.tsx")), PathBuf::from("o/frames/promo.layers"));
}

#[test]
fn the_page_covers_the_3d_only_where_it_painted() {
    let mut r = Recorder::new("x.layers".into(), "x.mp4".into());
    // 8x4 frame, 2x1 mask: the page painted the left sample (layer p), nothing on the right
    let px: Vec<u16> = (0..32).map(|i| if i % 8 < 4 { 1 } else { file::NONE }).collect();
    r.frame(0, 8, 4, &px, &[layer("p", "element")]);
    // the 3D has mesh m under both samples
    r.frame_3d(0, 8, &[1, 1], &[layer("m", "mesh")]);
    let b = r.bytes(30.0);
    let n = u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize;
    let words: Vec<u16> = b[12 + n..].chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    // p is 1, m is 2: left p, right m
    assert_eq!(words, vec![2, 0, 1, 1, 2, 1]);
}

#[test]
fn the_nearest_triangle_wins_and_one_behind_the_camera_is_cut() {
    // a 16x8 frame is a 4x2 mask; a big triangle at w = 2 (value 1) and a nearer one at w = 1 (value 2) over the
    // left half only
    let far = ([[-3.0 * 2.0, -3.0 * 2.0, 0.0, 2.0], [3.0 * 2.0, -3.0 * 2.0, 0.0, 2.0], [0.0, 3.0 * 2.0, 0.0, 2.0]], 1);
    let near = ([[-1.0, -1.0, 0.0, 1.0], [0.2, -1.0, 0.0, 1.0], [-1.0, 1.4, 0.0, 1.0]], 2);
    let m = raster((16, 8), [far, near]);
    assert_eq!(m.len(), 8);
    assert_eq!(m[0], 2, "the near one over the top left: {m:?}");
    assert_eq!(m[3], 1, "the far one over the top right: {m:?}");
    // all behind the camera: nothing
    let behind = ([[0.0, 0.0, 0.0, -1.0], [1.0, 0.0, 0.0, -1.0], [0.0, 1.0, 0.0, -1.0]], 3);
    assert!(raster((16, 8), [behind]).iter().all(|v| *v == 0));
}

#[test]
fn a_uses_root_is_named_after_its_component() {
    let dir = std::env::temp_dir().join(format!("raea-layers-names-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("comp.tsx"), "const a = 1;\n  <Card x={1} />\n  <motion.Div />\n").unwrap();
    assert_eq!(names::component_at(&dir, "comp.tsx:2:3").as_deref(), Some("Card"));
    assert_eq!(names::component_at(&dir, "comp.tsx:3:3").as_deref(), Some("motion.Div"));
    assert_eq!(names::component_at(&dir, "comp.tsx:1:1"), None);
    let mut r = Recorder::new("x.layers".into(), "x.mp4".into()).named_from(dir.clone());
    let root = Layer::of("card.tsx:1:1|comp.tsx:2:3".into(), "element", "div".into(), Some("top.tsx:1:1".into()), None);
    let inner = Layer::of("card.tsx:2:1|comp.tsx:2:3".into(), "element", "span".into(), Some(root.key.clone()), None);
    r.frame(0, 4, 4, &[1; 16], &[layer("top.tsx:1:1", "element"), root, inner]);
    let head = r.header(30.0);
    assert_eq!(head["layers"][1]["name"], "Card");
    assert_eq!(head["layers"][2]["name"], "span", "inside the use, not its root");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_layer_key_selects_its_element_alone() {
    assert_eq!(layer_selector("a.tsx:3:5"), r#"[data-raea-loc="a.tsx:3:5"]:not([data-raea-idx]):not([data-raea-use]), [data-raea-loc="a.tsx:3:5"]:not([data-raea-idx]):not([data-raea-use]) *"#);
    let s = layer_selector("card.tsx:2:10#2|comp.tsx:6:27#2|x.tsx:1:1");
    assert!(s.starts_with(r#"[data-raea-loc="card.tsx:2:10"][data-raea-idx="2"][data-raea-use="comp.tsx:6:27#2|x.tsx:1:1"]"#), "{s}");
}
