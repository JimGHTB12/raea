//! ComfyUI's partner API nodes (comfy.org bills them in its own credits) as made steps: the same vendor's model on
//! made.app, at the provider's price. A node's class names the vendor and what it does; its `model` setting, when it
//! has one, picks the version. 3D nodes (Tripo, Meshy, Rodin, Hunyuan) go to the same vendor on fal, since made makes
//! no meshes. The chat nodes
//! (Claude, OpenAI, Gemini) become text.generate on OpenRouter. Luma, Pixverse, Vidu and Runway stay ComfyUI.

use serde_json::{Map, Value as J, json};

use raea_io_gen_core::flow::Step;


/// Class prefix, what it does (`image`: generate, or edit when it's given an image; `video`: likewise with a video;
/// `upscale`, `vupscale`, `speech`, `lipsync`, …), and its made model unless VERSIONS picks another.
const TABLE: &[(&str, &str, &str)] = &[
    ("GeminiImage", "image", "nano_banana_2"),
    ("GeminiNanoBanana2", "image", "nano_banana_2"),
    ("OpenAIGPTImage", "image", "gpt_image_2"),
    ("OpenAIDalle", "image", "gpt_image_2"),
    ("ByteDanceSeedream", "image", "seedream_5_pro"),
    ("ByteDanceImageEdit", "image", "seedream_5_pro_edit"),
    ("GrokImage", "image", "grok_imagine_image_2"),
    ("Flux2Image", "image", "flux_2_pro"),
    ("Flux3Image", "image", "flux_2_pro"),
    ("FluxKontextMax", "image", "flux_kontext_max"),
    ("FluxKontext", "image", "flux_kontext_pro"),
    ("FluxProUltra", "image", "flux_1_1_pro"),
    ("FluxPro", "image", "flux_1_1_pro"),
    ("RecraftV4TextToImage", "image", "recraft_4_1"),
    ("RecraftTextToImage", "image", "recraft_4_1"),
    ("Krea", "image", "krea_2"),
    ("MetaMuseImage", "image", "muse_image"),
    ("QwenImage", "image", "qwen_image_3"),
    ("IdeogramV4", "image", "ideogram_v4_5_edit"),
    ("ReveImage", "image", "reve_create"),
    ("BriaRemoveBackground", "rembg", "bria_background_removal"),
    ("WavespeedImageUpscale", "upscale", "topaz_upscale_image"),
    ("MagnificImageUpscaler", "upscale", "topaz_upscale_image"),
    ("TopazImage", "upscale", "topaz_upscale_image"),
    ("WavespeedFlashVSR", "vupscale", "seedvr_video_upscale"),
    ("TopazVideo", "vupscale", "seedvr_video_upscale"),
    ("KlingOmniProEditVideo", "video", "kling_o3_edit_pro"),
    ("KlingOmniPro", "video", "kling_o3_pro"),
    ("Kling4", "video", "kling_4"),
    ("KlingLipSync", "lipsync", "kling_lipsync"),
    ("KlingAvatar", "lipsync", "kling_avatar_v2_pro"),
    ("KlingMotionControl", "video", "kling_3_motion_control_pro"),
    ("Kling", "video", "kling_3_pro"),
    ("Veo3", "video", "veo_3_1"),
    ("Veo", "video", "veo_3_1"),
    ("ByteDance2Reference", "video", "seedance_2_5_reference"),
    ("ByteDance2", "video", "seedance_2"),
    ("ByteDanceImageToVideo", "video", "seedance_1_5_pro"),
    ("ByteDanceTextToVideo", "video", "seedance_1_5_pro"),
    ("ByteDanceFirstLastFrame", "video", "seedance_1_5_pro"),
    ("MinimaxHailuo03Reference", "video", "minimax_h3_max_reference"),
    ("MinimaxHailuo03", "video", "minimax_h3_max"),
    ("MinimaxHailuo", "video", "hailuo_02"),
    ("Minimax", "video", "hailuo_02"),
    ("Wan3Reference", "video", "wan_3_0_prime_reference"),
    ("Wan3", "video", "wan_3"),
    ("Wan2VideoEdit", "video", "wan_3_reference_edit"),
    ("Wan2", "video", "wan_2_7"),
    ("WanImageToVideoApi", "video", "wan_2_5_i2v"),
    ("WanTextToVideoApi", "video", "wan_2_7"),
    ("WanReferenceVideoApi", "video", "wan_3_0_prime_reference"),
    ("HappyHorse", "video", "happy_horse_1_1"),
    ("LtxApi", "video", "ltx_2_5_fast"),
    ("GrokVideoExtend", "video", "grok_imagine_video_extend"),
    ("GrokVideo", "video", "grok_imagine_video_1_5"),
    ("FluxVideoEdit", "video", "flux_3"),
    ("SyncLipSync", "lipsync", "sync_lipsync_v3"),
    ("SyncTalkingImage", "lipsync", "omnihuman_1_5"),
    ("VeedFabric", "lipsync", "veed_fabric"),
    ("ElevenLabsTextToSpeech", "speech", "elevenlabs_turbo_v2_5"),
    ("HeyGenTextToSpeech", "speech", "elevenlabs_turbo_v2_5"),
    ("HeyGenVoiceTextToSpeech", "speech", "elevenlabs_turbo_v2_5"),
    ("ElevenLabsSoundEffects", "sfx", "elevenlabs_sound_effects_v2"),
    ("ElevenLabsMusic", "music", "elevenlabs_music_v2_5"),
    // the chat nodes write prompts: OpenRouter has each vendor's models
    ("ClaudeNode", "text", "anthropic/claude-sonnet-4.5"),
    ("OpenAIChatNode", "text", "openai/gpt-5"),
    ("OpenRouterLLMNode", "text", "moonshotai/kimi-k3.1-pro"),
    ("GeminiNode", "text", "google/gemini-2.5-flash"),
    // 3D: no made model makes meshes, so these go to fal directly (an endpoint id has a slash)
    ("TripoTextToModel", "mesh", "tripo3d/h3.1/text-to-3d"),
    ("TripoImageToModel", "mesh", "tripo3d/h3.1/image-to-3d"),
    ("TripoMultiviewToModel", "mesh", "tripo3d/h3.1/multiview-to-3d"),
    ("TripoP1ImageToModel", "mesh", "tripo3d/p1/image-to-3d"),
    ("TripoPSeriesImageToModel", "mesh", "tripo3d/p2/image-to-3d"),
    ("MeshyTextToModel", "mesh", "meshy/v7.1/text-to-3d"),
    ("MeshyImageToModel", "mesh", "meshy/v7.1/image-to-3d"),
    ("MeshyMultiImageToModel", "mesh", "meshy/v7.1/multi-image-to-3d"),
    ("Rodin3D", "mesh", "fal-ai/hyper3d/rodin/v2.5"),
    ("TencentImageToModel", "mesh", "fal-ai/hunyuan-3d/v3.1/pro/image-to-3d"),
    ("TencentTextToModel", "mesh", "fal-ai/hunyuan-3d/v3.1/pro/text-to-3d"),
];

/// A TABLE row's other models: the first whose text is in the node's `model` setting (lowercased) wins.
const VERSIONS: &[(&str, &str, &str)] = &[
    ("GeminiImage", "3-pro", "nano_banana_pro"), ("GeminiImage", "pro-image", "nano_banana_pro"), ("GeminiImage", "2.5", "nano_banana"),
    ("OpenAIGPTImage", "2.5", "gpt_image_2_5_flare"),
    ("ByteDanceSeedream", "4.5", "seedream_5_pro"), ("ByteDanceSeedream", "4", "seedream_4"),
    ("GrokImage", "quality", "grok_imagine_image_quality"),
    ("Kling", "2-6", "kling_2_6"), ("Kling", "2.6", "kling_2_6"), ("Kling", "2-5", "kling_2_5_turbo_pro"), ("Kling", "2.5", "kling_2_5_turbo_pro"),
    ("Veo3", "fast", "veo_3_fast"), ("Veo", "fast", "veo_3_fast"),
];

/// ComfyUI input names and the flow input each becomes.
const NAMES: &[(&str, &str)] = &[
    ("prompt", "prompt"), ("text", "prompt"), ("negative_prompt", "negative"), ("seed", "seed"), ("aspect_ratio", "aspect_ratio"),
    ("duration", "seconds"), ("resolution", "resolution"), ("generate_audio", "audio"),
    ("first_frame", "start_image"), ("start_frame", "start_image"), ("start_image", "start_image"),
    ("end_frame", "end_image"), ("last_frame", "end_image"), ("end_image", "end_image"),
    ("image", "image"), ("images", "image"), ("input_image", "image"), ("reference_images", "refs"), ("reference_image", "refs"),
    ("video", "video"), ("audio", "x_audio"),
];

/// A partner node as a step (its links still `[id, slot]`, for the lifter to point at steps), or None.
pub fn api_step(class: &str, inputs: &Map<String, J>) -> Option<Step> {
    let (row, does, model) = TABLE.iter().find(|(prefix, _, _)| class.starts_with(prefix))?;
    let setting = inputs.get("model").and_then(|m| m.as_str()).unwrap_or("").to_ascii_lowercase();
    let made = VERSIONS.iter().find(|(r, text, _)| r == row && setting.contains(text)).map_or(*model, |v| v.2);
    let mut i = Map::new();
    for (k, v) in inputs {
        // a dynamic combo's settings (`model.resolution`) are the chosen version's own
        let base = k.rsplit('.').next().unwrap_or(k);
        if let Some((_, to)) = NAMES.iter().find(|(from, _)| *from == base) {
            i.entry(to.to_string()).or_insert(v.clone());
        }
    }
    let (has_image, has_video) = (i.contains_key("image") || i.contains_key("refs"), i.contains_key("video"));
    let op = match *does {
        "image" if has_image => "image.edit",
        "image" => "image.generate",
        "video" if has_video => "video.edit",
        "video" => {
            if let Some(img) = i.remove("image") {
                i.entry("start_image".to_string()).or_insert(img);
            }
            "video.generate"
        }
        "upscale" => "image.upscale",
        "rembg" => "image.remove_background",
        "vupscale" => "video.upscale",
        "speech" => {
            if let Some(p) = i.remove("prompt") {
                i.insert("text".into(), p);
            }
            "audio.speech"
        }
        "sfx" => "audio.sfx",
        "music" => "audio.music",
        "mesh" => "mesh.generate",
        "text" => "text.generate",
        // lip sync: a video (or a still) and the audio it's synced to
        _ => {
            let media = i.remove("video").or_else(|| i.remove("image")).or_else(|| i.remove("start_image"));
            i.insert("video".into(), media.unwrap_or(J::Null));
            i.entry("prompt".to_string()).or_insert(json!(""));
            "video.edit"
        }
    };
    if op == "image.edit" && !i.contains_key("image") {
        if let Some(r) = i.remove("refs") {
            i.insert("image".into(), r);
        }
    }
    if op == "image.edit" || op == "video.edit" || op == "video.generate" {
        i.entry("prompt".to_string()).or_insert(json!(""));
    }
    // inputs an op doesn't take are carried as the model's own (x_…)
    let known = raea_io_gen_core::flow::op(op).map(|o| [o.2, o.3].concat()).unwrap_or_default();
    let i: Map<String, J> = i.into_iter().map(|(k, v)| if known.contains(&k.as_str()) || k.starts_with("x_") { (k, v) } else { (format!("x_{k}"), v) }).collect();
    let mut s = Step::new(op, J::Object(i));
    s.model = Some(match (*does, made.contains('/')) { ("text", _) => format!("openrouter:{made}"), (_, true) => format!("fal:{made}"), _ => format!("made:{made}") });
    s.from = Some(format!("comfy {class} ({})", if setting.is_empty() { "-" } else { &setting }));
    Some(s)
}
