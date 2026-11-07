//! T3d on a page: an `<img>` by a path relative to the app's folder and an autoplaying `<video>` on Rune's video
//! surface, each with the news @raea/ui hears; a `<style>` isn't drawn, and what can't map is reported.

use raea_rune_page::page::{Out, Page};
use raea_rune_page::session::Session;
use serde_json::{Value, json};

const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[test]
fn an_image_and_a_video_load_and_say_so() {
    let dir = std::env::temp_dir().join(format!("t3d-page-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let Ok(raea_rune_media::src::Src::Bytes(png)) = raea_rune_media::src::resolve(&format!("data:image/png;base64,{PNG}"), None) else { panic!() };
    std::fs::write(dir.join("dot.png"), png).unwrap();
    std::fs::copy(concat!(env!("CARGO_MANIFEST_DIR"), "/../media/tests/fixtures/clip.mp4"), dir.join("clip.mp4")).unwrap();
    raea_rune_media::set_base(&dir);
    let op = |id: u64, kind: &str, props: Value| json!({"op": "create", "id": id, "kind": kind, "props": props});
    let ops = [
        op(1, "img", json!({"src": "dot.png", "style": {"width": 20}})),
        op(2, "video", json!({"src": "clip.mp4", "autoplay": "", "muted": true})),
        op(3, "style", json!({})),
        json!({"op": "create", "id": 4, "kind": "text", "text": "@font-face{font-family:Later;src:url(x.woff2)} body{margin:0}"}),
        op(5, "div", json!({"style": {"fontFamily": "Nope"}})),
    ];
    let mut ops = ops.to_vec();
    ops.extend([(0, 1), (0, 2), (0, 3), (3, 4), (0, 5)].map(|(p, c)| json!({"op": "append", "parent": p, "child": c})));
    let mut s = Session::new(Page::default(), (800.0, 600.0));
    s.apply(&json!({"v": 1, "ops": ops}).to_string()).unwrap();
    // a 3 s clip at 60 fps, and a little over
    let mut news = Vec::new();
    for _ in 0..200 {
        s.frame();
        for o in s.take_out() {
            let Out::Deliver(m) = o else { continue };
            let v: Value = serde_json::from_str(&m).unwrap();
            if v["name"] == "media" {
                news.push(v["value"].clone());
            }
        }
    }
    let told: Vec<(u64, &str)> = news.iter().map(|v| (v["id"].as_u64().unwrap(), v["type"].as_str().unwrap())).collect();
    assert_eq!(told, [(1, "load"), (2, "loadedmetadata"), (2, "loadeddata"), (2, "ended")], "{news:?}");
    assert_eq!((news[0]["w"].as_u64(), news[1]["w"].as_u64(), news[1]["h"].as_u64(), news[1]["duration"].as_f64()), (Some(1), Some(320), Some(180), Some(3.0)));
    let el = |k: &str| s.rt.elements().into_iter().find(|e| e.key.as_deref() == Some(k));
    assert_eq!(el("w1").map(|e| [e.target[2], e.target[3]]), Some([20.0, 20.0]), "the image at its 1:1 aspect");
    assert_eq!(el("w2").map(|e| [e.target[2], e.target[3]]), Some([320.0, 180.0]), "the video at its natural size");
    assert!(el("w3").is_none(), "a <style> isn't drawn");
    let r = s.rt.state.report.borrow();
    let said: Vec<(&str, &str)> = r.all().map(|(p, seen)| (p, seen.value.as_str())).collect();
    for want in [("@font-face", "Later"), ("style", "rules"), ("fontFamily", "Nope")] {
        assert!(said.contains(&want), "{want:?} in {said:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}
