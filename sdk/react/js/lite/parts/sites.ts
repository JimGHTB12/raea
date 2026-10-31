// Hooks' records, and Z3's compiled sites (engine/bundle/src/z3.rs).
import { Element, Fragment, Host, Out, Slot, createElement, jsx } from "./all";

// ---- hooks

export interface Instance {
  fn: any;
  hooks: any[];
  effects: (() => void)[];
  cleanups: Map<number, () => void>;
  /** the component instances under it, by place */
  kids: Map<Slot, Instance>;
  seen: boolean;
  id: Slot | string;
  parent: Instance | null;
  // Z3's bailout (below): the element it was last rendered from, whether that was inside text, and, when it may be
  // reused, its last output with the contexts its subtree read (context, value, ...) and the components it rendered
  el: any;
  inText: boolean;
  out: Out[] | null;
  reads: any[] | null;
  codes: number[] | null;
  /** a state cell in it or under it changed since its last render */
  stale: boolean;
  /** reuses that failed in a row, and frames left before it's cached again (a subtree that reads the frame) */
  misses: number;
  cool: number;
  /** Z3's compiled element sites (engine/bundle/src/z3.rs): per site, its records this render; and which render this is */
  sites: Map<(r: Rec) => any, Sites> | null;
  stamp: number;
}

// ---- Z3's compiled sites (engine/bundle/src/z3.rs)

/** One occurrence of an element site in a component instance: the values it was last given and the element built
 * from them. */
interface Rec {
  v: any[];
  el: Element | null;
  t: any;
  k: any;
  /** a value differed from the last render's (or there was no element yet) */
  ch: boolean;
  /** the render that last reached it (keyed records unreached are let go) */
  u: number;
  /** the site's builder (engine/bundle/src/z3.rs): the props from the values */
  b: (r: Rec) => any;
  /** a key's literal prefix (__atk): the key is kp + k, made when the element is built */
  kp: string | null;
}

interface Sites {
  stamp: number;
  n: number;
  list: Rec[];
  keyed: Map<any, Rec> | null;
  /** keyed records reached this render */
  kn: number;
}

/** the module's state its files assign (each field was a top-level let) */ export const S: { stampSerial: number; current: Instance | null; components: number; hookIndex: number; contexts: Map<symbol, any>; idCounter: number; dirty: boolean; rerender: (() => void) | null; readN: number; codeN: number; pass: number; bailout: boolean; effectsN: number; rootHosts: Host[]; rootElement: any } = { stampSerial: 0, current: null, components: 0, hookIndex: 0, contexts: new Map(), idCounter: 0, dirty: false, rerender: null, readN: 0, codeN: 0, pass: 0, bailout: true, effectsN: 0, rootHosts: [], rootElement: null };

const newRec = (b: (r: Rec) => any): Rec => ({ v: [], el: null, t: undefined, k: null, ch: true, u: 0, b, kp: null });

/** The record for a site's next occurrence in the component rendering now (by its key, or its order among the
 * site's unkeyed occurrences); outside a render, a new one. */
export function __at(site: (r: Rec) => any, key: any): Rec {
  const inst = S.current;
  if (!inst) {
    const r = newRec(site);
    r.k = key;
    return r;
  }
  const map = inst.sites ?? (inst.sites = new Map());
  let st = map.get(site);
  if (st === undefined) map.set(site, (st = { stamp: -1, n: 0, list: [], keyed: null, kn: 0 }));
  if (st.stamp !== inst.stamp) {
    // a new render of the instance: keyed records left far behind (a game's sprites come and go) are let go
    if (st.keyed && st.keyed.size > 2 * st.kn + 32) for (const [k, r] of st.keyed) if (r.u !== st.stamp) st.keyed.delete(k);
    st.stamp = inst.stamp;
    st.n = 0;
    st.kn = 0;
  }
  let r: Rec | undefined;
  if (key === null || key === undefined) {
    r = st.list[st.n];
    if (r === undefined) st.list.push((r = newRec(site)));
    st.n++;
  } else {
    const m = st.keyed ?? (st.keyed = new Map());
    r = m.get(key);
    if (r === undefined) m.set(key, (r = newRec(site)));
    st.kn++;
  }
  r.u = inst.stamp;
  r.ch = r.el === null || !Object.is(r.k, key);
  r.k = key;
  r.kp = null;
  return r;
}

/** __at for a key made of a literal prefix and a value ("s" + i): looked up by the value, so the key's string isn't
 * made every render, only when the element is built. */
export function __atk(site: (r: Rec) => any, prefix: string, x: any): Rec {
  if (x === null || x === undefined || typeof x === "object") return __at(site, prefix + x);
  const r = __at(site, x);
  r.kp = prefix;
  return r;
}

/** A pure call's literal argument (engine/bundle/src/z3.rs): the last one while its values are the same, else one built from them. */
export function __o(r: Rec): any {
  if (!r.ch) return r.el;
  r.el = r.b(r);
  return r.el;
}

/** A site's value `i`: kept when it's the same as last render's (an array, when its items are the same ones: a
 * .map() of unchanged elements), else taken and the element marked to be built again. */
export function __v(r: Rec, i: number, x: any): Rec {
  const o = r.v[i];
  if (Object.is(o, x) || (i < r.v.length && sameItems(o, x))) return r;
  r.v[i] = x;
  r.ch = true;
  return r;
}

function sameItems(a: any, b: any): boolean {
  if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (!Object.is(a[i], b[i])) return false;
  return true;
}

/** The site's element: the last one while its type and values are the same, else one built from them. */
export function __e(r: Rec, type: any): Element {
  if (!r.ch && r.t === type) return r.el!;
  r.t = type;
  r.el = jsx(type, r.b(r), r.kp === null ? r.k : r.kp + r.k);
  return r.el;
}

export { createElement as __h, Fragment as __F };
