//! JavaScriptCore, the system's JS engine on macOS and iOS, through its C API: a context, global functions backed by
//! Rust, scripts evaluated, globals called. What raea runs its TypeScript on, on Apple (no engine shipped in the binary,
//! Apple's own GC; jitless on iOS as Apple requires). `Jsc` (jsc/host.rs) is this behind the crate's JsEngine.

use std::ffi::{c_char, c_void};
use std::ptr::null_mut;
use std::time::Duration;

type Ctx = *const c_void;
type GCtx = *mut c_void;
type Obj = *mut c_void;
type Val = *const c_void;
type Str = *mut c_void;

/// A global function's body: (the context, its arguments, where to put an exception) -> its value.
pub type Callback = unsafe extern "C" fn(ctx: Ctx, function: Obj, this: Obj, argc: usize, argv: *const Val, exception: *mut Val) -> Val;

unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopDefaultMode: *const c_void;
    fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after_source_handled: u8) -> i32;
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopCopyCurrentMode(rl: *mut c_void) -> *const c_void;
    fn CFRelease(cf: *const c_void);
}

/// This thread's run loop run once, without waiting: JavaScriptCore schedules its garbage collections (full and
/// eden, by its activity callbacks) as timers on the thread's run loop, so a JS thread that never runs one only ever
/// collects when allocation forces it, and its old generation grew (a live pong at 120 fps: 29 MB a minute). A JS
/// thread's loop calls this between events. Not from inside a run of the loop (JS on the main thread, under winit's
/// event handler: a Blitz page's step): that loop fires the timers itself, and running it again re-entered winit,
/// which panics ("tried to handle event while another event is currently being handled").
pub fn pump() {
    unsafe {
        let running = CFRunLoopCopyCurrentMode(CFRunLoopGetCurrent());
        if !running.is_null() {
            CFRelease(running);
            return;
        }
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.0, 0);
    }
}

#[link(name = "JavaScriptCore", kind = "framework")]
unsafe extern "C" {
    fn JSGlobalContextCreate(global_class: *mut c_void) -> GCtx;
    fn JSGlobalContextRelease(ctx: GCtx);
    fn JSContextGetGlobalObject(ctx: Ctx) -> Obj;
    fn JSStringCreateWithUTF8CString(s: *const c_char) -> Str;
    fn JSStringRelease(s: Str);
    fn JSStringGetMaximumUTF8CStringSize(s: Str) -> usize;
    fn JSStringGetUTF8CString(s: Str, buf: *mut c_char, size: usize) -> usize;
    fn JSEvaluateScript(ctx: Ctx, script: Str, this: Obj, source_url: Str, line: i32, exception: *mut Val) -> Val;
    fn JSObjectMakeFunctionWithCallback(ctx: Ctx, name: Str, f: Callback) -> Obj;
    fn JSObjectSetProperty(ctx: Ctx, obj: Obj, name: Str, value: Val, attrs: u32, exception: *mut Val);
    fn JSObjectGetProperty(ctx: Ctx, obj: Obj, name: Str, exception: *mut Val) -> Val;
    fn JSObjectCallAsFunction(ctx: Ctx, f: Obj, this: Obj, argc: usize, argv: *const Val, exception: *mut Val) -> Val;
    fn JSValueIsObject(ctx: Ctx, v: Val) -> bool;
    fn JSValueIsUndefined(ctx: Ctx, v: Val) -> bool;
    fn JSValueIsNull(ctx: Ctx, v: Val) -> bool;
    fn JSValueToStringCopy(ctx: Ctx, v: Val, exception: *mut Val) -> Str;
    fn JSValueToNumber(ctx: Ctx, v: Val, exception: *mut Val) -> f64;
    fn JSValueMakeNumber(ctx: Ctx, n: f64) -> Val;
    fn JSValueMakeString(ctx: Ctx, s: Str) -> Val;
    fn JSValueMakeNull(ctx: Ctx) -> Val;
    fn JSValueMakeUndefined(ctx: Ctx) -> Val;
    fn JSObjectMakeError(ctx: Ctx, argc: usize, argv: *const Val, exception: *mut Val) -> Obj;
    fn JSGarbageCollect(ctx: Ctx);
    fn JSContextGetGroup(ctx: Ctx) -> *const c_void;
    fn JSValueUnprotect(ctx: Ctx, v: Val);
    fn JSClassCreate(definition: *const ClassDefinition) -> *mut c_void;
    fn JSObjectMake(ctx: Ctx, class: *mut c_void, data: *mut c_void) -> Obj;
    fn JSObjectGetPrivate(obj: Obj) -> *mut c_void;
}

fn js_str(s: &str) -> Str {
    // written NUL-terminated (NULs dropped) into a buffer kept per thread (ZeroHeap: a String and a CString a call)
    thread_local! {
        static BUF: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    BUF.with_borrow_mut(|b| {
        b.clear();
        b.extend(s.bytes().filter(|&c| c != 0));
        b.push(0);
        unsafe { JSStringCreateWithUTF8CString(b.as_ptr() as *const c_char) }
    })
}

fn rust_str(s: Str) -> String {
    unsafe {
        let n = JSStringGetMaximumUTF8CStringSize(s);
        let mut buf = vec![0u8; n.max(1)];
        let len = JSStringGetUTF8CString(s, buf.as_mut_ptr() as *mut c_char, n);
        buf.truncate(len.saturating_sub(1));
        String::from_utf8_lossy(&buf).into_owned()
    }
}

/// A value as a string (JS's String(v)).
///
/// # Safety
/// `v` must belong to `ctx`.
pub unsafe fn to_string(ctx: Ctx, v: Val) -> String {
    unsafe {
        let s = JSValueToStringCopy(ctx, v, null_mut());
        if s.is_null() {
            return String::new();
        }
        let out = rust_str(s);
        JSStringRelease(s);
        out
    }
}

/// Argument `i` of a callback as a string ("" when missing).
///
/// # Safety
/// `argv` must hold `argc` values of `ctx`.
pub unsafe fn arg(ctx: Ctx, argc: usize, argv: *const Val, i: usize) -> String {
    if i >= argc {
        return String::new();
    }
    unsafe { to_string(ctx, *argv.add(i)) }
}

/// A string value of `ctx`.
pub fn string(ctx: Ctx, s: &str) -> Val {
    let j = js_str(s);
    let v = unsafe { JSValueMakeString(ctx, j) };
    unsafe { JSStringRelease(j) };
    v
}

pub fn number(ctx: Ctx, n: f64) -> Val {
    unsafe { JSValueMakeNumber(ctx, n) }
}

pub fn undefined(ctx: Ctx) -> Val {
    unsafe { JSValueMakeUndefined(ctx) }
}

/// An Error with `msg`, put in `exception` (a callback's way to throw).
///
/// # Safety
/// `exception` must be the callback's.
pub unsafe fn throw(ctx: Ctx, exception: *mut Val, msg: &str) -> Val {
    unsafe {
        let m = string(ctx, msg);
        let e = JSObjectMakeError(ctx, 1, &m, null_mut());
        if !exception.is_null() {
            *exception = e as Val;
        }
        JSValueMakeUndefined(ctx)
    }
}

/// JavaScriptCore's options for raea, set before its first VM (they're read once per process); every host that makes a
/// context calls this first.
/// - FTL, the top tier, off (JSC_useFTLJIT=true brings it back): its compiler took 75 MB at once ~90 s into a live game
///   and kept it, while the baseline and DFG tiers run raea's JS as fast here.
/// - Polling traps: the DFG tier's code checks for termination at loop heads, so a time limit (js-exec's timeout)
///   stops a loop it compiled; with signal traps such a loop ran on past the limit.
pub fn options() {
    static OPTIONS: std::sync::Once = std::sync::Once::new();
    OPTIONS.call_once(|| {
        for (k, v) in [("JSC_useFTLJIT", "false"), ("JSC_usePollingTraps", "true")] {
            if std::env::var_os(k).is_none() {
                unsafe { std::env::set_var(k, v) };
            }
        }
    });
}

/// JavaScriptCore's allocator gives its free pages back to the system now (WTF::releaseFastMallocFreeMemory, where the
/// system exports it). Freed memory stays resident in it otherwise, returned lazily: a live game's resident size crept
/// ~1 MB a minute with the bytes it held flat. Returns the call's time in ms, or None without it.
pub fn release_free_memory() -> Option<f64> {
    type Release = unsafe extern "C" fn();
    static RELEASE: std::sync::OnceLock<Option<Release>> = std::sync::OnceLock::new();
    let f = RELEASE.get_or_init(|| unsafe {
        let p = dlsym(RTLD_DEFAULT, c"_ZN3WTF27releaseFastMallocFreeMemoryEv".as_ptr());
        (!p.is_null()).then(|| std::mem::transmute::<*mut c_void, Release>(p))
    });
    let f = (*f)?;
    let t = std::time::Instant::now();
    unsafe { f() };
    Some(t.elapsed().as_secs_f64() * 1e3)
}

/// JSClassDefinition (JSObjectRef.h, version 0): only the class name, a finalizer and callAsFunction are set.
#[repr(C)]
struct ClassDefinition {
    version: i32,
    attributes: u32,
    class_name: *const c_char,
    parent_class: *mut c_void,
    static_values: *const c_void,
    static_functions: *const c_void,
    initialize: *const c_void,
    finalize: Option<unsafe extern "C" fn(Obj)>,
    has_property: *const c_void,
    get_property: *const c_void,
    set_property: *const c_void,
    delete_property: *const c_void,
    get_property_names: *const c_void,
    call_as_function: Option<Callback>,
    call_as_constructor: *const c_void,
    has_instance: *const c_void,
    convert_to_type: *const c_void,
}

/// The class of a function backed by a Rust closure: the closure is the object's private data (a boxed `Native`),
/// called by `call_native` and dropped by `drop_native` when JavaScriptCore collects the function.
fn native_class() -> *mut c_void {
    static CLASS: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *CLASS.get_or_init(|| {
        let def = ClassDefinition {
            version: 0,
            attributes: 0,
            class_name: c"RaeaNative".as_ptr(),
            parent_class: null_mut(),
            static_values: std::ptr::null(),
            static_functions: std::ptr::null(),
            initialize: std::ptr::null(),
            finalize: Some(drop_native),
            has_property: std::ptr::null(),
            get_property: std::ptr::null(),
            set_property: std::ptr::null(),
            delete_property: std::ptr::null(),
            get_property_names: std::ptr::null(),
            call_as_function: Some(call_native),
            call_as_constructor: std::ptr::null(),
            has_instance: std::ptr::null(),
            convert_to_type: std::ptr::null(),
        };
        unsafe { JSClassCreate(&def) as usize }
    }) as *mut c_void
}

unsafe extern "C" fn call_native(ctx: Ctx, function: Obj, _this: Obj, argc: usize, argv: *const Val, exception: *mut Val) -> Val {
    unsafe {
        let f = &*(JSObjectGetPrivate(function) as *const crate::Native);
        let args: Vec<String> = (0..argc).map(|i| to_string(ctx, *argv.add(i))).collect();
        match f(&args) {
            Ok(crate::Ret::Undefined) => undefined(ctx),
            Ok(crate::Ret::Num(n)) => number(ctx, n),
            Ok(crate::Ret::Str(s)) => string(ctx, &s),
            Err(e) => throw(ctx, exception, &e),
        }
    }
}

unsafe extern "C" fn drop_native(obj: Obj) {
    unsafe {
        let p = JSObjectGetPrivate(obj) as *mut crate::Native;
        if !p.is_null() {
            drop(Box::from_raw(p));
        }
    }
}

/// An argument to `Context::call`.
pub enum Arg<'a> {
    Num(f64),
    Str(&'a str),
    Null,
}

/// A JS context of its own (one global object), used from the thread that made it.
pub struct Context {
    ctx: GCtx,
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl Context {
    pub fn new() -> Context {
        options();
        Context { ctx: unsafe { JSGlobalContextCreate(null_mut()) } }
    }

    fn c(&self) -> Ctx {
        self.ctx as Ctx
    }

    /// The JSGlobalContextRef (for raw use alongside this, as `modules::Modules` does).
    pub fn raw(&self) -> *mut c_void {
        self.ctx
    }

    /// `name` on the global object, a function backed by `f`.
    pub fn set_fn(&self, name: &str, f: Callback) {
        unsafe {
            let n = js_str(name);
            let func = JSObjectMakeFunctionWithCallback(self.c(), n, f);
            JSObjectSetProperty(self.c(), JSContextGetGlobalObject(self.c()), n, func as Val, 0, null_mut());
            JSStringRelease(n);
        }
    }

    /// `name` on the global object, a function backed by the closure `f` (kept until the function is collected).
    pub fn set_native(&self, name: &str, f: crate::Native) {
        unsafe {
            let n = js_str(name);
            let func = JSObjectMake(self.c(), native_class(), Box::into_raw(Box::new(f)) as *mut c_void);
            JSObjectSetProperty(self.c(), JSContextGetGlobalObject(self.c()), n, func as Val, 0, null_mut());
            JSStringRelease(n);
        }
    }

    /// `src` run as a script (`name` for its stack traces); Err carries what it threw. Promise jobs it queued run before
    /// this returns (JSC drains them as the outermost API call ends).
    pub fn eval(&self, src: &str, name: &str) -> Result<(), String> {
        unsafe {
            let (s, u) = (js_str(src), js_str(name));
            let mut exc: Val = std::ptr::null();
            JSEvaluateScript(self.c(), s, null_mut(), u, 1, &mut exc);
            JSStringRelease(s);
            JSStringRelease(u);
            if !exc.is_null() { Err(self.describe(exc)) } else { Ok(()) }
        }
    }

    /// The global function `name` called with `args`: its value as a number (NaN when it isn't one), or what it threw.
    /// None when there's no such function.
    pub fn call_num(&self, name: &str, args: &[Arg]) -> Option<Result<f64, String>> {
        unsafe {
            let n = js_str(name);
            let f = JSObjectGetProperty(self.c(), JSContextGetGlobalObject(self.c()), n, null_mut());
            JSStringRelease(n);
            if f.is_null() || !JSValueIsObject(self.c(), f) {
                return None;
            }
            let val = |a: &Arg| match a {
                Arg::Num(x) => JSValueMakeNumber(self.c(), *x),
                Arg::Str(s) => string(self.c(), s),
                Arg::Null => JSValueMakeNull(self.c()),
            };
            // up to eight arguments on the stack (ZeroHeap: a Vec a call)
            let mut fixed: [Val; 8] = [std::ptr::null(); 8];
            let heap: Vec<Val>;
            let argv: &[Val] = if args.len() <= fixed.len() {
                for (slot, a) in fixed.iter_mut().zip(args) {
                    *slot = val(a);
                }
                &fixed[..args.len()]
            } else {
                heap = args.iter().map(val).collect();
                &heap
            };
            let mut exc: Val = std::ptr::null();
            let v = JSObjectCallAsFunction(self.c(), f as Obj, null_mut(), argv.len(), argv.as_ptr(), &mut exc);
            if !exc.is_null() {
                return Some(Err(self.describe(exc)));
            }
            if v.is_null() || JSValueIsUndefined(self.c(), v) || JSValueIsNull(self.c(), v) {
                return Some(Ok(f64::NAN));
            }
            Some(Ok(JSValueToNumber(self.c(), v, null_mut())))
        }
    }

    /// A script running longer than `limit` at a time is terminated (JavaScriptCore's watchdog, where the system
    /// exports it): js-exec's timeout for a script that never returns.
    pub fn time_limit(&self, limit: Duration) {
        type SetLimit = unsafe extern "C" fn(*const c_void, f64, Option<unsafe extern "C" fn(Ctx, *mut c_void) -> bool>, *mut c_void);
        unsafe extern "C" fn stop(_: Ctx, _: *mut c_void) -> bool {
            true
        }
        unsafe {
            let p = dlsym(RTLD_DEFAULT, c"JSContextGroupSetExecutionTimeLimit".as_ptr());
            if !p.is_null() {
                let f = std::mem::transmute::<*mut c_void, SetLimit>(p);
                f(JSContextGetGroup(self.c()), limit.as_secs_f64(), Some(stop), null_mut());
            }
        }
    }

    /// `value` (a protected JSValueRef, unprotected here) kept as the global `name`.
    pub fn keep(&self, name: &str, value: Val) {
        unsafe {
            let n = js_str(name);
            JSObjectSetProperty(self.c(), JSContextGetGlobalObject(self.c()), n, value, 0, null_mut());
            JSStringRelease(n);
            JSValueUnprotect(self.c(), value);
        }
    }

    /// A young-generation (eden) collection now, where the system exports one (JSSynchronousEdenCollectForDebugging):
    /// about a millisecond, as only what was made since the last collection is looked at. Run after each frame, a
    /// frame's garbage dies young instead of being promoted, so the old generation (and its full collections, ~30 ms
    /// stops in a live game) grows only with what lives. False where there's no such call.
    pub fn collect_young(&self) -> bool {
        type Eden = unsafe extern "C" fn(Ctx);
        static EDEN: std::sync::OnceLock<Option<Eden>> = std::sync::OnceLock::new();
        let f = EDEN.get_or_init(|| unsafe {
            let p = dlsym(RTLD_DEFAULT, c"JSSynchronousEdenCollectForDebugging".as_ptr());
            (!p.is_null()).then(|| std::mem::transmute::<*mut c_void, Eden>(p))
        });
        match f {
            Some(f) => {
                unsafe { f(self.c()) };
                true
            }
            None => false,
        }
    }

    fn describe(&self, exc: Val) -> String {
        unsafe {
            let msg = to_string(self.c(), exc);
            // the stack, when the thrown value has one
            if JSValueIsObject(self.c(), exc) {
                let k = js_str("stack");
                let st = JSObjectGetProperty(self.c(), exc as Obj, k, null_mut());
                JSStringRelease(k);
                if !st.is_null() && !JSValueIsUndefined(self.c(), st) {
                    return format!("{msg}\n{}", to_string(self.c(), st));
                }
            }
            msg
        }
    }

    /// A collection now (between frames, as V8's posted GC jobs ran): JavaScriptCore's synchronous full collection
    /// where the system exports it (JSSynchronousGarbageCollectForDebugging), else a request (JSGarbageCollect, which
    /// only schedules one).
    pub fn collect(&self) {
        type Sync = unsafe extern "C" fn(Ctx);
        static SYNC: std::sync::OnceLock<Option<Sync>> = std::sync::OnceLock::new();
        let f = SYNC.get_or_init(|| unsafe {
            let p = dlsym(RTLD_DEFAULT, c"JSSynchronousGarbageCollectForDebugging".as_ptr());
            (!p.is_null()).then(|| std::mem::transmute::<*mut c_void, Sync>(p))
        });
        unsafe {
            match f {
                Some(f) => f(self.c()),
                None => JSGarbageCollect(self.c()),
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { JSGlobalContextRelease(self.ctx) }
    }
}

/// ES modules on a JavaScriptCore context, through the system's Objective-C module loader (JSScript and a
/// JSModuleLoaderDelegate; macOS 10.15, iOS 13): JavaScriptCore resolves each import against its importer's URL and
/// asks `fetch` for the source at the absolute URL, once per URL (its registry keeps the rest: cycles, live bindings,
/// top-level await all as the spec has them). Bare names never reach `fetch`: the host rewrites them to URLs first
/// (raea-modules' rewrite_specifiers).
pub mod modules;

mod host;
pub use host::Jsc;
