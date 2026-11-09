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
