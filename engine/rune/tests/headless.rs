//! R4b: `raea-rune-host --headless app.js --script steps.json --png out/` writes a PNG per step and the element tree,
//! on Rune's virtual clock (the same files every run).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

mod common;

fn run(out: &Path) {
    let _ = std::fs::remove_dir_all(out);
    std::fs::create_dir_all(out).unwrap();
    let script = out.join("steps.json");
    std::fs::write(&script, r#"[{"press": "inc"}, {"press": "inc"}, {"press": "inc"}, {"press": "reset"}]"#).unwrap();
    let app = out.join("counter.js");
    std::fs::write(&app, common::script(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/apps/counter.tsx"))).unwrap();
    let r = Command::new(env!("CARGO_BIN_EXE_raea-rune-host")).arg("--headless").arg(&app).arg("--script").arg(&script).arg("--png").arg(out.join("png")).output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
}

fn pixels(p: &Path) -> (u32, u32, Vec<u8>) {
    let dec = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(p).unwrap()));
    let mut r = dec.read_info().unwrap();
    let mut buf = vec![0; r.output_buffer_size().unwrap()];
    let info = r.next_frame(&mut buf).unwrap();
    (info.width, info.height, buf)
}

fn at(img: &(u32, u32, Vec<u8>), x: f32, y: f32) -> [u8; 3] {
    let i = ((y as u32 * img.0 + x as u32) * 4) as usize;
    [img.2[i], img.2[i + 1], img.2[i + 2]]
}

fn near(a: [u8; 3], hex: u32) -> bool {
    let b = [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8];
    a.iter().zip(b).all(|(x, y)| x.abs_diff(y) <= 2)
}

#[test]
fn a_png_and_the_tree_per_step_the_same_every_run() {
    let tmp = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let (a, b) = (tmp.join("r4b-a"), tmp.join("r4b-b"));
    run(&a);
    let names: Vec<String> = ["00-mount", "01-press-inc", "02-press-inc", "03-press-inc", "04-press-reset"].iter().map(|n| format!("{n}.png")).collect();
    for n in &names {
        assert!(a.join("png").join(n).exists(), "{n}");
    }
    let trees: Value = serde_json::from_str(&std::fs::read_to_string(a.join("png/tree.json")).unwrap()).unwrap();
    assert_eq!(trees.as_array().unwrap().len(), 5);
    let el = |step: usize, t: &str| trees[step]["tree"].as_array().unwrap().iter().find(|e| e["testid"] == t).cloned();
    assert_eq!(el(1, "count").unwrap()["text"], "Count: 1");
    let inc = el(1, "inc").unwrap();
    assert_eq!((inc["role"].as_str(), inc["key"].as_str()), (Some("button"), Some("w4")));
    assert!(el(2, "reset").is_none() && el(3, "reset").is_some() && el(4, "reset").is_none());

    // the pixels: the button is #2a2d35 at an even count and #3b82f6 at an odd one, on the app's #111318
    let r: Vec<f32> = inc["rect"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap() as f32).collect();
    let (cx, cy) = (r[0] + 4.0, r[1] + r[3] / 2.0);
    let mount = pixels(&a.join("png/00-mount.png"));
    assert_eq!((mount.0, mount.1), (800, 600));
    assert!(near(at(&mount, cx, cy), 0x2a2d35), "{:?}", at(&mount, cx, cy));
    assert!(near(at(&pixels(&a.join("png/01-press-inc.png")), cx, cy), 0x3b82f6));
    assert!(near(at(&mount, 4.0, 4.0), 0x111318), "the app's background");
    assert!(near(at(&mount, 700.0, 500.0), 0xffffff), "the canvas outside it");

    run(&b);
    for n in names.iter().chain([&"tree.json".to_string()]) {
        assert!(std::fs::read(a.join("png").join(n)).unwrap() == std::fs::read(b.join("png").join(n)).unwrap(), "{n} differs between runs");
    }
}
