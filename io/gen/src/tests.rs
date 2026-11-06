use serde_json::json;

use crate::comfy::{lift::lift, read::read, schema, write::write};
use crate::flow::{Flow, Step};
use crate::route::{Backend, Policy, route};
use crate::run::Local;

/// The classic text-to-image graph in API format, as ComfyUI's own example saves it.
fn sd_api() -> serde_json::Value {
    json!({
        "4": {"class_type": "CheckpointLoaderSimple", "inputs": {"ckpt_name": "sd_xl_base_1.0.safetensors"}},
        "10": {"class_type": "LoraLoader", "inputs": {"model": ["4", 0], "clip": ["4", 1], "lora_name": "ink.safetensors", "strength_model": 0.8, "strength_clip": 0.8}},
        "6": {"class_type": "CLIPTextEncode", "inputs": {"clip": ["10", 1], "text": "a lighthouse in fog"}},
        "7": {"class_type": "CLIPTextEncode", "inputs": {"clip": ["10", 1], "text": "blurry"}},
        "5": {"class_type": "EmptyLatentImage", "inputs": {"width": 1216, "height": 832, "batch_size": 1}},
        "3": {"class_type": "KSampler", "inputs": {"model": ["10", 0], "positive": ["6", 0], "negative": ["7", 0], "latent_image": ["5", 0], "seed": 7, "steps": 30, "cfg": 6.5, "sampler_name": "euler", "scheduler": "normal", "denoise": 1.0}},
        "8": {"class_type": "VAEDecode", "inputs": {"samples": ["3", 0], "vae": ["4", 2]}},
        "9": {"class_type": "SaveImage", "inputs": {"images": ["8", 0], "filename_prefix": "x"}},
    })
}

#[test]
fn a_sampler_chain_lifts_to_one_generate_step() {
    let s = schema::load();
    let l = lift(&read(&sd_api(), &s).unwrap(), &s);
    assert_eq!(l.flow.out, vec!["n8"]);
    let st = &l.flow.steps["n8"];
    assert_eq!(st.op, "image.generate");
    assert_eq!(st.inputs["prompt"], "a lighthouse in fog");
    assert_eq!(st.inputs["negative"], "blurry");
    assert_eq!(st.inputs["weights"], "sd_xl_base_1.0.safetensors");
    assert_eq!(st.inputs["loras"][0]["name"], "ink.safetensors");
    assert_eq!((st.inputs["width"].as_u64(), st.inputs["seed"].as_u64()), (Some(1216), Some(7)));
    l.flow.check().unwrap();
}

#[test]
fn weights_route_to_made_unless_exact() {
    let s = schema::load();
    let l = lift(&read(&sd_api(), &s).unwrap(), &s);
    let st = &l.flow.steps["n8"];
    let (b, note) = route(st, &Policy::default()).unwrap();
    assert_eq!(b, Backend::Made("nano_banana_2".into()));
    assert!(note.unwrap().contains("sd_xl_base"));
    assert_eq!(route(st, &Policy { exact: true, ..Default::default() }).unwrap().0, Backend::Comfy);
}

#[test]
fn a_lifted_chain_writes_back_and_lifts_the_same() {
    let s = schema::load();
    let once = lift(&read(&sd_api(), &s).unwrap(), &s).flow;
    let api = write(&once).unwrap();
    let twice = lift(&read(&api, &s).unwrap(), &s).flow;
    assert_eq!(once.steps.values().map(|s| &s.inputs).collect::<Vec<_>>(), twice.steps.values().map(|s| &s.inputs).collect::<Vec<_>>());
}

#[test]
fn unknown_nodes_stay_comfy_with_their_media_bound() {
    let g = json!({
        "1": {"class_type": "LoadImage", "inputs": {"image": "cat.png"}},
        "2": {"class_type": "SomePackFilter", "inputs": {"image": ["1", 0], "amount": 3}},
        "3": {"class_type": "SaveImage", "inputs": {"images": ["2", 0]}},
    });
    let s = schema::load();
    let f = lift(&read(&g, &s).unwrap(), &s).flow;
    let st = &f.steps["n2"];
    assert_eq!(st.op, "comfy");
    assert_eq!(st.inputs["bind"]["2.image"], "@n1");
    assert_eq!(f.steps["n1"].op, "media.load");
    // written back, the bound input is a link to the load again
    let api = write(&f).unwrap();
    assert_eq!(api["n2.2"]["inputs"]["image"], json!(["n1", 0]));
}

#[test]
fn partner_nodes_become_made_steps() {
    let g = json!({
        "1": {"class_type": "LoadImage", "inputs": {"image": "a.png"}},
        "2": {"class_type": "KlingFirstLastFrameNode", "inputs": {"prompt": "a slow push in", "duration": 5, "first_frame": ["1", 0], "end_frame": ["1", 0], "generate_audio": false, "model": "kling-v3", "model.resolution": "1080p", "seed": 1}},
        "3": {"class_type": "SaveVideo", "inputs": {"video": ["2", 0]}},
    });
    let s = schema::load();
    let f = lift(&read(&g, &s).unwrap(), &s).flow;
    let st = &f.steps["n2"];
    assert_eq!((st.op.as_str(), st.model.as_deref()), ("video.generate", Some("made:kling_3_pro")));
    assert_eq!((st.inputs["start_image"].as_str(), st.inputs["end_image"].as_str(), st.inputs["seconds"].as_u64()), (Some("@n1"), Some("@n1"), Some(5)));
    f.check().unwrap();
}

#[test]
fn ui_widgets_are_named_and_subgraphs_inlined() {
    // a UI-format save: a subgraph holding the prompt encode, a reroute, a muted note
    let ui = json!({
        "nodes": [
            {"id": 4, "type": "CheckpointLoaderSimple", "mode": 0, "inputs": [], "outputs": [{"type": "MODEL"}, {"type": "CLIP"}, {"type": "VAE"}], "widgets_values": ["flux1-dev.safetensors"]},
            {"id": 20, "type": "sub-1", "mode": 0, "inputs": [{"name": "clip", "type": "CLIP", "link": 1}], "outputs": [{"type": "CONDITIONING"}], "widgets_values": []},
            {"id": 5, "type": "EmptyLatentImage", "mode": 0, "inputs": [], "widgets_values": [512, 768, 1]},
            {"id": 3, "type": "KSampler", "mode": 0, "inputs": [{"name": "model", "link": 2}, {"name": "positive", "link": 3}, {"name": "negative", "link": 3}, {"name": "latent_image", "link": 4}], "widgets_values": [42, "randomize", 20, 3.5, "euler", "simple", 1.0]},
            {"id": 8, "type": "VAEDecode", "mode": 0, "inputs": [{"name": "samples", "link": 5}, {"name": "vae", "link": 6}], "widgets_values": []},
            {"id": 30, "type": "Reroute", "mode": 0, "inputs": [{"name": "", "link": 7}], "widgets_values": []},
            {"id": 9, "type": "SaveImage", "mode": 0, "inputs": [{"name": "images", "link": 8}], "widgets_values": ["out"]},
            {"id": 11, "type": "Note", "mode": 0, "inputs": [], "widgets_values": ["hello"]}
        ],
        "links": [[1, 4, 1, 20, 0, "CLIP"], [2, 4, 0, 3, 0, "MODEL"], [3, 20, 0, 3, 1, "CONDITIONING"], [4, 5, 0, 3, 3, "LATENT"], [5, 3, 0, 8, 0, "LATENT"], [6, 4, 2, 8, 1, "VAE"], [7, 8, 0, 30, 0, "IMAGE"], [8, 30, 0, 9, 0, "IMAGE"]],
        "definitions": {"subgraphs": [{
            "id": "sub-1", "inputs": [{"name": "clip", "type": "CLIP"}], "outputs": [{"name": "CONDITIONING"}],
            "nodes": [{"id": 1, "type": "CLIPTextEncode", "mode": 0, "inputs": [{"name": "clip", "link": 10}], "widgets_values": ["a red kite"]}],
            "links": [{"id": 10, "origin_id": -10, "origin_slot": 0, "target_id": 1, "target_slot": 0}, {"id": 11, "origin_id": 1, "origin_slot": 0, "target_id": -20, "target_slot": 0}]
        }]}
    });
    let s = schema::load();
    let g = read(&ui, &s).unwrap();
    assert_eq!(g["3"].inputs["steps"], 20);
    assert_eq!(g["20:1"].inputs["clip"], json!(["4", 1]));
    assert_eq!(g["3"].inputs["positive"], json!(["20:1", 0]));
    assert_eq!(g["9"].inputs["images"], json!(["8", 0]));
    assert!(!g.contains_key("11"));
    let f = lift(&g, &s).flow;
    assert_eq!(f.steps["n8"].inputs["prompt"], "a red kite");
    assert_eq!(f.steps["n8"].inputs["cfg"], 3.5);
}

#[test]
fn keys_change_with_what_a_step_reads() {
    let mut f = Flow::default();
    f.steps.insert("a".into(), Step::new("image.generate", json!({"prompt": "one"})));
    f.steps.insert("b".into(), Step::new("image.upscale", json!({"image": "@a"})));
    f.out = vec!["b".into()];
    let k1 = f.keys().unwrap();
    f.steps.get_mut("a").unwrap().inputs.insert("prompt".into(), json!("two"));
    let k2 = f.keys().unwrap();
    assert_ne!(k1["a"], k2["a"]);
    assert_ne!(k1["b"], k2["b"]);
    assert_eq!(Flow::parse(&f.to_json()).unwrap(), f);
}

#[test]
fn made_bodies_fit_the_model() {
    let model = json!({"id": "kling_3_pro_i2v", "durations": [3, 4, 5, 10], "mediaRoles": {"start_image": "x", "end_image": "y"}, "allowedParams": ["negative_prompt"], "inputMap": {"audio": {}}});
    let s = Step::new("video.generate", json!({"prompt": "push in", "seconds": 6.2, "start_image": "@a", "seed": 3, "negative": "blur"}));
    let (b, notes) = crate::route::made_body(&s, &model, &|_| Ok("job_1".into())).unwrap();
    assert_eq!(b["duration"], 5);
    assert_eq!(b["medias"], json!([{"value": "job_1", "role": "start_image"}]));
    assert_eq!(b["negative_prompt"], "blur");
    assert!(b.get("seed").is_none() && notes.iter().any(|n| n.contains("seed")));
}

struct FakeLocal;

impl crate::run::Local for FakeLocal {
    fn names(&self) -> Vec<String> {
        vec!["sao".into()]
    }
    fn make(&self, name: &str, step: &Step, dir: &std::path::Path, key: &str) -> Result<std::path::PathBuf, String> {
        let f = dir.join(format!("{key}.wav"));
        std::fs::write(&f, format!("{name}:{}", step.inputs["prompt"].as_str().unwrap_or(""))).map_err(|e| e.to_string())?;
        Ok(f)
    }
}

#[test]
fn local_models_run_and_cache() {
    let dir = std::env::temp_dir().join(format!("raea-gen-test-{}", std::process::id()));
    let mut f = Flow::default();
    f.steps.insert("rain".into(), Step::new("audio.sfx", json!({"prompt": "rain on a tin roof", "seconds": 3})));
    f.out = vec!["rain".into()];
    let local = FakeLocal;
    let mut r = crate::run::Run::new(dir.clone(), dir.clone());
    r.policy.local = local.names();
    r.local = Some(&local);
    let out = r.run(&f).unwrap();
    assert_eq!(out["steps"][0]["runs_on"], "local:sao");
    let file = out["out"]["rain"][0].as_str().unwrap().to_string();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "sao:rain on a tin roof");
    // the second run finds it by key
    assert_eq!(r.run(&f).unwrap()["steps"][0]["cached"], true);
    // without the host's sao, effects go to made
    assert_eq!(route(&f.steps["rain"], &Policy::default()).unwrap().0, Backend::Made("elevenlabs_sound_effects_v2".into()));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn made_requests_map_to_fal_fields() {
    let model = json!({"id": "kling_3_pro_i2v", "provider": "fal", "workflowId": "fal-ai/kling-video/v3/pro/image-to-video",
        "mediaRoles": {"start_image": "start_image_url", "end_image": "end_image_url"},
        "inputMap": {"audio": {"field": "generate_audio"}, "duration": {"stringify": true}, "aspect_ratio": {"drop": true}},
        "providerParams": {"cfg_scale": 0.5}});
    let made = json!({"model": "kling_3_pro_i2v", "prompt": "push in", "duration": 5, "audio": false, "aspect_ratio": "16:9",
        "medias": [{"value": "https://x/a.png", "role": "start_image"}], "request_id": "r", "max_cost_cents": 9});
    let (endpoint, body) = crate::direct::body(&model, &made).unwrap();
    assert_eq!(endpoint, "fal-ai/kling-video/v3/pro/image-to-video");
    assert_eq!(body, json!({"prompt": "push in", "duration": "5", "generate_audio": false, "start_image_url": "https://x/a.png", "cfg_scale": 0.5}));
    // list fields collect every media of their role; an edit endpoint is used when there's media
    let nb = json!({"id": "nano_banana_2", "provider": "fal", "workflowId": "fal-ai/nano-banana-2", "editWorkflowId": "fal-ai/nano-banana-2/edit", "mediaRoles": {"image": "image_urls[]"}});
    let (e, b) = crate::direct::body(&nb, &json!({"model": "nano_banana_2", "prompt": "p", "medias": [{"value": "u1", "role": "image"}, {"value": "u2", "role": "image"}]})).unwrap();
    assert_eq!((e.as_str(), &b["image_urls"]), ("fal-ai/nano-banana-2/edit", &json!(["u1", "u2"])));
    assert!(!crate::direct::possible(&json!({"provider": "reapi", "workflowId": "x"})));
}

#[test]
fn partner_3d_and_chat_nodes_route_to_fal_and_openrouter() {
    let g = json!({
        "1": {"class_type": "LoadImage", "inputs": {"image": "toy.png"}},
        "2": {"class_type": "TripoImageToModelNode", "inputs": {"image": ["1", 0], "seed": 3}},
        "3": {"class_type": "SaveGLB", "inputs": {"mesh": ["2", 0]}},
        "4": {"class_type": "ClaudeNode", "inputs": {"prompt": "describe a toy"}},
        "5": {"class_type": "PreviewAny", "inputs": {"source": ["4", 0]}},
        "6": {"class_type": "SaveText", "inputs": {"text": ["4", 0]}},
    });
    let s = schema::load();
    let f = lift(&read(&g, &s).unwrap(), &s).flow;
    assert_eq!((f.steps["n2"].op.as_str(), f.steps["n2"].model.as_deref()), ("mesh.generate", Some("fal:tripo3d/h3.1/image-to-3d")));
    assert_eq!(f.steps["n2"].inputs["image"], "@n1");
    assert_eq!(f.steps["n4"].op, "text.generate");
    assert!(f.steps["n4"].model.as_deref().unwrap().starts_with("openrouter:"));
    f.check().unwrap();
}

#[test]
fn text_steps_feed_prompts_by_reference() {
    let mut f = Flow::default();
    f.steps.insert("idea".into(), Step::new("text.generate", json!({"prompt": "write a prompt"})));
    f.steps.insert("pic".into(), Step::new("image.generate", json!({"prompt": "@idea"})));
    f.out = vec!["pic".into()];
    f.check().unwrap();
    assert_eq!(f.order().unwrap(), vec!["idea", "pic"]);
    assert!(matches!(route(&f.steps["idea"], &Policy::default()).unwrap().0, Backend::OpenRouter(_)));
}

#[test]
fn exported_text_steps_link_their_words() {
    let mut f = Flow::default();
    f.steps.insert("idea".into(), Step::new("text.generate", json!({"prompt": "write a prompt"})));
    f.steps.insert("pic".into(), Step::new("image.generate", json!({"prompt": "@idea"})));
    f.out = vec!["pic".into(), "idea".into()];
    let g = crate::comfy::export(&f).unwrap();
    // RaeaMade's STRING output is slot 4; a text output is shown, not saved as a picture
    assert_eq!(g["pic"]["inputs"]["prompt"], json!(["idea", 4]));
    assert_eq!(g["save1"]["class_type"], "PreviewAny");
}

#[test]
fn image_utilities_lift_run_here_and_export_as_core_nodes() {
    let dir = std::env::temp_dir().join(format!("raea-gen-native-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    image::RgbImage::from_pixel(64, 32, image::Rgb([200, 30, 30])).save(dir.join("red.png")).unwrap();
    image::RgbImage::from_pixel(16, 16, image::Rgb([30, 30, 200])).save(dir.join("blue.png")).unwrap();
    let g = json!({
        "1": {"class_type": "LoadImage", "inputs": {"image": "red.png"}},
        "2": {"class_type": "LoadImage", "inputs": {"image": "blue.png"}},
        "3": {"class_type": "ImageCrop", "inputs": {"image": ["1", 0], "width": 32, "height": 32, "x": 16, "y": 0}},
        "4": {"class_type": "ImageScale", "inputs": {"image": ["2", 0], "upscale_method": "lanczos", "width": 0, "height": 32, "crop": "disabled"}},
        "5": {"class_type": "ImageStitch", "inputs": {"image1": ["3", 0], "image2": ["4", 0], "direction": "right", "match_image_size": true, "spacing_width": 0, "spacing_color": "white"}},
        "6": {"class_type": "SaveImage", "inputs": {"images": ["5", 0], "filename_prefix": "x"}},
    });
    let s = schema::load();
    let f = lift(&read(&g, &s).unwrap(), &s).flow;
    let ops: Vec<&str> = f.order().unwrap().iter().map(|id| f.steps[id].op.as_str()).collect();
    assert!(ops.contains(&"image.crop") && ops.contains(&"image.scale") && ops.contains(&"image.stitch"), "{ops:?}");
    assert!(f.steps.values().all(|s| s.op != "comfy"));
    // no model, no server: raea runs them
    let r = crate::run::Run::new(dir.clone(), dir.clone());
    let out = r.run(&f).unwrap();
    let file = out["out"][&f.out[0]][0].as_str().unwrap().to_string();
    let pic = image::open(&file).unwrap().to_rgb8();
    assert_eq!(pic.dimensions(), (64, 32));
    assert_eq!(pic.get_pixel(5, 5).0, [200, 30, 30]);
    assert_eq!(pic.get_pixel(50, 5).0, [30, 30, 200]);
    // and back out as ComfyUI's own nodes
    let w = write(&f).unwrap();
    let classes: Vec<&str> = w.as_object().unwrap().values().filter_map(|n| n["class_type"].as_str()).collect();
    for c in ["ImageCrop", "ImageScale", "ImageStitch"] {
        assert!(classes.contains(&c), "{classes:?}");
    }
    let _ = std::fs::remove_dir_all(dir);
}

/// A stand-in for Replicate's predictions API on 127.0.0.1: a model's POST starts a prediction, its urls.get answers
/// "processing" once and then "succeeded" with an output URL, which serves a PNG. Every request is kept for asserts.
fn fake_replicate() -> (String, std::sync::Arc<std::sync::Mutex<Vec<String>>>) {
    use std::io::{BufRead, BufReader, Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let (b, s) = (base.clone(), seen.clone());
    let mut png = Vec::new();
    image::RgbImage::from_pixel(8, 8, image::Rgb([10, 200, 10])).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    std::thread::spawn(move || {
        let mut polls = 0;
        for c in l.incoming().flatten() {
            let mut r = BufReader::new(c.try_clone().unwrap());
            let (mut head, mut line, mut len) = (String::new(), String::new(), 0);
            while r.read_line(&mut line).unwrap_or(0) > 2 {
                if let Some(n) = line.to_ascii_lowercase().strip_prefix("content-length:") { len = n.trim().parse().unwrap_or(0); }
                head += &line;
                line.clear();
            }
            let mut body = vec![0; len];
            r.read_exact(&mut body).unwrap();
            let first = head.lines().next().unwrap_or("").to_string();
            s.lock().unwrap().push(format!("{first} {} {}", head.contains("Bearer r8_test"), String::from_utf8_lossy(&body)));
            let (mime, out): (&str, Vec<u8>) = if first.starts_with("POST /v1/models/") {
                ("application/json", json!({"id": "p1", "status": "starting", "urls": {"get": format!("{b}/v1/predictions/p1")}}).to_string().into_bytes())
            } else if first.starts_with("GET /v1/predictions/p1") {
                polls += 1;
                let st = if polls < 2 { json!({"status": "processing", "logs": "50%", "urls": {"get": format!("{b}/v1/predictions/p1")}}) } else { json!({"status": "succeeded", "output": [format!("{b}/out.png")], "urls": {"get": format!("{b}/v1/predictions/p1")}}) };
                ("application/json", st.to_string().into_bytes())
            } else {
                ("image/png", png.clone())
            };
            let mut c = c;
            let _ = write!(c, "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", out.len());
            let _ = c.write_all(&out);
        }
    });
    (base, seen)
}

#[test]
fn replicate_steps_run_through_the_predictions_api() {
    let (base, seen) = fake_replicate();
    // SAFETY: only this test reads these two settings
    unsafe {
        std::env::set_var("REPLICATE_API_TOKEN", "r8_test");
        std::env::set_var("RAEA_REPLICATE_API", format!("{base}/v1"));
    }
    let dir = std::env::temp_dir().join(format!("raea-gen-replicate-{}", std::process::id()));
    let mut f = Flow::default();
    let mut s = Step::new("image.generate", json!({"prompt": "a green square", "x_num_outputs": 1}));
    s.model = Some("replicate:black-forest-labs/flux-schnell".into());
    f.steps.insert("sq".into(), s);
    f.out = vec!["sq".into()];
    let out = crate::run::Run::new(dir.clone(), dir.clone()).run(&f).unwrap();
    assert_eq!(out["steps"][0]["runs_on"], "replicate:black-forest-labs/flux-schnell");
    let pic = image::open(out["out"]["sq"][0].as_str().unwrap()).unwrap().to_rgb8();
    assert_eq!(pic.get_pixel(0, 0).0, [10, 200, 10]);
    let seen = seen.lock().unwrap();
    // created on the model's own route with the token, its input as the step said; polled until it succeeded
    assert!(seen[0].starts_with("POST /v1/models/black-forest-labs/flux-schnell/predictions HTTP/1.1 true"), "{seen:?}");
    assert!(seen[0].contains(r#""prompt":"a green square""#) && seen[0].contains(r#""num_outputs":1"#), "{seen:?}");
    assert_eq!(seen.iter().filter(|r| r.starts_with("GET /v1/predictions/p1")).count(), 2);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_batch_reaches_a_model_as_each_of_its_pictures() {
    use std::collections::BTreeMap;
    let mut done = BTreeMap::new();
    done.insert("batch".to_string(), crate::run::Made1 { files: vec!["a.png".into(), "b.png".into()], ..Default::default() });
    done.insert("mesh".to_string(), crate::run::Made1 { files: vec!["m.glb".into(), "t.png".into()], ..Default::default() });
    let s = Step::new("image.edit", json!({"image": "@batch", "refs": ["@mesh", "@batch.1"], "prompt": "x"}));
    let out = crate::run::spread_batches(&s, &done);
    assert_eq!(out.inputs["image"], json!(["@batch.0", "@batch.1"]));
    // a mesh with its textures stays whole; an indexed reference stays as it is
    assert_eq!(out.inputs["refs"], json!(["@mesh", "@batch.1"]));
    assert_eq!(crate::run::file_of(&done, &json!("@batch.1")).unwrap().to_str(), Some("b.png"));
}

#[test]
fn comfy_auto_aspect_fits_what_made_offers() {
    let model = json!({"id": "nano_banana_pro", "aspectRatios": ["match_input_image", "1:1", "16:9", "9:16"], "mediaRoles": {"image": "image_urls[]"}});
    let s = Step::new("image.edit", json!({"image": "@a", "prompt": "x", "aspect_ratio": "auto"}));
    let (b, notes) = crate::route::made_body(&s, &model, &|_| Ok("https://x/a.png".into())).unwrap();
    assert_eq!(b["aspect_ratio"], "match_input_image");
    assert!(notes.iter().any(|n| n.contains("auto")));
    let s = Step::new("image.generate", json!({"prompt": "x", "aspect_ratio": "17:9"}));
    assert_eq!(crate::route::made_body(&s, &model, &|_| Ok(String::new())).unwrap().0["aspect_ratio"], "16:9");
}

#[test]
fn a_workflow_says_the_packs_and_weights_it_needs() {
    // UI format: each node's pack is its properties' registry id (cnr_id) or repo (aux_id); models carry their URLs
    let wf = json!({"nodes": [
        {"id": 1, "type": "LoadImage", "properties": {"cnr_id": "comfy-core"}},
        {"id": 2, "type": "ImageResizeKJv2", "properties": {"cnr_id": "comfyui-kjnodes"}},
        {"id": 3, "type": "FL_PixelSort", "properties": {"aux_id": "filliptm/ComfyUI_Fill-Nodes"}},
        {"id": 4, "type": "LoadBackgroundRemovalModel", "properties": {"models": [{"name": "birefnet.safetensors", "url": "https://huggingface.co/x/birefnet.safetensors", "directory": "background_removal"}]}},
    ], "links": []});
    let mut s = crate::comfy::schema::Schema::new();
    s.insert("LoadImage".into(), Default::default());
    s.insert("LoadBackgroundRemovalModel".into(), Default::default());
    let n = crate::comfy::needs::needs(&wf, &s);
    assert_eq!(n["packs"], json!(["comfyui-kjnodes"]));
    assert_eq!(n["repos"], json!(["https://github.com/filliptm/ComfyUI_Fill-Nodes"]));
    assert_eq!(n["models"][0]["directory"], "background_removal");
    assert_eq!(n["unknown"], json!([]));
}

#[test]
fn a_made_model_fal_also_serves_goes_to_its_twin() {
    let model = json!({"id": "gpt_image_2_5_fast", "provider": "made_batch", "workflowId": "gpt-image-2.5-flare-low"});
    assert!(crate::direct::possible(&model));
    let (e, b) = crate::direct::body(&model, &json!({"model": "gpt_image_2_5_fast", "prompt": "a kite", "aspect_ratio": "16:9"})).unwrap();
    assert_eq!(e, "openai/gpt-image-2.5/flare/text-to-image");
    assert_eq!((b["quality"].as_str(), b["image_size"].as_str()), (Some("low"), Some("landscape_16_9")));
    let (e, b) = crate::direct::body(&model, &json!({"prompt": "edit", "medias": [{"value": "https://x/a.png", "role": "image"}]})).unwrap();
    assert_eq!((e.as_str(), &b["image_urls"]), ("openai/gpt-image-2.5/flare/edit", &json!(["https://x/a.png"])));
}

#[test]
fn a_kept_node_saved_from_several_outputs_gives_each() {
    let g = json!({
        "1": {"class_type": "LoadImage", "inputs": {"image": "a.png"}},
        "2": {"class_type": "SplitterOfSomePack", "inputs": {"image": ["1", 0]}},
        "3": {"class_type": "SaveImage", "inputs": {"images": ["2", 0], "filename_prefix": "a"}},
        "4": {"class_type": "SaveImage", "inputs": {"images": ["2", 2], "filename_prefix": "b"}},
    });
    let s = schema::load();
    let f = lift(&read(&g, &s).unwrap(), &s).flow;
    let st = f.steps.values().find(|s| s.op == "comfy").unwrap();
    assert_eq!(st.inputs["slots"], json!([0, 2]));
    assert_eq!(f.out.len(), 1);
    let w = write(&f).unwrap();
    // one Save per slot, in order, which the runner fetches as the step's files (@step.0, @step.1)
    let src = |save: &str| w[save]["inputs"]["images"].clone();
    assert_eq!(src("save0")[1], json!(0));
    assert_eq!(src("save0.1")[1], json!(2));
}

/// fish:<model> against a stand-in for Fish's /v1/tts: the key, the model header and the step's words, voice and speed
/// as Fish reads them, an MP3 kept, its bytes' price quoted in the plan and charged in the run.
#[test]
fn fish_speech_steps_send_the_users_key_and_words() {
    use std::io::{BufRead, BufReader, Read, Write};
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", l.local_addr().unwrap());
    let seen = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let s2 = seen.clone();
    std::thread::spawn(move || {
        for c in l.incoming().flatten() {
            let mut r = BufReader::new(c.try_clone().unwrap());
            let (mut head, mut line, mut len) = (String::new(), String::new(), 0);
            while r.read_line(&mut line).unwrap_or(0) > 2 {
                if let Some(n) = line.to_ascii_lowercase().strip_prefix("content-length:") { len = n.trim().parse().unwrap_or(0); }
                head += &line;
                line.clear();
            }
            let mut body = vec![0; len];
            r.read_exact(&mut body).unwrap();
            *s2.lock().unwrap() = format!("{head}{}", String::from_utf8_lossy(&body));
            let mp3 = b"ID3fake-mp3";
            let mut c = c;
            let _ = write!(c, "HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", mp3.len());
            let _ = c.write_all(mp3);
        }
    });
    // SAFETY: only this test reads these two settings
    unsafe {
        std::env::set_var("FISH_API_KEY", "fish_test");
        std::env::set_var("RAEA_FISH_API", &base);
    }
    let dir = std::env::temp_dir().join(format!("raea-gen-fish-{}", std::process::id()));
    let mut f = Flow::default();
    let mut s = Step::new("audio.speech", json!({"text": "Three drops, every morning.", "voice": "abc", "speed": 0.9}));
    s.model = Some("fish:".into());
    f.steps.insert("vo".into(), s);
    f.out = vec!["vo".into()];
    let run = crate::run::Run::new(dir.clone(), dir.clone());
    let plan = run.plan(&f).unwrap();
    assert_eq!((plan["steps"][0]["runs_on"].as_str(), plan["steps"][0]["cents"].as_u64()), (Some("fish:s2.1-pro"), Some(1)), "{plan}");
    let out = run.run(&f).unwrap();
    assert_eq!(std::fs::read(out["out"]["vo"][0].as_str().unwrap()).unwrap(), b"ID3fake-mp3");
    assert_eq!(out["cents"], 1);
    let seen = seen.lock().unwrap().to_ascii_lowercase();
    assert!(seen.starts_with("post /v1/tts ") && seen.contains("authorization: bearer fish_test") && seen.contains("model: s2.1-pro"), "{seen}");
    assert!(seen.contains(r#""text":"three drops, every morning.""#) && seen.contains(r#""reference_id":"abc""#) && seen.contains(r#""prosody":{"speed":0.9}"#), "{seen}");
    let _ = std::fs::remove_dir_all(dir);
}
