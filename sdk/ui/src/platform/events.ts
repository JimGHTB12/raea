// DOMException, Event and EventTarget, reportError and the console, timers' arguments, AbortController.
import { on } from "../wire";
import { g, named, put, tag } from "./all";

// ---------------------------------------------------------------- DOMException

const CODES: Record<string, number> = {
  IndexSizeError: 1, HierarchyRequestError: 3, WrongDocumentError: 4, InvalidCharacterError: 5, NoModificationAllowedError: 7, NotFoundError: 8,
  NotSupportedError: 9, InvalidStateError: 11, SyntaxError: 12, InvalidModificationError: 13, NamespaceError: 14, InvalidAccessError: 15,
  TypeMismatchError: 17, SecurityError: 18, NetworkError: 19, AbortError: 20, URLMismatchError: 21, QuotaExceededError: 22, TimeoutError: 23,
  InvalidNodeTypeError: 24, DataCloneError: 25,
};

export class DOMException_ extends Error {
  readonly code: number;
  constructor(message = "", name = "Error") {
    super(String(message));
    Object.defineProperty(this, "name", { value: String(name), enumerable: false, configurable: true, writable: true });
    this.code = CODES[name] ?? 0;
  }
}

for (const [k, v] of Object.entries(CODES)) {
  const c = k.replace(/Error$/, "").replace(/([a-z])([A-Z])/g, "$1_$2").toUpperCase() + "_ERR";
  (DOMException_ as any)[c] = v;
}

named(DOMException_, "DOMException");

put("DOMException", DOMException_);

export const DOMEx = (): typeof DOMException_ => g.DOMException;

// ---------------------------------------------------------------- Event and EventTarget (the DOM layer's replace them)

class Event_ {
  type: string;
  bubbles: boolean;
  cancelable: boolean;
  composed: boolean;
  defaultPrevented = false;
  timeStamp = g.performance?.now?.() ?? 0;
  target: any = null;
  currentTarget: any = null;
  eventPhase = 0;
  isTrusted = false;
  _stop = false;
  _stopNow = false;
  constructor(type: string, init: { bubbles?: boolean; cancelable?: boolean; composed?: boolean } = {}) {
    if (arguments.length === 0) throw new TypeError("Failed to construct 'Event': 1 argument required, but only 0 present.");
    this.type = String(type);
    this.bubbles = !!init.bubbles;
    this.cancelable = !!init.cancelable;
    this.composed = !!init.composed;
  }
  preventDefault() {
    if (this.cancelable) this.defaultPrevented = true;
  }
  stopPropagation() {
    this._stop = true;
  }
  stopImmediatePropagation() {
    this._stop = this._stopNow = true;
  }
  composedPath() {
    return this.currentTarget ? [this.currentTarget] : [];
  }
}

export class EventTarget_ {
  _ls?: Map<string, { fn: any; once: boolean }[]>;
  addEventListener(type: string, fn: any, o?: boolean | { once?: boolean; signal?: any }) {
    if (!fn) return;
    const ls = (this._ls ??= new Map());
    const l = ls.get(type) ?? [];
    if (l.some((x) => x.fn === fn)) return;
    const once = typeof o === "object" && !!o?.once;
    const sig = typeof o === "object" ? o?.signal : undefined;
    if (sig?.aborted) return;
    l.push({ fn, once });
    ls.set(type, l);
    sig?.addEventListener?.("abort", () => this.removeEventListener(type, fn));
  }
  removeEventListener(type: string, fn: any) {
    const l = this._ls?.get(type);
    if (l) this._ls!.set(type, l.filter((x) => x.fn !== fn));
  }
  dispatchEvent(ev: any) {
    ev.target ??= this;
    ev.currentTarget = this;
    ev.eventPhase = 2;
    const on = (this as any)["on" + ev.type];
    const l = (this._ls?.get(ev.type) ?? []).slice();
    for (const x of l) {
      if (x.once) this.removeEventListener(ev.type, x.fn);
      try {
        typeof x.fn === "function" ? x.fn.call(this, ev) : x.fn.handleEvent(ev);
      } catch (e) {
        reportError_(e);
      }
      if (ev._stopNow) break;
    }
    if (typeof on === "function" && !ev._stopNow) {
      try {
        on.call(this, ev);
      } catch (e) {
        reportError_(e);
      }
    }
    ev.currentTarget = null;
    ev.eventPhase = 0;
    return !ev.defaultPrevented;
  }
}

put("Event", named(Event_, "Event"));

put("EventTarget", named(EventTarget_, "EventTarget"));

put(
  "CustomEvent",
  named(
    class CustomEvent extends (g.Event as typeof Event_) {
      detail: unknown;
      constructor(type: string, init: { detail?: unknown; bubbles?: boolean; cancelable?: boolean } = {}) {
        super(type, init);
        this.detail = init.detail ?? null;
      }
    },
    "CustomEvent",
  ),
);

/** An event of `type` with `fields`, made from whatever Event class is global when it fires (the DOM layer's, once it
 *  installed), so listeners see one kind of event. */
export function event(type: string, fields: Record<string, unknown> = {}, init: Record<string, unknown> = {}) {
  const ev = new g.Event(type, init);
  for (const k in fields) {
    try {
      Object.defineProperty(ev, k, { value: fields[k], configurable: true, enumerable: true });
    } catch {}
  }
  return ev;
}

/** A target for the objects here (an AbortSignal, a WebSocket): the global EventTarget when it fires (the DOM layer
 *  installs its own after this module ran), so `instanceof EventTarget` holds either way. */
function Target(): typeof EventTarget_ {
  return EventTarget_;
}

// ---------------------------------------------------------------- reportError, the console's rest

function reportError_(e: unknown) {
  let prevented = false;
  try {
    if (typeof g.dispatchEvent === "function") {
      const ev = event("error", { message: "Uncaught " + (e instanceof Error ? `${e.name}: ${e.message}` : String(e)), error: e, filename: "", lineno: 0, colno: 0 }, { cancelable: true });
      prevented = !g.dispatchEvent(ev);
    }
  } catch {}
  if (!prevented) g.console?.error?.(`Uncaught ${(e as Error)?.stack ?? e}`);
}

put("reportError", reportError_);

if (g.console) {
  const c = g.console;
  const counts = new Map<string, number>();
  const timers = new Map<string, number>();
  let depth = 0;
  const now = () => g.performance?.now?.() ?? Date.now();
  const ind = (f: (...a: unknown[]) => void) => (...a: unknown[]) => f(...(depth ? ["  ".repeat(depth), ...a] : a));
  c.debug ??= () => {};
  c.trace ??= (...a: unknown[]) => c.log("Trace:", ...a, new Error().stack ?? "");
  c.assert ??= (ok: unknown, ...a: unknown[]) => void (!ok && c.error("Assertion failed:", ...a));
  c.count ??= (l = "default") => {
    const n = (counts.get(l) ?? 0) + 1;
    counts.set(l, n);
    c.info(`${l}: ${n}`);
  };
  c.countReset ??= (l = "default") => void counts.set(l, 0);
  c.time ??= (l = "default") => void timers.set(l, now());
  c.timeLog ??= (l = "default", ...a: unknown[]) => c.info(`${l}: ${(now() - (timers.get(l) ?? now())).toFixed(3)} ms`, ...a);
  c.timeEnd ??= (l = "default") => {
    c.timeLog(l);
    timers.delete(l);
  };
  c.group ??= (...a: unknown[]) => {
    if (a.length) c.log(...a);
    depth++;
  };
  c.groupCollapsed ??= c.group;
  c.groupEnd ??= () => void (depth = Math.max(0, depth - 1));
  c.table ??= (d: unknown) => c.log(JSON.stringify(d));
  c.dir ??= (d: unknown) => c.log(d);
  c.dirxml ??= c.dir;
  c.clear ??= () => {};
  if (!c.__indented) {
    c.log = ind(c.log);
    c.__indented = true;
  }
}

// ---------------------------------------------------------------- setInterval's arguments, setTimeout's strings

if (g.setInterval && !g.setInterval.__args) {
  const si = g.setInterval;
  const st = g.setTimeout;
  g.setInterval = Object.assign((fn: any, ms?: number, ...args: unknown[]) => si(typeof fn === "function" ? () => fn(...args) : () => (0, eval)(String(fn)), Number(ms) || 0), { __args: true });
  g.setTimeout = Object.assign((fn: any, ms?: number, ...args: unknown[]) => st(typeof fn === "function" ? fn : () => (0, eval)(String(fn)), Number(ms) || 0, ...args), { __args: true });
}

// ---------------------------------------------------------------- AbortController and AbortSignal

export class AbortSignal_ extends EventTarget_ {
  aborted = false;
  reason: unknown = undefined;
  onabort: ((e: unknown) => void) | null = null;
  throwIfAborted() {
    if (this.aborted) throw this.reason;
  }
  _abort(reason: unknown) {
    if (this.aborted) return;
    this.aborted = true;
    this.reason = reason === undefined ? new (DOMEx())("signal is aborted without reason", "AbortError") : reason;
    this.dispatchEvent(event("abort"));
  }
  static abort(reason?: unknown) {
    const s = new AbortSignal_();
    s._abort(reason);
    return s;
  }
  static timeout(ms: number) {
    const s = new AbortSignal_();
    g.setTimeout(() => s._abort(new (DOMEx())("signal timed out", "TimeoutError")), ms);
    return s;
  }
  static any(signals: Iterable<AbortSignal_>) {
    const s = new AbortSignal_();
    for (const x of signals) {
      if (x.aborted) {
        s._abort(x.reason);
        return s;
      }
    }
    for (const x of signals) x.addEventListener("abort", () => s._abort(x.reason), { once: true });
    return s;
  }
}

class AbortController_ {
  readonly signal = new AbortSignal_();
  abort(reason?: unknown) {
    this.signal._abort(reason);
  }
}

put("AbortSignal", named(AbortSignal_, "AbortSignal"));

put("AbortController", named(AbortController_, "AbortController"));

tag(AbortSignal_, "AbortSignal");

tag(AbortController_, "AbortController");
