//! A flow: generation steps by what they make, not by who runs them. Each step is an op from OPS (image.generate,
//! video.generate, audio.sfx, …) with its inputs, literal or `@step` (another step's output); a step may name a model
//! (`made:kling_3_pro_i2v`, `fal:<endpoint>`, `replicate:owner/name`, `openrouter:<model>`, `modal:<endpoint>`, `comfy`,
//! `local:sao`) or leave it to the router. A step the importer couldn't name keeps its
//! ComfyUI nodes whole (`comfy`), so nothing is lost and it runs where ComfyUI runs.
//!
//! On disk it's JSON (`{"flow": 1, "steps": {...}, "out": [...]}`); agents write it in TypeScript with raea/gen's
//! flow(), which prints the same JSON.

use std::collections::BTreeMap;

use serde_json::{Map, Value as J, json};
use sha2::{Digest, Sha256};

/// What a step makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Image,
    Video,
    Audio,
    /// a 3D model (glb)
    Mesh,
    /// words (a prompt an LLM wrote): a step reading one gets its text in place of the reference
    Text,
    Any,
}

/// The ops a step may be: its name, what it makes, the inputs it reads (the first ones it can't go without).
pub const OPS: &[(&str, Kind, &[&str], &[&str])] = &[
    ("image.generate", Kind::Image, &["prompt"], &["negative", "width", "height", "aspect_ratio", "seed", "steps", "cfg", "sampler", "scheduler", "weights", "loras", "refs", "resolution"]),
    ("image.edit", Kind::Image, &["image", "prompt"], &["mask", "strength", "negative", "seed", "steps", "cfg", "sampler", "scheduler", "weights", "loras", "refs", "aspect_ratio", "resolution"]),
    ("image.upscale", Kind::Image, &["image"], &["factor", "weights"]),
    ("image.remove_background", Kind::Image, &["image"], &[]),
    ("image.expand", Kind::Image, &["image", "aspect_ratio"], &["prompt"]),
    // run by raea itself (io/gen native.rs), no model: ComfyUI's ImageCrop, ImageScale* and ImageStitch
    ("image.crop", Kind::Image, &["image", "width", "height"], &["x", "y"]),
    ("image.scale", Kind::Image, &["image"], &["width", "height", "factor", "megapixels", "fit"]),
    ("image.stitch", Kind::Image, &["images"], &["direction"]),
    ("video.generate", Kind::Video, &[], &["prompt", "negative", "sampler", "scheduler", "seconds", "fps", "width", "height", "aspect_ratio", "start_image", "end_image", "refs", "audio", "seed", "steps", "cfg", "weights", "loras", "resolution"]),
    ("video.edit", Kind::Video, &["video", "prompt"], &["refs", "seconds", "aspect_ratio", "resolution"]),
    ("video.reframe", Kind::Video, &["video", "aspect_ratio"], &[]),
    ("video.upscale", Kind::Video, &["video"], &["factor"]),
    ("audio.music", Kind::Audio, &["prompt"], &["seconds", "seed", "weights"]),
    ("audio.sfx", Kind::Audio, &["prompt"], &["seconds", "video", "seed", "weights"]),
    ("audio.speech", Kind::Audio, &["text"], &["voice", "speed"]),
    ("mesh.generate", Kind::Mesh, &[], &["prompt", "image", "refs", "seed", "texture"]),
    ("text.generate", Kind::Text, &["prompt"], &["system", "image", "refs"]),
    ("media.load", Kind::Any, &[], &["path", "url"]),
    ("comfy", Kind::Any, &["nodes", "output"], &["bind", "slots"]),
];

pub fn op(name: &str) -> Option<&'static (&'static str, Kind, &'static [&'static str], &'static [&'static str])> {
    OPS.iter().find(|o| o.0 == name)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub op: String,
    pub inputs: Map<String, J>,
    /// `made:<id>`, `comfy`, `replicate`, `local:<name>`, or None for the router's choice
    pub model: Option<String>,
    /// how the step came to be (`comfy: KSampler 3 ← CheckpointLoaderSimple 4, …`), for a reader
    pub from: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Flow {
    pub steps: BTreeMap<String, Step>,
    pub out: Vec<String>,
}

/// `@id` (or `@id.N`, an output by index): the step it points at.
pub fn reference(v: &J) -> Option<&str> {
    v.as_str()?.strip_prefix('@').map(|r| r.split_once('.').map_or(r, |(step, _)| step))
}

/// Which of a step's files a reference names: `@step.N` the Nth (a batch's pictures); None for `@step`.
pub fn index(v: &J) -> Option<usize> {
    v.as_str()?.strip_prefix('@')?.split_once('.')?.1.parse().ok()
}

/// Every step a value points at, nested in arrays and objects too (`refs: ["@a", "@b"]`).
pub fn references(v: &J, out: &mut Vec<String>) {
    match v {
        J::String(_) => out.extend(reference(v).map(str::to_string)),
        J::Array(a) => a.iter().for_each(|x| references(x, out)),
        J::Object(o) => o.values().for_each(|x| references(x, out)),
        _ => {}
    }
}

impl Step {
    pub fn new(op: &str, inputs: J) -> Step {
        Step { op: op.into(), inputs: inputs.as_object().cloned().unwrap_or_default(), model: None, from: None }
    }

    pub fn needs(&self) -> Vec<String> {
        let mut out = Vec::new();
        // a comfy step's own nodes link by [id, slot], never by @; only its bindings point at other steps
        let inputs: Vec<&J> = if self.op == "comfy" { self.inputs.get("bind").into_iter().collect() } else { self.inputs.values().collect() };
        inputs.into_iter().for_each(|v| references(v, &mut out));
        out.sort();
        out.dedup();
        out
    }
}

impl Flow {
    pub fn parse(v: &J) -> Result<Flow, String> {
        if v["flow"].as_u64() != Some(1) {
            return Err("not a flow: it says {\"flow\": 1, \"steps\": {...}} at the top".into());
        }
        let mut f = Flow::default();
        for (id, s) in v["steps"].as_object().ok_or("a flow's steps are an object by id")? {
            let op = s["op"].as_str().ok_or(format!("step `{id}` has no op"))?;
            let mut step = Step::new(op, s["in"].clone());
            step.model = s["model"].as_str().map(str::to_string);
            step.from = s["from"].as_str().map(str::to_string);
            f.steps.insert(id.clone(), step);
        }
        f.out = v["out"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default();
        f.check()?;
        Ok(f)
    }

    pub fn to_json(&self) -> J {
        let steps: Map<String, J> = self.steps.iter().map(|(id, s)| {
            let mut o = json!({"op": s.op, "in": s.inputs});
            if let Some(m) = &s.model {
                o["model"] = json!(m);
            }
            if let Some(f) = &s.from {
                o["from"] = json!(f);
            }
            (id.clone(), o)
        }).collect();
        json!({"flow": 1, "steps": steps, "out": self.out})
    }

    /// Ops known, required inputs given, references to steps that exist, no cycles, outputs that exist.
    pub fn check(&self) -> Result<(), String> {
        for (id, s) in &self.steps {
            let (_, _, required, optional) = op(&s.op).ok_or_else(|| format!("step `{id}`: unknown op `{}` (ops: {})", s.op, OPS.iter().map(|o| o.0).collect::<Vec<_>>().join(", ")))?;
            for r in required.iter() {
                if !s.inputs.contains_key(*r) {
                    return Err(format!("step `{id}` ({}) needs `{r}`", s.op));
                }
            }
            if s.op != "comfy" {
                if let Some(k) = s.inputs.keys().find(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()) && !k.starts_with("x_")) {
                    return Err(format!("step `{id}` ({}) doesn't take `{k}` (it takes {}; a model's own setting goes in as x_<name>)", s.op, required.iter().chain(optional.iter()).copied().collect::<Vec<_>>().join(", ")));
                }
            }
            for n in s.needs() {
                if !self.steps.contains_key(&n) {
                    return Err(format!("step `{id}` points at `@{n}`, which isn't a step"));
                }
            }
        }
        for o in &self.out {
            if !self.steps.contains_key(o) {
                return Err(format!("out names `{o}`, which isn't a step"));
            }
        }
        self.order().map(|_| ())
    }

    /// Steps in an order that runs each after what it reads.
    pub fn order(&self) -> Result<Vec<String>, String> {
        let (mut done, mut out) = (std::collections::BTreeSet::new(), Vec::new());
        while out.len() < self.steps.len() {
            let ready: Vec<String> = self.steps.iter().filter(|(id, s)| !done.contains(*id) && s.needs().iter().all(|n| done.contains(n))).map(|(id, _)| id.clone()).collect();
            if ready.is_empty() {
                let left: Vec<&String> = self.steps.keys().filter(|k| !done.contains(*k)).collect();
                return Err(format!("these steps read each other in a circle: {left:?}"));
            }
            for r in ready {
                done.insert(r.clone());
                out.push(r);
            }
        }
        Ok(out)
    }

    /// Each step's key: a hash of its op, model and inputs, with each reference replaced by that step's key, so a
    /// step's key changes when anything it's made from does. Cached outputs are found by it.
    pub fn keys(&self) -> Result<BTreeMap<String, String>, String> {
        let mut keys = BTreeMap::new();
        for id in self.order()? {
            let s = &self.steps[&id];
            let inputs = J::Object(s.inputs.clone());
            let resolved = substitute(&inputs, &|r| keys.get(r).map(|k: &String| format!("key:{k}")));
            let text = format!("{}\u{0}{}\u{0}{}", s.op, s.model.as_deref().unwrap_or(""), canonical(&resolved));
            keys.insert(id, format!("{:x}", Sha256::digest(text.as_bytes()))[..24].to_string());
        }
        Ok(keys)
    }
}

/// A value with every `@ref` string replaced by what `f` gives for it (or kept).
pub fn substitute(v: &J, f: &dyn Fn(&str) -> Option<String>) -> J {
    match v {
        J::String(_) => reference(v).and_then(f).map(J::String).unwrap_or_else(|| v.clone()),
        J::Array(a) => J::Array(a.iter().map(|x| substitute(x, f)).collect()),
        J::Object(o) => J::Object(o.iter().map(|(k, x)| (k.clone(), substitute(x, f))).collect()),
        _ => v.clone(),
    }
}

/// JSON with object keys sorted and whole floats written as integers, so the same inputs always hash the same.
pub fn canonical(v: &J) -> String {
    match v {
        J::Object(o) => {
            let mut ks: Vec<&String> = o.keys().collect();
            ks.sort();
            format!("{{{}}}", ks.iter().map(|k| format!("{}:{}", J::String((*k).clone()), canonical(&o[*k]))).collect::<Vec<_>>().join(","))
        }
        J::Array(a) => format!("[{}]", a.iter().map(canonical).collect::<Vec<_>>().join(",")),
        J::Number(n) => match n.as_f64() {
            Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f as i64),
            _ => n.to_string(),
        },
        _ => v.to_string(),
    }
}
