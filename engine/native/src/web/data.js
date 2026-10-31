// A piece of web.js's one function body (lib.rs joins the pieces inside `(function (g) { "use strict"; ... })(globalThis)`,
// so they share its helpers): structuredClone, crypto, Blob and Headers.

// ---- structuredClone ----------------------------------------------------------------------------------------------
def("structuredClone", (value) => {
  const seen = new Map();
  const clone = (v) => {
    if (v === null || typeof v !== "object") {
      if (typeof v === "function" || typeof v === "symbol") throw new DOMEx(`${String(v)} could not be cloned.`, "DataCloneError");
      return v;
    }
    if (seen.has(v)) return seen.get(v);
    let out;
    if (v instanceof Date) out = new Date(v.getTime());
    else if (v instanceof RegExp) out = new RegExp(v.source, v.flags);
    else if (v instanceof ArrayBuffer) out = v.slice(0);
    else if (ArrayBuffer.isView(v)) {
      const buf = clone(v.buffer);
      out = v instanceof DataView ? new DataView(buf, v.byteOffset, v.byteLength) : new v.constructor(buf, v.byteOffset, v.length);
    } else if (v instanceof Map) {
      out = new Map();
      seen.set(v, out);
      for (const [k, x] of v) out.set(clone(k), clone(x));
      return out;
    } else if (v instanceof Set) {
      out = new Set();
      seen.set(v, out);
      for (const x of v) out.add(clone(x));
      return out;
    } else if (v instanceof Error) {
      out = new v.constructor(v.message);
      seen.set(v, out);
      if (v.stack) out.stack = v.stack;
      return out;
    } else if (Array.isArray(v)) {
      out = new Array(v.length);
      seen.set(v, out);
      for (let i = 0; i < v.length; i++) out[i] = clone(v[i]);
      return out;
    } else {
      out = {};
      seen.set(v, out);
      for (const k of Object.keys(v)) out[k] = clone(v[k]);
      return out;
    }
    seen.set(v, out);
    return out;
  };
  return clone(value);
});

// ---- crypto -------------------------------------------------------------------------------------------------------
if (typeof g.crypto === "undefined") {
  const getRandomValues = (arr) => {
    if (!ArrayBuffer.isView(arr) || arr instanceof Float32Array || arr instanceof Float64Array) throw new TypeError("getRandomValues takes an integer typed array");
    const b = bytesFrom64(call("web.random", { n: arr.byteLength }));
    new Uint8Array(arr.buffer, arr.byteOffset, arr.byteLength).set(b);
    return arr;
  };
  const randomUUID = () => {
    const b = getRandomValues(new Uint8Array(16));
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    const h = Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
    return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
  };
  const subtle = {
    digest(algo, data) {
      try {
        const name = typeof algo === "string" ? algo : algo.name;
        const out = bytesFrom64(call("web.digest", { algo: name, data: bytesTo64(bytesOf(data)) }));
        return Promise.resolve(out.buffer.slice(out.byteOffset, out.byteOffset + out.byteLength));
      } catch (e) {
        return Promise.reject(e);
      }
    },
  };
  def("crypto", { getRandomValues, randomUUID, subtle });
}

// ---- Blob ---------------------------------------------------------------------------------------------------------
class BlobShim {
  constructor(parts = [], options = {}) {
    const chunks = [];
    for (const p of parts) {
      if (p instanceof BlobShim) chunks.push(p._bytes);
      else if (typeof p === "string") chunks.push(new g.TextEncoder().encode(p));
      else chunks.push(new Uint8Array(bytesOf(p)));
    }
    const n = chunks.reduce((a, c) => a + c.length, 0);
    this._bytes = new Uint8Array(n);
    let o = 0;
    for (const c of chunks) {
      this._bytes.set(c, o);
      o += c.length;
    }
    this.type = options.type ? String(options.type).toLowerCase() : "";
  }
  get size() {
    return this._bytes.length;
  }
  slice(start = 0, end = this.size, type = "") {
    const b = new BlobShim([], { type });
    b._bytes = this._bytes.slice(start, end);
    return b;
  }
  arrayBuffer() {
    return Promise.resolve(this._bytes.slice().buffer);
  }
  bytes() {
    return Promise.resolve(this._bytes.slice());
  }
  text() {
    return Promise.resolve(new g.TextDecoder().decode(this._bytes));
  }
  // its bytes as a stream of one chunk, where the host has ReadableStream (the page engine's web layer: jetway #1)
  stream() {
    const bytes = this._bytes;
    return new g.ReadableStream({ start(c) { c.enqueue(bytes.slice()); c.close(); } });
  }
}
def("Blob", BlobShim);

// ---- Headers --------------------------------------------------------------------------------------------------------
const normName = (n) => {
  n = String(n);
  if (!/^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/.test(n)) throw new TypeError(`Invalid header name: ${n}`);
  return n.toLowerCase();
};
class HeadersShim {
  constructor(init) {
    this._m = new Map();
    if (init instanceof HeadersShim) init.forEach((v, k) => this.append(k, v));
    else if (Array.isArray(init) || (init && typeof init[Symbol.iterator] === "function")) for (const [k, v] of init) this.append(k, v);
    else if (init && typeof init === "object") for (const k of Object.keys(init)) this.append(k, init[k]);
  }
  append(k, v) {
    k = normName(k);
    v = String(v).trim();
    this._m.set(k, this._m.has(k) ? (k === "set-cookie" ? this._m.get(k) + "\n" + v : this._m.get(k) + ", " + v) : v);
  }
  set(k, v) {
    this._m.set(normName(k), String(v).trim());
  }
  get(k) {
    const v = this._m.get(normName(k));
    return v === undefined ? null : v;
  }
  has(k) {
    return this._m.has(normName(k));
  }
  delete(k) {
    this._m.delete(normName(k));
  }
  getSetCookie() {
    const v = this._m.get("set-cookie");
    return v === undefined ? [] : v.split("\n");
  }
  forEach(fn, self) {
    for (const [k, v] of this.entries()) fn.call(self, v, k, this);
  }
  keys() {
    return [...this._m.keys()].sort()[Symbol.iterator]();
  }
  values() {
    return [...this.entries()].map((e) => e[1])[Symbol.iterator]();
  }
  entries() {
    return [...this._m.keys()].sort().map((k) => [k, this._m.get(k)])[Symbol.iterator]();
  }
  [Symbol.iterator]() {
    return this.entries();
  }
}
def("Headers", HeadersShim);
