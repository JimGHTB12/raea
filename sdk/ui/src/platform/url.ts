// URL and URLSearchParams (WHATWG, the special schemes).
import { Blob_, at, named, origin, put, tag, utf8, uuid } from "./all";

// ---------------------------------------------------------------- URL and URLSearchParams (WHATWG, the special schemes)

const SPECIAL: Record<string, string> = { "http:": "80", "https:": "443", "ws:": "80", "wss:": "443", "ftp:": "21", "file:": "" };

const hex = (b: number) => "%" + b.toString(16).toUpperCase().padStart(2, "0");

/** percent-encodes the code points of `s` in `set` (and every non-ASCII one), as the URL spec's encode sets do */
function pct(s: string, set: (c: number) => boolean) {
  let out = "";
  for (const ch of s) {
    const c = ch.codePointAt(0)!;
    if (c < 0x20 || c > 0x7e || set(c)) for (const b of utf8(ch)) out += hex(b);
    else out += ch;
  }
  return out;
}

const C0 = (_c: number) => false;

const FRAG = (c: number) => c === 0x20 || c === 0x22 || c === 0x3c || c === 0x3e || c === 0x60;

const QUERY = (c: number) => c === 0x20 || c === 0x22 || c === 0x23 || c === 0x3c || c === 0x3e;

const SQUERY = (c: number) => QUERY(c) || c === 0x27;

const PATH = (c: number) => QUERY(c) || c === 0x3f || c === 0x5e || c === 0x60 || c === 0x7b || c === 0x7d;

const USER = (c: number) => PATH(c) || c === 0x2f || c === 0x3a || c === 0x3b || c === 0x3d || c === 0x40 || (c >= 0x5b && c <= 0x5d) || c === 0x7c;

const FORM = (c: number) => !(/[A-Za-z0-9*\-._]/.test(String.fromCharCode(c)));

type Parts = { protocol: string; username: string; password: string; hostname: string; port: string; path: string[] | string; search: string; hash: string };

function parseHost(h: string, special: boolean): string {
  if (h.startsWith("[")) {
    if (!h.endsWith("]")) throw 0;
    return h.toLowerCase();
  }
  if (!special) return pct(h, (c) => c === 0x20 || c === 0x22 || c === 0x3c || c === 0x3e || c === 0x60);
  let d = h;
  try {
    d = decodeURIComponent(h);
  } catch {}
  d = d.toLowerCase();
  // Chrome keeps a space in a host, percent-encoded (the URL standard refuses it)
  if (d === "" || /[\x00-\x1f#%/:<>?@[\\\]^|]/.test(d)) throw 0;
  if (d.includes(" ")) return d.replace(/ /g, "%20");
  // IPv4 in its forms (a.b.c.d, numbers, hex)
  const parts = d.split(".");
  if (parts[parts.length - 1] === "") parts.pop();
  const num = (p: string) => (/^0x[0-9a-f]*$/i.test(p) ? parseInt(p.slice(2) || "0", 16) : /^0[0-7]*$/.test(p) && p.length > 1 ? parseInt(p, 8) : /^\d+$/.test(p) ? parseInt(p, 10) : NaN);
  if (parts.length && parts.length <= 4 && /^(0x[0-9a-f]*|\d+)$/i.test(parts[parts.length - 1])) {
    const ns = parts.map(num);
    if (ns.some((n) => isNaN(n))) throw 0;
    let v = ns[ns.length - 1];
    if (v >= 256 ** (5 - ns.length)) throw 0;
    for (let i = 0; i < ns.length - 1; i++) {
      if (ns[i] > 255) throw 0;
      v += ns[i] * 256 ** (3 - i);
    }
    return [v >>> 24, (v >>> 16) & 255, (v >>> 8) & 255, v & 255].join(".");
  }
  return d;
}

function normPath(segs: string[], special: boolean): string[] {
  const out: string[] = [];
  for (let i = 0; i < segs.length; i++) {
    const raw = segs[i];
    const s = raw.toLowerCase();
    const last = i === segs.length - 1;
    if (s === ".." || s === ".%2e" || s === "%2e." || s === "%2e%2e") {
      out.pop();
      if (last) out.push("");
    } else if (s === "." || s === "%2e") {
      if (last) out.push("");
    } else out.push(pct(raw, PATH));
  }
  return special || out.length ? out : out;
}

function parseURL(input: string, base?: Parts): Parts {
  let s = input.replace(/^[\x00-\x20]+|[\x00-\x20]+$/g, "").replace(/[\t\n\r]/g, "");
  const m = /^([a-zA-Z][a-zA-Z0-9+.-]*):(.*)$/s.exec(s);
  let hash = "";
  let search = "";
  const cut = (x: string) => {
    const h = x.indexOf("#");
    if (h >= 0) {
      hash = "#" + pct(x.slice(h + 1), FRAG);
      x = x.slice(0, h);
    }
    const q = x.indexOf("?");
    if (q >= 0) {
      search = "?" + x.slice(q + 1);
      x = x.slice(0, q);
    }
    return x;
  };
  const enc = (sp: boolean) => (search ? "?" + pct(search.slice(1), sp ? SQUERY : QUERY) : "");
  if (m) {
    const protocol = m[1].toLowerCase() + ":";
    let rest = m[2];
    const special = protocol in SPECIAL;
    if (special && base && base.protocol === protocol && !/^[/\\]/.test(rest)) return parseURL(rest, base);
    if (special || rest.startsWith("//")) {
      if (special) rest = rest.replace(/\\/g, "/");
      if (protocol === "file:") {
        rest = cut(rest);
        let host = "";
        let p = rest;
        if (p.startsWith("//")) {
          const e = p.indexOf("/", 2);
          host = p.slice(2, e < 0 ? undefined : e);
          p = e < 0 ? "" : p.slice(e);
          if (host === "localhost") host = "";
        }
        const segs = normPath(p.replace(/^\/+/, "/").split("/").slice(1), true);
        return { protocol, username: "", password: "", hostname: host ? parseHost(host, true) : "", port: "", path: segs.length ? segs : [""], search: enc(true), hash };
      }
      rest = rest.replace(/^\/*/, (x) => (special || x.length >= 2 ? "" : x));
      rest = cut(rest);
      const e = rest.search(/\//);
      let auth = e < 0 ? rest : rest.slice(0, e);
      const p = e < 0 ? "" : rest.slice(e);
      let username = "";
      let password = "";
      const at = auth.lastIndexOf("@");
      if (at >= 0) {
        const cred = auth.slice(0, at);
        auth = auth.slice(at + 1);
        const c = cred.indexOf(":");
        username = pct(c < 0 ? cred : cred.slice(0, c), USER);
        password = c < 0 ? "" : pct(cred.slice(c + 1), USER);
      }
      const pm = /^(\[[^\]]*\]|[^:]*)(?::(.*))?$/.exec(auth)!;
      let port = pm[2] ?? "";
      if (port && !/^\d+$/.test(port)) throw new TypeError(`Failed to construct 'URL': Invalid URL`);
      if (port) {
        const n = parseInt(port, 10);
        if (n > 65535) throw new TypeError(`Failed to construct 'URL': Invalid URL`);
        port = String(n);
        if (SPECIAL[protocol] === port) port = "";
      }
      let hostname: string;
      try {
        hostname = parseHost(pm[1], special);
      } catch {
        throw new TypeError(`Failed to construct 'URL': Invalid URL`);
      }
      if (special && !hostname) throw new TypeError(`Failed to construct 'URL': Invalid URL`);
      const segs = p ? normPath(p.split("/").slice(1), special) : special ? [""] : [];
      return { protocol, username, password, hostname, port, path: segs, search: enc(special), hash };
    }
    // an opaque path (mailto:, data:, blob:, javascript:)
    rest = cut(rest);
    if (rest.startsWith("/")) {
      return { protocol, username: "", password: "", hostname: "", port: "", path: normPath(rest.split("/").slice(1), false), search: enc(false), hash, opaqueHostless: true } as Parts;
    }
    return { protocol, username: "", password: "", hostname: "", port: "", path: pct(rest, C0), search: enc(false), hash };
  }
  if (!base) throw new TypeError(`Failed to construct 'URL': Invalid URL`);
  const special = base.protocol in SPECIAL;
  if (special) s = s.replace(/\\/g, "/");
  if (typeof base.path === "string") {
    if (!s.startsWith("#")) throw new TypeError(`Failed to construct 'URL': Invalid URL`);
    return { ...base, hash: "#" + pct(s.slice(1), FRAG) };
  }
  if (s.startsWith("//")) return parseURL(base.protocol + s);
  s = cut(s);
  const sq = enc(special);
  if (s === "") return { ...base, search: input.includes("?") ? sq : base.search, hash: input.includes("#") ? hash : "" };
  if (s.startsWith("/")) return { ...base, path: normPath(s.split("/").slice(1), special), search: sq, hash };
  const dir = (base.path as string[]).slice(0, -1);
  return { ...base, path: normPath([...dir, ...s.split("/")], special), search: sq, hash };
}

export class URLSearchParams_ {
  _l: [string, string][] = [];
  _url?: URL_;
  constructor(init: unknown = "") {
    if (typeof init === "string") this._parse(init);
    else if (init && typeof (init as any)[Symbol.iterator] === "function") {
      for (const p of init as Iterable<unknown>) {
        const a = [...(p as Iterable<unknown>)];
        if (a.length !== 2) throw new TypeError("Failed to construct 'URLSearchParams': The provided value cannot be converted to a sequence.");
        this._l.push([String(a[0]), String(a[1])]);
      }
    } else if (init && typeof init === "object") for (const k of Object.keys(init)) this._l.push([k, String((init as any)[k])]);
    else if (init !== undefined && init !== null) this._parse(String(init));
  }
  _parse(s: string) {
    this._l = [];
    s = s.replace(/^\?/, "");
    for (const part of s.split("&")) {
      if (!part) continue;
      const i = part.indexOf("=");
      const dec = (x: string) => {
        x = x.replace(/\+/g, " ");
        try {
          return decodeURIComponent(x);
        } catch {
          return x.replace(/%([0-9a-fA-F]{2})/g, (_m, h) => String.fromCharCode(parseInt(h, 16)));
        }
      };
      this._l.push(i < 0 ? [dec(part), ""] : [dec(part.slice(0, i)), dec(part.slice(i + 1))]);
    }
  }
  _update() {
    if (this._url) {
      const q = this.toString();
      this._url._p.search = q ? "?" + q : "";
    }
  }
  get size() {
    return this._l.length;
  }
  append(k: string, v: string) {
    this._l.push([String(k), String(v)]);
    this._update();
  }
  delete(k: string, v?: string) {
    this._l = this._l.filter(([a, b]) => !(a === String(k) && (v === undefined || b === String(v))));
    this._update();
  }
  get(k: string) {
    return this._l.find(([a]) => a === String(k))?.[1] ?? null;
  }
  getAll(k: string) {
    return this._l.filter(([a]) => a === String(k)).map(([, b]) => b);
  }
  has(k: string, v?: string) {
    return this._l.some(([a, b]) => a === String(k) && (v === undefined || b === String(v)));
  }
  set(k: string, v: string) {
    k = String(k);
    const i = this._l.findIndex(([a]) => a === k);
    if (i < 0) this._l.push([k, String(v)]);
    else {
      this._l[i] = [k, String(v)];
      this._l = this._l.filter(([a], j) => a !== k || j === i);
    }
    this._update();
  }
  sort() {
    this._l = this._l.map((e, i) => [e, i] as const).sort((x, y) => (x[0][0] < y[0][0] ? -1 : x[0][0] > y[0][0] ? 1 : x[1] - y[1])).map(([e]) => e);
    this._update();
  }
  forEach(fn: (v: string, k: string, p: URLSearchParams_) => void, self?: unknown) {
    for (const [k, v] of this._l.slice()) fn.call(self, v, k, this);
  }
  keys() {
    return this._l.map(([k]) => k)[Symbol.iterator]();
  }
  values() {
    return this._l.map(([, v]) => v)[Symbol.iterator]();
  }
  entries() {
    return this._l.map(([k, v]) => [k, v] as [string, string])[Symbol.iterator]();
  }
  [Symbol.iterator]() {
    return this.entries();
  }
  toString() {
    return this._l.map(([k, v]) => pct(k, FORM).replace(/%20/g, "+") + "=" + pct(v, FORM).replace(/%20/g, "+")).join("&");
  }
}

export const blobs = new Map<string, Blob_>();

let blobSeq = 0;

export class URL_ {
  _p: Parts;
  _sp?: URLSearchParams_;
  constructor(url: unknown, base?: unknown) {
    const b = base === undefined ? undefined : base instanceof URL_ ? base._p : new URL_(String(base))._p;
    this._p = parseURL(String(url), b);
  }
  static canParse(url: unknown, base?: unknown) {
    try {
      new URL_(url, base);
      return true;
    } catch {
      return false;
    }
  }
  static parse(url: unknown, base?: unknown) {
    try {
      return new URL_(url, base);
    } catch {
      return null;
    }
  }
  static createObjectURL(b: Blob_) {
    const u = `blob:${origin()}/${uuid(++blobSeq)}`;
    blobs.set(u, b);
    return u;
  }
  static revokeObjectURL(u: string) {
    blobs.delete(String(u));
  }
  get protocol() {
    return this._p.protocol;
  }
  set protocol(v: string) {
    const p = String(v).replace(/:.*$/, "").toLowerCase() + ":";
    if (/^[a-z][a-z0-9+.-]*:$/.test(p) && (p in SPECIAL) === (this._p.protocol in SPECIAL) && p !== "file:") {
      this._p.protocol = p;
      if (SPECIAL[p] === this._p.port) this._p.port = "";
    }
  }
  get username() {
    return this._p.username;
  }
  set username(v: string) {
    if (this._p.hostname) this._p.username = pct(String(v), USER);
  }
  get password() {
    return this._p.password;
  }
  set password(v: string) {
    if (this._p.hostname) this._p.password = pct(String(v), USER);
  }
  get host() {
    return this._p.hostname + (this._p.port ? ":" + this._p.port : "");
  }
  set host(v: string) {
    try {
      const u = new URL_(this._p.protocol + "//" + String(v));
      this._p.hostname = u._p.hostname;
      if (u._p.port || /:\d*$/.test(String(v))) this._p.port = u._p.port;
    } catch {}
  }
  get hostname() {
    return this._p.hostname;
  }
  set hostname(v: string) {
    try {
      this._p.hostname = parseHost(String(v), this._p.protocol in SPECIAL);
    } catch {}
  }
  get port() {
    return this._p.port;
  }
  set port(v: string) {
    const m = /^\d*/.exec(String(v))![0];
    if (m === "" && String(v) !== "") return;
    const n = m === "" ? "" : String(parseInt(m, 10));
    if (n !== "" && +n > 65535) return;
    this._p.port = SPECIAL[this._p.protocol] === n ? "" : n;
  }
  get origin() {
    const p = this._p.protocol;
    if (p === "blob:") {
      try {
        return new URL_(this.pathname).origin;
      } catch {
        return "null";
      }
    }
    // Chrome's file: origin is "file://" (the standard's is opaque)
    return p === "file:" ? "file://" : p in SPECIAL ? p + "//" + this.host : "null";
  }
  get pathname() {
    const p = this._p.path;
    return typeof p === "string" ? p : p.length || this._p.protocol in SPECIAL ? "/" + p.join("/") : "";
  }
  set pathname(v: string) {
    if (typeof this._p.path === "string") return;
    const special = this._p.protocol in SPECIAL;
    let s = String(v);
    if (special) s = s.replace(/\\/g, "/");
    this._p.path = normPath((s.startsWith("/") ? s : "/" + s).split("/").slice(1), special);
  }
  get search() {
    return this._p.search.length > 1 ? this._p.search : "";
  }
  set search(v: string) {
    const s = String(v).replace(/^\?/, "");
    this._p.search = s ? "?" + pct(s, this._p.protocol in SPECIAL ? SQUERY : QUERY) : "";
    this._sp?._parse(s);
  }
  get searchParams() {
    if (!this._sp) {
      this._sp = new URLSearchParams_(this.search);
      this._sp._url = this;
    }
    return this._sp;
  }
  get hash() {
    return this._p.hash.length > 1 ? this._p.hash : "";
  }
  set hash(v: string) {
    const s = String(v).replace(/^#/, "");
    this._p.hash = s ? "#" + pct(s, FRAG) : "";
  }
  get href() {
    const p = this._p;
    const auth = p.username || p.password ? p.username + (p.password ? ":" + p.password : "") + "@" : "";
    const hostful = p.hostname || p.protocol in SPECIAL;
    const lead = typeof p.path !== "string" && !hostful && p.path.length > 1 && p.path[0] === "" ? "/." : "";
    return p.protocol + (hostful ? "//" + auth + this.host : (p as any).opaqueHostless ? "" : "") + lead + this.pathname + p.search + p.hash;
  }
  set href(v: string) {
    this._p = parseURL(String(v));
    this._sp?._parse(this.search);
  }
  toString() {
    return this.href;
  }
  toJSON() {
    return this.href;
  }
}

put("URL", named(URL_, "URL"));

put("URLSearchParams", named(URLSearchParams_, "URLSearchParams"));

tag(URLSearchParams_, "URLSearchParams");
