//! A made request sent to fal itself, at fal's price. made says `markupPercent: 0`, but since 2026-10-29 it rounds each
//! job up to a whole credit (10¢): a 1¢ FLUX Klein image is billed 10¢ (its /v1/cost answers providerCostCents 1,
//! customerCostCents 10). For a model made runs on fal, its catalog row says exactly how made calls fal: the endpoint
//! (`workflowId`, or `editWorkflowId` when given media), each media role's fal field (`mediaRoles`), and how settings
//! are renamed, scaled, stringified or dropped (`inputMap`, `providerParams`). So when made would charge more than the
//! provider does, the same request goes to fal directly (FAL_KEY), unless RAEA_GEN_VIA=made. A model made runs
//! elsewhere goes the same way when fal serves the same model (TWINS).

use serde_json::{Map, Value as J, json};

/// made models it runs elsewhere that fal also serves, the same model at the same setting: (made id, fal endpoint,
/// fal endpoint given pictures, fal settings that make it that model). gpt_image_2_5_fast is made_batch's
/// gpt-image-2.5-flare-low: fal's page prices low quality at 0.4-0.6¢ an image (10-29), made bills 10¢.
const TWINS: &[(&str, &str, &str, &str)] = &[
    ("gpt_image_2_5_fast", "openai/gpt-image-2.5/flare/text-to-image", "openai/gpt-image-2.5/flare/edit", r#"{"quality":"low"}"#),
];

fn twin(model: &J) -> Option<&'static (&'static str, &'static str, &'static str, &'static str)> {
    TWINS.iter().find(|t| model["id"] == t.0)
}

/// Whether this model can go straight to fal: made runs it there (or fal serves its twin), and none of its settings
/// need made's own handling.
pub fn possible(model: &J) -> bool {
    twin(model).is_some() || model["provider"] == "fal" && model["workflowId"].is_string() && !model["inputMap"].as_object().is_some_and(|m| m.values().any(|v| v["dimensions"] == true))
}

/// A made body for a twin: the prompt, pictures as image_urls, the aspect as fal's image_size, the twin's settings.
fn twin_body(t: &(&str, &str, &str, &str), made: &J) -> (String, J) {
    let medias = made["medias"].as_array().cloned().unwrap_or_default();
    let mut out: Map<String, J> = serde_json::from_str(t.3).unwrap_or_default();
    out.insert("prompt".into(), made["prompt"].clone());
    if let Some(size) = made["aspect_ratio"].as_str().and_then(|a| match a {
        "1:1" => Some("square_hd"), "4:3" => Some("landscape_4_3"), "3:4" => Some("portrait_4_3"),
        "16:9" => Some("landscape_16_9"), "9:16" => Some("portrait_16_9"), _ => None,
    }) {
        out.insert("image_size".into(), json!(size));
    }
    if medias.is_empty() {
        return (t.1.to_string(), J::Object(out));
    }
    out.insert("image_urls".into(), J::Array(medias.iter().map(|m| m["value"].clone()).collect()));
    (t.2.to_string(), J::Object(out))
}

/// A made body (model, prompt, medias with values as URLs, settings) as (fal endpoint, fal input).
pub fn body(model: &J, made: &J) -> Result<(String, J), String> {
    if let Some(t) = twin(model) {
        return Ok(twin_body(t, made));
    }
    let medias = made["medias"].as_array().cloned().unwrap_or_default();
    let endpoint = match (medias.is_empty(), model["editWorkflowId"].as_str()) {
        (false, Some(e)) => e,
        _ => model["workflowId"].as_str().ok_or("no workflowId")?,
    }.to_string();
    let map = model["inputMap"].as_object().cloned().unwrap_or_default();
    let mut out = Map::new();
    for (k, v) in made.as_object().into_iter().flatten() {
        if matches!(k.as_str(), "model" | "medias" | "request_id" | "max_cost_cents" | "count") {
            continue;
        }
        let rule = map.get(k).cloned().unwrap_or(J::Null);
        if rule["drop"] == true {
            continue;
        }
        let mut v = v.clone();
        if let Some(scale) = rule["scale"].as_f64() {
            v = json!(v.as_f64().unwrap_or(0.0) * scale);
        }
        if let Some(to) = rule["values"].get(v.to_string().trim_matches('"')).or_else(|| rule["values"].get(v.as_str().unwrap_or(""))) {
            v = to.clone();
        } else if rule["stringify"] == true {
            v = json!(v.to_string().trim_matches('"'));
        }
        out.insert(rule["field"].as_str().unwrap_or(k).to_string(), v);
    }
    let roles = model["mediaRoles"].as_object().cloned().unwrap_or_default();
    for m in &medias {
        let role = m["role"].as_str().unwrap_or("");
        let field = roles.get(role).and_then(|f| f.as_str()).ok_or(format!("{} has no fal field for role {role}", model["id"]))?;
        match field.strip_suffix("[]") {
            Some(list) => out.entry(list).or_insert(json!([])).as_array_mut().ok_or("media field clash")?.push(m["value"].clone()),
            None => { out.insert(field.to_string(), m["value"].clone()); }
        }
    }
    for (k, v) in model["providerParams"].as_object().into_iter().flatten() {
        out.entry(k.clone()).or_insert(v.clone());
    }
    Ok((endpoint, J::Object(out)))
}
