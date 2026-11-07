//! One clef-omni call on Workers AI (as apps/app/host/critique.ts makes it): the state paragraph, the questions by id,
//! the images as data URLs (each made a JPEG at most 1600 px wide first). Keys: CLOUDFLARE_ACCOUNT_ID and
//! CLOUDFLARE_API_TOKEN from raea's settings (the environment, then ~/.raea/env).

use std::path::PathBuf;

use base64::Engine as _;
use serde_json::{Value, json};

fn jpeg(p: &PathBuf) -> Result<String, String> {
    let img = image::open(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let img = if img.width() > 1600 { img.resize(1600, u32::MAX, image::imageops::FilterType::Triangle) } else { img };
    let mut buf = std::io::Cursor::new(Vec::new());
    img.to_rgb8().write_to(&mut buf, image::ImageFormat::Jpeg).map_err(|e| e.to_string())?;
    Ok(format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(buf.into_inner())))
}

/// The answers by question id, and the call's cost in dollars ($0.15 a million input tokens).
pub fn ask(state: &str, questions: &Value, images: &[PathBuf]) -> Result<(Value, f64), String> {
    let (Some(acct), Some(token)) = (raea_io_home::setting("CLOUDFLARE_ACCOUNT_ID"), raea_io_home::setting("CLOUDFLARE_API_TOKEN")) else {
        return Err("judge: clef-omni on Workers AI needs CLOUDFLARE_ACCOUNT_ID and CLOUDFLARE_API_TOKEN (the environment or ~/.raea/env)".into());
    };
    let imgs: Vec<String> = images.iter().map(jpeg).collect::<Result<_, _>>()?;
    let body = json!({"state": state, "questions": questions, "images": imgs});
    let url = format!("https://api.cloudflare.com/client/v4/accounts/{acct}/ai/run/@cf/cloudflare/clef-omni");
    let res = ureq::post(&url).set("authorization", &format!("Bearer {token}")).timeout(std::time::Duration::from_secs(180)).send_json(body);
    let j: Value = match res {
        Ok(r) => r.into_json().map_err(|e| format!("clef: {e}"))?,
        Err(ureq::Error::Status(code, r)) => return Err(format!("clef: HTTP {code}: {}", r.into_string().unwrap_or_default().chars().take(300).collect::<String>())),
        Err(e) => return Err(format!("clef: {e}")),
    };
    if j["success"] != true {
        return Err(format!("clef: {}", j["errors"].to_string().chars().take(300).collect::<String>()));
    }
    let cost = j["result"]["usage"]["input_tokens"].as_f64().unwrap_or(0.0) * 0.15 / 1e6;
    Ok((j["result"]["answers"].clone(), cost))
}
