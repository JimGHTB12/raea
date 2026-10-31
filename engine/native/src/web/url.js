// A piece of web.js's one function body (lib.rs joins the pieces inside `(function (g) { "use strict"; ... })(globalThis)`,
// so they share its helpers): URLSearchParams and URL.

// ---- URLSearchParams / URL ----------------------------------------------------------------------------------------
const encodeForm = (s) =>
  encodeURIComponent(s).replace(/%20/g, "+").replace(/[!'()~]/g, (c) => "%" + c.charCodeAt(0).toString(16).toUpperCase());
const decodeForm = (s) => {
  try {
    return decodeURIComponent(s.replace(/\+/g, " "));
  } catch (_) {
    return s.replace(/\+/g, " ");
  }
};
class URLSearchParamsShim {
  constructor(init = "") {
    this._list = [];
    this._url = null;
    if (typeof init === "string") this._parse(init);
    else if (init && typeof init[Symbol.iterator] === "function") {
      for (const pair of init) {
        const p = Array.from(pair);
        if (p.length !== 2) throw new TypeError("each pair needs a name and a value");
        this._list.push([String(p[0]), String(p[1])]);
      }
    } else if (init && typeof init === "object") {
      for (const k of Object.keys(init)) this._list.push([k, String(init[k])]);
    }
  }
  _parse(s) {
    this._list = [];
    s = String(s);
    if (s.startsWith("?")) s = s.slice(1);
    for (const part of s.split("&")) {
      if (!part) continue;
      const i = part.indexOf("=");
      this._list.push(i < 0 ? [decodeForm(part), ""] : [decodeForm(part.slice(0, i)), decodeForm(part.slice(i + 1))]);
    }
  }
  _update() {
    if (this._url) this._url._setSearch(this.toString());
  }
  get size() {
    return this._list.length;
  }
  append(k, v) {
    this._list.push([String(k), String(v)]);
    this._update();
  }
  delete(k, v) {
    this._list = this._list.filter(([a, b]) => !(a === String(k) && (v === undefined || b === String(v))));
    this._update();
  }
  get(k) {
    const e = this._list.find(([a]) => a === String(k));
    return e ? e[1] : null;
  }
  getAll(k) {
    return this._list.filter(([a]) => a === String(k)).map(([, b]) => b);
  }
  has(k, v) {
    return this._list.some(([a, b]) => a === String(k) && (v === undefined || b === String(v)));
  }
  set(k, v) {
    k = String(k);
    const i = this._list.findIndex(([a]) => a === k);
    if (i < 0) this._list.push([k, String(v)]);
    else {
      this._list[i][1] = String(v);
      this._list = this._list.filter(([a], j) => a !== k || j === i);
    }
    this._update();
  }
  sort() {
    this._list = this._list.map((e, i) => [e, i]).sort((x, y) => (x[0][0] < y[0][0] ? -1 : x[0][0] > y[0][0] ? 1 : x[1] - y[1])).map((e) => e[0]);
    this._update();
  }
  forEach(fn, self) {
    for (const [k, v] of this._list) fn.call(self, v, k, this);
  }
  keys() {
    return this._list.map(([k]) => k)[Symbol.iterator]();
  }
  values() {
    return this._list.map(([, v]) => v)[Symbol.iterator]();
  }
  entries() {
    return this._list.map(([k, v]) => [k, v])[Symbol.iterator]();
  }
  [Symbol.iterator]() {
    return this.entries();
  }
  toString() {
    return this._list.map(([k, v]) => encodeForm(k) + "=" + encodeForm(v)).join("&");
  }
}
def("URLSearchParams", URLSearchParamsShim);

const PARTS = ["href", "origin", "protocol", "username", "password", "host", "hostname", "port", "pathname", "search", "hash"];
class URLShim {
  constructor(url, base) {
    const p = call("url.parse", base === undefined ? { url: String(url) } : { url: String(url), base: String(base) });
    if (!p) throw new TypeError(`Invalid URL: ${url}`);
    this._p = p;
    this._sp = new g.URLSearchParams(p.search);
    this._sp._url = this;
  }
  static canParse(url, base) {
    return !!call("url.parse", base === undefined ? { url: String(url) } : { url: String(url), base: String(base) });
  }
  static parse(url, base) {
    try {
      return new URLShim(url, base);
    } catch (_) {
      return null;
    }
  }
  _set(part, value) {
    this._p = call("url.set", { href: this._p.href, part, value: String(value) });
    if (part === "search" || part === "href") this._sp._parse(this._p.search);
  }
  _setSearch(q) {
    this._p = call("url.set", { href: this._p.href, part: "search", value: q });
  }
  get searchParams() {
    return this._sp;
  }
  toString() {
    return this._p.href;
  }
  toJSON() {
    return this._p.href;
  }
}
for (const k of PARTS) {
  Object.defineProperty(URLShim.prototype, k, {
    get() {
      return this._p[k];
    },
    set(v) {
      if (k !== "origin") this._set(k, v);
    },
    configurable: true,
  });
}
def("URL", URLShim);
