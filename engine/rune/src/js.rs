//! A bundled TSX app run in the host (R4a): raea-runtime-jsengine's engine (JavaScriptCore on Apple, QuickJS
//! elsewhere) with the natives @raea/ui's wire.ts talks to a host through. What the JS hands the host (op batches, asks,
//! calls) and what the host has for the JS (events, host events, layout snapshots) go back and forth until neither has
//! anything left: `Js::pump`, against a `Host` (the headless session, or a window's page, R7a). JS time is the host's
//! clock: virtual headless, so a run is the same every time.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use raea_io_net_calls::{CALLS, Net};
use raea_runtime_jsengine::{Arg, Engine, JsEngine, Native, Ret, arg};
use serde_json::{Value, json};

use crate::page::{Out, Page};
use crate::session::Session;

/// The platform a bundle expects (console, microtasks, timers on the host's clock).
pub const PRELUDE: &str = include_str!("prelude.js");

/// What the JS pumps against.
pub trait Host {
    /// The clock, in seconds.
    fn clock(&self) -> f64;
    /// An op batch from the JS.
    fn apply(&mut self, batch: &str) -> Result<(), String>;
    /// An `__ui_emit` from the JS.
    fn emit(&mut self, name: &str, v: &Value);
    /// What the JS hasn't been sent, taken.
    fn take_out(&mut self) -> Vec<Out>;
}

/// What the JS handed the host since the last pump.
#[derive(Default)]
struct Asks {
    flushed: Vec<String>,
    emits: Vec<(String, String)>,
    /// a call's id, name and args (JSON)
    calls: Vec<(u64, String, String)>,
}

/// The host calls the native target answers (docs/targets.feature T2a): net.* by raea-io-net-calls, as the web
/// target does. Their answers and a socket's events (the wire's `answer` and `named` messages) come back on `rx`, from
/// whichever thread has them, and go to the JS at the next pump.
struct Calls {
    net: Net<()>,
    tx: Sender<Value>,
    rx: Receiver<Value>,
    refused: Refused,
}

/// The calls this host refused (T2c), by name: why, and how many times. The check lists them beside unmapped.json.
pub type Refused = Rc<RefCell<BTreeMap<String, (String, u32)>>>;

/// Why `name` isn't answered here: the world API is the web target's until T6, anything else this host lacks.
fn refusal(name: &str) -> String {
    if name.starts_with("world.") || name.starts_with("registry.") || name.starts_with("schedule.") {
        format!("{name}: no world on the native target (targets.feature T6)")
    } else {
        format!("the Rune host has no {name}")
    }
}

fn refuse(refused: &Refused, name: &str) -> String {
    refuse_as(refused, name, refusal(name))
}

fn refuse_as(refused: &Refused, name: &str, why: String) -> String {
    refused.borrow_mut().entry(name.to_string()).or_insert_with(|| (why.clone(), 0)).1 += 1;
    why
}

impl Calls {
    fn new(refused: Refused) -> Calls {
        let (tx, rx) = channel();
        let push = tx.clone();
        let net = Net::new(Arc::new(move |(), ev| {
            let _ = push.send(ev);
        }));
        Calls { net, tx, rx, refused }
    }

    /// Call `id`: answered here (now or later), or refused as a call this host doesn't have.
    fn call(&self, id: u64, name: &str, args: &str) {
        let tx = self.tx.clone();
        let done = Box::new(move |r| {
            let _ = tx.send(raea_core_wire::answer(id, r));
        });
        let args = || serde_json::from_str(args).unwrap_or(Value::Null);
        if CALLS.contains(&name) {
            self.net.call((), name, args(), done);
        } else if raea_io_os::CALLS.contains(&name) {
            // a headless pick, save or dialog no step answers is refused and listed, as a call this host lacks is
            match raea_io_os::unscripted(name) {
                Some(why) => done(Err(refuse_as(&self.refused, name, why))),
                None => raea_io_os::call(name, args(), done),
            }
        } else {
            done(Err(refuse(&self.refused, name)));
        }
    }
}

/// The engine with a bundle running in it.
pub struct Js {
    js: Box<dyn JsEngine>,
    asks: Rc<RefCell<Asks>>,
    calls: Calls,
    now: Rc<Cell<f64>>,
    /// what the page logged, as `level: text` (0 log, 1 warn, 2 error)
    pub logs: Rc<RefCell<Vec<String>>>,
    /// the calls, async and sync, this host refused
    pub refused: Refused,
}

/// Rounds of back and forth one pump allows: a page and host still answering each other past this are a loop.
const ROUNDS: usize = 256;

impl Js {
    /// A fresh engine with the natives and the prelude; `bundle` runs with `eval` once the host is ready for it.
    pub fn new() -> Result<Js, String> {
        let asks: Rc<RefCell<Asks>> = Rc::default();
        let now = Rc::new(Cell::new(0.0));
        let logs: Rc<RefCell<Vec<String>>> = Rc::default();
        let refused: Refused = Rc::default();
        let mut js = Engine::default().start()?;
        let push = |a: &Rc<RefCell<Asks>>, f: fn(&mut Asks, &[String])| -> Native {
            let a = a.clone();
            Box::new(move |x| {
                let args: Vec<String> = (0..3).map(|i| arg(x, i).to_string()).collect();
                f(&mut a.borrow_mut(), &args);
                Ok(Ret::Undefined)
            })
        };
        let natives: [(&str, Native); 6] = [
            ("__ui_flush", push(&asks, |a, x| a.flushed.push(x[0].clone()))),
            ("__ui_emit", push(&asks, |a, x| a.emits.push((x[0].clone(), x[1].clone())))),
            ("__ui_call", push(&asks, |a, x| a.calls.push((x[0].parse::<f64>().unwrap_or(0.0) as u64, x[1].clone(), x[2].clone())))),
            // no synchronous host calls yet: each answers that it isn't there, as a host without it does
            ("__ui_sync", {
                let r = refused.clone();
                Box::new(move |x| Ok(Ret::Str(json!({"err": refuse(&r, arg(x, 0))}).to_string())))
            }),
            ("__ui_log", {
                let l = logs.clone();
                Box::new(move |x| {
                    let (level, text) = (arg(x, 0), arg(x, 1));
                    if level != "0" {
                        eprintln!("raea-rune-host: js: {text}");
                    }
                    l.borrow_mut().push(format!("{level}: {text}"));
                    Ok(Ret::Undefined)
                })
            }),
            ("__ui_now", {
                let n = now.clone();
                Box::new(move |_| Ok(Ret::Num(n.get())))
            }),
        ];
        for (name, f) in natives {
            js.native(name, f)?;
        }
        js.eval(PRELUDE, "prelude.js")?;
        Ok(Js { js, asks, calls: Calls::new(refused.clone()), now, logs, refused })
    }

    /// `bundle` (a classic script: an IIFE that mounts its app) run, then the pump.
    pub fn run(&mut self, bundle: &str, h: &mut impl Host) -> Result<(), String> {
        self.now.set(h.clock() * 1e3);
        self.js.eval(bundle, "app.js")?;
        self.pump(h)
    }

    /// The JS's asks to the host and the host's messages to the JS, until neither has any.
    pub fn pump(&mut self, h: &mut impl Host) -> Result<(), String> {
        for _ in 0..ROUNDS {
            self.now.set(h.clock() * 1e3);
            let asks = std::mem::take(&mut *self.asks.borrow_mut());
            let out = h.take_out();
            let answered: Vec<Value> = self.calls.rx.try_iter().collect();
            if asks.flushed.is_empty() && asks.emits.is_empty() && asks.calls.is_empty() && out.is_empty() && answered.is_empty() {
                return Ok(());
            }
            for m in &answered {
                self.call("__ui_deliver", &m.to_string())?;
            }
            for b in &asks.flushed {
                h.apply(b)?;
            }
            for (name, v) in &asks.emits {
                h.emit(name, &serde_json::from_str(v).unwrap_or(Value::Null));
            }
            for (id, name, args) in &asks.calls {
                self.calls.call(*id, name, args);
            }
            for o in out {
                match o {
                    Out::Deliver(m) => self.call("__ui_deliver", &m)?,
                    Out::Layout(l) => self.call("__ui_layout", &l)?,
                }
            }
        }
        Err(format!("the page and the host were still answering each other after {ROUNDS} rounds"))
    }

    /// Frame `n` at the host's clock: the JS's frame number, its timers due, then the pump. The ms to the next timer
    /// (none: None).
    pub fn frame(&mut self, n: u64, h: &mut impl Host) -> Result<Option<f64>, String> {
        self.now.set(h.clock() * 1e3);
        self.js.call("__ui_setFrame", &[Arg::Num(n as f64)]);
        let next = self.timers()?;
        self.pump(h)?;
        Ok(next)
    }

    /// Each JS timer due by now, one call each (its microtasks run before the next); the ms to the next one after.
    pub fn timers(&mut self) -> Result<Option<f64>, String> {
        let now = self.now.get();
        for _ in 0..10_000 {
            match self.js.call("__ui_timers", &[Arg::Num(now)]) {
                Some(Ok(n)) if n == 0.0 => continue,
                Some(Ok(n)) if n > 0.0 => return Ok(Some(n)),
                Some(Err(e)) => return Err(format!("timers: {e}")),
                _ => return Ok(None),
            }
        }
        Err("timers kept coming due at the same time".into())
    }

    fn call(&mut self, f: &str, s: &str) -> Result<(), String> {
        match self.js.call(f, &[Arg::Str(s)]) {
            Some(Err(e)) => Err(format!("{f}: {e}")),
            _ => Ok(()),
        }
    }
}

impl Host for Session {
    fn clock(&self) -> f64 {
        Session::clock(self)
    }
    fn apply(&mut self, batch: &str) -> Result<(), String> {
        Session::apply(self, batch)
    }
    fn emit(&mut self, name: &str, v: &Value) {
        Session::emit(self, name, v)
    }
    fn take_out(&mut self) -> Vec<Out> {
        Session::take_out(self)
    }
}

/// A bundle run headless: the engine and the session its page lives in.
pub struct App {
    js: Js,
    pub session: Session,
    pub logs: Rc<RefCell<Vec<String>>>,
    pub refused: Refused,
}

impl App {
    /// `bundle` run in a fresh engine over a `viewport` page.
    pub fn new(bundle: &str, viewport: (f32, f32)) -> Result<App, String> {
        let mut js = Js::new()?;
        let mut session = Session::new(Page::default(), viewport);
        js.run(bundle, &mut session)?;
        Ok(App { logs: js.logs.clone(), refused: js.refused.clone(), js, session })
    }

    /// The pump (Js::pump) against the session.
    pub fn pump(&mut self) -> Result<(), String> {
        self.js.pump(&mut self.session)
    }

    /// A frame: the session's (the clock moves on, Rune lays out), then the JS's.
    pub fn frame(&mut self) -> Result<(), String> {
        self.session.frame();
        self.js.frame(self.session.frames(), &mut self.session).map(drop)
    }

    /// Input through the session (`f` drives it: a click, keys, a wheel), then the JS's timers and the pump.
    pub fn input(&mut self, f: impl FnOnce(&mut Session)) -> Result<(), String> {
        f(&mut self.session);
        self.js.now.set(self.session.clock() * 1e3);
        self.js.timers()?;
        self.pump()
    }
}
