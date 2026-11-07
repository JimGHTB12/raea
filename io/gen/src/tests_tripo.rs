//! raea gen's Tripo tests: routing, the CLI's arguments, and naming outputs (kept apart from tests.rs for its size).
use serde_json::json;

use crate::flow::Step;
use crate::route::{Backend, Policy, route};

#[test]
fn tripo_routes_and_args() {
    let mut s = Step::new("mesh.generate", json!({"prompt": "a CRT television", "x_texture_quality": "detailed", "x_face_limit": 8000, "seed": 7}));
    // a host with the CLI and a key: Tripo itself; without: the same model on fal
    assert_eq!(route(&s, &Policy { tripo: true, ..Default::default() }).unwrap().0, Backend::Tripo(String::new()));
    assert_eq!(route(&s, &Policy::default()).unwrap().0, Backend::Fal("tripo3d/p2/text-to-3d".into()));
    s.model = Some("tripo:H3.1".into());
    assert_eq!(route(&s, &Policy::default()).unwrap().0, Backend::Tripo("H3.1".into()));
    let a = crate::tripo::args("H3.1", &s.inputs, &[], std::path::Path::new("/tmp/o"), false);
    let line = a.join(" ");
    assert!(line.starts_with("make a CRT television --model H3.1 --seed 7"), "{line}");
    assert!(line.contains("-p texture_quality=detailed") && line.contains("-p face_limit=8000") && line.ends_with("-o /tmp/o"), "{line}");
    // pictures replace the prompt; a dry run writes nothing
    let a = crate::tripo::args("", &s.inputs, &["/x/front.png".into(), "/x/back.png".into()], std::path::Path::new("."), true);
    assert_eq!((a[1].as_str(), a[2].as_str(), a.last().unwrap().as_str()), ("/x/front.png", "/x/back.png", "--dry-run"));
    assert_eq!(crate::tripo::fal_twin("H3.1", 1), "tripo3d/h3.1/image-to-3d");
    assert_eq!(crate::tripo::fal_twin("tripo-p1", 3), "tripo3d/p1/multiview-to-3d");
    // the defaults: a step's own x_ wins (face_limit 8000 stays, no second one); pictures face their own way; a
    // textured model in metres with its texture seeded like its geometry
    assert_eq!(line.matches("face_limit=").count(), 1, "{line}");
    assert!(line.contains("-p auto_size=true") && line.contains("-p pbr=true") && line.contains("-p texture_seed=7") && !line.contains("orientation"), "{line}");
    let bare = Step::new("mesh.generate", json!({"prompt": "a CRT television"}));
    let a = crate::tripo::args("H3.1", &bare.inputs, &["/x/tv.png".into()], std::path::Path::new("/tmp/o"), false).join(" ");
    assert!(a.contains("-p orientation=align_image") && a.contains("-p face_limit=30000"), "{a}");
    // no seed given: one from the step, the same every run
    let again = crate::tripo::args("H3.1", &bare.inputs, &["/x/tv.png".into()], std::path::Path::new("/tmp/o"), false).join(" ");
    assert_eq!(a, again);
    // P2 keeps its own topology; bare geometry gets no texture settings
    let geo = Step::new("mesh.generate", json!({"prompt": "a CRT television", "texture": false}));
    let p2 = crate::tripo::args("P2", &geo.inputs, &[], std::path::Path::new("/tmp/o"), false).join(" ");
    assert!(!p2.contains("face_limit") && !p2.contains("auto_size") && !p2.contains("pbr=") && p2.contains("texture=false"), "{p2}");
}

#[test]
fn naming_an_output_never_truncates_it() {
    let dir = std::env::temp_dir().join(format!("raea-gen-name-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("k")).unwrap();
    let glb = dir.join("k/model.glb");
    std::fs::write(&glb, b"glTF-bytes").unwrap();
    let png = dir.join("k/preview.png");
    std::fs::write(&png, b"png").unwrap();
    // someone linked the name to the step's file already
    #[cfg(unix)]
    std::os::unix::fs::symlink(&glb, dir.join("prop.glb")).unwrap();
    crate::run::name_output_for_test(&dir, "prop", &[glb.clone(), png]).unwrap();
    assert_eq!(std::fs::read(&glb).unwrap(), b"glTF-bytes");
    assert_eq!(std::fs::read(dir.join("prop.glb")).unwrap(), b"glTF-bytes");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_mesh_steps_size_is_fitted_after_and_leaves_its_key() {
    // the same step with and without a size has the same key: fitting after doesn't make another model
    let mut f = crate::flow::Flow::default();
    f.steps.insert("m".into(), Step::new("mesh.generate", json!({"prompt": "a chair"})));
    let k0 = f.keys().unwrap()["m"].clone();
    f.steps.insert("m".into(), Step::new("mesh.generate", json!({"prompt": "a chair", "size": {"h": 0.9}})));
    assert_eq!(f.keys().unwrap()["m"], k0);
    f.check().unwrap();
    assert_eq!(crate::fit::want_of(&json!({"h": 0.9, "dims": [0.5, 0.5, 0.9], "turn": 90})).h, Some(0.9));
}
