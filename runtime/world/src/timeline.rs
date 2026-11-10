//! A frame's page and timeline as entities: the page's layers in document order (footage, pictures, runs of text),
//! its Sequences as shots, its sounds as cues and the lines sung or said on it. The page's text comes from its layout
//! (page.rs) when it was laid out; else from the tree, with `needs: "render"`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::{Children, World};
use raea_core_facts::*;
use raea_runtime_react::{ReactKind, ReactProps, ReactText};
use raea_core_scene::num;
use serde_json::{Map, Value};

/// An element (or text node) of the frame's page, in document order, with its parent's index.
pub(crate) struct El {
    pub(crate) kind: String,
    pub(crate) props: Map<String, Value>,
    pub(crate) text: Option<String>,
    pub(crate) parent: Option<usize>,
}

/// The page under the composition's surface, in document order; a 3D holder's 3D is the scene's (scene.rs).
pub(crate) fn tree(world: &mut World) -> Vec<El> {
    let Some(surf) = raea_runtime_react::scene::surface(world) else { return Vec::new() };
    let mut out = Vec::new();
    let mut stack: Vec<(bevy::ecs::entity::Entity, Option<usize>)> = vec![(surf, None)];
    while let Some((e, parent)) = stack.pop() {
        let (Some(kind), props) = (world.get::<ReactKind>(e), world.get::<ReactProps>(e)) else { continue };
        let el = El { kind: kind.0.clone(), props: props.map(|p| p.0.clone()).unwrap_or_default(), text: world.get::<ReactText>(e).map(|t| t.0.clone()), parent };
        let holds_3d = el.props.contains_key("data-3d");
        out.push(el);
        if let (false, Some(kids)) = (holds_3d, world.get::<Children>(e)) {
            let at = out.len() - 1;
            stack.extend(kids.iter().rev().map(|k| (*k, Some(at))));
        }
    }
    out
}

thread_local! {
    static HASHES: std::cell::RefCell<HashMap<PathBuf, Option<String>>> = std::cell::RefCell::new(HashMap::new());
}

/// A file's SHA-256, read once a process.
fn hash(p: &Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    HASHES.with(|h| {
        h.borrow_mut()
            .entry(p.to_path_buf())
            .or_insert_with(|| {
                let mut f = std::fs::File::open(p).ok()?;
                let mut d = Sha256::new();
                std::io::copy(&mut f, &mut d).ok()?;
                Some(format!("{:x}", d.finalize()))
            })
            .clone()
    })
}

/// A transcript file's words (raea_core_facts::transcript_words, as `raea check` reads them); none when it isn't one.
pub(crate) fn words(path: &Path, fps: f64) -> Vec<Word> {
    std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str::<Value>(&s).ok()).map_or_else(Vec::new, |v| transcript_words(&v, fps))
}

/// The project's transcript (its words.json) as lines: a line ends at a stop (. ? !) or a pause of half a second.
pub(crate) fn project_lines(given: Option<&Value>, dir: &Path, fps: f64) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    let mut open = true;
    // the transcript `--words` gives the composition, else the project's words.json
    for w in given.map_or_else(|| words(&dir.join("words.json"), fps), |v| transcript_words(v, fps)) {
        match lines.last_mut() {
            Some(l) if open && w.start - l.to.unwrap_or(l.from) < 0.5 => {
                l.text = format!("{} {}", l.text, w.text);
                l.to = Some(w.end);
                l.words.push(w.clone());
            }
            _ => lines.push(Line { text: w.text.clone(), from: w.start, to: Some(w.end), words: vec![w.clone()], needs: None }),
        }
        open = !w.text.ends_with(['.', '?', '!']);
    }
    lines
}

/// The lines on frame `f`: the project transcript's that span it, and each line raea says (<Speech>) on it.
pub(crate) fn put_lines(facts: &mut Facts, lines: &[Line], tree: &[El], f: i64, fps: f64) {
    let t = f as f64 / fps;
    for l in lines.iter().filter(|l| l.from <= t && t <= l.to.unwrap_or(l.from)) {
        let e = facts.spawn(None);
        facts.put(e, l);
    }
    for el in tree {
        let Some(said) = el.props.get("data-speech").and_then(Value::as_str).and_then(|s| serde_json::from_str::<Value>(s).ok()) else { continue };
        let text = said["text"].as_str().unwrap_or_default().to_string();
        let from = said["start"].as_f64().unwrap_or(f as f64) / fps;
        let e = facts.spawn(None);
        facts.put(e, &Line { text, from, to: None, words: Vec::new(), needs: Some("transcript".into()) });
    }
}

/// A run of text the page laid out (a render manifest's text entry) as a Text, in the frame's 0..1 units, its size
/// in px of a frame 1080 on its short side.
pub(crate) fn laid_text(t: &Value, (w, h): (u32, u32)) -> Text {
    let b: Vec<f64> = t["box"].as_array().map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
    let (w, h) = (w as f64, h as f64);
    Text {
        words: t["text"].as_str().unwrap_or_default().to_string(),
        font: t["family"].as_str().map(str::to_string),
        size: t["size"].as_f64().map(|s| (s * 1080.0 / w.min(h) * 10.0).round() / 10.0),
        box_: (b.len() == 4).then(|| [b[0] / w, b[1] / h, (b[0] + b[2]) / w, (b[1] + b[3]) / h].map(|v| (v * 1e4).round() / 1e4)),
        extruded: 0.0,
        lines: t["lines"].as_u64().map(|n| n as u32),
        needs: None,
    }
}

fn style(props: &Map<String, Value>, k: &str) -> Option<String> {
    props.get("style").and_then(|s| s.get(k)).map(text_of)
}

/// A page layer: its place in document order, what it is, the element it is (its props), and its own components.
#[derive(Default)]
struct Paged {
    at: usize,
    medium: Option<Medium>,
    opacity: Option<f64>,
    props: Option<usize>,
    text: Option<Text>,
    clip: Option<Clip>,
    cue: Option<Cue>,
    source: Option<Source>,
}

/// The frame's page and timeline into `facts`, its layers after the 3D's (`order` goes on from there).
pub(crate) fn put(facts: &mut Facts, tree: &[El], laid: Option<&Value>, (w, h, fps): (u32, u32, f64), dir: &Path, order: &mut u32) {
    let mut paged: Vec<Paged> = Vec::new();
    let mut shots: HashMap<usize, u32> = HashMap::new();
    for (i, el) in tree.iter().enumerate() {
        let p = &el.props;
        if let Some(from) = num(p, "data-raea-start").filter(|_| p.contains_key("data-sequence") || p.contains_key("data-raea-end")) {
            // a Sequence's box: a shot, inside the shot around it
            let up = std::iter::successors(el.parent, |&k| tree[k].parent).find_map(|k| shots.get(&k).copied());
            let e = facts.spawn(up);
            facts.put(e, &Shot { name: p.get("data-sequence").and_then(Value::as_str).map(str::to_string), from: from as i64, to: num(p, "data-raea-end").map(|t| t as i64) });
            shots.insert(i, e);
        }
        let src = p.get("src").and_then(Value::as_str).filter(|s| !s.is_empty());
        // a URL or an asset reference (a generated sound's `asset:sao/...`) as it is; a file where the render finds it
        let file = src.map(|s| if s.starts_with("http") || s.starts_with("asset:") { PathBuf::from(s) } else { raea_core_scene::media_path(dir, s) });
        let sounds = num(p, "data-volume").unwrap_or(1.0) > 0.0 && p.get("data-muted").and_then(Value::as_str) != Some("true");
        let cue = |file: &Path| Cue {
            source: file.display().to_string(),
            at: num(p, "data-from").unwrap_or(0.0) / fps,
            gain: num(p, "data-gain-db").or_else(|| num(p, "data-volume").map(|v| (20.0 * v.log10() * 10.0).round() / 10.0)),
            bus: p.get("data-bus").and_then(Value::as_str).map(str::to_string),
        };
        match (el.kind.as_str(), file) {
            ("video", Some(file)) => {
                let (from, out) = (num(p, "data-start-from").unwrap_or(0.0) / fps, num(p, "data-end-at").map(|t| t / fps));
                let heard = list_words(&file, fps, from, out);
                let clip = Clip { source: file.display().to_string(), hash: hash(&file), time: num(p, "data-source-time"), in_: from, out, at: num(p, "data-from").unwrap_or(0.0) as i64, words: heard };
                let cue = sounds.then(|| cue(&file));
                paged.push(Paged { at: i, medium: Some(Medium::Footage), props: Some(i), clip: Some(clip), cue, ..Default::default() });
            }
            ("audio", Some(file)) if sounds => {
                let e = facts.spawn(None);
                facts.put(e, &cue(&file));
                if let Some(s) = crate::source::of(p, dir) {
                    facts.put(e, &s);
                }
            }
            ("img", Some(_)) => paged.push(Paged { at: i, medium: Some(Medium::Graphic2d), props: Some(i), ..Default::default() }),
            _ => {}
        }
    }
    match laid.and_then(|l| l["text"].as_array()) {
        Some(texts) => {
            // each laid-out run at its owner's place in the tree (its stamp and list key), in the page's order
            let mut used = vec![false; tree.len()];
            for t in texts {
                let (loc, idx) = (t["loc"].as_str(), t["index"].as_str());
                let owner = loc.and_then(|loc| (0..tree.len()).find(|&k| !used[k] && tree[k].props.get("data-raea-loc").and_then(Value::as_str) == Some(loc) && tree[k].props.get("data-raea-idx").map(text_of).as_deref() == idx));
                if let Some(k) = owner {
                    used[k] = true;
                }
                let text = laid_text(t, (w, h));
                let mut stamp = Map::new();
                for (k, from) in [("data-raea-loc", "loc"), ("data-raea-idx", "index")] {
                    if let Some(v) = t[from].as_str() {
                        stamp.insert(k.into(), Value::String(v.into()));
                    }
                }
                if let Some(u) = t["use"].as_array() {
                    stamp.insert("data-raea-use".into(), Value::String(u.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("|")));
                }
                let source = crate::source::of(&stamp, dir);
                paged.push(Paged { at: owner.unwrap_or(tree.len()), medium: Some(Medium::Type2d), opacity: t["opacity"].as_f64(), props: owner, text: Some(text), source, ..Default::default() });
            }
        }
        None => {
            // not laid out: each stamped element's words, with no box or size
            let mut owned: Vec<(usize, Vec<String>)> = Vec::new();
            for (i, el) in tree.iter().enumerate() {
                let Some(words) = el.text.as_deref().map(str::trim).filter(|t| !t.is_empty()) else { continue };
                // a <style>'s or <script>'s text is code, not words on the frame
                if std::iter::successors(Some(i), |&k| tree[k].parent).any(|k| matches!(tree[k].kind.as_str(), "style" | "script")) {
                    continue;
                }
                let owner = std::iter::successors(Some(i), |&k| tree[k].parent).find(|&k| tree[k].props.contains_key("data-raea-loc")).unwrap_or(i);
                match owned.iter_mut().find(|(o, _)| *o == owner) {
                    Some((_, ws)) => ws.push(words.to_string()),
                    None => owned.push((owner, vec![words.to_string()])),
                }
            }
            for (owner, ws) in owned {
                let text = Text { words: ws.join(" "), font: style(&tree[owner].props, "fontFamily"), size: None, box_: None, extruded: 0.0, lines: None, needs: Some("render".into()) };
                paged.push(Paged { at: owner, medium: Some(Medium::Type2d), props: Some(owner), text: Some(text), ..Default::default() });
            }
        }
    }
    paged.sort_by_key(|p| p.at);
    for p in paged {
        let e = facts.spawn(None);
        let props = p.props.map(|k| &tree[k].props);
        // laid-out text has its opacity as computed (its parents' with it); anything else its own style's
        let opacity = p.opacity.or_else(|| props.and_then(|pr| style(pr, "opacity")).and_then(|o| o.parse::<f64>().ok())).unwrap_or(1.0);
        facts.put(e, &Layer { medium: p.medium.unwrap_or(Medium::Page), order: *order, opacity, blend: props.and_then(|pr| style(pr, "mixBlendMode")).filter(|b| b != "normal") });
        *order += 1;
        if let Some(s) = p.source.or_else(|| props.and_then(|pr| crate::source::of(pr, dir))) {
            facts.put(e, &s);
        }
        if let Some(t) = &p.text {
            facts.put(e, t);
        }
        if let Some(c) = &p.clip {
            facts.put(e, c);
        }
        if let Some(c) = &p.cue {
            facts.put(e, c);
        }
    }
}

/// The words a take's transcript (TAKE.words.json, beside it) has between `from` and `out` seconds of the take.
fn list_words(take: &Path, fps: f64, from: f64, out: Option<f64>) -> Vec<Word> {
    // any word the clip holds part of: one its in or out point cuts through is in it (a check of where a cut lands reads it)
    words(&take.with_extension("words.json"), fps).into_iter().filter(|w| w.end > from && out.is_none_or(|o| w.start < o)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn el(kind: &str, props: Value, text: Option<&str>, parent: Option<usize>) -> El {
        El { kind: kind.into(), props: props.as_object().cloned().unwrap_or_default(), text: text.map(str::to_string), parent }
    }

    #[test]
    fn a_page_reads_as_shots_footage_cues_and_text_in_document_order() {
        let tree = vec![
            el("div", serde_json::json!({"data-sequence": "intro", "data-raea-start": 0, "data-raea-end": 90}), None, None),
            el("video", serde_json::json!({"src": "/abs/take.mp4", "data-source-time": 1.5, "data-start-from": 30, "data-from": 0, "data-volume": 0.5}), None, Some(0)),
            el("h1", serde_json::json!({"data-raea-loc": "edit.tsx:9:7", "style": {"fontFamily": "Inter"}}), None, Some(0)),
            el("#text", serde_json::json!({}), Some("OUT NOW"), Some(2)),
            el("audio", serde_json::json!({"src": "/abs/hit.wav", "data-from": 45, "data-bus": "sfx"}), None, Some(0)),
            el("style", serde_json::json!({"data-raea-loc": "edit.tsx:3:5"}), None, Some(0)),
            el("#text", serde_json::json!({}), Some("@font-face { font-family: Inter }"), Some(5)),
        ];
        let mut facts = Facts::new(10, 1920, 1080, 30.0);
        let mut order = 1;
        put(&mut facts, &tree, None, (1920, 1080, 30.0), Path::new("/nowhere"), &mut order);
        let shot: Shot = facts.get(facts.find(&["Shot"])[0].id).unwrap();
        assert_eq!((shot.name.as_deref(), shot.from, shot.to), (Some("intro"), 0, Some(90)));
        let take = facts.find(&["Clip"])[0];
        let clip: Clip = facts.get(take.id).unwrap();
        assert_eq!((clip.in_, clip.time, clip.hash), (1.0, Some(1.5), None), "a file that isn't there has no hash");
        assert_eq!(facts.get::<Cue>(take.id).unwrap().gain, Some(-6.0), "a sounding take is a cue too");
        assert_eq!(facts.get::<Layer>(take.id).unwrap().order, 1, "the page's layers go on from the 3D's");
        assert_eq!(facts.find(&["Text"]).len(), 1, "a <style>'s CSS is not words on the frame");
        let t = facts.find(&["Text", "Layer"])[0];
        let text: Text = facts.get(t.id).unwrap();
        assert_eq!((text.words.as_str(), text.font.as_deref(), text.needs.as_deref()), ("OUT NOW", Some("Inter"), Some("render")));
        assert_eq!(facts.get::<Layer>(t.id).unwrap().medium, Medium::Type2d);
        assert_eq!(facts.get::<Source>(t.id).unwrap().line, Some(9));
        let hit: Cue = facts.get(facts.find(&["Cue"]).iter().find(|e| !e.components.contains_key("Clip")).unwrap().id).unwrap();
        assert_eq!((hit.at, hit.bus.as_deref()), (1.5, Some("sfx")));
    }

    #[test]
    fn laid_out_text_is_in_the_frames_units_at_1080() {
        let t = serde_json::json!({"text": "OUT", "box": [960.0, 108.0, 480.0, 216.0], "size": 96.0, "family": "Inter", "lines": 2});
        let text = laid_text(&t, (1920, 1080));
        assert_eq!((text.box_, text.size, text.lines), (Some([0.5, 0.1, 0.75, 0.3]), Some(96.0), Some(2)));
        assert_eq!(laid_text(&t, (3840, 2160)).size, Some(48.0), "96 px in a 4K frame is 48 px at 1080");
    }

    #[test]
    fn a_transcript_becomes_lines_at_its_stops_and_pauses() {
        let dir = std::env::temp_dir().join(format!("raea-world-lines-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ws = serde_json::json!([{"text": "give", "start": 0.0, "end": 0.3}, {"text": "it", "start": 0.35, "end": 0.5}, {"text": "back.", "start": 0.55, "end": 0.9},
            {"text": "now", "start": 1.0, "end": 1.2}, {"text": "later", "start": 2.5, "end": 3.0}]);
        std::fs::write(dir.join("words.json"), ws.to_string()).unwrap();
        let lines = project_lines(None, &dir, 30.0);
        assert_eq!(lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(), ["give it back.", "now", "later"]);
        let mut facts = Facts::new(15, 1080, 1920, 30.0);
        put_lines(&mut facts, &lines, &[], 15, 30.0);
        assert_eq!(facts.get::<Line>(facts.find(&["Line"])[0].id).unwrap().text, "give it back.", "frame 15 is 0.5 s, in the first line");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
