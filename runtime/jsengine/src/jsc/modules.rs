use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char, c_void};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObject};
use objc2::{AnyThread, DefinedClass, define_class, msg_send};
use objc2::{Encoding, RefEncode};

/// JSGlobalContextRef's pointee, typed so the message to contextWithJSGlobalContextRef: carries its encoding
/// (`^{OpaqueJSContext=}`); objc2 checks a message's argument types in debug builds, and `*mut c_void` is `^v`.
#[repr(C)]
struct OpaqueJSContext {
    _p: [u8; 0],
}

unsafe impl RefEncode for OpaqueJSContext {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("OpaqueJSContext", &[]));
}

/// JSValueRef's pointee, likewise (`^{OpaqueJSValue=}`).
#[repr(C)]
struct OpaqueJSValue {
    _p: [u8; 0],
}

unsafe impl RefEncode for OpaqueJSValue {
    const ENCODING_REF: Encoding = Encoding::Pointer(&Encoding::Struct("OpaqueJSValue", &[]));
}

/// The source of the module at an absolute URL, or why there's none.
pub type Fetch = Box<dyn Fn(&str) -> Result<String, String>>;

pub struct State {
    fetch: Fetch,
    /// entry modules being evaluated: JavaScriptCore fetches an entry's own URL too
    entries: RefCell<HashMap<String, String>>,
}

fn ns(s: &str) -> Retained<AnyObject> {
    let c = CString::new(s.replace('\0', "")).unwrap();
    unsafe { msg_send![objc2::class!(NSString), stringWithUTF8String: c.as_ptr()] }
}

fn rs(o: &AnyObject) -> String {
    unsafe {
        let p: *const c_char = msg_send![o, UTF8String];
        if p.is_null() { String::new() } else { CStr::from_ptr(p).to_string_lossy().into_owned() }
    }
}

/// A module script for `url`, or JavaScriptCore's reason it isn't one (a syntax error).
fn script(vm: &AnyObject, url: &str, source: &str) -> Result<Retained<AnyObject>, String> {
    unsafe {
        let u: Option<Retained<AnyObject>> = msg_send![objc2::class!(NSURL), URLWithString: &*ns(url)];
        let Some(u) = u else { return Err(format!("{url}: not a URL")) };
        let mut err: *mut AnyObject = std::ptr::null_mut();
        let s: Option<Retained<AnyObject>> = msg_send![objc2::class!(JSScript), scriptOfType: 1isize, withSource: &*ns(source), andSourceURL: &*u, andBytecodeCache: std::ptr::null::<AnyObject>(), inVirtualMachine: vm, error: &mut err];
        match s {
            Some(s) => Ok(s),
            None if !err.is_null() => {
                let d: Retained<AnyObject> = msg_send![&*err, localizedDescription];
                Err(format!("{url}: {}", rs(&d)))
            }
            None => Err(format!("{url}: not a module")),
        }
    }
}

fn call1(f: &AnyObject, arg: &AnyObject) {
    unsafe {
        let arr: Retained<AnyObject> = msg_send![objc2::class!(NSArray), arrayWithObject: arg];
        let _: Option<Retained<AnyObject>> = msg_send![f, callWithArguments: &*arr];
    }
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RaeaJSCModuleLoader"]
    #[ivars = State]
    struct Loader;

    impl Loader {
        #[unsafe(method(context:fetchModuleForIdentifier:withResolveHandler:andRejectHandler:))]
        fn fetch_module(&self, ctx: &AnyObject, id: &AnyObject, resolve: &AnyObject, reject: &AnyObject) {
            let url = unsafe {
                let s: Retained<AnyObject> = msg_send![id, toString];
                rs(&s)
            };
            let st = self.ivars();
            let source = match st.entries.borrow().get(&url) {
                Some(s) => Ok(s.clone()),
                None => (st.fetch)(&url),
            };
            let vm: Retained<AnyObject> = unsafe { msg_send![ctx, virtualMachine] };
            match source.and_then(|s| script(&vm, &url, &s)) {
                Ok(s) => call1(resolve, &s),
                Err(e) => unsafe {
                    // an Error, so the import's rejection reads as one
                    let err: Retained<AnyObject> = msg_send![objc2::class!(JSValue), valueWithNewErrorFromMessage: &*ns(&e), inContext: ctx];
                    call1(reject, &err);
                },
            }
        }
    }
);

/// A context's module loader: keep it as long as the context runs modules (JavaScriptCore holds its delegate weakly).
pub struct Modules {
    ctx: Retained<AnyObject>,
    loader: Retained<Loader>,
}

impl Modules {
    /// Modules for the context `gctx` (a JSGlobalContextRef), their sources from `fetch`.
    pub fn new(gctx: *mut c_void, fetch: Fetch) -> Modules {
        unsafe {
            let ctx: Retained<AnyObject> = msg_send![objc2::class!(JSContext), contextWithJSGlobalContextRef: gctx as *mut OpaqueJSContext];
            let loader = Loader::alloc().set_ivars(State { fetch, entries: RefCell::new(HashMap::new()) });
            let loader: Retained<Loader> = msg_send![super(loader), init];
            let _: () = msg_send![&*ctx, setModuleLoaderDelegate: &*loader];
            Modules { ctx, loader }
        }
    }

    /// `source` evaluated as the module at `url` (its imports fetched as they're reached): the evaluation's promise
    /// (a JSValueRef protected for the caller, who unprotects it), or why it couldn't start (a syntax error).
    pub fn eval(&self, url: &str, source: &str) -> Result<*const c_void, String> {
        unsafe {
            let vm: Retained<AnyObject> = msg_send![&*self.ctx, virtualMachine];
            let s = script(&vm, url, source)?;
            self.loader.ivars().entries.borrow_mut().insert(url.to_string(), source.to_string());
            let p: Option<Retained<AnyObject>> = msg_send![&*self.ctx, evaluateJSScript: &*s];
            self.loader.ivars().entries.borrow_mut().remove(url);
            let Some(p) = p else { return Err(format!("{url}: not evaluated")) };
            let v: *const OpaqueJSValue = msg_send![&*p, JSValueRef];
            let v = v as *const c_void;
            let g: *mut OpaqueJSContext = msg_send![&*self.ctx, JSGlobalContextRef];
            JSValueProtect(g as *const c_void, v);
            Ok(v)
        }
    }
}

#[link(name = "JavaScriptCore", kind = "framework")]
unsafe extern "C" {
    fn JSValueProtect(ctx: *const c_void, v: *const c_void);
}
