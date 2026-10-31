// A piece of web.js's one function body (lib.rs joins the pieces inside `(function (g) { "use strict"; ... })(globalThis)`,
// so they share its helpers): base64, DOMException, atob and btoa, TextEncoder and TextDecoder.

const call = (name, args) => {
  const out = g.__raea_call(name, JSON.stringify(args));
  return out === undefined || out === "" ? undefined : JSON.parse(out);
};
const def = (name, value) => {
  if (typeof g[name] === "undefined") Object.defineProperty(g, name, { value, writable: true, configurable: true, enumerable: false });
};

// ---- base64 -----------------------------------------------------------------------------------------------------
const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const B64I = new Int16Array(128).fill(-1);
for (let i = 0; i < 64; i++) B64I[B64.charCodeAt(i)] = i;
function bytesTo64(b) {
  let s = "";
  let i = 0;
  for (; i + 2 < b.length; i += 3) {
    const n = (b[i] << 16) | (b[i + 1] << 8) | b[i + 2];
    s += B64[n >> 18] + B64[(n >> 12) & 63] + B64[(n >> 6) & 63] + B64[n & 63];
  }
  if (i < b.length) {
    const n = (b[i] << 16) | ((i + 1 < b.length ? b[i + 1] : 0) << 8);
    s += B64[n >> 18] + B64[(n >> 12) & 63] + (i + 1 < b.length ? B64[(n >> 6) & 63] : "=") + "=";
  }
  return s;
}
function bytesFrom64(s) {
  const clean = String(s).replace(/[\t\n\f\r ]/g, "").replace(/=+$/, "");
  if (clean.length % 4 === 1 || /[^A-Za-z0-9+/]/.test(clean)) throw new DOMExceptionShim("The string to be decoded is not correctly encoded.", "InvalidCharacterError");
  const out = new Uint8Array(Math.floor((clean.length * 3) / 4));
  let o = 0;
  for (let i = 0; i < clean.length; i += 4) {
    const a = B64I[clean.charCodeAt(i)], b = B64I[clean.charCodeAt(i + 1)];
    const c = i + 2 < clean.length ? B64I[clean.charCodeAt(i + 2)] : 0;
    const d = i + 3 < clean.length ? B64I[clean.charCodeAt(i + 3)] : 0;
    const n = (a << 18) | (b << 12) | (c << 6) | d;
    out[o++] = n >> 16;
    if (i + 2 < clean.length) out[o++] = (n >> 8) & 255;
    if (i + 3 < clean.length) out[o++] = n & 255;
  }
  return out.subarray(0, o);
}
const bytesOf = (src) => {
  if (src instanceof Uint8Array) return src;
  if (ArrayBuffer.isView(src)) return new Uint8Array(src.buffer, src.byteOffset, src.byteLength);
  if (src instanceof ArrayBuffer || (typeof SharedArrayBuffer !== "undefined" && src instanceof SharedArrayBuffer)) return new Uint8Array(src);
  throw new TypeError("expected an ArrayBuffer or a view of one");
};

// ---- DOMException -----------------------------------------------------------------------------------------------
const CODES = { IndexSizeError: 1, NotFoundError: 8, NotSupportedError: 9, InvalidStateError: 11, SyntaxError: 12, InvalidCharacterError: 5, NetworkError: 19, AbortError: 20, TimeoutError: 23, DataCloneError: 25 };
class DOMExceptionShim extends Error {
  constructor(message = "", name = "Error") {
    super(message);
    Object.defineProperty(this, "name", { value: name, configurable: true, writable: true });
  }
  get code() {
    return CODES[this.name] || 0;
  }
}
def("DOMException", DOMExceptionShim);
const DOMEx = g.DOMException;

// ---- atob / btoa ------------------------------------------------------------------------------------------------
def("btoa", (s) => {
  s = String(s);
  const b = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    if (c > 255) throw new DOMEx("The string to be encoded contains characters outside of the Latin1 range.", "InvalidCharacterError");
    b[i] = c;
  }
  return bytesTo64(b);
});
def("atob", (s) => {
  const b = bytesFrom64(s);
  let out = "";
  for (let i = 0; i < b.length; i += 8192) out += String.fromCharCode.apply(null, b.subarray(i, i + 8192));
  return out;
});

// ---- TextEncoder / TextDecoder (UTF-8; TextDecoder also takes latin1/ascii and utf-16le) ---------------------------
class TextEncoderShim {
  get encoding() {
    return "utf-8";
  }
  encode(input = "") {
    const s = String(input);
    const out = new Uint8Array(s.length * 3);
    return out.slice(0, this._into(s, out).written);
  }
  encodeInto(input, dest) {
    return this._into(String(input), dest);
  }
  _into(s, out) {
    let o = 0, i = 0;
    for (; i < s.length; i++) {
      let c = s.charCodeAt(i);
      if (c >= 0xd800 && c <= 0xdbff && i + 1 < s.length) {
        const d = s.charCodeAt(i + 1);
        if (d >= 0xdc00 && d <= 0xdfff) {
          c = 0x10000 + ((c - 0xd800) << 10) + (d - 0xdc00);
          if (o + 4 > out.length) break;
          out[o++] = 0xf0 | (c >> 18);
          out[o++] = 0x80 | ((c >> 12) & 63);
          out[o++] = 0x80 | ((c >> 6) & 63);
          out[o++] = 0x80 | (c & 63);
          i++;
          continue;
        }
      }
      if (c >= 0xd800 && c <= 0xdfff) c = 0xfffd; // a lone surrogate
      if (c < 0x80) {
        if (o + 1 > out.length) break;
        out[o++] = c;
      } else if (c < 0x800) {
        if (o + 2 > out.length) break;
        out[o++] = 0xc0 | (c >> 6);
        out[o++] = 0x80 | (c & 63);
      } else {
        if (o + 3 > out.length) break;
        out[o++] = 0xe0 | (c >> 12);
        out[o++] = 0x80 | ((c >> 6) & 63);
        out[o++] = 0x80 | (c & 63);
      }
    }
    return { read: i, written: o };
  }
}
def("TextEncoder", TextEncoderShim);

const LABELS = { "utf-8": "utf-8", utf8: "utf-8", "unicode-1-1-utf-8": "utf-8", "utf-16le": "utf-16le", "utf-16": "utf-16le", latin1: "windows-1252", "iso-8859-1": "windows-1252", ascii: "windows-1252", "us-ascii": "windows-1252", "windows-1252": "windows-1252" };
class TextDecoderShim {
  constructor(label = "utf-8", options = {}) {
    const enc = LABELS[String(label).trim().toLowerCase()];
    if (!enc) throw new RangeError(`The encoding label provided ('${label}') is invalid.`);
    this._enc = enc;
    this._fatal = !!options.fatal;
    this._ignoreBOM = !!options.ignoreBOM;
    this._pending = new Uint8Array(0);
    this._bomSeen = false;
  }
  get encoding() {
    return this._enc;
  }
  get fatal() {
    return this._fatal;
  }
  get ignoreBOM() {
    return this._ignoreBOM;
  }
  decode(input, options = {}) {
    let b = input === undefined ? new Uint8Array(0) : bytesOf(input);
    if (this._pending.length) {
      const j = new Uint8Array(this._pending.length + b.length);
      j.set(this._pending);
      j.set(b, this._pending.length);
      b = j;
      this._pending = new Uint8Array(0);
    }
    const stream = !!options.stream;
    let out;
    if (this._enc === "utf-8") out = this._utf8(b, stream);
    else if (this._enc === "utf-16le") out = this._utf16(b, stream);
    else {
      out = "";
      for (let i = 0; i < b.length; i += 8192) out += String.fromCharCode.apply(null, b.subarray(i, i + 8192));
    }
    if (!this._ignoreBOM && !this._bomSeen && out.length) {
      if (out.charCodeAt(0) === 0xfeff && this._enc !== "windows-1252") out = out.slice(1);
      this._bomSeen = true;
    }
    if (!stream) this._bomSeen = false;
    return out;
  }
  _bad() {
    if (this._fatal) throw new TypeError("The encoded data was not valid for encoding utf-8");
    return 0xfffd;
  }
  _utf8(b, stream) {
    const parts = [];
    let codes = [];
    let i = 0;
    const n = b.length;
    while (i < n) {
      const c = b[i];
      let cp, len;
      if (c < 0x80) { cp = c; len = 1; }
      else if (c >= 0xc2 && c < 0xe0) { cp = c & 31; len = 2; }
      else if (c >= 0xe0 && c < 0xf0) { cp = c & 15; len = 3; }
      else if (c >= 0xf0 && c < 0xf5) { cp = c & 7; len = 4; }
      else { codes.push(this._bad()); i++; continue; }
      if (i + len > n) {
        // an incomplete sequence at the end: keep it for the next chunk when streaming
        let ok = true;
        for (let k = 1; k < n - i; k++) if ((b[i + k] & 0xc0) !== 0x80) ok = false;
        if (stream && ok) { this._pending = b.slice(i); break; }
        codes.push(this._bad());
        i++;
        while (i < n && (b[i] & 0xc0) === 0x80) i++;
        continue;
      }
      let bad = false;
      for (let k = 1; k < len; k++) {
        const d = b[i + k];
        if ((d & 0xc0) !== 0x80) { bad = true; len = k; break; }
        cp = (cp << 6) | (d & 63);
      }
      if (!bad && ((len === 3 && (cp < 0x800 || (cp >= 0xd800 && cp <= 0xdfff))) || (len === 4 && (cp < 0x10000 || cp > 0x10ffff)))) bad = true;
      if (bad) { codes.push(this._bad()); i += Math.max(1, len === 1 ? 1 : len); continue; }
      if (cp > 0xffff) { cp -= 0x10000; codes.push(0xd800 + (cp >> 10), 0xdc00 + (cp & 1023)); }
      else codes.push(cp);
      i += len;
      if (codes.length > 8192) { parts.push(String.fromCharCode.apply(null, codes)); codes = []; }
    }
    parts.push(String.fromCharCode.apply(null, codes));
    return parts.join("");
  }
  _utf16(b, stream) {
    const even = b.length - (b.length % 2);
    if (b.length % 2) {
      if (stream) this._pending = b.slice(even);
    }
    let out = "";
    const codes = [];
    for (let i = 0; i < even; i += 2) {
      codes.push(b[i] | (b[i + 1] << 8));
      if (codes.length > 8192) { out += String.fromCharCode.apply(null, codes); codes.length = 0; }
    }
    out += String.fromCharCode.apply(null, codes);
    if (b.length % 2 && !stream) out += String.fromCharCode(this._bad());
    return out;
  }
}
def("TextDecoder", TextDecoderShim);
