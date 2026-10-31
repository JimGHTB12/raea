// Fingerprints: a declaration (or a whole props object) by content.
import { useRef } from "react";
import type { AnimTrack, Animation } from "../animate.gen";
import { Wire } from "./all";

// ---- fingerprints: a declaration (or a whole props object) by content, walked without allocating ----

const FN_IDS = new WeakMap<object, number>();

const SYM_IDS = new Map<symbol, number>();

/** the module's state its files assign (each field was a top-level let) */ export const S: { nextId: number } = { nextId: 1 };

export const mix = (h: number, x: number) => Math.imul(h ^ x, 0x01000193) >>> 0;

const idOf = (f: object) => {
  let i = FN_IDS.get(f);
  if (i === undefined) FN_IDS.set(f, (i = S.nextId++));
  return i;
};

/** FNV-1a over a value's numbers and strings. Functions (components, handlers) count by identity, React elements by
 * type, key and props; an inline `animate={{ y: track(...) }}` or `style={{...}}` is a new object every render, and this
 * tells it's the same content. Assumes props aren't mutated in place (React's own assumption). */
export function fingerprint(v: unknown, h: number, depth = 0): number {
  if (v === null || v === undefined) return mix(h, v === null ? 0x9e37 : 0x9e38);
  switch (typeof v) {
    case "number": {
      // the number's bits, through its integer and fractional parts (no typed-array view, no string)
      const i = Math.floor(v);
      return mix(mix(h, i | 0), ((v - i) * 4294967296) | 0);
    }
    case "string":
      for (let i = 0; i < v.length; i++) h = mix(h, v.charCodeAt(i));
      return mix(h, 0x22);
    case "boolean":
      return mix(h, v ? 0x31 : 0x30);
    case "function":
      return mix(mix(h, 0x66), idOf(v));
    case "symbol": {
      let i = SYM_IDS.get(v);
      if (i === undefined) SYM_IDS.set(v, (i = S.nextId++));
      return mix(mix(h, 0x73), i);
    }
    case "object": {
      if (depth > 48) return mix(h, S.nextId++); // too deep to walk: never the same
      const o = v as Record<string, unknown>;
      if (typeof o.$$typeof === "symbol") {
        // a React element: its type, key and props
        h = fingerprint(o.type, mix(h, 0x3c), depth + 1);
        h = fingerprint(o.key, h, depth + 1);
        return fingerprint(o.props, h, depth + 1);
      }
      if (Array.isArray(v)) {
        h = mix(h, 0x5b);
        for (let i = 0; i < v.length; i++) h = fingerprint(v[i], h, depth + 1);
        return mix(h, 0x5d);
      }
      h = mix(h, 0x7b);
      for (const k in o) {
        h = fingerprint(k, h, depth + 1);
        h = fingerprint(o[k], h, depth + 1);
      }
      return mix(h, 0x7d);
    }
    default:
      return h;
  }
}

/** A props object's fingerprint: what <Animate> and <Stagger> compare to know a render would give the same element,
 * and what a runtime's bailout can key on (`Animate.fingerprint`). */
export function propsFingerprint(props: object): number {
  return fingerprint(props, 0x811c9dc5);
}

/** The Animation text for these tracks at this place in time: built and serialised only when the declaration's
 * content or its place in time changes, and the same string object otherwise (so React sends it once and a frame
 * allocates nothing for it). */
type In = unknown;

export function useWire(build: () => AnimTrack[], i0: In, i1: In, i2: In, i3: In, i4: In, i5: In, i6: In, origin: number, freeze: number | null, fps: number, window: boolean): Wire {
  const last = useRef<(Wire & { key: number; ins: In[] }) | null>(null);
  const prev = last.current;
  // the same objects as last time (a declaration hoisted out of the render): nothing to look at, nothing allocated
  if (prev && prev.anim.origin === origin && prev.anim.freeze === freeze && prev.anim.fps === fps && prev.window === window) {
    const p = prev.ins;
    if (p[0] === i0 && p[1] === i1 && p[2] === i2 && p[3] === i3 && p[4] === i4 && p[5] === i5 && p[6] === i6) return prev;
  }
  let key = 0x811c9dc5;
  key = fingerprint(i6, fingerprint(i5, fingerprint(i4, fingerprint(i3, fingerprint(i2, fingerprint(i1, fingerprint(i0, key)))))));
  key = fingerprint(origin, fingerprint(freeze, fingerprint(fps, fingerprint(window, key))));
  if (prev && prev.key === key) {
    // the same content in new objects: remembered in place (nothing allocated), so the next frame's identity check
    // can hit if the caller starts passing the same objects
    const p = prev.ins;
    p[0] = i0; p[1] = i1; p[2] = i2; p[3] = i3; p[4] = i4; p[5] = i5; p[6] = i6;
    return prev;
  }
  const ins = [i0, i1, i2, i3, i4, i5, i6];
  const tracks = build();
  const anim: Animation = { origin, freeze, fps, tracks, show_from: null, show_until: null };
  if (window) {
    // mount="during": shown from the first key, and (with an exit) hidden again after the last
    let from = Infinity;
    let until = -Infinity;
    for (const tr of tracks) for (const k of tr.keys) { from = Math.min(from, k.at); until = Math.max(until, k.at); }
    if (from !== Infinity) anim.show_from = from;
    if (until !== -Infinity && i4 !== undefined) anim.show_until = until + 1;
  }
  const json = JSON.stringify(anim);
  if (prev && prev.json === json) {
    prev.key = key;
    prev.ins = ins;
    return prev;
  }
  const w = { json, anim, key, ins, window };
  last.current = w;
  return w;
}
