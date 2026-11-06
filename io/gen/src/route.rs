//! Where each step runs. A step that names its model goes there (`made:<id>`, `fal:<endpoint>`,
//! `replicate:<owner/name>`, `openrouter:<model>`, `modal:<endpoint>`, `fish:<model>`, `tripo:<model>`, `comfy`, `local:<name>`). mesh.generate goes to
//! Tripo 3D: through the tripo CLI on the founder's own key when this host has both (tripo.rs), else the same Tripo model on fal. text.generate goes to
//! OpenRouter (RAEA_GEN_TEXT_MODEL, else Kimi). Otherwise: a `comfy` step on the ComfyUI server; a step with ComfyUI weights on the server too when
//! the flow asks for the exact weights (`--exact`), else made's model for that op, which is noted as standing in for
//! them; a local model when the host has one for the op (sao for effects); made for the rest.

use serde_json::{Map, Value as J, json};

use crate::flow::{Step, reference};

#[derive(Clone, Debug, PartialEq)]
pub enum Backend {
    Made(String),
    Fal(String),
    Replicate(String),
    OpenRouter(String),
    Modal(String),
    /// Fish Audio's text-to-speech, with the founder's FISH_API_KEY (raea-io-gen-core's fish.rs)
    Fish(String),
    /// Tripo 3D through its own CLI and the founder's key (raea-io-gen-core's tripo.rs); the name is Tripo's model id or alias
    Tripo(String),
    Comfy,
    Local(String),
    Load,
    /// raea itself: native.rs
    Native,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Backend::Made(m) => write!(f, "made:{m}"),
            Backend::Fal(m) => write!(f, "fal:{m}"),
            Backend::Replicate(m) => write!(f, "replicate:{m}"),
            Backend::OpenRouter(m) => write!(f, "openrouter:{m}"),
            Backend::Modal(m) => write!(f, "modal:{m}"),
            Backend::Fish(m) => write!(f, "fish:{}", if m.is_empty() { crate::fish::DEFAULT_MODEL } else { m }),
            Backend::Tripo(m) => write!(f, "tripo:{m}"),
            Backend::Comfy => write!(f, "comfy"),
            Backend::Local(m) => write!(f, "local:{m}"),
            Backend::Load => write!(f, "load"),
            Backend::Native => write!(f, "raea"),
        }
    }
}

/// made's model for an op when the step doesn't name one: a good default at a fair price, by what the step is given.
pub fn made_default(s: &Step) -> Option<&'static str> {
    let has = |k: &str| s.inputs.contains_key(k);
    Some(match s.op.as_str() {
        "image.generate" => "nano_banana_2",
        "image.edit" => "nano_banana_2",
        "image.upscale" => "topaz_upscale_image",
        "image.remove_background" => "bria_background_removal",
        "image.expand" => "flux_2_pro_outpaint",
        "video.generate" if has("start_image") || has("end_image") => "kling_3_pro_i2v",
        "video.generate" if has("refs") => "seedance_2_5_reference",
        "video.generate" => "kling_3_pro",
        "video.edit" => "kling_o3_edit_pro",
        "video.upscale" => "seedvr_video_upscale",
        "audio.music" => "elevenlabs_music_v2_5",
        "audio.sfx" if has("video") => "mirelo_sfx_1_5",
        "audio.sfx" => "elevenlabs_sound_effects_v2",
        "audio.speech" => "elevenlabs_turbo_v2_5",
        _ => return None,
    })
}

/// The local model an op prefers when the host has it (raea's generation policy: SAO for effects).
pub fn local_default(s: &Step) -> Option<&'static str> {
    match s.op.as_str() {
        "audio.sfx" if !s.inputs.contains_key("video") => Some("sao"),
        _ => None,
    }
}

/// Settings for routing a whole flow.
#[derive(Clone, Debug, Default)]
pub struct Policy {
    /// steps with ComfyUI weights run those weights (on ComfyUI) rather than made's nearest model
    pub exact: bool,
    /// the local models the host can run
    pub local: Vec<String>,
    /// comfy steps go to Replicate's any-comfyui-workflow rather than a ComfyUI server (RAEA_COMFY_URL)
    pub comfy_on_replicate: bool,
    /// the tripo CLI and a key are on this host (tripo::available), so mesh steps run on Tripo itself
    pub tripo: bool,
}

/// Where a step runs, and a note when it stands in for something it was asked for.
pub fn route(s: &Step, p: &Policy) -> Result<(Backend, Option<String>), String> {
    if let Some(m) = &s.model {
        let (kind, name) = m.split_once(':').unwrap_or((m.as_str(), ""));
        return Ok((match kind {
            "made" => Backend::Made(name.into()),
            "fal" => Backend::Fal(name.into()),
            "replicate" => Backend::Replicate(name.into()),
            "openrouter" => Backend::OpenRouter(name.into()),
            "modal" => Backend::Modal(name.into()),
            "fish" => Backend::Fish(name.into()),
            "tripo" => Backend::Tripo(name.into()),
            "comfy" => Backend::Comfy,
            "local" => Backend::Local(name.into()),
            _ => return Err(format!("model `{m}`: say made:<id>, fal:<endpoint>, replicate:<owner/name>, openrouter:<model>, modal:<endpoint>, fish:<model>, tripo:<model>, comfy or local:<name>")),
        }, None));
    }
    match s.op.as_str() {
        "media.load" => return Ok((Backend::Load, None)),
        op if crate::native::OPS.contains(&op) => return Ok((Backend::Native, None)),
        "comfy" if p.comfy_on_replicate => return Ok((Backend::Replicate(crate::replicate::COMFY.into()), None)),
        "comfy" => return Ok((Backend::Comfy, None)),
        "text.generate" => return Ok((Backend::OpenRouter(crate::setting("RAEA_GEN_TEXT_MODEL").unwrap_or_else(|| "moonshotai/kimi-k3.1-pro".into())), None)),
        "mesh.generate" => {
            // Tripo by default (hunyuan-3d rapid on fal errored three times running, 10-29): its CLI on the founder's key when
            // the host has both, else fal's resale of the same model
            if p.tripo {
                return Ok((Backend::Tripo(String::new()), None));
            }
            let views = usize::from(s.inputs.contains_key("image")) + s.inputs.get("refs").and_then(|r| r.as_array()).map_or(0, Vec::len);
            return Ok((Backend::Fal(crate::tripo::fal_twin("", views)), Some("no tripo CLI with a key here (npm install -g tripo-cli; TRIPO_API_KEY): the same Tripo model on fal".into())));
        }
        _ => {}
    }
    if let Some(w) = s.inputs.get("weights").filter(|w| !w.is_null()) {
        if p.exact {
            return Ok((Backend::Comfy, None));
        }
        let m = made_default(s).ok_or(format!("{} has no made model", s.op))?;
        return Ok((Backend::Made(m.into()), Some(format!("made's {m} stands in for {w}{} (--exact runs them on ComfyUI)", if s.inputs.contains_key("loras") { " and its LoRAs" } else { "" }))));
    }
    if let Some(l) = local_default(s).filter(|l| p.local.iter().any(|x| x.as_str() == *l)) {
        return Ok((Backend::Local(l.into()), None));
    }
    let m = made_default(s).ok_or(format!("nothing runs {} yet: give it a model (made:<id>, fal:<endpoint>, comfy)", s.op))?;
    Ok((Backend::Made(m.into()), None))
}

/// Which made media role an input takes, from the roles the model declares.
fn role(input: &str, roles: &Map<String, J>) -> Option<String> {
    let wants: &[&str] = match input {
        "image" | "refs" => &["image", "reference_image", "reference_images", "image_urls", "start_image", "character_image"],
        "start_image" => &["start_image", "image", "first_frame"],
        "end_image" => &["end_image", "last_frame"],
        "video" => &["video", "source_video", "reference_video", "input_video"],
        "mask" => &["mask", "mask_image"],
        "x_audio" => &["audio", "audio_url"],
        _ => &[],
    };
    wants.iter().find(|w| roles.contains_key(**w)).map(|w| w.to_string()).or_else(|| (input == "image" || input == "refs").then(|| roles.keys().next().cloned()).flatten())
}

/// The nearest of a model's offered values (durations, aspect ratios) to what a step asks for.
fn nearest(want: f64, offered: &[f64]) -> Option<f64> {
    offered.iter().map(|&o| (o, (o - want).abs())).min_by(|a, b| a.1.total_cmp(&b.1)).map(|(o, _)| o)
}

/// The aspect ratio offered nearest to width:height.
fn ratio(w: f64, h: f64, offered: &[J]) -> Option<String> {
    let parse = |s: &str| s.split_once(':').and_then(|(a, b)| Some(a.parse::<f64>().ok()? / b.parse::<f64>().ok()?));
    offered.iter().filter_map(|o| o.as_str()).filter_map(|o| Some((o, parse(o)?))).min_by(|a, b| ((a.1 / (w / h)).ln().abs()).total_cmp(&(b.1 / (w / h)).ln().abs())).map(|(o, _)| o.to_string())
}

/// A step as a made request: its text, timing and shape fitted to what the model offers, and `media(input)` for
/// each media input (a `job_…`, `media_…` or https URL). Returns the body and what was changed or dropped to fit.
pub fn made_body(s: &Step, model: &J, media: &dyn Fn(&J) -> Result<String, String>) -> Result<(J, Vec<String>), String> {
    let mut b = json!({"model": model["id"]});
    let mut notes = Vec::new();
    let roles = model["mediaRoles"].as_object().cloned().unwrap_or_default();
    let mut medias = Vec::new();
    for (k, v) in &s.inputs {
        let vals: Vec<&J> = match v { J::Array(a) if a.iter().any(|x| reference(x).is_some()) => a.iter().collect(), _ => vec![v] };
        if vals.iter().any(|x| reference(x).is_some()) {
            let r = role(k, &roles).ok_or_else(|| format!("{} takes no {k} (its media roles: {})", model["id"], roles.keys().cloned().collect::<Vec<_>>().join(", ")))?;
            for x in vals {
                medias.push(json!({"value": media(x)?, "role": r}));
            }
            continue;
        }
        match k.as_str() {
            "prompt" | "text" => b["prompt"] = v.clone(),
            "negative" => b["negative_prompt"] = v.clone(),
            "seconds" => {
                let want = v.as_f64().unwrap_or(5.0);
                let offered: Vec<f64> = model["durations"].as_array().map(|a| a.iter().filter_map(|d| d.as_f64()).collect()).unwrap_or_default();
                let d = nearest(want, &offered).unwrap_or(want.round());
                if (d - want).abs() > 0.01 {
                    notes.push(format!("{want}s asked, {} offers {d}s", model["id"]));
                }
                b["duration"] = json!(d as u64);
            }
            "aspect_ratio" | "resolution" | "audio" | "seed" => b[k.as_str()] = v.clone(),
            k if k.starts_with("x_") => b[&k[2..]] = v.clone(),
            "width" | "height" | "steps" | "cfg" | "sampler" | "scheduler" | "weights" | "loras" | "strength" | "fps" | "factor" => {}
            _ => b[k.as_str()] = v.clone(),
        }
    }
    // an aspect the model doesn't offer: ComfyUI's "auto" is made's match_input_image (or the model's default), a
    // ratio the nearest one offered
    if let (Some(a), Some(offered)) = (b.get("aspect_ratio").and_then(|a| a.as_str()).map(str::to_string), model["aspectRatios"].as_array()) {
        if !offered.iter().any(|o| o == a.as_str()) {
            let has = |x: &str| offered.iter().any(|o| o == x);
            let pick = match a.split_once(':').and_then(|(w, h)| Some((w.parse::<f64>().ok()?, h.parse::<f64>().ok()?))) {
                Some((w, h)) => ratio(w, h, offered),
                None if has("match_input_image") && !medias.is_empty() => Some("match_input_image".into()),
                None => None,
            };
            notes.push(format!("aspect {a} isn't one {} offers: {}", model["id"], pick.as_deref().unwrap_or("its default")));
            match pick {
                Some(p) => b["aspect_ratio"] = json!(p),
                None => { b.as_object_mut().map(|o| o.remove("aspect_ratio")); }
            }
        }
    }
    if b.get("aspect_ratio").is_none() {
        if let (Some(w), Some(h), Some(offered)) = (s.inputs.get("width").and_then(|v| v.as_f64()), s.inputs.get("height").and_then(|v| v.as_f64()), model["aspectRatios"].as_array()) {
            if let Some(r) = ratio(w, h, offered) {
                b["aspect_ratio"] = json!(r);
            }
        }
    }
    if !medias.is_empty() {
        b["medias"] = J::Array(medias);
    }
    if b.get("prompt").is_none() || b["prompt"].as_str().is_some_and(|p| p.trim().is_empty()) {
        b.as_object_mut().map(|o| o.remove("prompt"));
    }
    let (fitted, dropped) = crate::made::fit(model, b);
    notes.extend(dropped.into_iter().map(|d| format!("{} doesn't take `{d}`; left out", model["id"])));
    Ok((fitted, notes))
}
