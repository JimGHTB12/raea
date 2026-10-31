//! JsEngine on JavaScriptCore: a Context whose natives are closures (Context::set_native), promise jobs drained as the
//! outermost API call ends, ES modules through the system's loader (modules.rs), and the thread's run loop pumped
//! between frames for JavaScriptCore's GC timers.

use super::{Context, modules::Modules, pump};
use crate::{Arg, Fetch, JsEngine, Native};

pub struct Jsc {
    ctx: Context,
    /// the module loader, kept while the context runs modules (JavaScriptCore holds its delegate weakly)
    modules: Option<Modules>,
}

impl Jsc {
    pub fn new() -> Jsc {
        Jsc { ctx: Context::new(), modules: None }
    }
}

impl Default for Jsc {
    fn default() -> Self {
        Self::new()
    }
}

impl JsEngine for Jsc {
    fn name(&self) -> &'static str {
        "jsc"
    }

    fn native(&mut self, name: &str, f: Native) -> Result<(), String> {
        self.ctx.set_native(name, f);
        Ok(())
    }

    fn eval(&mut self, src: &str, file: &str) -> Result<(), String> {
        self.ctx.eval(src, file)
    }

    fn call(&mut self, func: &str, args: &[Arg]) -> Option<Result<f64, String>> {
        let args: Vec<super::Arg> = args
            .iter()
            .map(|a| match a {
                Arg::Num(n) => super::Arg::Num(*n),
                Arg::Str(s) => super::Arg::Str(s),
                Arg::Null => super::Arg::Null,
            })
            .collect();
        self.ctx.call_num(func, &args)
    }

    fn module(&mut self, url: &str, src: &str, fetch: Fetch, global: &str) -> Result<(), String> {
        let modules = self.modules.insert(Modules::new(self.ctx.raw(), fetch));
        let p = modules.eval(url, src)?;
        self.ctx.keep(global, p);
        Ok(())
    }

    fn time_limit(&mut self, limit: std::time::Duration) {
        self.ctx.time_limit(limit);
    }

    fn collect(&mut self) {
        self.ctx.collect();
    }

    fn collect_young(&mut self) -> bool {
        self.ctx.collect_young()
    }

    fn release_memory(&mut self) -> Option<f64> {
        super::release_free_memory()
    }

    fn idle(&mut self) {
        pump();
    }
}
