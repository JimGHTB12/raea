//! raea's window on the feed (docs/app.feature AP5/AP6): with RAEA_PAGE_HOST (the feed's Node host, http://127.0.0.1:PORT),
//! the page's /host/* is that host's, and so is a /media/* that isn't on disk (a render the host made after the page
//! loaded); everything else is page_fetch's.

use base64::Engine as _;

use crate::page_fetch;

/// The page's fetch of its own origin: its host's for /host/* (and a /media/* the roots don't have), else page_fetch.
/// None, as page_fetch: not the page's origin, the network's.
pub fn fetch(args: &serde_json::Value) -> Option<serde_json::Value> {
    let u = args["url"].as_str().and_then(|u| url::Url::parse(u).ok())?;
    let own = u.scheme() == "http" && u.host_str() == Some("localhost") && u.port().is_none();
    if own && u.path().starts_with("/host/") {
        if let Some(r) = to_host(args, &u) {
            return Some(r);
        }
    }
    let r = page_fetch(args)?;
    if r["status"] == 404 && u.path().starts_with("/media/") {
        return Some(to_host(args, &u).unwrap_or(r));
    }
    Some(r)
}

/// The request sent on to RAEA_PAGE_HOST (method, headers and body as the page gave them), its answer as `fetch` gives
/// one; None when there's no host. A host that doesn't answer is a 502, so the page sees the failure, not a hang.
fn to_host(args: &serde_json::Value, u: &url::Url) -> Option<serde_json::Value> {
    let host = std::env::var("RAEA_PAGE_HOST").ok().filter(|h| !h.is_empty())?;
    let mut a = args.clone();
    a["url"] = serde_json::Value::String(format!("{}{}{}", host.trim_end_matches('/'), u.path(), u.query().map(|q| format!("?{q}")).unwrap_or_default()));
    Some(raea_io_net::http::fetch(&a).unwrap_or_else(|e| {
        let body = base64::engine::general_purpose::STANDARD.encode(e.as_bytes());
        serde_json::json!({ "status": 502, "statusText": "Bad Gateway", "url": args["url"], "redirected": false, "headers": [], "body": body })
    }))
}
