use super::*;

#[test]
fn a_new_fish_take_is_refused() {
    let s = Store { dir: std::env::temp_dir().join(format!("render-fish-{}", std::process::id())) };
    let e = s.make("fish", "Three drops.", 1, &json!({})).unwrap_err();
    assert!(e.contains("no longer part of raea") && e.contains("<Speech>"), "{e}");
    let _ = std::fs::remove_dir_all(&s.dir);
}

#[test]
fn a_new_lyria_take_is_refused() {
    let s = Store { dir: std::env::temp_dir().join(format!("render-lyria-{}", std::process::id())) };
    assert!(s.make("lyria", "warm lo-fi bed", 1, &json!({})).unwrap_err().contains("lyria is retired"));
    let _ = std::fs::remove_dir_all(&s.dir);
}

#[test]
fn the_key_ignores_param_order_but_not_seed_or_params() {
    let k = make_key("sao", "rain", 3, &json!({"seconds": 6, "a": 1}));
    assert_eq!(k, make_key("sao", "rain", 3, &json!({"a": 1, "seconds": 6})));
    assert_ne!(k, make_key("sao", "rain", 4, &json!({"a": 1, "seconds": 6})));
    assert_ne!(k, make_key("sao", "rain", 3, &json!({"a": 1, "seconds": 7})));
    // a whole number is one key however it was written (a Tone graph's `1`, Rust's `1.0`); fractions stay apart
    assert_eq!(make_key("sao", "rain", 1, &json!({"seconds": 1})), make_key("sao", "rain", 1, &json!({"seconds": 1.0})));
    assert_ne!(make_key("sao", "rain", 1, &json!({"seconds": 1.5})), make_key("sao", "rain", 1, &json!({"seconds": 1})));
}

#[test]
fn an_asset_indexed_under_the_old_key_is_still_found() {
    let dir = std::env::temp_dir().join(format!("render-legacy-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let s = Store { dir: dir.clone() };
    // an <Audio>'s 40 s bed, indexed with its f64 as written before whole numbers were made integers
    let p = json!({"seconds": 40.0});
    let old = legacy_key("sao", "room tone", 1, &p);
    assert_ne!(old, make_key("sao", "room tone", 1, &p));
    let line = json!({"key": old, "hash": "abc", "ext": "wav", "origin": {"how": "made"}});
    std::fs::write(dir.join("made.jsonl"), format!("{line}\n")).unwrap();
    assert_eq!(s.made("sao", "room tone", 1, &p).unwrap().map(|a| a.hash), Some("abc".into()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_local_model_without_its_install_is_not_installed() {
    let dir = std::env::temp_dir().join(format!("render-noinstall-{}", std::process::id()));
    assert!(!installed("sao", &dir) && !installed("tokensynth", &dir));
    assert!(setup_script("sao").exists() && setup_script("tokensynth").exists());
}

#[test]
fn make_indexes_once_and_refuses_unknown_models() {
    let dir = std::env::temp_dir().join(format!("render-make-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let s = Store { dir: dir.clone() };
    assert_eq!(s.made("stand-in", "a cat", 1, &json!({})).unwrap(), None);
    let a = s.make("stand-in", "a cat", 1, &json!({})).unwrap();
    assert_eq!(s.make("stand-in", "a cat", 1, &json!({})).unwrap(), a);
    assert_eq!(s.made("stand-in", "a cat", 1, &json!({})).unwrap(), Some(a));
    assert_eq!(std::fs::read_to_string(dir.join("made.jsonl")).unwrap().lines().count(), 1);
    assert!(s.make("dall-e", "a cat", 1, &json!({})).unwrap_err().contains("stand-in, tokensynth, sao, fish, lyria"));
}

#[test]
fn matchmove_params_fill_defaults_and_refuse_nonsense() {
    let p = matchmove_params(&json!({"from": 2, "subject": [0.4, 0.2, 0.2, 0.7]})).unwrap();
    assert_eq!(p, json!({"from": 2.0, "seconds": 0.0, "subject": [0.4, 0.2, 0.2, 0.7], "height": 1.6, "fov": 0.0, "smooth": 1.0}));
    // a default written out is the same solve as one left out
    let k = |p: &J| make_key("matchmove", "sha256:x", 1, &matchmove_params(p).unwrap());
    assert_eq!(k(&json!({"from": 2})), k(&json!({"from": 2, "height": 1.6, "subject": null})));
    assert_ne!(k(&json!({"from": 2})), k(&json!({"from": 3})));
    assert!(matchmove_params(&json!({"subject": [0.4, 0.2, 1.5, 0.7]})).unwrap_err().contains("subject"));
    assert!(matchmove_params(&json!({"height": -1})).is_err());
    assert!(matchmove_params(&json!({"from": "two"})).unwrap_err().contains("from must be a number"));
    assert!(!installed("matchmove", &std::env::temp_dir().join("render-no-matchmove")) && setup_script("matchmove").exists());
}
