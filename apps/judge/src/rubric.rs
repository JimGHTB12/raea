//! The frozen rubric (the 11-06 prompt pack, section 3, in clef-omni's native question types): a 0-10 score per category
//! as an ordered six-level rubric, and every named defect as a yes/no question, so a defect is a probability rather
//! than prose. clef-omni answers questions, not essays; the defects it can name are the ones asked. Nothing here says
//! where the pass line is (the loop decides, in code: `pass`).

use serde_json::{Value, json};

/// The six levels each category is scored on, lowest first (clef's `score`: the probability-weighted level), read as
/// 0, 2, 4, 6, 8, 10.
fn levels(what: &str) -> Value {
    json!([
        format!("0: no {what} to speak of; broken or missing"),
        format!("2: {what} is crude: primitives and placeholders"),
        format!("4: {what} is a rough draft, plainly unfinished"),
        format!("6: {what} is a believable draft"),
        format!("8: {what} is a production base an artist would accept"),
        format!("10: {what} is indistinguishable from the real thing"),
    ])
}

/// (id, the sheet it is judged on, the question)
pub const SCORES: [(&str, &str, &str); 7] = [
    ("composition", "colour", "Composition: the camera, framing and layout; every major object's position and scale against the brief's space."),
    ("form", "clay", "Form, judged on these grey clay tiles: silhouettes read at a glance, parts have real thickness, nothing blobby or melted, no primitive standing in for a thing, repeated things vary."),
    ("assembly", "clay", "Assembly, judged on these grey clay tiles: nothing floats, sinks, pokes through or rests on the wrong thing; joints touch; the low tile shows the feet resting."),
    ("materials", "colour", "Materials: every surface reads as its real material with wear where use puts it; nothing a flat default colour; glass is glass."),
    ("light", "colour", "Light: motivated by the place's own sources; exposure and contrast believable; no surface implausibly dark or bright."),
    ("detail", "colour", "Detail: close surfaces hold real geometry or texture at that distance, not smooth stand-ins; printed matter reads."),
    ("world", "colour", "World: one coherent real place matching the brief; nothing off-era, no real brands, nothing invented beyond the brief."),
];

/// (id, sheet, the yes/no question, blocking when yes)
pub const DEFECTS: [(&str, &str, &str, bool); 12] = [
    ("floating", "clay", "Does any object float above the surface it should rest on, with a visible gap under it?", true),
    ("intersecting", "clay", "Does any object pass through another object, a wall or the floor?", true),
    ("blobby", "clay", "Is any object a blobby, melted or lumpy shape where the real thing has crisp form?", true),
    ("stand_in", "clay", "Is any object just a plain box, ball or cylinder standing in for a more detailed real thing?", true),
    ("clones", "clay", "Are several copies of one thing identical and evenly spaced in an obviously artificial way?", false),
    ("flat_colour", "colour", "Is any large surface a flat, uniform default colour with no texture, variation or wear?", true),
    ("plastic", "colour", "Do the surfaces look like shiny uniform plastic or CG rather than their real materials?", true),
    ("garbled_text", "colour", "Is any visible text garbled, tiled or unreadable where it should read?", false),
    ("too_dark", "colour", "Is the scene so dark that important objects are hard to see?", true),
    ("too_bright", "colour", "Is the scene washed out, with large areas blown to white?", true),
    ("brands", "colour", "Is a real brand name or logo visible?", true),
    ("not_doing", "colour", "Does the scene show anything the brief says it must not contain?", true),
];

/// The paragraph clef reads before the sheet: what it is looking at, and the brief's words (no code, no log, no line).
pub fn state(sheet: &str, brief: &Value) -> String {
    let look = brief["look"].as_str().unwrap_or("");
    let logline = brief["logline"].as_str().unwrap_or("");
    let not: Vec<&str> = brief["notDoing"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    let tiles = "six tiles of one 3D scene, left to right and top to bottom: hero (the shot's own camera), front, right, top, iso, and low (a camera 0.3 m off the floor)";
    let what = if sheet == "clay" { format!("The image is {tiles}, rendered in clay: every surface plain grey, so only form and assembly show.") } else { format!("The image is {tiles}, in full colour.") };
    let target = if sheet == "colour" { " A second image, when given, is the target picture the scene should read as." } else { "" };
    let mut s = format!("{what}{target} Judge only what is in the pixels, as the person who will use the scene would.");
    if !logline.is_empty() || !look.is_empty() {
        s.push_str(&format!(" The brief: {logline} Look: {look}."));
    }
    if !not.is_empty() {
        s.push_str(&format!(" The scene must not contain: {}.", not.join("; ")));
    }
    s
}

/// The questions for one sheet, by id.
pub fn questions(sheet: &str) -> Value {
    let mut q = serde_json::Map::new();
    for (id, on, ask) in SCORES {
        if on == sheet {
            q.insert(format!("score_{id}"), json!({"type": "score", "instructions": ask, "criteria": levels(id)}));
        }
    }
    for (id, on, ask, _) in DEFECTS {
        if on == sheet {
            q.insert(format!("defect_{id}"), json!({"type": "noul", "instructions": ask}));
        }
    }
    Value::Object(q)
}

/// The blind read on the hero tile alone: is this a photograph (reported, never gated, until calibrated).
pub fn blind() -> Value {
    json!({"photo": {"type": "noul", "instructions": "Is this a photograph of a real place, rather than a computer render?"}})
}

/// clef's answers read into scores (0-10) and defects (probability of yes, blocking ones flagged).
pub fn read(answers: &Value) -> (serde_json::Map<String, Value>, Vec<Value>) {
    let mut scores = serde_json::Map::new();
    let mut defects = Vec::new();
    for (id, _, _) in SCORES {
        if let Some(a) = answers.get(format!("score_{id}")) {
            let level = a["score"].as_f64().unwrap_or(0.0);
            scores.insert(id.into(), json!((level * 2.0 * 100.0).round() / 100.0));
        }
    }
    for (id, _, ask, blocking) in DEFECTS {
        if let Some(a) = answers.get(format!("defect_{id}")) {
            let p = a["noul"].as_f64().unwrap_or(0.0);
            defects.push(json!({"defect": id, "p": (p * 1000.0).round() / 1000.0, "present": p >= 0.5, "level": if blocking { "blocking" } else { "detail" }, "asked": ask}));
        }
    }
    (scores, defects)
}

/// The loop's pass, in code: no blocking defect present, the lowest score 6 or more and the mean 7 or more (raea's
/// 0-10 scale; the prompt pack's line). Scores mapped by a calibration first, when there is one.
pub fn pass(scores: &serde_json::Map<String, Value>, defects: &[Value]) -> (bool, Vec<String>) {
    let vals: Vec<f64> = scores.values().filter_map(Value::as_f64).collect();
    let mut why = Vec::new();
    let blocking = blocking(defects);
    if !blocking.is_empty() {
        why.push(format!("blocking: {}", blocking.join(", ")));
    }
    let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let mean = if vals.is_empty() { 0.0 } else { vals.iter().sum::<f64>() / vals.len() as f64 };
    if vals.len() < SCORES.len() {
        why.push(format!("only {} of {} categories scored", vals.len(), SCORES.len()));
    }
    if min < 6.0 {
        let low: Vec<String> = scores.iter().filter(|(_, v)| v.as_f64().is_some_and(|x| x < 6.0)).map(|(k, _)| k.clone()).collect();
        why.push(format!("below a draft: {}", low.join(", ")));
    }
    if mean < 7.0 {
        why.push("the whole is under a production base".into());
    }
    (why.is_empty(), why)
}

/// The blocking defects present, by name.
pub fn blocking(defects: &[Value]) -> Vec<&str> {
    defects.iter().filter(|d| d["present"] == true && d["level"] == "blocking").filter_map(|d| d["defect"].as_str()).collect()
}
