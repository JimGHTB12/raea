// A piece of web.js's one function body (lib.rs joins the pieces inside `(function (g) { "use strict"; ... })(globalThis)`,
// so they share its helpers): Event, EventTarget, CustomEvent, MessageChannel, AbortController and AbortSignal.

// ---- Event / EventTarget / CustomEvent ----------------------------------------------------------------------------
class EventShim {
  constructor(type, init = {}) {
    if (arguments.length === 0) throw new TypeError("Event needs a type");
    this.type = String(type);
    this.bubbles = !!init.bubbles;
    this.cancelable = !!init.cancelable;
    this.composed = !!init.composed;
    this.defaultPrevented = false;
    this.timeStamp = typeof g.performance !== "undefined" ? g.performance.now() : Date.now();
    this.target = null;
    this.currentTarget = null;
    this.isTrusted = false;
    this._stop = false;
  }
  preventDefault() {
    if (this.cancelable) this.defaultPrevented = true;
  }
  stopPropagation() {}
  stopImmediatePropagation() {
    this._stop = true;
  }
}
def("Event", EventShim);
class CustomEventShim extends g.Event {
  constructor(type, init = {}) {
    super(type, init);
    this.detail = init.detail === undefined ? null : init.detail;
  }
}
def("CustomEvent", CustomEventShim);
const LISTENERS = Symbol("listeners");
class EventTargetShim {
  constructor() {
    Object.defineProperty(this, LISTENERS, { value: new Map() });
  }
  addEventListener(type, fn, opts) {
    if (!fn) return;
    const once = typeof opts === "object" && opts !== null && !!opts.once;
    const signal = typeof opts === "object" && opts !== null ? opts.signal : undefined;
    if (signal && signal.aborted) return;
    let list = this[LISTENERS].get(type);
    if (!list) this[LISTENERS].set(type, (list = []));
    if (list.some((l) => l.fn === fn)) return;
    const entry = { fn, once };
    list.push(entry);
    if (signal) signal.addEventListener("abort", () => this.removeEventListener(type, fn));
  }
  removeEventListener(type, fn) {
    const list = this[LISTENERS].get(type);
    if (!list) return;
    const i = list.findIndex((l) => l.fn === fn);
    if (i >= 0) list.splice(i, 1);
  }
  dispatchEvent(ev) {
    if (!(ev instanceof g.Event)) throw new TypeError("dispatchEvent takes an Event");
    try { Object.defineProperty(ev, "target", { value: this, configurable: true, writable: true }); } catch (_) {}
    try { Object.defineProperty(ev, "currentTarget", { value: this, configurable: true, writable: true }); } catch (_) {}
    const handler = this["on" + ev.type];
    const list = (this[LISTENERS].get(ev.type) || []).slice();
    if (typeof handler === "function") list.unshift({ fn: handler, once: false, prop: true });
    for (const l of list) {
      if (l.once) this.removeEventListener(ev.type, l.fn);
      try {
        if (typeof l.fn === "function") l.fn.call(this, ev);
        else if (l.fn && typeof l.fn.handleEvent === "function") l.fn.handleEvent(ev);
      } catch (e) {
        if (typeof g.reportError === "function") g.reportError(e);
        else Promise.reject(e);
      }
      if (ev._stop) break;
    }
    return !ev.defaultPrevented;
  }
}
def("EventTarget", EventTargetShim);

// MessageChannel: two linked ports, a message delivered to the other port as a macrotask (setTimeout 0), its data
// structured-cloned, through onmessage and "message" listeners. React's browser builds schedule work with it
// (react-dom/server.browser makes one as it loads; the bundles raea's bundler makes for the browser carry them).
class MessageEventShim extends g.Event {
  constructor(type, init = {}) {
    super(type, init);
    this.data = init.data;
    this.origin = "";
    this.lastEventId = "";
    this.source = null;
    this.ports = [];
  }
}
class MessagePortShim extends g.EventTarget {
  constructor() {
    super();
    this._other = null;
    this._closed = false;
    this.onmessage = null;
    this.onmessageerror = null;
  }
  postMessage(data) {
    const to = this._other;
    if (this._closed || !to || to._closed) return;
    const copy = typeof g.structuredClone === "function" ? g.structuredClone(data) : data;
    g.setTimeout(() => {
      if (to._closed) return;
      const ev = new MessageEventShim("message", { data: copy });
      if (typeof to.onmessage === "function") to.onmessage.call(to, ev);
      to.dispatchEvent(ev);
    }, 0);
  }
  start() {}
  close() { this._closed = true; }
}
class MessageChannelShim {
  constructor() {
    this.port1 = new MessagePortShim();
    this.port2 = new MessagePortShim();
    this.port1._other = this.port2;
    this.port2._other = this.port1;
  }
}
def("MessageEvent", MessageEventShim);
def("MessagePort", MessagePortShim);
def("MessageChannel", MessageChannelShim);

// ---- AbortController / AbortSignal --------------------------------------------------------------------------------
class AbortSignalShim extends g.EventTarget {
  constructor() {
    super();
    this.aborted = false;
    this.reason = undefined;
    this.onabort = null;
  }
  throwIfAborted() {
    if (this.aborted) throw this.reason;
  }
  _abort(reason) {
    if (this.aborted) return;
    this.aborted = true;
    this.reason = reason === undefined ? new DOMEx("This operation was aborted", "AbortError") : reason;
    this.dispatchEvent(new g.Event("abort"));
  }
  static abort(reason) {
    const s = new AbortSignalShim();
    s._abort(reason);
    return s;
  }
  static timeout(ms) {
    const s = new AbortSignalShim();
    g.setTimeout(() => s._abort(new DOMEx("The operation timed out.", "TimeoutError")), ms);
    return s;
  }
  static any(signals) {
    const s = new AbortSignalShim();
    for (const x of signals) {
      if (x.aborted) { s._abort(x.reason); return s; }
    }
    for (const x of signals) x.addEventListener("abort", () => s._abort(x.reason), { once: true });
    return s;
  }
}
def("AbortSignal", AbortSignalShim);
class AbortControllerShim {
  constructor() {
    this.signal = new g.AbortSignal();
  }
  abort(reason) {
    this.signal._abort(reason);
  }
}
def("AbortController", AbortControllerShim);
