//! A check's script (R4b): the steps both targets play, each element by its data-testid. The same JSON raea-rune-host
//! reads (engine/rune/src/script.rs): a press, text typed into a field, a wheel over an element, a key at the focus,
//! frames with no input, and the answer to the app's next pick, save or dialog. Any other field on a step (a `name`) is
//! left alone.

use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Press(String),
    Type { text: String, into: String },
    Wheel { over: String, dy: f64 },
    Key(String),
    Frames(u32),
    /// the answer to the app's next fs.pick, fs.save or dialog.message (`{"pick": [file]}`, `{"save": file}`,
    /// `{"dialog": index}`, docs/targets.feature T2d): a headless check opens no dialog
    Os(Value),
}

/// `json`, an array of steps.
pub fn parse(json: &str) -> Result<Vec<Step>, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("the script: {e}"))?;
    let list = v.as_array().ok_or("the script: an array of steps")?;
    list.iter().enumerate().map(|(i, s)| step(s).ok_or_else(|| format!("the script's step {}: {s} is none of press, type+into, wheel+dy, key, frames, pick, save, dialog", i + 1))).collect()
}

fn step(s: &Value) -> Option<Step> {
    let str_ = |k: &str| s.get(k).and_then(Value::as_str).map(str::to_string);
    if let Some(p) = str_("press") {
        return Some(Step::Press(p));
    }
    if let (Some(text), Some(into)) = (str_("type"), str_("into")) {
        return Some(Step::Type { text, into });
    }
    if let (Some(over), Some(dy)) = (str_("wheel"), s.get("dy").and_then(Value::as_f64)) {
        return Some(Step::Wheel { over, dy });
    }
    if let Some(k) = str_("key") {
        return Some(Step::Key(k));
    }
    if ["pick", "save", "dialog"].iter().any(|k| s.get(k).is_some()) {
        return Some(Step::Os(s.clone()));
    }
    s.get("frames").and_then(Value::as_u64).map(|n| Step::Frames(n as u32))
}

/// A step's file name: what it did, by its element (as raea-rune-host names it)
pub fn name(s: &Step) -> String {
    let n = match s {
        Step::Press(p) => format!("press-{p}"),
        Step::Type { into, .. } => format!("type-{into}"),
        Step::Wheel { over, .. } => format!("wheel-{over}"),
        Step::Key(k) => format!("key-{k}"),
        Step::Frames(n) => format!("frames-{n}"),
        Step::Os(v) => match v.get("pick").and_then(Value::as_array) {
            Some(f) => format!("pick-{}", f.len()),
            None => v.get("dialog").map_or("save".into(), |d| format!("dialog-{d}")),
        },
    };
    n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect()
}
