//! T0b: the verbs read the target from the app (or --target), and each target is handed what its host takes.
use std::path::PathBuf;

use raea_app_targets::steps::{self, Step};
use raea_app_targets::verbs::{Verb, parse};
use raea_app_targets::{Target, native};

fn app(src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("raea-targets-verbs-{}-{}", std::process::id(), src.len()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("app.tsx");
    std::fs::write(&f, src).unwrap();
    f
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn the_app_says_its_target_and_window_and_flags_win() {
    let f = app("export const app = { target: \"native\", title: \"Tally\", size: [390, 844] };\nexport default function A() { return null }\n");
    let p = f.display().to_string();
    let a = parse(&s(&["check", &p, "--png", "o"]), None).unwrap();
    assert_eq!((a.verb, a.config.target, a.size, a.title.as_str()), (Verb::Check, Target::Native, (390, 844), "Tally"));
    let a = parse(&s(&["run", &p, "--target", "web", "--size", "640x480"]), None).unwrap();
    assert_eq!((a.config.target, a.size), (Target::Web, (640, 480)));
}

#[test]
fn an_app_with_no_setting_is_web_and_rune_is_native() {
    let f = app("export default function A() { return null }\n");
    let p = f.display().to_string();
    assert_eq!(parse(&s(&["run", &p]), None).unwrap().config.target, Target::Web);
    assert_eq!(parse(&s(&["run", &p]), Some(Target::Native)).unwrap().config.target, Target::Native);
    assert!(parse(&s(&["run", &p, "--target", "web"]), Some(Target::Native)).is_err());
    // the native host's own flags aren't taken quietly for a web app
    assert!(parse(&s(&["check", &p, "--video", "x.mp4"]), None).unwrap_err().contains("native"));
}

#[test]
fn native_gets_raea_rune_hosts_arguments() {
    let f = app("export const app = { target: \"native\" };\n");
    let p = f.display().to_string();
    let a = parse(&s(&["check", &p, "--script", "steps.json", "--png", "out", "--video", "v.mp4"]), None).unwrap();
    assert_eq!(native::args(&a), s(&["--headless", &p, "--png", "out", "--size", "800x600", "--settle", "12", "--script", "steps.json", "--video", "v.mp4"]));
    let a = parse(&s(&["bundle", &p, "-o", "T.app"]), None).unwrap();
    assert_eq!(native::args(&a), s(&["bundle", &p, "--size", "800x600", "--title", "app", "-o", "T.app"]));
}

#[test]
fn a_script_is_the_same_steps_on_both_targets() {
    let v = steps::parse(r#"[{"press":"add","name":"press-add"},{"type":"Tacos","into":"what"},{"wheel":"list","dy":120},{"key":"Enter"},{"frames":30}]"#).unwrap();
    assert_eq!(v, vec![Step::Press("add".into()), Step::Type { text: "Tacos".into(), into: "what".into() }, Step::Wheel { over: "list".into(), dy: 120.0 }, Step::Key("Enter".into()), Step::Frames(30)]);
    assert_eq!(steps::name(&v[0]), "press-add");
    assert!(steps::parse(r#"[{"tap":"x"}]"#).unwrap_err().contains("step 1"));
}

#[test]
fn bundle_signs_as_the_developer_asks() {
    let f = app("export const app = { target: \"native\" };\nexport default function A() { return null }\n");
    let p = f.display().to_string();
    let a = parse(&s(&["bundle", &p, "--identity", "Developer ID Application: Someone (TEAM)", "--notarize", "my-profile"]), None).unwrap();
    assert_eq!((a.sign.identity.as_deref(), a.sign.notarize.as_deref()), (Some("Developer ID Application: Someone (TEAM)"), Some("my-profile")));
    // neither reaches the host: raea signs the .app after the host makes it
    assert!(!native::args(&a).iter().any(|x| x.contains("notarize") || x.contains("identity")));
    // signing is bundle's alone, and --notarize names its profile
    assert!(parse(&s(&["check", &p, "--identity", "X"]), None).unwrap_err().contains("bundle"));
    assert!(parse(&s(&["bundle", &p, "--notarize"]), None).unwrap_err().contains("needs a value"));
}
