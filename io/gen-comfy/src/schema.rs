//! What each ComfyUI node class takes, in order: a UI workflow keeps a node's settings as `widgets_values`, a list by
//! position, and only the class's schema names them. The schema comes from a ComfyUI server's /object_info, kept as
//! ~/.cache/raea-gen/comfy-object-info.json by `raea gen comfy schema [URL]`; without it, CORE names the settings of
//! the classes most workflows are built from.

use std::collections::BTreeMap;

use serde_json::Value as J;

/// One setting a node shows as a widget: its name, its type, whether a seed-style "control after generate" value
/// follows it in widgets_values, and (a dynamic combo) the settings each choice adds after it.
#[derive(Clone, Debug)]
pub struct Widget {
    pub name: String,
    pub ty: String,
    pub control: bool,
    pub choices: BTreeMap<String, Vec<Widget>>,
}

#[derive(Clone, Debug, Default)]
pub struct Class {
    pub widgets: Vec<Widget>,
    /// link-only inputs (MODEL, IMAGE, …) by name
    pub links: Vec<(String, String)>,
    pub outputs: Vec<String>,
    /// a partner API node (comfy.org's paid nodes: Kling, Veo, Gemini, …)
    pub api: bool,
    pub output_node: bool,
}

pub type Schema = BTreeMap<String, Class>;

/// The settings, in widget order, of the classes most workflows use. `+` marks a seed followed by its control value.
const CORE: &[(&str, &str)] = &[
    ("CheckpointLoaderSimple", "ckpt_name"),
    ("CLIPTextEncode", "text"),
    ("KSampler", "seed+ steps cfg sampler_name scheduler denoise"),
    ("KSamplerAdvanced", "add_noise noise_seed+ steps cfg sampler_name scheduler start_at_step end_at_step return_with_leftover_noise"),
    ("EmptyLatentImage", "width height batch_size"),
    ("EmptySD3LatentImage", "width height batch_size"),
    ("SaveImage", "filename_prefix"),
    ("LoadImage", "image upload"),
    ("LoraLoader", "lora_name strength_model strength_clip"),
    ("LoraLoaderModelOnly", "lora_name strength_model"),
    ("UNETLoader", "unet_name weight_dtype"),
    ("CLIPLoader", "clip_name type device"),
    ("DualCLIPLoader", "clip_name1 clip_name2 type device"),
    ("VAELoader", "vae_name"),
    ("KSamplerSelect", "sampler_name"),
    ("BasicScheduler", "scheduler steps denoise"),
    ("RandomNoise", "noise_seed+"),
    ("FluxGuidance", "guidance"),
    ("CFGGuider", "cfg"),
    ("ModelSamplingSD3", "shift"),
    ("ModelSamplingAuraFlow", "shift"),
    ("SaveVideo", "filename_prefix format codec"),
    ("CreateVideo", "fps"),
    ("LoadVideo", "file upload"),
    ("ImageScaleBy", "upscale_method scale_by"),
    ("UpscaleModelLoader", "model_name"),
    ("SaveAudio", "filename_prefix"),
    ("PrimitiveStringMultiline", "value"),
    ("PrimitiveString", "value"),
    ("PrimitiveInt", "value+"),
    ("PrimitiveFloat", "value"),
    ("PrimitiveBoolean", "value"),
    ("WanImageToVideo", "width height length batch_size"),
    ("EmptyHunyuanLatentVideo", "width height length batch_size"),
];

fn core() -> Schema {
    CORE.iter().map(|(class, ws)| {
        let widgets = ws.split_whitespace().map(|w| Widget { name: w.trim_end_matches('+').into(), ty: String::new(), control: w.ends_with('+'), choices: BTreeMap::new() }).collect();
        (class.to_string(), Class { widgets, ..Default::default() })
    }).collect()
}

const WIDGET_TYPES: [&str; 6] = ["INT", "FLOAT", "STRING", "BOOLEAN", "COMBO", "COMFY_DYNAMICCOMBO_V3"];

fn widget(name: &str, spec: &J) -> Option<Widget> {
    let ty = match &spec[0] {
        J::Array(_) => "COMBO".to_string(),
        J::String(s) => s.clone(),
        _ => return None,
    };
    let opts = &spec[1];
    if !WIDGET_TYPES.contains(&ty.as_str()) || opts["forceInput"] == true {
        return None;
    }
    let control = !matches!(opts.get("control_after_generate"), None | Some(J::Bool(false)));
    let mut choices = BTreeMap::new();
    if ty == "COMFY_DYNAMICCOMBO_V3" {
        for o in opts["options"].as_array().into_iter().flatten() {
            let key = o["key"].as_str().unwrap_or_default().to_string();
            choices.insert(key, ordered(&o["inputs"], None).0);
        }
    }
    Some(Widget { name: name.into(), ty, control, choices })
}

/// Widgets and links of an `input` block, in the order the class declares (input_order, else the block's own).
fn ordered(input: &J, order: Option<&J>) -> (Vec<Widget>, Vec<(String, String)>) {
    let (mut ws, mut ls) = (Vec::new(), Vec::new());
    for part in ["required", "optional"] {
        let block = &input[part];
        let names: Vec<String> = match order.and_then(|o| o[part].as_array()) {
            Some(a) => a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
            None => block.as_object().map(|o| o.keys().cloned().collect()).unwrap_or_default(),
        };
        for n in names {
            let spec = &block[&n];
            match widget(&n, spec) {
                Some(w) => ws.push(w),
                None => ls.push((n, spec[0].as_str().unwrap_or("*").to_string())),
            }
        }
    }
    (ws, ls)
}

/// A schema from /object_info's JSON.
pub fn from_object_info(v: &J) -> Schema {
    v.as_object().map(|o| o.iter().map(|(class, c)| {
        let (widgets, links) = ordered(&c["input"], c.get("input_order"));
        let outputs = c["output"].as_array().map(|a| a.iter().map(|t| String::from(t.as_str().unwrap_or("*"))).collect()).unwrap_or_default();
        (class.clone(), Class { widgets, links, outputs, api: c["api_node"] == true, output_node: c["output_node"] == true })
    }).collect()).unwrap_or_default()
}

/// The server's node classes, kept in raea gen's cache under this name (made::cached).
pub const CACHE_NAME: &str = "comfy-object-info.json";

/// The schema this machine has: the cached /object_info over CORE.
pub fn load() -> Schema {
    let mut s = core();
    if let Some(v) = std::fs::read(raea_io_gen_core::made::cached(CACHE_NAME)).ok().and_then(|b| serde_json::from_slice::<J>(&b).ok()) {
        s.extend(from_object_info(&v));
    }
    s
}

/// The value kept at a widget slot: an index into widgets_values, or a dynamic combo's setting by name.
pub fn slot_value(values: &J, at: &J) -> J {
    match at {
        J::String(k) => values[k.as_str()].clone(),
        _ => values[at.as_u64().unwrap_or(0) as usize].clone(),
    }
}

/// A UI node's widgets_values named: (name, value) for each, the control values after seeds skipped, a dynamic
/// combo's added settings as `combo.setting`. Values beyond what the schema names keep their place as `widget_N`.
pub fn name_widgets(class: Option<&Class>, values: &J) -> Vec<(String, J)> {
    widget_slots(class, values).into_iter().map(|(name, at)| {
        let v = slot_value(values, &at);
        (name, v)
    }).collect()
}

/// Where each named setting is in a UI node's widgets_values: its index, or its key when a node pack saves its
/// widgets by name (an edit writes back there; apps/app's whiteboards change a flow's settings this way).
pub fn widget_slots(class: Option<&Class>, values: &J) -> Vec<(String, J)> {
    // some node packs (VideoHelperSuite) save their widgets by name already
    if let Some(o) = values.as_object() {
        return o.keys().filter(|k| k.as_str() != "videopreview").map(|k| (k.clone(), J::String(k.clone()))).collect();
    }
    let vals = values.as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();
    let mut i = 0;
    fn walk(ws: &[Widget], prefix: &str, vals: &[J], i: &mut usize, out: &mut Vec<(String, J)>) {
        for w in ws {
            let Some(v) = vals.get(*i) else { return };
            out.push((format!("{prefix}{}", w.name), J::from(*i)));
            *i += 1;
            // a control value ("randomize", "fixed", …) rides after a seed; an older save may leave it out
            if w.control && vals.get(*i).and_then(|c| c.as_str()).is_some_and(|c| ["fixed", "increment", "decrement", "randomize"].contains(&c)) {
                *i += 1;
            }
            if let Some(sub) = v.as_str().and_then(|k| w.choices.get(k)) {
                walk(sub, &format!("{}.", w.name), vals, i, out);
            }
        }
    }
    if let Some(c) = class {
        walk(&c.widgets, "", &vals, &mut i, &mut out);
    }
    for k in i..vals.len() {
        out.push((format!("widget_{k}"), J::from(k)));
    }
    out
}
