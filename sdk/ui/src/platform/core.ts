// The global object's own pieces: the host's calls, structuredClone, the clipboard, and install.
import { call } from "../wire";
import { Blob_, DOMEx, DOMException_, EventTarget_, File_ } from "./all";

export const g = globalThis as any;

export const put = (k: string, v: unknown) => {
  if (g[k] === undefined) {
    try {
      Object.defineProperty(g, k, { value: v, writable: true, configurable: true, enumerable: false });
    } catch {
      g[k] = v;
    }
  }
};

export const named = (cls: any, name: string) => {
  try {
    Object.defineProperty(cls, "name", { value: name });
  } catch {}
  return cls;
};

export const tag = (cls: any, name: string) => {
  try {
    Object.defineProperty(cls.prototype, Symbol.toStringTag, { value: name, configurable: true });
  } catch {}
};

// ---------------------------------------------------------------- structuredClone

function cloneErr(what: string) {
  return new (DOMEx())(`Failed to execute 'structuredClone' on 'Window': ${what} could not be cloned.`, "DataCloneError");
}

export function structuredClone_(v: unknown, _o?: { transfer?: unknown[] }): any {
  const seen = new Map<object, unknown>();
  const go = (x: any): any => {
    if (x === null || (typeof x !== "object" && typeof x !== "function")) {
      if (typeof x === "symbol") throw cloneErr(String(x));
      return x;
    }
    if (typeof x === "function") throw cloneErr(String(x).length > 60 ? `${String(x).slice(0, 60)}…` : String(x));
    if (seen.has(x)) return seen.get(x);
    let out: any;
    const tagOf = Object.prototype.toString.call(x).slice(8, -1);
    if (x instanceof Boolean || x instanceof Number || x instanceof String) out = Object(x.valueOf());
    else if (tagOf === "BigInt") out = Object(x.valueOf());
    else if (x instanceof Date) out = new Date(x.getTime());
    else if (x instanceof RegExp) out = new RegExp(x.source, x.flags);
    else if (x instanceof ArrayBuffer) out = x.slice(0);
    else if (ArrayBuffer.isView(x)) {
      const buf = go(x.buffer);
      out = x instanceof DataView ? new DataView(buf, x.byteOffset, x.byteLength) : new (x.constructor as any)(buf, x.byteOffset, (x as any).length);
    } else if (x instanceof Map) {
      out = new Map();
      seen.set(x, out);
      for (const [k, val] of x) out.set(go(k), go(val));
      return out;
    } else if (x instanceof Set) {
      out = new Set();
      seen.set(x, out);
      for (const k of x) out.add(go(k));
      return out;
    } else if (x instanceof Error) {
      const ctors: Record<string, any> = { EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError };
      const C = ctors[x.name] ?? Error;
      out = new C(x.message);
      if ("cause" in x) out.cause = go((x as any).cause);
      if (x.stack !== undefined) out.stack = x.stack;
      if (x instanceof DOMException_) out = new (DOMEx())(x.message, x.name);
    } else if (x instanceof File_) out = new File_([x], x.name, { type: x.type, lastModified: x.lastModified });
    else if (x instanceof Blob_) out = x.slice(0, x.size, x.type);
    else if (Array.isArray(x)) {
      out = new Array(x.length);
      seen.set(x, out);
      for (const k of Object.keys(x)) out[k] = go(x[k]);
      return out;
    } else if (tagOf === "Object" || Object.getPrototypeOf(x) === null || isPlainish(x)) {
      out = {};
      seen.set(x, out);
      for (const k of Object.keys(x)) out[k] = go(x[k]);
      return out;
    } else throw cloneErr(tagOf === "Object" ? "#<Object>" : `#<${tagOf}>`);
    seen.set(x, out);
    return out;
  };
  return go(v);
}

function isPlainish(x: object) {
  // class instances clone as plain objects of their own enumerable properties (Chrome does), host objects don't
  const p = Object.getPrototypeOf(x);
  return !!p && typeof p.constructor === "function" && !/\[native code\]/.test(Function.prototype.toString.call(p.constructor)) && !(x instanceof EventTarget_) && !isPromise(x) && !(x instanceof WeakMap) && !(x instanceof WeakSet);
}

const isPromise = (x: any) => x instanceof Promise;

put("structuredClone", structuredClone_);

// ---------------------------------------------------------------- the clipboard (the host's, else this process's)

let clip = "";

export const clipboard = {
  writeText(t: string) {
    clip = String(t);
    return call("net.clipboard.write", { text: clip }).then(
      () => undefined,
      () => undefined,
    );
  },
  readText() {
    return call<string>("net.clipboard.read", {}).then(
      (t) => (typeof t === "string" ? t : clip),
      () => clip,
    );
  },
  write(items: { types: string[]; getType(t: string): Promise<Blob_> }[]) {
    const it = items[0];
    if (!it || !it.types.includes("text/plain")) return Promise.resolve();
    return it.getType("text/plain").then((b) => b.text()).then((t) => this.writeText(t));
  },
  read() {
    return this.readText().then((t) => [new ClipboardItem_({ "text/plain": new Blob_([t], { type: "text/plain" }) })]);
  },
};

class ClipboardItem_ {
  _d: Record<string, unknown>;
  presentationStyle = "unspecified";
  constructor(d: Record<string, unknown>) {
    this._d = d;
  }
  get types() {
    return Object.keys(this._d);
  }
  getType(t: string) {
    if (!(t in this._d)) return Promise.reject(new (DOMEx())("Failed to execute 'getType' on 'ClipboardItem': The type was not found", "NotFoundError"));
    return Promise.resolve(this._d[t]).then((v) => (v instanceof Blob_ ? v : new Blob_([String(v)], { type: t })));
  }
  static supports(t: string) {
    return ["text/plain", "text/html", "image/png"].includes(t);
  }
}

put("ClipboardItem", named(ClipboardItem_, "ClipboardItem"));

/** navigator.clipboard as this layer has it, onto the navigator the DOM layer installed (it puts its own over ours). */
export function fixNavigator() {
  const n = g.navigator;
  if (n && (!n.clipboard || n.clipboard.__stub || !n.clipboard.read)) {
    try {
      Object.defineProperty(n, "clipboard", { value: clipboard, configurable: true });
    } catch {
      n.clipboard = clipboard;
    }
  }
}

if (!g.navigator) put("navigator", { userAgent: "Mozilla/5.0 (raea)", language: "en-US", languages: ["en-US", "en"], onLine: true, clipboard });
