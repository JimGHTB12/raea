//! QuickJS through rquickjs: the portable engine (a C engine built with the crate: macOS, iOS, Linux, Windows, Android
//! and wasm alike; no JIT, which iOS forbids anyway). Natives are closures; promise jobs are run after every eval and
//! call, as JavaScriptCore does at the end of its outermost call.

use std::time::{Duration, Instant};

use rquickjs::context::EvalOptions;
use rquickjs::function::{Args, Rest};
use rquickjs::loader::{ImportAttributes, Loader, Resolver};
use rquickjs::module::Declared;
use rquickjs::prelude::Coerced;
use rquickjs::{CatchResultExt, Context, Ctx, Exception, Function, Module, Runtime, Value};

use crate::{Arg, Fetch, JsEngine, Native, Ret};

pub struct Quick {
    rt: Runtime,
    ctx: Context,
}

impl Quick {
    pub fn new() -> Result<Quick, String> {
        let rt = Runtime::new().map_err(|e| e.to_string())?;
        // React's commit recurses through the tree; give it room (the host runs on the main thread's 8 MB stack)
        rt.set_max_stack_size(4 << 20);
        let ctx = Context::full(&rt).map_err(|e| e.to_string())?;
        Ok(Quick { rt, ctx })
    }

    /// The promise jobs queued, run until none is left (one that throws is logged, as an unhandled rejection is).
    fn jobs(&self) {
        loop {
            match self.rt.execute_pending_job() {
                Ok(true) => {}
                Ok(false) => break,
                Err(e) => {
                    let msg = e.0.with(|ctx| ctx.catch().into_exception().map(|x| x.to_string()).unwrap_or_else(|| "a job threw".into()));
                    eprintln!("raea-runtime-jsengine: quickjs: {msg}");
                }
            }
        }
    }
}

/// Imports name their modules by URL already (the host rewrote them); one relative to its importer is joined to its folder.
struct Urls;

impl Resolver for Urls {
    fn resolve<'js>(&mut self, _: &Ctx<'js>, base: &str, name: &str, _: Option<ImportAttributes<'js>>) -> rquickjs::Result<String> {
        if !name.starts_with('.') {
            return Ok(name.to_string());
        }
        let dir = base.rsplit_once('/').map_or("", |(d, _)| d);
        Ok(format!("{dir}/{}", name.trim_start_matches("./")))
    }
}

/// A module declared with its import.meta.url (QuickJS sets none for a module it didn't load from a file itself;
/// JavaScriptCore does, and Node's createRequire(import.meta.url) needs it).
fn declare<'js>(ctx: &Ctx<'js>, url: &str, src: String) -> rquickjs::Result<Module<'js, Declared>> {
    let m = Module::declare(ctx.clone(), url, src)?;
    m.meta()?.set("url", url)?;
    Ok(m)
}

/// Each module's source from the host's fetch.
struct Sources(Fetch);

impl Loader for Sources {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str, _: Option<ImportAttributes<'js>>) -> rquickjs::Result<Module<'js, Declared>> {
        let src = (self.0)(name).map_err(|e| Exception::throw_message(ctx, &e))?;
        declare(ctx, name, src)
    }
}

/// `f` as the global `name`: its arguments coerced to strings, its Ret made a JS value, its Err thrown as an Error.
fn install<'js>(ctx: &Ctx<'js>, name: &str, f: Native) -> rquickjs::Result<()> {
    let func = Function::new(ctx.clone(), move |ctx: Ctx<'js>, args: Rest<Coerced<String>>| -> rquickjs::Result<Value<'js>> {
        let args: Vec<String> = args.0.into_iter().map(|a| a.0).collect();
        match f(&args) {
            Ok(Ret::Undefined) => Ok(Value::new_undefined(ctx)),
            Ok(Ret::Num(n)) => Ok(Value::new_float(ctx, n)),
            Ok(Ret::Str(s)) => Ok(rquickjs::String::from_str(ctx, &s)?.into_value()),
            Err(e) => Err(Exception::throw_message(&ctx, &e)),
        }
    })?;
    ctx.globals().set(name, func)
}

impl JsEngine for Quick {
    fn name(&self) -> &'static str {
        "quickjs"
    }

    fn native(&mut self, name: &str, f: Native) -> Result<(), String> {
        self.ctx.with(|ctx| install(&ctx, name, f).map_err(|e| e.to_string()))
    }

    fn eval(&mut self, src: &str, file: &str) -> Result<(), String> {
        // a classic script, sloppy as a browser runs one (rquickjs's options default to strict), named in its traces
        let r = self.ctx.with(|ctx| {
            let mut o = EvalOptions::default();
            o.strict = false;
            o.filename = Some(file.to_string());
            ctx.eval_with_options::<(), _>(src, o).catch(&ctx).map_err(|e| format!("{file}: {e}"))
        });
        self.jobs();
        r
    }

    fn call(&mut self, func: &str, args: &[Arg]) -> Option<Result<f64, String>> {
        let r = self.ctx.with(|ctx| {
            let f: Function = ctx.globals().get::<_, Value>(func).ok()?.into_function()?;
            let mut a = Args::new(ctx.clone(), args.len());
            for x in args {
                let pushed = match x {
                    Arg::Num(n) => a.push_arg(*n),
                    Arg::Str(s) => a.push_arg(*s),
                    Arg::Null => a.push_arg(rquickjs::Null),
                };
                if let Err(e) = pushed {
                    return Some(Err(e.to_string()));
                }
            }
            Some(match f.call_arg::<Value>(a).catch(&ctx) {
                Ok(v) => Ok(v.as_number().unwrap_or(f64::NAN)),
                Err(e) => Err(e.to_string()),
            })
        });
        self.jobs();
        r
    }

    fn module(&mut self, url: &str, src: &str, fetch: Fetch, global: &str) -> Result<(), String> {
        self.rt.set_loader(Urls, Sources(fetch));
        // QuickJS fails an import that won't load (or a syntax error) here, where JavaScriptCore rejects the promise: both
        // end as a rejected promise, so the host watches one thing
        let r = self.ctx.with(|ctx| -> rquickjs::Result<()> {
            match declare(&ctx, url, src.to_string()).and_then(|m| m.eval()).map(|(_, p)| p).catch(&ctx) {
                Ok(p) => ctx.globals().set(global, p),
                Err(e) => {
                    ctx.globals().set("__raea_failed", format!("{url}: {e}"))?;
                    let p: Value = ctx.eval("Promise.reject(new Error(globalThis.__raea_failed))")?;
                    ctx.globals().set(global, p)
                }
            }
        })
        .map_err(|e| e.to_string());
        self.jobs();
        r
    }

    fn time_limit(&mut self, limit: Duration) {
        let until = Instant::now() + limit;
        self.rt.set_interrupt_handler(Some(Box::new(move || Instant::now() > until)));
    }

    fn collect(&mut self) {
        self.rt.run_gc();
    }
}
