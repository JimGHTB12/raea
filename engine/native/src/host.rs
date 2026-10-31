//! The native-call layer on a JS engine: `__raea_call` (the registry, under this thread's allow list) and the web shims
//! bundles expect, on any engine raea-runtime-jsengine has.

use raea_runtime_jsengine::{JsEngine, Ret, arg};

/// `__raea_call` on the engine's global, then the web shims run in it.
pub fn install(js: &mut dyn JsEngine) -> Result<(), String> {
    js.native("__raea_call", Box::new(raea_call))?;
    js.eval(crate::WEB_JS, "web.js").map_err(|e| format!("web.js: {e}"))
}

/// `__raea_call(name, json)`: the call `name` with its JSON arguments, if this thread allows it; its JSON answer.
fn raea_call(a: &[String]) -> Result<Ret, String> {
    let name = arg(a, 0);
    if !crate::allowed(name) {
        return Err(format!("{name}: not allowed here"));
    }
    let v: serde_json::Value = a.get(1).and_then(|j| serde_json::from_str(j).ok()).unwrap_or(serde_json::Value::Null);
    crate::call(name, &v).map(|out| Ret::Str(out.to_string())).map_err(|e| format!("{name}: {e}"))
}
