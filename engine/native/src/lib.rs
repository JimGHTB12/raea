//! raea's one native-call layer. TypeScript running on JavaScriptCore (raea render, run and test, the iPhone
//! probes, the host app's composition scripts) reaches Rust through one function, `__raea_call(name, json) -> json`; every
//! name it can call is registered here once, with the TypeScript types of its arguments and answer, by whichever crate
//! implements it (raea-runtime-react's solids, worlds and coders; the sandbox's commands; the web shims' own calls below). A
//! host links the crates it wants and calls their `register()`; `typescript()` writes the types the SDK imports, so a
//! call that doesn't exist, or takes the wrong arguments, fails to type-check.
//!
//! The registry needs no JS engine (the sandbox links it without one); `host::install` binds it into one.

use std::sync::Arc;

use serde_json::{Value, json};

/// `__raea_call` and the web shims on a JS engine (raea-runtime-jsengine: JavaScriptCore on Apple, QuickJS elsewhere)
pub mod host;

/// The web shims' source (plain JS, run once per context after the host's own prelude), for either engine: the globals
/// bundled npm packages expect from a browser or Node, each defined only where the context lacks it. Anything with real
/// work behind it (randomness, digests, URL parsing) is a native call through __raea_call; the rest is plain JS. The
/// pieces in web/ are one function's body, in this order.
pub const WEB_JS: &str = concat!(
    "(function (g) {\n\"use strict\";\n",
    include_str!("web/base.js"),
    include_str!("web/events.js"),
    include_str!("web/url.js"),
    include_str!("web/data.js"),
    include_str!("web/streams.js"),
    "})(globalThis);\n"
);

thread_local! {
    /// The native calls this thread's context may make (names, or "prefix.*"); None: all. js-exec's contexts are limited.
    pub(crate) static ALLOW: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn allowed(name: &str) -> bool {
    ALLOW.with(|a| a.borrow().as_ref().is_none_or(|list| list.iter().any(|p| p == name || p.strip_suffix('*').is_some_and(|pre| name.starts_with(pre)))))
}
#[cfg(feature = "modules")]
pub mod js;
/// Node's core for raea's own tools (js::run_node)
#[cfg(feature = "modules")]
pub mod node;

pub use raea_core_calls::{Call, Handler, R, dec64, enc64, register, s};

/// The shell's own calls (the web shims', fs, net, sql, archive, js.run) in the registry, once; everything here that
/// reads the registry seeds it first.
fn seed() {
    static SEEDED: std::sync::Once = std::sync::Once::new();
    SEEDED.call_once(|| {
        #[allow(unused_mut)] // js.run joins only with modules
        let mut all: Vec<Call> = web_calls().into_iter().chain(raea_io_shell::calls()).collect();
        #[cfg(feature = "modules")]
        all.push(Call {
            name: "js.run".into(),
            args: "{ root: string; cwd: string; file?: string; code?: string; ts?: boolean; argv?: string[]; env?: Record<string, string>; stdin?: string; allow?: string[]; timeoutMs?: number }".into(),
            answer: "{ exit: number; stdout: string; stderr: string }".into(),
            doc: "JavaScript or TypeScript run as an ES module in a fresh isolate (imports only inside root, native calls only from allow)".into(),
            f: Arc::new(js::call),
        });
        raea_core_calls::register_all(all);
    });
}

/// Whether `name` is registered.
pub fn has(name: &str) -> bool {
    seed();
    raea_core_calls::has(name)
}

/// Runs a call; an unknown name says so.
pub fn call(name: &str, args: &Value) -> R<Value> {
    seed();
    raea_core_calls::call(name, args)
}

/// Every registered call, by name.
pub fn calls() -> Vec<Call> {
    seed();
    raea_core_calls::calls()
}

/// The TypeScript the SDK imports (sdk/raea/src/native-calls.ts): one entry per call, its arguments and answer.
pub fn typescript() -> String {
    seed();
    raea_core_calls::typescript()
}

/// A WHATWG URL's parts, as the shims' URL class reads them.
fn url_parts(u: &url::Url) -> Value {
    json!({
        "href": u.as_str(),
        "origin": u.origin().ascii_serialization(),
        "protocol": format!("{}:", u.scheme()),
        "username": u.username(),
        "password": u.password().unwrap_or(""),
        "host": match (u.host_str(), u.port()) { (Some(h), Some(p)) => format!("{h}:{p}"), (Some(h), None) => h.to_string(), _ => String::new() },
        "hostname": u.host_str().unwrap_or(""),
        "port": u.port().map(|p| p.to_string()).unwrap_or_default(),
        "pathname": u.path(),
        "search": u.query().map(|q| if q.is_empty() { String::new() } else { format!("?{q}") }).unwrap_or_default(),
        "hash": u.fragment().map(|f| if f.is_empty() { String::new() } else { format!("#{f}") }).unwrap_or_default(),
    })
}

/// The host's own calls (its environment, its platform) and the ones the web shims stand on: randomness, digests and URL
/// parsing (the `url` crate is WHATWG's parser).
fn web_calls() -> Vec<Call> {
    let mk = Call::new;
    vec![
        mk("host.env", "{ name: string }", "string | null", "an environment variable of the host process (null when unset)", Arc::new(|v| {
            Ok(std::env::var(s(v, "name")?).map(Value::String).unwrap_or(Value::Null))
        })),
        mk("host.has", "{ name: string }", "boolean", "whether this host answers a native call", Arc::new(|v| Ok(json!(has(s(v, "name")?))))),
        mk("host.platform", "{}", "\"macos\" | \"ios\" | \"linux\" | \"windows\" | \"other\"", "the host's platform", Arc::new(|_| {
            Ok(json!(if cfg!(target_os = "ios") { "ios" } else if cfg!(target_os = "macos") { "macos" } else if cfg!(target_os = "linux") { "linux" } else if cfg!(windows) { "windows" } else { "other" }))
        })),
        mk("web.random", "{ n: number }", "string", "n random bytes (the OS's), base64", Arc::new(|v| {
            let n = v.get("n").and_then(Value::as_u64).ok_or("n: a count")? as usize;
            if n > 65536 {
                return Err(format!("getRandomValues: {n} bytes is over the 65536 limit"));
            }
            let mut b = vec![0u8; n];
            getrandom::fill(&mut b).map_err(|e| e.to_string())?;
            Ok(json!(enc64(&b)))
        })),
        mk("web.digest", "{ algo: string; data: string }", "string", "SHA-1, SHA-256, SHA-384 or SHA-512 of base64 bytes, base64", Arc::new(|v| {
            use sha2::Digest as _;
            let data = dec64(s(v, "data")?)?;
            let out: Vec<u8> = match s(v, "algo")?.to_ascii_uppercase().as_str() {
                "SHA-1" => sha1::Sha1::digest(&data).to_vec(),
                "SHA-256" => sha2::Sha256::digest(&data).to_vec(),
                "SHA-384" => sha2::Sha384::digest(&data).to_vec(),
                "SHA-512" => sha2::Sha512::digest(&data).to_vec(),
                a => return Err(format!("digest: {a} isn't supported (SHA-1, SHA-256, SHA-384, SHA-512)")),
            };
            Ok(json!(enc64(&out)))
        })),
        mk("zlib.gzip", "{ data: string; level?: number }", "string", "base64 bytes gzipped (level 0-9, default 6), base64", Arc::new(|v| {
            use std::io::Write as _;
            let level = v.get("level").and_then(Value::as_u64).unwrap_or(6).min(9) as u32;
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
            e.write_all(&dec64(s(v, "data")?)?).map_err(|e| e.to_string())?;
            Ok(json!(enc64(&e.finish().map_err(|e| e.to_string())?)))
        })),
        mk("zlib.gunzip", "{ data: string }", "string", "gzip (all members) inflated, base64", Arc::new(|v| {
            use std::io::Read as _;
            let mut out = Vec::new();
            flate2::read::MultiGzDecoder::new(&dec64(s(v, "data")?)?[..]).read_to_end(&mut out).map_err(|e| format!("incorrect header check or corrupt data: {e}"))?;
            Ok(json!(enc64(&out)))
        })),
        mk("zlib.deflate", "{ data: string; level?: number; raw?: boolean }", "string", "zlib (or raw) deflate, base64", Arc::new(|v| {
            use std::io::Write as _;
            let level = flate2::Compression::new(v.get("level").and_then(Value::as_u64).unwrap_or(6).min(9) as u32);
            let data = dec64(s(v, "data")?)?;
            let out = if v.get("raw").and_then(Value::as_bool).unwrap_or(false) {
                let mut e = flate2::write::DeflateEncoder::new(Vec::new(), level);
                e.write_all(&data).map_err(|e| e.to_string())?;
                e.finish()
            } else {
                let mut e = flate2::write::ZlibEncoder::new(Vec::new(), level);
                e.write_all(&data).map_err(|e| e.to_string())?;
                e.finish()
            };
            Ok(json!(enc64(&out.map_err(|e| e.to_string())?)))
        })),
        mk("zlib.inflate", "{ data: string; raw?: boolean }", "string", "zlib (or raw) inflate, base64", Arc::new(|v| {
            use std::io::Read as _;
            let data = dec64(s(v, "data")?)?;
            let mut out = Vec::new();
            if v.get("raw").and_then(Value::as_bool).unwrap_or(false) {
                flate2::read::DeflateDecoder::new(&data[..]).read_to_end(&mut out)
            } else {
                flate2::read::ZlibDecoder::new(&data[..]).read_to_end(&mut out)
            }
            .map_err(|e| format!("incorrect header check or corrupt data: {e}"))?;
            Ok(json!(enc64(&out)))
        })),
        mk("url.parse", "{ url: string; base?: string }", "{ href: string; origin: string; protocol: string; username: string; password: string; host: string; hostname: string; port: string; pathname: string; search: string; hash: string } | null", "a URL parsed by the WHATWG rules (null when invalid)", Arc::new(|v| {
            let input = s(v, "url")?;
            let u = match v.get("base").and_then(Value::as_str) {
                Some(b) => url::Url::parse(b).and_then(|b| b.join(input)),
                None => url::Url::parse(input),
            };
            Ok(u.map(|u| url_parts(&u)).unwrap_or(Value::Null))
        })),
        mk("url.set", "{ href: string; part: string; value: string }", "{ href: string; origin: string; protocol: string; username: string; password: string; host: string; hostname: string; port: string; pathname: string; search: string; hash: string }", "a URL with one part set, as the URL setters do (an invalid value leaves it unchanged)", Arc::new(|v| {
            let mut u = url::Url::parse(s(v, "href")?).map_err(|e| e.to_string())?;
            let val = s(v, "value")?;
            match s(v, "part")? {
                "href" => u = url::Url::parse(val).map_err(|e| format!("Invalid URL: {e}"))?,
                "protocol" => { let _ = u.set_scheme(val.trim_end_matches(':')); }
                "username" => { let _ = u.set_username(val); }
                "password" => { let _ = u.set_password(Some(val).filter(|p| !p.is_empty())); }
                "host" => {
                    let (h, p) = match val.rsplit_once(':') { Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) => (h, p.parse().ok()), _ => (val, None) };
                    if u.set_host(Some(h)).is_ok() { let _ = u.set_port(p); }
                }
                "hostname" => { let _ = u.set_host(Some(val)); }
                "port" => { let _ = u.set_port(if val.is_empty() { None } else { val.parse().ok() }); }
                "pathname" => u.set_path(val),
                "search" => u.set_query(Some(val.trim_start_matches('?')).filter(|q| !q.is_empty())),
                "hash" => u.set_fragment(Some(val.trim_start_matches('#')).filter(|f| !f.is_empty())),
                p => return Err(format!("URL has no settable part {p}")),
            }
            Ok(url_parts(&u))
        })),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_calls_and_types() {
        register("test.add", "{ a: number; b: number }", "number", "a + b", |v| Ok(json!(v["a"].as_f64().unwrap() + v["b"].as_f64().unwrap())));
        assert_eq!(call("test.add", &json!({"a": 1, "b": 2})).unwrap(), json!(3.0));
        assert!(call("test.nothing", &json!({})).unwrap_err().contains("no native call"));
        let ts = typescript();
        assert!(ts.contains("\"test.add\": { args: { a: number; b: number }; answer: number };"));
        assert!(ts.contains("\"url.parse\""));
    }

    #[test]
    fn urls_and_digests() {
        let p = call("url.parse", &json!({"url": "../c?x=1#h", "base": "https://a.example:8080/b/d"})).unwrap();
        assert_eq!(p["href"], "https://a.example:8080/c?x=1#h");
        assert_eq!(p["port"], "8080");
        assert_eq!(call("url.parse", &json!({"url": "not a url"})).unwrap(), Value::Null);
        let d = call("web.digest", &json!({"algo": "SHA-256", "data": enc64(b"abc")})).unwrap();
        assert_eq!(dec64(d.as_str().unwrap()).unwrap()[..4], [0xba, 0x78, 0x16, 0xbf]);
    }
}

/// Runs `f` in `cwd` with this process's stdout and stderr captured (into temporary files, so a long output can't
/// block), then puts both back: how an in-process command line (raea-render, raea) answers a shell that can't spawn processes
/// (the iPhone). One at a time: the descriptors and the working folder are the process's own.
#[cfg(unix)]
pub fn capture<T>(cwd: &std::path::Path, f: impl FnOnce() -> T) -> R<(T, String, String)> {
    use std::io::{Read as _, Seek as _, Write as _};
    use std::os::fd::AsRawFd as _;
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    unsafe extern "C" {
        fn dup(fd: i32) -> i32;
        fn dup2(fd: i32, fd2: i32) -> i32;
        fn close(fd: i32) -> i32;
    }
    let dir = std::env::temp_dir();
    let mk = |n: &str| -> R<std::fs::File> {
        let p = dir.join(format!("raea-capture-{}-{n}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0)));
        let f = std::fs::OpenOptions::new().create(true).read(true).write(true).truncate(true).open(&p).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&p);
        Ok(f)
    };
    let (mut out, mut err) = (mk("out")?, mk("err")?);
    let old_cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(cwd).map_err(|e| format!("{}: {e}", cwd.display()))?;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    let (so, se) = unsafe { (dup(1), dup(2)) };
    unsafe {
        dup2(out.as_raw_fd(), 1);
        dup2(err.as_raw_fd(), 2);
    }
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    unsafe {
        dup2(so, 1);
        dup2(se, 2);
        close(so);
        close(se);
    }
    let _ = std::env::set_current_dir(old_cwd);
    let read = |f: &mut std::fs::File| -> String {
        let mut s = Vec::new();
        let _ = f.seek(std::io::SeekFrom::Start(0));
        let _ = f.read_to_end(&mut s);
        String::from_utf8_lossy(&s).into_owned()
    };
    let (o, e) = (read(&mut out), read(&mut err));
    match r {
        Ok(t) => Ok((t, o, e)),
        Err(p) => {
            let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| "it panicked".into());
            Err(format!("{msg}\n{e}"))
        }
    }
}
