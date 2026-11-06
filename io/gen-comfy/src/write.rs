//! A flow written as a ComfyUI workflow (API format). A step with ComfyUI weights and no made model becomes the
//! classic chain (checkpoint, LoRAs, prompts, latent, KSampler, decode); a `comfy` step goes back as its own nodes;
//! a load becomes LoadImage, LoadVideo or LoadAudio; every other step becomes a RaeaMade node, which tools/comfy/raea_made
//! (a custom node pack) runs on made.app from inside ComfyUI (text.generate on OpenRouter, its words on a STRING output). Each output gets a Save node of its kind.

use std::collections::BTreeMap;

use serde_json::{Map, Value as J, json};

use raea_io_gen_core::flow::{Flow, Kind, Step, op, reference};

/// A step's output in the written graph: the node, the slot, what kind of thing it is.
type Out = (String, usize, Kind);

struct W {
    nodes: Map<String, J>,
    outs: BTreeMap<String, Out>,
    /// a comfy step's outputs when it gives several slots (`slots`), in its order: `@step.N` is the Nth
    slots: BTreeMap<String, Vec<Out>>,
}

impl W {
    fn add(&mut self, id: String, class: &str, inputs: J) -> String {
        self.nodes.insert(id.clone(), json!({"class_type": class, "inputs": inputs}));
        id
    }

    fn link(&self, v: &J) -> Option<J> {
        let r = reference(v)?;
        let (n, s, _) = match crate::flow_index(v).and_then(|i| self.slots.get(r)?.get(i)) {
            Some(o) => o,
            None => self.outs.get(r)?,
        };
        Some(json!([n, s]))
    }
}

fn kind_of(step: &Step, w: &W) -> Kind {
    match op(&step.op).map(|o| o.1) {
        Some(Kind::Any) | None => {
            if step.op == "media.load" {
                let p = step.inputs.get("path").or(step.inputs.get("url")).and_then(|p| p.as_str()).unwrap_or("").to_ascii_lowercase();
                return if [".mp4", ".mov", ".webm", ".mkv"].iter().any(|e| p.ends_with(e)) { Kind::Video } else if [".wav", ".mp3", ".flac", ".m4a", ".ogg"].iter().any(|e| p.ends_with(e)) { Kind::Audio } else { Kind::Image };
            }
            let _ = w;
            Kind::Image
        }
        Some(k) => k,
    }
}

fn get<'a>(s: &'a Step, k: &str) -> Option<&'a J> {
    s.inputs.get(k)
}

/// Whether a step is written as the classic sampler chain: it names ComfyUI weights and isn't sent to made.
fn local_chain(s: &Step) -> bool {
    matches!(s.op.as_str(), "image.generate" | "image.edit") && get(s, "weights").is_some_and(|w| w.is_string()) && !s.model.as_deref().is_some_and(|m| m.starts_with("made:"))
}

pub fn write(flow: &Flow) -> Result<J, String> {
    let mut w = W { nodes: Map::new(), outs: BTreeMap::new(), slots: BTreeMap::new() };
    for id in flow.order()? {
        let s = &flow.steps[&id];
        let kind = kind_of(s, &w);
        let out: Out = if s.op == "media.load" {
            let p = get(s, "path").or(get(s, "url")).and_then(|p| p.as_str()).unwrap_or("input.png");
            let name = std::path::Path::new(p).file_name().and_then(|n| n.to_str()).unwrap_or(p).to_string();
            let (class, key) = match kind { Kind::Video => ("LoadVideo", "file"), Kind::Audio => ("LoadAudio", "audio"), _ => ("LoadImage", "image") };
            (w.add(id.clone(), class, json!({key: name})), 0, kind)
        } else if s.op == "comfy" {
            comfy(&mut w, &id, s)?
        } else if let Some(o) = native(&mut w, &id, s) {
            o
        } else if local_chain(s) {
            chain(&mut w, &id, s)
        } else {
            made(&mut w, &id, s, kind)
        };
        w.outs.insert(id, out);
    }
    for (k, o) in flow.out.iter().enumerate() {
        // one Save per output; a step giving several slots gets one per slot: save<k>, save<k>.1, save<k>.2, …
        let outs = w.slots.get(o).cloned().unwrap_or_else(|| vec![w.outs[o].clone()]);
        for (j, (n, slot, kind)) in outs.into_iter().enumerate() {
            let (class, key) = match kind { Kind::Video => ("SaveVideo", "video"), Kind::Audio => ("SaveAudio", "audio"), Kind::Text => ("PreviewAny", "source"), _ => ("SaveImage", "images") };
            let mut inputs = json!({key: [n, slot], "filename_prefix": format!("raea/{o}")});
            if class == "PreviewAny" {
                inputs = json!({key: [n, slot]});
            }
            if class == "SaveVideo" {
                inputs["format"] = json!("auto");
                inputs["codec"] = json!("auto");
            }
            w.add(if j == 0 { format!("save{k}") } else { format!("save{k}.{j}") }, class, inputs);
        }
    }
    Ok(J::Object(w.nodes))
}

/// A `comfy` step's nodes, ids prefixed by the step's, links rewritten, bound inputs pointed at the steps they name.
fn comfy(w: &mut W, id: &str, s: &Step) -> Result<Out, String> {
    let nodes = get(s, "nodes").and_then(|n| n.as_object()).ok_or(format!("step `{id}`: comfy nodes aren't an object"))?;
    let bind = get(s, "bind").and_then(|b| b.as_object()).cloned().unwrap_or_default();
    let pre = |n: &str| format!("{id}.{n}");
    for (nid, n) in nodes {
        let mut inputs = n["inputs"].as_object().cloned().unwrap_or_default();
        for (name, v) in inputs.iter_mut() {
            if let Some(target) = bind.get(&format!("{nid}.{name}")) {
                *v = w.link(target).ok_or(format!("step `{id}` binds {nid}.{name} to {target}, which isn't written yet"))?;
            } else if let Some(a) = v.as_array_mut().filter(|a| a.len() == 2 && a[1].is_u64() && a[0].is_string()) {
                a[0] = json!(pre(a[0].as_str().unwrap_or_default()));
            }
        }
        w.add(pre(nid), n["class_type"].as_str().unwrap_or_default(), J::Object(inputs));
    }
    let output = get(s, "output").and_then(|o| o.as_str()).ok_or(format!("step `{id}`: no output node"))?;
    let class = nodes.get(output).and_then(|n| n["class_type"].as_str()).unwrap_or("");
    let schema = crate::schema::load();
    let slots: Vec<usize> = get(s, "slots").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_u64()).map(|x| x as usize).collect()).filter(|v: &Vec<usize>| !v.is_empty()).unwrap_or(vec![0]);
    let mut outs = Vec::new();
    for slot in slots {
        // what the node puts out there, from the server's schema where it's known, else read from its name
        let ty = schema.get(class).and_then(|c| c.outputs.get(slot)).cloned().unwrap_or_default();
        outs.push(if ty == "MASK" {
            // a mask is saved as the picture of it
            let m = w.add(pre(&format!("{output}.picture{slot}")), "MaskToImage", json!({"mask": [pre(output), slot]}));
            (m, 0, Kind::Image)
        } else {
            let kind = match ty.as_str() {
                "VIDEO" => Kind::Video,
                "AUDIO" => Kind::Audio,
                "IMAGE" => Kind::Image,
                _ if class.contains("Video") => Kind::Video,
                _ if class.contains("Audio") => Kind::Audio,
                _ => Kind::Image,
            };
            (pre(output), slot, kind)
        });
    }
    let first = outs[0].clone();
    if outs.len() > 1 {
        w.slots.insert(id.to_string(), outs);
    }
    Ok(first)
}

/// raea's own image utilities as ComfyUI's core nodes: ImageCrop, ImageScale (or ScaleBy, ScaleToTotalPixels), and
/// ImageStitch chained for more than two pictures.
/// A step's setting `k`, else `d`.
fn num(s: &Step, k: &str, d: J) -> J {
    get(s, k).cloned().unwrap_or(d)
}

fn native(w: &mut W, id: &str, s: &Step) -> Option<Out> {
    let img = |w: &W, k: &str| get(s, k).and_then(|v| w.link(v));
    let node = match s.op.as_str() {
        "image.crop" => w.add(id.into(), "ImageCrop", json!({"image": img(w, "image")?, "width": num(s, "width", json!(512)), "height": num(s, "height", json!(512)), "x": num(s, "x", json!(0)), "y": num(s, "y", json!(0))})),
        "image.scale" => {
            let image = img(w, "image")?;
            if let Some(f) = get(s, "factor") {
                w.add(id.into(), "ImageScaleBy", json!({"image": image, "upscale_method": "lanczos", "scale_by": f}))
            } else if let Some(mp) = get(s, "megapixels") {
                w.add(id.into(), "ImageScaleToTotalPixels", json!({"image": image, "upscale_method": "lanczos", "megapixels": mp, "resolution_steps": 1}))
            } else {
                // contain has no core node: it scales as stretch would, to the box
                let crop = if get(s, "fit").is_some_and(|f| f == "cover") { "center" } else { "disabled" };
                w.add(id.into(), "ImageScale", json!({"image": image, "upscale_method": "lanczos", "width": num(s, "width", json!(0)), "height": num(s, "height", json!(0)), "crop": crop}))
            }
        }
        "image.stitch" => {
            let links: Vec<J> = get(s, "images")?.as_array()?.iter().filter_map(|v| w.link(v)).collect();
            let dir = num(s, "direction", json!("right"));
            let mut acc = links.first()?.clone();
            let mut last = String::new();
            for (k, next) in links.iter().enumerate().skip(1) {
                let nid = if k + 1 == links.len() { id.to_string() } else { format!("{id}.{k}") };
                last = w.add(nid, "ImageStitch", json!({"image1": acc, "image2": next, "direction": dir, "match_image_size": true, "spacing_width": 0, "spacing_color": "white"}));
                acc = json!([last, 0]);
            }
            if last.is_empty() {
                // one picture: nothing to stitch
                return Some((acc[0].as_str()?.to_string(), acc[1].as_u64()? as usize, Kind::Image));
            }
            last
        }
        _ => return None,
    };
    Some((node, 0, Kind::Image))
}

/// The classic chain for a step with weights: checkpoint, each LoRA, the prompts, a latent (empty, or the image
/// encoded), KSampler, decode.
fn chain(w: &mut W, id: &str, s: &Step) -> Out {
    let n = |k: &str| format!("{id}.{k}");
    w.add(n("ckpt"), "CheckpointLoaderSimple", json!({"ckpt_name": get(s, "weights")}));
    let (mut model, mut clip) = (json!([n("ckpt"), 0]), json!([n("ckpt"), 1]));
    for (k, l) in get(s, "loras").and_then(|l| l.as_array()).cloned().unwrap_or_default().iter().enumerate() {
        let lid = w.add(n(&format!("lora{k}")), "LoraLoader", json!({"model": model, "clip": clip, "lora_name": l["name"], "strength_model": l["strength"].as_f64().unwrap_or(1.0), "strength_clip": l["strength"].as_f64().unwrap_or(1.0)}));
        (model, clip) = (json!([lid, 0]), json!([lid, 1]));
    }
    w.add(n("pos"), "CLIPTextEncode", json!({"clip": clip, "text": num(s, "prompt", json!(""))}));
    w.add(n("neg"), "CLIPTextEncode", json!({"clip": clip, "text": num(s, "negative", json!(""))}));
    let (latent, denoise) = match get(s, "image").and_then(|v| w.link(v)) {
        Some(img) => {
            w.add(n("enc"), "VAEEncode", json!({"pixels": img, "vae": [n("ckpt"), 2]}));
            (json!([n("enc"), 0]), num(s, "strength", json!(0.75)))
        }
        None => {
            w.add(n("latent"), "EmptyLatentImage", json!({"width": num(s, "width", json!(1024)), "height": num(s, "height", json!(1024)), "batch_size": 1}));
            (json!([n("latent"), 0]), json!(1.0))
        }
    };
    w.add(n("sample"), "KSampler", json!({
        "model": model, "positive": [n("pos"), 0], "negative": [n("neg"), 0], "latent_image": latent,
        "seed": num(s, "seed", json!(0)), "steps": num(s, "steps", json!(25)), "cfg": num(s, "cfg", json!(7.0)),
        "sampler_name": num(s, "sampler", json!("euler")), "scheduler": num(s, "scheduler", json!("normal")), "denoise": denoise,
    }));
    w.add(n("decode"), "VAEDecode", json!({"samples": [n("sample"), 0], "vae": [n("ckpt"), 2]}));
    (n("decode"), 0, Kind::Image)
}

/// A RaeaMade node: the op, the made model, the text inputs as settings, the rest as JSON; media as links.
fn made(w: &mut W, id: &str, s: &Step, kind: Kind) -> Out {
    let mut inputs = Map::new();
    let mut params = Map::new();
    let mut refs = 0;
    for (k, v) in &s.inputs {
        let links: Vec<J> = match v {
            J::Array(a) => a.iter().filter_map(|x| w.link(x)).collect(),
            _ => w.link(v).into_iter().collect(),
        };
        if !links.is_empty() {
            for l in links {
                // ComfyUI inputs are one link each: a list of references becomes ref_1, ref_2, …
                let name = if matches!(v, J::Array(_)) || k == "refs" { refs += 1; format!("ref_{refs}") } else { k.clone() };
                inputs.insert(name, l);
            }
        } else if k == "prompt" || k == "text" {
            inputs.insert("prompt".into(), v.clone());
        } else {
            params.insert(k.clone(), v.clone());
        }
    }
    inputs.insert("op".into(), json!(s.op));
    inputs.insert("model".into(), json!(s.model.as_deref().and_then(|m| m.strip_prefix("made:")).unwrap_or("auto")));
    inputs.insert("params".into(), json!(J::Object(params).to_string()));
    inputs.entry("prompt").or_insert(json!(""));
    // RaeaMade's outputs: IMAGE, VIDEO, AUDIO, MASK, STRING
    let slot = match kind { Kind::Video => 1, Kind::Audio => 2, Kind::Text => 4, _ => 0 };
    let node = w.add(id.to_string(), "RaeaMade", J::Object(inputs));
    if s.op == "image.remove_background" {
        // the cut-out's transparency rides as the mask: joined back so it's saved
        let join = w.add(format!("{id}.alpha"), "JoinImageWithAlpha", json!({"image": [node, 0], "alpha": [node, 3]}));
        return (join, 0, kind);
    }
    (node, slot, kind)
}
