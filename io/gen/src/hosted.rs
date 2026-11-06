//! The hosted backends that take media inline (fal, Replicate, OpenRouter, a Modal endpoint): each step's references
//! become an https URL when the step that made them has one, else a data URI of its file.

use std::collections::BTreeMap;

use serde_json::{Map, Value as J, json};

use crate::flow::{Step, reference};
use crate::run::{Made1, Run};

/// Puts a local file where a hosted API can fetch it; its URL (fal's storage). Without one, files go as data URIs.
type Upload<'a> = Option<&'a dyn Fn(&std::path::Path) -> Result<String, String>>;

/// A made step's output as a value a hosted API reads.
fn media(done: &BTreeMap<String, Made1>, v: &J, up: Upload) -> Result<Option<String>, String> {
    let file = |f: &std::path::Path| match up { Some(u) => u(f), None => crate::fal::data_uri(f) };
    let Some(r) = reference(v) else { return Ok(None) };
    let up = done.get(r).ok_or(format!("@{r} isn't made yet"))?;
    if crate::flow::index(v).is_some() {
        return crate::run::file_of(done, v).ok_or(format!("{v} names no file of @{r}")).and_then(|f| file(f)).map(Some);
    }
    match (&up.url, up.files.first()) {
        (Some(u), _) => Ok(Some(u.clone())),
        (None, Some(f)) => file(f).map(Some),
        _ => Err(format!("@{r} made nothing")),
    }
}

/// The step's literal inputs, and its media as (input, value) in input order (arrays of references spread out).
fn split(s: &Step, done: &BTreeMap<String, Made1>, up: Upload) -> Result<(Map<String, J>, Vec<(String, String)>), String> {
    let (mut lit, mut med) = (Map::new(), Vec::new());
    for (k, v) in &s.inputs {
        let vals: Vec<&J> = match v { J::Array(a) if a.iter().any(|x| reference(x).is_some()) => a.iter().collect(), _ => vec![v] };
        let mut any = false;
        for x in vals {
            if let Some(m) = media(done, x, up)? {
                any = true;
                // a model's own media input (x_<name>) keeps its name
                if k.starts_with("x_") { lit.insert(k.clone(), json!(m)); } else { med.push((k.clone(), m)); }
            }
        }
        if !any {
            lit.insert(k.clone(), v.clone());
        }
    }
    Ok((lit, med))
}

fn stems(run: &Run, key: &str, k: usize) -> std::path::PathBuf {
    run.dir.join(if k == 0 { key.to_string() } else { format!("{key}-{k}") })
}

impl Run<'_> {
    pub(crate) fn fal(&self, endpoint: &str, s: &Step, key: &str, done: &BTreeMap<String, Made1>) -> Result<Made1, String> {
        let fal = crate::fal::Fal::from_settings()?;
        let upload = |f: &std::path::Path| fal.upload(f);
        let (lit, med) = split(s, done, Some(&upload))?;
        let resp = fal.run(endpoint, &crate::fal::body(&lit, &med, &fal.inputs(endpoint)), self.stall)?;
        let mut urls = Vec::new();
        crate::fal::urls(&resp, &mut urls);
        let files = if urls.is_empty() { vec![] } else { crate::run::save_all(&urls, &self.dir, key)? };
        if files.is_empty() {
            return Err(format!("fal {endpoint} answered with no files: {}", resp.to_string().chars().take(300).collect::<String>()));
        }
        Ok(Made1 { files, ..Default::default() })
    }

    /// Replicate: the inputs by their own names (x_<name> as <name>; prompt, image, seed as they are), media inline.
    /// A comfy step on Replicate sends its graph as any-comfyui-workflow's workflow_json.
    pub(crate) fn replicate(&self, model: &str, s: &Step, key: &str, done: &BTreeMap<String, Made1>) -> Result<Made1, String> {
        let rep = crate::replicate::Replicate::from_settings()?;
        let input = if s.op == "comfy" {
            if !s.needs().is_empty() {
                return Err("a comfy step on Replicate can't read other steps' files yet (any-comfyui-workflow takes one input_file); run it on a ComfyUI server".into());
            }
            let mut mini = crate::flow::Flow::default();
            mini.steps.insert("g".into(), Step { model: None, ..s.clone() });
            mini.out = vec!["g".into()];
            json!({"workflow_json": crate::comfy::export(&mini)?.to_string(), "output_format": "png"})
        } else {
            let (lit, med) = split(s, done, None)?;
            let mut b = json!({});
            for (k, v) in lit {
                let name = k.strip_prefix("x_").map(str::to_string).unwrap_or(if k == "negative" { "negative_prompt".into() } else { k });
                b[name.as_str()] = v;
            }
            for (k, v) in med {
                let name = match k.as_str() { "start_image" => "start_image", "end_image" => "end_image", "video" => "video", "mask" => "mask", _ => "image" };
                b[name] = json!(v);
            }
            b
        };
        let out = rep.run(model, &input, self.stall)?;
        let urls = crate::replicate::urls(&out);
        let files = if urls.is_empty() { vec![] } else { crate::run::save_all(&urls, &self.dir, key)? };
        if files.is_empty() {
            return Err(format!("Replicate {model} gave no files: {out}"));
        }
        Ok(Made1 { files, ..Default::default() })
    }

    /// OpenRouter: text.generate writes words (kept as .txt); image ops ask an image model for pictures.
    pub(crate) fn openrouter(&self, model: &str, s: &Step, key: &str, done: &BTreeMap<String, Made1>) -> Result<(Made1, u64), String> {
        let or = crate::openrouter::OpenRouter::from_settings()?;
        let (lit, med) = split(s, done, None)?;
        let prompt = lit.get("prompt").or(lit.get("text")).and_then(|p| p.as_str()).unwrap_or("").to_string();
        let system = lit.get("system").and_then(|p| p.as_str());
        let extra: Map<String, J> = lit.iter().filter_map(|(k, v)| k.strip_prefix("x_").map(|n| (n.to_string(), v.clone()))).collect();
        let images: Vec<String> = med.into_iter().map(|(_, v)| v).collect();
        let text = s.op == "text.generate";
        let (words, pics, cents) = or.complete(model, &prompt, system, &images, !text, &extra)?;
        let files = if text {
            let f = stems(self, key, 0).with_extension("txt");
            std::fs::write(&f, words.trim()).map_err(|e| format!("{}: {e}", f.display()))?;
            vec![f]
        } else {
            pics.iter().enumerate().map(|(k, u)| crate::openrouter::save(u, &stems(self, key, k))).collect::<Result<Vec<_>, _>>()?
        };
        if files.is_empty() {
            return Err(format!("OpenRouter {model} made no image (it said: {})", words.chars().take(200).collect::<String>()));
        }
        Ok((Made1 { files, ..Default::default() }, cents.ceil() as u64))
    }

    pub(crate) fn modal(&self, name: &str, s: &Step, key: &str, done: &BTreeMap<String, Made1>) -> Result<(Made1, u64), String> {
        let (mut lit, med) = split(s, done, None)?;
        for (k, v) in med {
            lit.insert(k, json!(v));
        }
        let (files, cents) = crate::modal::run(name, &s.op, &J::Object(lit), &stems(self, key, 0))?;
        Ok((Made1 { files, ..Default::default() }, cents))
    }

    /// A speech step on Fish Audio: its words (a text step's already put in by run), voice and speed.
    pub(crate) fn fish(&self, model: &str, s: &Step, key: &str) -> Result<(Made1, u64), String> {
        let (file, cents) = crate::fish::run(model, &s.op, &s.inputs, &stems(self, key, 0))?;
        Ok((Made1 { files: vec![file], ..Default::default() }, cents))
    }
}
