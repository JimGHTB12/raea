//! A page root's JS engine (docs/ts-native.feature P21): raea-runtime-jsengine's, JavaScriptCore on Apple and QuickJS
//! elsewhere, with the natives a root's bundle talks to the host through, so nothing engine-specific reaches the host
//! or the bundle.
//!
//! The natives every root gets before the prelude runs: `__ui_flush(json)` (a batch of ops), `__ui_emit(name, json)`
//! (an ask), `__ui_call(id, name, json)` (a host call), `__ui_log(level, text)`, `__ui_now()` (ms since the process
//! started) and `__ui_sync(name, json)` (a synchronous host call, src/calls.rs, answered before it returns). What the
//! first three are handed lands in the root's `Outbox`, which the host drains after each call into the engine.

use std::cell::RefCell;
use std::rc::Rc;

pub use raea_runtime_jsengine::{Arg, Engine, JsEngine, now_ms};
use raea_runtime_jsengine::{Native, Ret, arg, clock};

/// What a root's JS handed the host since it was last drained.
#[derive(Default)]
pub struct Outbox {
    /// op batches, as `{v, ops}` JSON (src/ops.rs)
    pub flushed: Vec<String>,
    /// asks: (name, JSON)
    pub emits: Vec<(String, String)>,
    /// host calls: (call id, name, JSON args)
    pub requests: Vec<(u64, String, String)>,
}

pub type Shared = Rc<RefCell<Outbox>>;

/// A fresh `engine` whose natives write into `out`.
pub fn start(engine: Engine, out: Shared) -> Result<Box<dyn JsEngine>, String> {
    let mut js = engine.start()?;
    let natives: [(&str, Native); 6] = [
        ("__ui_flush", {
            let o = out.clone();
            Box::new(move |x| {
                o.borrow_mut().flushed.push(arg(x, 0).into());
                Ok(Ret::Undefined)
            })
        }),
        ("__ui_emit", {
            let o = out.clone();
            Box::new(move |x| {
                o.borrow_mut().emits.push((arg(x, 0).into(), arg(x, 1).into()));
                Ok(Ret::Undefined)
            })
        }),
        ("__ui_call", {
            let o = out.clone();
            Box::new(move |x| {
                let id = arg(x, 0).parse::<f64>().unwrap_or(0.0) as u64;
                o.borrow_mut().requests.push((id, arg(x, 1).into(), arg(x, 2).into()));
                Ok(Ret::Undefined)
            })
        }),
        ("__ui_log", Box::new(move |x| {
            log(arg(x, 0), arg(x, 1));
            Ok(Ret::Undefined)
        })),
        ("__ui_now", clock()),
        ("__ui_sync", Box::new(move |x| Ok(Ret::Str(crate::calls::sync(arg(x, 0), arg(x, 1)))))),
    ];
    for (name, f) in natives {
        js.native(name, f)?;
    }
    Ok(js)
}

/// `__ui_log`: warnings and errors always, the rest with BEVY_TSX_TRACE (or the feed's RAEA_FEED_BEVY_TRACE).
pub fn log(level: &str, text: &str) {
    if level != "0" || crate::trace().is_some() {
        eprintln!("raea-runtime-js: js: {text}");
    }
}

/// The platform shims a bundle expects (js/prelude.js).
pub const PRELUDE: &str = include_str!("../js/prelude.js");
