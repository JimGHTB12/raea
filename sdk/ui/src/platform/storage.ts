// Storage, location and history (in memory).
import { call } from "../wire";
import { DOMEx, URL_, event, g, named, put, structuredClone_, tag } from "./all";

// ---------------------------------------------------------------- Storage

class Storage_ {
  constructor(m: Map<string, string>, persist: string | null) {
    // configurable, so the proxy may list only the stored keys as its own (Object.keys(localStorage))
    Object.defineProperty(this, "_m", { value: m, configurable: true });
    Object.defineProperty(this, "_persist", { value: persist, configurable: true });
    return new Proxy(this, {
      get(t, k, r) {
        if (typeof k === "string" && !(k in t) && m.has(k)) return m.get(k);
        return Reflect.get(t, k, r);
      },
      set(t, k, v) {
        if (typeof k === "string" && !(k in t)) t.setItem(k, v);
        return true;
      },
      deleteProperty(t, k) {
        if (typeof k === "string") t.removeItem(k);
        return true;
      },
      has(t, k) {
        return (typeof k === "string" && m.has(k)) || k in t;
      },
      ownKeys() {
        return [...m.keys()];
      },
      getOwnPropertyDescriptor(_t, k) {
        return typeof k === "string" && m.has(k) ? { value: m.get(k), writable: true, enumerable: true, configurable: true } : undefined;
      },
    });
  }
  declare _m: Map<string, string>;
  declare _persist: string | null;
  get length() {
    return this._m.size;
  }
  key(i: number) {
    return [...this._m.keys()][i] ?? null;
  }
  getItem(k: string) {
    return this._m.get(String(k)) ?? null;
  }
  setItem(k: string, v: unknown) {
    const key = String(k);
    const old = this.getItem(key);
    const val = String(v);
    let size = 0;
    for (const [a, b] of this._m) if (a !== key) size += a.length + b.length;
    if (size + key.length + val.length > 5 * 1024 * 1024) throw new (DOMEx())(`Failed to execute 'setItem' on 'Storage': Setting the value of '${key}' exceeded the quota.`, "QuotaExceededError");
    this._m.set(key, val);
    if (old !== val) this._save({ k: key, v: val });
  }
  removeItem(k: string) {
    if (this._m.delete(String(k))) this._save({ k: String(k), v: null });
  }
  clear() {
    if (!this._m.size) return;
    this._m.clear();
    this._save({ clear: true });
  }
  _save(change: unknown) {
    if (this._persist) call("net.storage.set", { area: this._persist, change }).catch(() => {});
  }
}

tag(Storage_, "Storage");

put("Storage", named(Storage_, "Storage"));

const local = new Map<string, string>();

put("localStorage", new Storage_(local, "local"));

put("sessionStorage", new Storage_(new Map(), null));

// what an earlier run kept, as soon as the host answers (a bundle reading it at load sees what's been set since)
call<Record<string, string>>("net.storage.get", { area: "local" })
  .then((kept) => {
    for (const k in kept ?? {}) if (!local.has(k)) local.set(k, kept[k]);
  })
  .catch(() => {});

// ---------------------------------------------------------------- location and history (in memory)

export const entries: { url: string; state: unknown }[] = [{ url: "http://localhost/", state: null }];

export let at = 0;

const here = () => new URL_(entries[at].url);

function navigate(url: string) {
  const u = new URL_(url, entries[at].url);
  const h = here();
  if (u.href.replace(/#.*$/, "") === h.href.replace(/#.*$/, "") && u.hash !== h.hash) {
    entries.splice(at + 1);
    entries.push({ url: u.href, state: null });
    at++;
    // Chrome: popstate during the navigation, hashchange after a task
    const oldURL = h.href;
    g.dispatchEvent?.(event("popstate", { state: null, hasUAVisualTransition: false }));
    g.setTimeout(() => g.dispatchEvent?.(event("hashchange", { oldURL, newURL: u.href })), 0);
    return;
  }
  // a document load: nothing to load into here; the URL changes as an in-memory navigation would leave it
  entries.splice(at + 1);
  entries.push({ url: u.href, state: null });
  at++;
}

class Location_ {
  get href() {
    return here().href;
  }
  set href(v: string) {
    navigate(String(v));
  }
  get origin() {
    return here().origin;
  }
  get protocol() {
    return here().protocol;
  }
  get host() {
    return here().host;
  }
  get hostname() {
    return here().hostname;
  }
  get port() {
    return here().port;
  }
  get pathname() {
    return here().pathname;
  }
  set pathname(v: string) {
    const u = here();
    u.pathname = v;
    navigate(u.href);
  }
  get search() {
    return here().search;
  }
  set search(v: string) {
    const u = here();
    u.search = v;
    navigate(u.href);
  }
  get hash() {
    return here().hash;
  }
  set hash(v: string) {
    const u = here();
    u.hash = v;
    navigate(u.href);
  }
  get ancestorOrigins() {
    return { length: 0, item: () => null, contains: () => false };
  }
  assign(u: string) {
    navigate(String(u));
  }
  replace(u: string) {
    entries[at] = { url: new URL_(String(u), entries[at].url).href, state: null };
  }
  reload() {}
  toString() {
    return this.href;
  }
}

tag(Location_, "Location");

class History_ {
  scrollRestoration = "auto";
  get length() {
    return entries.length;
  }
  get state() {
    return entries[at].state;
  }
  _url(u: unknown) {
    if (u === undefined || u === null) return entries[at].url;
    const next = new URL_(String(u), entries[at].url);
    if (next.origin !== here().origin) throw new (DOMEx())(`Failed to execute 'pushState' on 'History': A history state object with URL '${next.href}' cannot be created in a document with origin '${here().origin}' and URL '${entries[at].url}'.`, "SecurityError");
    return next.href;
  }
  pushState(state: unknown, _title: string, url?: string | URL_ | null) {
    const u = this._url(url);
    entries.splice(at + 1);
    entries.push({ url: u, state: structuredClone_(state) });
    at++;
  }
  replaceState(state: unknown, _title: string, url?: string | URL_ | null) {
    entries[at] = { url: this._url(url), state: structuredClone_(state) };
  }
  go(n = 0) {
    const to = at + Number(n || 0);
    if (n === 0 || to < 0 || to >= entries.length) return;
    // a traversal is a task: popstate (and hashchange when only the fragment moved) fire after it
    g.setTimeout(() => {
      const was = entries[at].url;
      at = to;
      const now = entries[at].url;
      g.dispatchEvent?.(event("popstate", { state: entries[at].state, hasUAVisualTransition: false }));
      if (was.replace(/#.*$/, "") === now.replace(/#.*$/, "") && new URL_(was).hash !== new URL_(now).hash) g.dispatchEvent?.(event("hashchange", { oldURL: was, newURL: now }));
    }, 0);
  }
  back() {
    this.go(-1);
  }
  forward() {
    this.go(1);
  }
}

tag(History_, "History");

put("Location", named(Location_, "Location"));

put("History", named(History_, "History"));

put("location", new Location_());

put("history", new History_());
