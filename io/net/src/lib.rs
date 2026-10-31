//! What a URL names, read, for every crate that reads one: a `data:` URL decoded in memory, an HTTP(S) request
//! (`http`: the page's net.fetch, and the request a policy-checked fetch makes each hop), a WebSocket on its own thread
//! (`ws`), and raea's URL cache (`cache`: an http(s) URL fetched once into ~/Library/Caches/raea).

pub mod cache;
pub mod http;
#[cfg(feature = "ws")]
pub mod ws;

pub use ureq;

/// A `data:` URL's media type (lowercase, "" when it names none) and its bytes: base64 when a `;base64` parameter says
/// so (standard or URL-safe, whitespace and padding skipped), else percent-encoded.
pub fn data_url(src: &str) -> Option<(String, Vec<u8>)> {
    let (meta, body) = src.strip_prefix("data:")?.split_once(',')?;
    let mime = meta.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    let base64 = meta.split(';').skip(1).any(|p| p.trim().eq_ignore_ascii_case("base64"));
    Some((mime, if base64 { base64_lenient(body)? } else { percent_decode(body) }))
}

/// `%XX` escapes decoded; a `%` without two hex digits after it is kept as it is.
pub fn percent_decode(s: &str) -> Vec<u8> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = |c: u8| (c as char).to_digit(16);
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2]))
        {
            out.push((h * 16 + l) as u8);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

/// percent_decode as text (invalid UTF-8 replaced): a path or a name in a URL.
pub fn percent_decode_str(s: &str) -> String {
    String::from_utf8_lossy(&percent_decode(s)).into_owned()
}

/// Base64 as browsers read it in a data: URL: either alphabet, whitespace and `=` ignored. None on any other byte.
fn base64_lenient(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' | b' ' | b'\t' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_data_urls() {
        assert_eq!(data_url("data:text/plain;base64,aGk=").unwrap(), ("text/plain".into(), b"hi".to_vec()));
        assert_eq!(data_url("data:Image/SVG+xml;charset=utf-8;BASE64,aGVs\nbG8=").unwrap(), ("image/svg+xml".into(), b"hello".to_vec()));
        assert_eq!(data_url("data:,a%20b").unwrap(), (String::new(), b"a b".to_vec()));
        assert_eq!(data_url("data:;base64,a$b"), None);
        assert_eq!(data_url("text/plain,hi"), None);
    }

    #[test]
    fn percent_decodes() {
        assert_eq!(percent_decode("a%20b%2"), b"a b%2");
        assert_eq!(percent_decode("%zz%41"), b"%zzA");
        assert_eq!(percent_decode_str("caf%C3%A9"), "café");
    }
}
