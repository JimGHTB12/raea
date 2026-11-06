//! HTTP through curl (raea's io crates link no HTTP client). The URL, headers and body go to curl's stdin as its
//! config, so a key never shows in the process list.

use std::io::Write;
use std::path::Path;

use serde_json::Value as J;

/// What came back: the status, the content type and the body (empty when it went to a file).
pub struct Reply {
    pub code: u16,
    pub mime: String,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.code)
    }

    pub fn json(&self) -> Result<J, String> {
        serde_json::from_slice(&self.body).map_err(|e| format!("not JSON ({e}): {}", self.text(200)))
    }

    pub fn text(&self, n: usize) -> String {
        String::from_utf8_lossy(&self.body).chars().take(n).collect()
    }
}

/// What to send: a JSON body, a file's bytes (PUT uploads), or nothing.
pub enum Body<'a> {
    None,
    Json(&'a J),
    File(&'a Path),
    /// form fields and files, as curl's -F takes them (ComfyUI's /upload/image)
    Form(&'a [(String, String)]),
}

fn q(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// One request. `out` takes the body when given (downloads); otherwise it comes back in the reply.
pub fn call(method: &str, url: &str, headers: &[String], body: Body, out: Option<&Path>, secs: u32) -> Result<Reply, String> {
    raea_io_home::offline(&format!("raea gen's request to {}", redact(url)))?;
    let mut config = format!("url = {}\nrequest = {}\nheader = \"User-Agent: raea-gen\"\nmax-time = {secs}\nlocation\n", q(url), q(method));
    for h in headers {
        config += &format!("header = {}\n", q(h));
    }
    match body {
        Body::None => {}
        Body::Json(j) => config += &format!("header = \"Content-Type: application/json\"\ndata-binary = {}\n", q(&j.to_string())),
        Body::File(p) => config += &format!("upload-file = {}\n", q(&p.display().to_string())),
        Body::Form(fields) => {
            for (k, v) in fields {
                config += &format!("form = {}\n", q(&format!("{k}={v}")));
            }
        }
    }
    let tmp;
    let dest = match out {
        Some(p) => p,
        None => {
            // one name per call: two requests at once in one process never share a file
            static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            tmp = std::env::temp_dir().join(format!("raea-gen-{}-{n}", std::process::id()));
            &tmp
        }
    };
    let mut child = std::process::Command::new("curl")
        .args(["-sS", "-K", "-", "-o"])
        .arg(dest)
        .args(["-w", "%{http_code} %{content_type}"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("curl: {e}"))?;
    child.stdin.take().ok_or("curl: no stdin")?.write_all(config.as_bytes()).map_err(|e| format!("curl: {e}"))?;
    let o = child.wait_with_output().map_err(|e| format!("curl: {e}"))?;
    if !o.status.success() {
        let _ = out.is_none().then(|| std::fs::remove_file(dest));
        return Err(format!("{method} {}: {}", redact(url), String::from_utf8_lossy(&o.stderr).trim()));
    }
    let w = String::from_utf8_lossy(&o.stdout).to_string();
    let (code, mime) = w.split_once(' ').unwrap_or((&w, ""));
    let body = if out.is_none() {
        let b = std::fs::read(dest).unwrap_or_default();
        let _ = std::fs::remove_file(dest);
        b
    } else {
        Vec::new()
    };
    Ok(Reply { code: code.parse().unwrap_or(0), mime: mime.to_string(), body })
}

/// A URL as an error may show it: no query (signed links carry their signature there).
/// The Authorization header for a hosted service's key, read with crate::setting: `Authorization: SCHEME KEY`,
/// or `missing` (where to get one) when it isn't set.
pub fn key_header(name: &str, scheme: &str, missing: &str) -> Result<String, String> {
    Ok(format!("Authorization: {scheme} {}", crate::setting(name).ok_or(missing)?))
}

pub fn redact(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// Text for an error: data URIs cut to their head, the whole kept under 600 characters.
pub fn clip(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("data:") {
        out += &rest[..i];
        let tail = &rest[i..];
        let end = tail.find(|c: char| c == '"' || c == ' ' || c == '\'').unwrap_or(tail.len());
        out += &format!("{}…", &tail[..end.min(40)]);
        rest = &tail[end..];
    }
    out += rest;
    out.chars().take(600).collect()
}
