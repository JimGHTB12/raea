//! A ComfyUI graph lifted to a flow: each output's chain named by what it makes. A sampler chain (KSampler or
//! SamplerCustomAdvanced, its loaders, prompts and latent) becomes image.generate, image.edit or video.generate with
//! its weights, LoRAs and settings kept as inputs; a partner API node (Gemini, Kling, Veo, Seedance, …) becomes the
//! same op on the made model that runs that vendor's model; loaders become media.load. What has no name stays ComfyUI
//! nodes in a `comfy` step, with the steps it reads bound in, so it runs exactly on a ComfyUI server.

use std::collections::BTreeMap;

use serde_json::{Map, Value as J, json};

use crate::api_nodes::api_step;
use crate::read::{Graph, link, upstream};
use raea_io_gen_core::flow::{Flow, Step};

const SAMPLERS: [&str; 4] = ["KSampler", "KSamplerAdvanced", "SamplerCustomAdvanced", "SamplerCustom"];
const LOADS: [(&str, &str); 5] = [("LoadImage", "image"), ("LoadImageMask", "image"), ("LoadVideo", "file"), ("VHS_LoadVideo", "video"), ("LoadAudio", "audio")];
/// latent or conditioning nodes that make a sampler's output a video, and the image inputs they carry
const VIDEO_LATENTS: [&str; 12] = ["EmptyHunyuanLatentVideo", "EmptyLTXVLatentVideo", "EmptyMochiLatentVideo", "EmptyCosmosLatentVideo", "WanImageToVideo", "WanFirstLastFrameToVideo", "Wan22ImageToVideoLatent", "LTXVImgToVideo", "HunyuanImageToVideo", "CosmosImageToVideoLatent", "WanVaceToVideo", "EmptyWanLatentVideo"];

/// What a lift found: the flow, and each sink's step (by sink node id) for a reader.
pub struct Lifted {
    pub flow: Flow,
    pub sinks: BTreeMap<String, String>,
}

fn is_sink(class: &str, schema_out: bool) -> bool {
    schema_out || class.starts_with("Save") || class.starts_with("Preview") || class == "VHS_VideoCombine"
}

struct Cx<'a> {
    g: &'a Graph,
    flow: Flow,
    /// comfy node id → step id, once lifted
    done: BTreeMap<String, String>,
}

impl Cx<'_> {
    fn input(&self, id: &str, name: &str) -> Option<&J> {
        self.g.get(id)?.inputs.get(name)
    }

    /// An input followed to a literal: through primitive and string nodes (a node's own value, text or string), a
    /// switch's chosen branch (the other when that one leads nowhere: an LLM enhancer), string edits to their source.
    fn lit(&self, id: &str, name: &str) -> Option<J> {
        self.lit_at(self.input(id, name)?, 0)
    }

    fn lit_at(&self, v: &J, depth: usize) -> Option<J> {
        let Some((src, _)) = link(v) else { return Some(v.clone()) };
        if depth > 12 {
            return None;
        }
        let n = self.g.get(src)?;
        let follow = |k: &str| n.inputs.get(k).and_then(|x| self.lit_at(x, depth + 1));
        if n.class.contains("Switch") {
            let on = follow("switch").and_then(|b| b.as_bool()).unwrap_or(false);
            let (a, b) = if on { ("on_true", "on_false") } else { ("on_false", "on_true") };
            return follow(a).filter(|x| x.as_str().is_none_or(|s| !s.trim().is_empty())).or_else(|| follow(b));
        }
        ["value", "text", "string", "prompt", "string_a", "text_a"].iter().find_map(|k| follow(k))
    }

    /// The first node upstream of `id` (itself not counted) whose class passes `f`.
    fn find(&self, id: &str, f: impl Fn(&str) -> bool) -> Option<String> {
        upstream(self.g, id).into_iter().skip(1).find(|n| self.g.get(n).is_some_and(|x| f(&x.class)))
    }

    /// The text a conditioning input leads to: the nearest text-encode node's words.
    fn text(&self, id: &str, input: &str) -> Option<String> {
        let (src, _) = link(self.input(id, input)?)?;
        let enc = std::iter::once(src.to_string()).chain(upstream(self.g, src)).find(|n| self.g.get(n).is_some_and(|x| x.class.contains("TextEncode")))?;
        ["text", "prompt", "t5xxl", "clip_l"].iter().find_map(|k| self.lit(&enc, k)).and_then(|v| v.as_str().map(str::to_string))
    }

    fn step(&mut self, comfy_id: &str, mut step: Step) -> String {
        let id = format!("n{}", comfy_id.replace(':', "_"));
        if step.from.is_none() {
            step.from = Some(format!("comfy {} {comfy_id}", self.g[comfy_id].class));
        }
        self.flow.steps.insert(id.clone(), step);
        self.done.insert(comfy_id.to_string(), id.clone());
        id
    }

    /// The step that makes what `id`'s output is: lifted when it can be named, else kept as ComfyUI nodes.
    fn lift(&mut self, id: &str) -> String {
        if let Some(s) = self.done.get(id) {
            return s.clone();
        }
        let class = self.g[id].class.clone();
        if let Some((_, key)) = LOADS.iter().find(|(c, _)| *c == class) {
            let path = self.lit(id, key).unwrap_or(J::Null);
            return self.step(id, Step::new("media.load", json!({"path": path})));
        }
        if class == "VAEDecode" || class == "VAEDecodeTiled" {
            let why = match self.find(id, |c| SAMPLERS.contains(&c)) {
                Some(s) => match self.sampler(&s) {
                    Ok(step) => return self.step(id, step),
                    Err(why) => why,
                },
                None => "no sampler upstream".to_string(),
            };
            let kept = self.keep(id);
            self.flow.steps.get_mut(&kept).unwrap().from = Some(format!("comfy VAEDecode {id}, kept: {why}"));
            return kept;
        }
        if class == "ImageUpscaleWithModel" {
            if let Some((img, _)) = self.input(id, "image").and_then(link).map(|(a, b)| (a.to_string(), b)) {
                let model = self.input(id, "upscale_model").and_then(link).and_then(|(m, _)| self.lit(m, "model_name"));
                let factor = model.as_ref().and_then(|m| m.as_str()).and_then(|m| m.chars().next().and_then(|c| c.to_digit(10))).unwrap_or(4);
                let image = self.lift(&img);
                return self.step(id, Step::new("image.upscale", json!({"image": format!("@{image}"), "factor": factor, "weights": model})));
            }
        }
        if let Some(step) = self.utility(id, &class) {
            return self.step(id, step);
        }
        let named: Map<String, J> = self.g[id].inputs.clone();
        if let Some(mut step) = api_step(&class, &named) {
            // its media inputs are other nodes' outputs: lift them and point at their steps
            for v in step.inputs.values_mut() {
                self.point(v);
            }
            return self.step(id, step);
        }
        self.keep(id)
    }

    /// Links inside a lifted step's inputs (`[id, slot]`, or arrays of them) replaced by `@step`.
    fn point(&mut self, v: &mut J) {
        if let Some((src, _)) = link(v).map(|(a, b)| (a.to_string(), b)) {
            *v = json!(format!("@{}", self.lift(&src)));
        } else if let Some(a) = v.as_array_mut() {
            for x in a {
                self.point(x);
            }
        }
    }

    /// A node with no name kept as ComfyUI: it and what it reads, up to media that other steps make (a load, a
    /// lifted generation), which are bound in as `bind: {"<node>.<input>": "@step"}`.
    fn keep(&mut self, id: &str) -> String {
        let mut nodes = Map::new();
        let mut bind = Map::new();
        let mut queue = vec![id.to_string()];
        while let Some(n) = queue.pop() {
            if nodes.contains_key(&n) {
                continue;
            }
            let node = self.g[&n].clone();
            for (name, v) in &node.inputs {
                let Some((src, _)) = link(v) else { continue };
                let sc = &self.g[src].class;
                let media = LOADS.iter().any(|(c, _)| c == sc) || ((sc == "VAEDecode" || sc == "VAEDecodeTiled") && src != id);
                if media && src != id {
                    let step = self.lift(src);
                    bind.insert(format!("{n}.{name}"), json!(format!("@{step}")));
                } else {
                    queue.push(src.to_string());
                }
            }
            nodes.insert(n.clone(), json!({"class_type": node.class, "inputs": node.inputs}));
        }
        let mut inputs = json!({"nodes": nodes, "output": id});
        if !bind.is_empty() {
            inputs["bind"] = J::Object(bind);
        }
        self.step(id, Step::new("comfy", inputs))
    }

    /// A sampler's chain as one step.
    fn sampler(&mut self, s: &str) -> Result<Step, String> {
        let class = self.g[s].class.clone();
        let mut i = Map::new();
        // the guider holds the prompts on SamplerCustomAdvanced; the sampler itself does elsewhere
        let cond = if class == "SamplerCustomAdvanced" { self.input(s, "guider").and_then(link).map(|(g, _)| g.to_string()).ok_or("its guider isn't linked")? } else { s.to_string() };
        let prompt = ["positive", "conditioning", "cond1"].iter().find_map(|k| self.text(&cond, k)).ok_or_else(|| format!("no prompt found up {}'s conditioning", self.g[&cond].class))?;
        i.insert("prompt".into(), json!(prompt));
        if let Some(neg) = self.text(&cond, "negative").or_else(|| self.text(&cond, "cond2")).filter(|t| !t.trim().is_empty()) {
            i.insert("negative".into(), json!(neg));
        }
        let pick = |cx: &Cx, from: Option<String>, keys: &[(&str, &str)], i: &mut Map<String, J>| {
            if let Some(n) = from {
                for (k, to) in keys {
                    if let Some(v) = cx.lit(&n, k).filter(|v| !v.is_null()) {
                        i.insert(to.to_string(), v);
                    }
                }
            }
        };
        pick(self, Some(s.into()), &[("seed", "seed"), ("noise_seed", "seed"), ("steps", "steps"), ("cfg", "cfg"), ("sampler_name", "sampler"), ("scheduler", "scheduler")], &mut i);
        if class == "SamplerCustomAdvanced" {
            let up = |name: &str| self.input(s, name).and_then(link).map(|(n, _)| n.to_string());
            pick(self, up("noise"), &[("noise_seed", "seed")], &mut i);
            pick(self, up("sigmas"), &[("steps", "steps"), ("scheduler", "scheduler"), ("denoise", "x_denoise")], &mut i);
            pick(self, up("sampler"), &[("sampler_name", "sampler")], &mut i);
            pick(self, Some(cond.clone()), &[("cfg", "cfg")], &mut i);
        }
        if let Some(g) = self.find(&cond, |c| c == "FluxGuidance") {
            pick(self, Some(g), &[("guidance", "cfg")], &mut i);
        }
        // weights and LoRAs: every loader up the model input
        let model_root = self.input(s, "model").or_else(|| self.input(&cond, "model")).and_then(link).map(|(m, _)| m.to_string());
        if let Some(m) = model_root {
            let chain = upstream(self.g, &m);
            let mut weights = Vec::new();
            let mut loras = Vec::new();
            for n in &chain {
                for k in ["ckpt_name", "unet_name"] {
                    if let Some(w) = self.lit(n, k) {
                        weights.push(w);
                    }
                }
                if let Some(l) = self.lit(n, "lora_name") {
                    loras.push(json!({"name": l, "strength": self.lit(n, "strength_model").unwrap_or(json!(1.0))}));
                }
            }
            if !weights.is_empty() {
                i.insert("weights".into(), if weights.len() == 1 { weights.remove(0) } else { J::Array(weights) });
            }
            if !loras.is_empty() {
                i.insert("loras".into(), J::Array(loras));
            }
        }
        // the latent says what kind of step this is
        let latent = self.input(s, "latent_image").and_then(link).map(|(l, _)| l.to_string());
        let video = self.find(s, |c| VIDEO_LATENTS.contains(&c) || c.ends_with("ToVideo") || c.contains("ToVideoLatent"));
        if let Some(v) = video {
            for (k, to) in [("width", "width"), ("height", "height"), ("length", "x_frames")] {
                if let Some(x) = self.lit(&v, k) {
                    i.insert(to.into(), x);
                }
            }
            for (k, to) in [("start_image", "start_image"), ("image", "start_image"), ("end_image", "end_image")] {
                if let Some((src, _)) = self.input(&v, k).and_then(link).map(|(a, b)| (a.to_string(), b)) {
                    let st = self.lift(&src);
                    i.insert(to.into(), json!(format!("@{st}")));
                }
            }
            return Ok(Step::new("video.generate", J::Object(i)));
        }
        let mut latent = latent.ok_or("its latent isn't linked")?;
        // a second pass (a refiner, a high- then low-noise sampler) or an upscaled latent: back to the first latent
        for _ in 0..8 {
            let n = &self.g[&latent];
            let next = if SAMPLERS.contains(&n.class.as_str()) { n.inputs.get("latent_image") } else if n.class.starts_with("LatentUpscale") { n.inputs.get("samples") } else { None };
            match next.and_then(link) {
                Some((l, _)) => latent = l.to_string(),
                None => break,
            }
        }
        let lclass = self.g[&latent].class.clone();
        if lclass.starts_with("Empty") {
            pick(self, Some(latent), &[("width", "width"), ("height", "height")], &mut i);
            return Ok(Step::new("image.generate", J::Object(i)));
        }
        // an encoded image: img2img (or inpainting, with a mask on the way)
        let enc = std::iter::once(latent.clone()).chain(upstream(self.g, &latent)).find(|n| self.g.get(n).is_some_and(|x| x.class.starts_with("VAEEncode") || x.class == "InpaintModelConditioning")).ok_or_else(|| format!("its latent comes from {}, neither empty nor an encoded image", lclass))?;
        let (px, _) = self.input(&enc, "pixels").and_then(link).map(|(a, b)| (a.to_string(), b)).ok_or("the encoded image isn't linked")?;
        let image = self.lift(&px);
        i.insert("image".into(), json!(format!("@{image}")));
        if let Some((m, _)) = self.input(&enc, "mask").or_else(|| self.input(&latent, "mask")).and_then(link).map(|(a, b)| (a.to_string(), b)) {
            let mask = self.lift(&m);
            i.insert("mask".into(), json!(format!("@{mask}")));
        }
        if let Some(d) = self.lit(s, "denoise") {
            i.insert("strength".into(), d);
        }
        Ok(Step::new("image.edit", J::Object(i)))
    }
}

impl Cx<'_> {
    /// The image utilities raea runs itself (image.crop, image.scale, image.stitch), from ComfyUI's core nodes; None
    /// for any other node, or one whose image or numbers aren't plain.
    fn utility(&mut self, id: &str, class: &str) -> Option<Step> {
        let num = |cx: &Self, k: &str| cx.lit(id, k).filter(|v| v.is_number());
        let image = |cx: &mut Self, k: &str| -> Option<J> {
            let (src, _) = cx.input(id, k).and_then(link).map(|(a, b)| (a.to_string(), b))?;
            Some(json!(format!("@{}", cx.lift(&src))))
        };
        let mut i = Map::new();
        match class {
            "ImageCrop" => {
                for k in ["width", "height", "x", "y"] {
                    i.insert(k.into(), num(self, k)?);
                }
                i.insert("image".into(), image(self, "image")?);
                Some(Step::new("image.crop", J::Object(i)))
            }
            "ImageScale" | "ImageScaleBy" | "ImageScaleToTotalPixels" => {
                match class {
                    // 0 keeps the picture's proportions on that side
                    "ImageScale" => {
                        for k in ["width", "height"] {
                            if let Some(v) = num(self, k).filter(|v| v.as_f64() != Some(0.0)) {
                                i.insert(k.into(), v);
                            }
                        }
                        let cover = self.lit(id, "crop").is_some_and(|c| c == "center");
                        i.insert("fit".into(), json!(if cover { "cover" } else { "stretch" }));
                    }
                    "ImageScaleBy" => { i.insert("factor".into(), num(self, "scale_by")?); }
                    _ => { i.insert("megapixels".into(), num(self, "megapixels")?); }
                }
                i.insert("image".into(), image(self, "image")?);
                Some(Step::new("image.scale", J::Object(i)))
            }
            "ImageStitch" => {
                let mut images = vec![image(self, "image1")?];
                images.extend(image(self, "image2"));
                i.insert("images".into(), J::Array(images));
                if let Some(d) = self.lit(id, "direction") {
                    i.insert("direction".into(), d);
                }
                Some(Step::new("image.stitch", J::Object(i)))
            }
            _ => None,
        }
    }
}

/// Every output of a graph lifted. A video sink (CreateVideo, VHS_VideoCombine) gives its step seconds from frames
/// and fps.
pub fn lift(g: &Graph, schema: &crate::schema::Schema) -> Lifted {
    let mut cx = Cx { g, flow: Flow::default(), done: BTreeMap::new() };
    let mut sinks = BTreeMap::new();
    for (id, n) in g {
        if !is_sink(&n.class, schema.get(&n.class).is_some_and(|c| c.output_node)) || n.class == "PreviewAny" {
            continue;
        }
        // what the sink saves: its first linked input, through CreateVideo-style wrappers that only add fps
        let Some((mut src, slot)) = n.inputs.values().find_map(link).map(|(a, b)| (a.to_string(), b)) else { continue };
        let mut fps = n.inputs.get("frame_rate").or(n.inputs.get("fps")).cloned();
        if g[&src].class == "CreateVideo" {
            fps = cx.lit(&src, "fps").or(fps);
            if let Some((img, _)) = g[&src].inputs.get("images").and_then(link) {
                src = img.to_string();
            }
        }
        let step = cx.lift(&src);
        if let (Some(fps), Some(s)) = (fps.and_then(|f| f.as_f64()), cx.flow.steps.get_mut(&step)) {
            if s.op == "video.generate" || s.inputs.contains_key("x_frames") {
                s.op = "video.generate".into();
                if let Some(frames) = s.inputs.remove("x_frames").and_then(|f| f.as_f64()) {
                    s.inputs.insert("seconds".into(), json!(((frames / fps) * 100.0).round() / 100.0));
                }
                s.inputs.insert("fps".into(), json!(fps));
            }
        }
        // a kept node saved from several of its outputs (a stem separator's four): the step gives each slot, in order
        if let Some(st) = cx.flow.steps.get_mut(&step).filter(|st| st.op == "comfy" && st.inputs.get("output").and_then(|o| o.as_str()) == Some(src.as_str())) {
            let slots = st.inputs.entry("slots").or_insert(json!([]));
            if let Some(a) = slots.as_array_mut().filter(|a| !a.contains(&json!(slot))) {
                a.push(json!(slot));
            }
        }
        if !cx.flow.out.contains(&step) {
            cx.flow.out.push(step.clone());
        }
        sinks.insert(id.clone(), step);
    }
    // one slot, the first: the plain case, no list
    for st in cx.flow.steps.values_mut() {
        if st.inputs.get("slots") == Some(&json!([0])) {
            st.inputs.remove("slots");
        }
    }
    Lifted { flow: cx.flow, sinks }
}
