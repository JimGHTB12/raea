// A piece of web.js's one function body (lib.rs joins the pieces inside `(function (g) { "use strict"; ... })(globalThis)`,
// so they share its helpers): Buffer, WritableStream, and the small globals packages probe for.

// ---- Buffer (the part packages use: bytes from and to strings) ------------------------------------------------------
if (typeof g.Buffer === "undefined") {
  class BufferShim extends Uint8Array {
    static from(v, enc) {
      if (typeof v === "string") {
        const e = (enc || "utf8").toLowerCase();
        let b;
        if (e === "base64" || e === "base64url") b = bytesFrom64(e === "base64url" ? v.replace(/-/g, "+").replace(/_/g, "/") : v);
        else if (e === "hex") {
          b = new Uint8Array(v.length >> 1);
          for (let i = 0; i < b.length; i++) b[i] = parseInt(v.substr(i * 2, 2), 16);
        } else if (e === "latin1" || e === "binary" || e === "ascii") {
          b = new Uint8Array(v.length);
          for (let i = 0; i < v.length; i++) b[i] = v.charCodeAt(i) & 255;
        } else b = new g.TextEncoder().encode(v);
        const out = new BufferShim(b.length);
        out.set(b);
        return out;
      }
      if (v instanceof ArrayBuffer) return new BufferShim(v);
      const src = ArrayBuffer.isView(v) ? new Uint8Array(v.buffer, v.byteOffset, v.byteLength) : Uint8Array.from(v);
      const out = new BufferShim(src.length);
      out.set(src);
      return out;
    }
    static alloc(n, fill = 0) {
      return new BufferShim(n).fill(typeof fill === "number" ? fill : 0);
    }
    static isBuffer(b) {
      return b instanceof BufferShim;
    }
    static byteLength(s, enc) {
      return typeof s === "string" ? BufferShim.from(s, enc).length : s.byteLength;
    }
    static concat(list, total) {
      const n = total === undefined ? list.reduce((a, b) => a + b.length, 0) : total;
      const out = new BufferShim(n);
      let o = 0;
      for (const b of list) {
        out.set(b.subarray(0, Math.max(0, Math.min(b.length, n - o))), o);
        o += b.length;
        if (o >= n) break;
      }
      return out;
    }
    toString(enc, start = 0, end = this.length) {
      const b = this.subarray(start, end);
      const e = (enc || "utf8").toLowerCase();
      if (e === "base64") return bytesTo64(b);
      if (e === "base64url") return bytesTo64(b).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
      if (e === "hex") return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
      if (e === "latin1" || e === "binary" || e === "ascii") {
        let s = "";
        for (let i = 0; i < b.length; i += 8192) s += String.fromCharCode.apply(null, b.subarray(i, i + 8192));
        return s;
      }
      return new g.TextDecoder().decode(b);
    }
    equals(o) {
      return this.length === o.length && this.every((x, i) => x === o[i]);
    }
  }
  def("Buffer", BufferShim);
}

// ---- WritableStream (the part a writer to a sink needs: write, close, abort, ready; writes go in order) ------------
class WritableStreamShim {
  constructor(sink = {}) {
    this._sink = sink;
    this._locked = false;
    this._chain = Promise.resolve();
    this._closed = false;
    if (typeof sink.start === "function") this._chain = Promise.resolve(sink.start(this));
  }
  get locked() {
    return this._locked;
  }
  _run(fn) {
    const p = this._chain.then(fn);
    this._chain = p.catch(() => {});
    return p;
  }
  getWriter() {
    if (this._locked) throw new TypeError("WritableStream is locked");
    this._locked = true;
    const st = this;
    return {
      get desiredSize() {
        return st._closed ? 0 : 1;
      },
      get ready() {
        return st._chain;
      },
      get closed() {
        return st._chain;
      },
      write(chunk) {
        if (st._closed) return Promise.reject(new TypeError("the stream is closed"));
        return st._run(() => (typeof st._sink.write === "function" ? st._sink.write(chunk, st) : undefined));
      },
      close() {
        st._closed = true;
        return st._run(() => (typeof st._sink.close === "function" ? st._sink.close() : undefined));
      },
      abort(reason) {
        st._closed = true;
        return st._run(() => (typeof st._sink.abort === "function" ? st._sink.abort(reason) : undefined));
      },
      releaseLock() {
        st._locked = false;
      },
    };
  }
  close() {
    return this.getWriter().close();
  }
  abort(reason) {
    return this.getWriter().abort(reason);
  }
}
def("WritableStream", WritableStreamShim);

// ---- small globals packages probe for -------------------------------------------------------------------------------
def("globalThis", g);
def("self", g);
if (typeof g.reportError === "undefined") def("reportError", (e) => { if (g.console && g.console.error) g.console.error(e && e.stack ? e.stack : String(e)); });
