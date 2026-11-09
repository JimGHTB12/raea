// Curves, tracks and moves.
import { MoveLike, fingerprint, mix, toMove } from "./all";

// ---- curves: text names, which cross the bridge (the engine parses the same names) ----

/** Curve names. Any of these, or `bezier(x1,y1,x2,y2)`, `steps(n)`, `spring(mass,damping,stiffness)`,
 * `reverse(<curve>)`. */
export type CurveName =
  | "linear" | "ease"
  | `${"in" | "out" | "in-out"}-${"quad" | "cubic" | "quart" | "quint" | "sine" | "expo" | "circ" | "back" | "elastic" | "bounce"}`;

export type Curve = CurveName | (string & {});

/** Curve helpers: `curve.bezier(0.05, 0.7, 0.1, 1)`, `curve.spring({ damping: 15, stiffness: 120 })`. */
export const curve = {
  bezier: (x1: number, y1: number, x2: number, y2: number): Curve => `bezier(${x1},${y1},${x2},${y2})`,
  steps: (n: number): Curve => `steps(${n})`,
  /** a spring stretched over its segment (its settle lands on the next key) */
  spring: (c: { mass?: number; damping?: number; stiffness?: number } = {}): Curve => `spring(${c.mass ?? 1},${c.damping ?? 10},${c.stiffness ?? 100})`,
  reverse: (c: Curve): Curve => `reverse(${c})`,
};

// ---- tracks and moves ----

/** Keyframes by local frame: numbers, or colour text for colour properties (a material's `animate` takes other
 * values for its own fields, LinearRgba for an emissive). */
export type Keys<V = number | string> = Record<number, V>;

/** One property's keyframes: values by frame, the curve between each pair (one for all, or one per segment). */
export interface TrackSpec<V = number | string> {
  keys: [number, V][];
  ease: Curve | Curve[];
  unit?: "px" | "%";
  repeat?: "hold" | "loop" | "pingpong" | `${"loop" | "pingpong"}:${number}`;
  digits?: number;
  /** a `text` counter's text before and after the number, and between its thousands ("," writes 12,345) */
  prefix?: string;
  suffix?: string;
  separator?: string;
}

// Declarations built inline in a render (`track({ 0: 30, 6: 0 })`, `pop({ peak: 1.1 })`) come back as the same object
// for the same arguments: kept by the arguments' fingerprint, checked equal on a hit, so a frame that builds the same
// declaration again allocates only its argument literal.
const MADE = new Map<number, { args: unknown[]; value: unknown }>();

const MADE_MAX = 4096;

function same(a: unknown, b: unknown, depth = 0): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || a === null || b === null || depth > 32) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  let n = 0;
  for (const k in a as Record<string, unknown>) {
    if (!same((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k], depth + 1)) return false;
    n++;
  }
  for (const _ in b as Record<string, unknown>) n--;
  return n === 0;
}

export function made<T>(tag: number, a0: unknown, a1: unknown, a2: unknown, build: () => T): T {
  const key = fingerprint(a2, fingerprint(a1, fingerprint(a0, mix(0x811c9dc5, tag))));
  const hit = MADE.get(key);
  if (hit && same(hit.args[0], a0) && same(hit.args[1], a1) && same(hit.args[2], a2)) return hit.value as T;
  const value = build();
  if (MADE.size >= MADE_MAX) MADE.clear();
  // the arguments copied as they were (a caller may reuse and change its object)
  MADE.set(key, { args: [structured(a0), structured(a1), structured(a2)], value });
  return value;
}

function structured(v: unknown): unknown {
  if (typeof v !== "object" || v === null) return v;
  if (Array.isArray(v)) return v.map(structured);
  const o: Record<string, unknown> = {};
  for (const k in v as Record<string, unknown>) o[k] = structured((v as Record<string, unknown>)[k]);
  return o;
}

/** A track: `track({ 0: 30, 6: 0 })` is a 30 px rise over 6 frames, out-cubic by default. */
export function track<V = number | string>(keys: Keys<V>, ease: Curve | Curve[] = "out-cubic", opts: Omit<TrackSpec, "keys" | "ease"> = NO_OPTS): TrackSpec<V> {
  return made(1, keys, ease, opts, () => makeTrack(keys, ease, opts));
}

const NO_OPTS = {};

function makeTrack<V>(keys: Keys<V>, ease: Curve | Curve[], opts: Omit<TrackSpec, "keys" | "ease">): TrackSpec<V> {
  const ks = Object.keys(keys).map(Number).sort((a, b) => a - b).map((f) => [f, keys[f]] as [number, V]);
  if (ks.length === 0) throw new Error("track: no keys");
  if (Array.isArray(ease) && ease.length !== ks.length - 1) throw new RangeError(`track: ${ks.length} keys take ${ks.length - 1} curves, got ${ease.length}`);
  return { keys: ks, ease, ...opts };
}

/** Tracks by property, in frames from the move's own start. */
export interface Move {
  tracks: Record<string, TrackSpec | TrackSpec[]>;
  /** how long it runs (its last key) */
  frames: number;
}

/** A move from its tracks, by property: `move({ opacity: track({ 0: 0, 6: 1 }), y: track({ 0: 30, 6: 0 }) })`. Its
 * length is its last key. The named moves (rise, pop and the rest) are built with it. */
export function move(tracks: Record<string, TrackSpec | TrackSpec[] | Keys>): Move {
  const out: Record<string, TrackSpec | TrackSpec[]> = {};
  let frames = 0;
  for (const [k, v] of Object.entries(tracks)) {
    const t = Array.isArray(v) || (v as TrackSpec).keys ? (v as TrackSpec | TrackSpec[]) : track(v as Keys);
    out[k] = t;
    for (const s of Array.isArray(t) ? t : [t]) frames = Math.max(frames, s.keys[s.keys.length - 1][0]);
  }
  return { tracks: out, frames };
}

/** The same move later by `frames`. */
export function delay(m: MoveLike, frames: number): Move {
  const mm = toMove(m);
  return { frames: mm.frames + frames, tracks: mapTracks(mm, (t) => ({ ...t, keys: t.keys.map(([f, v]) => [f + frames, v]) })) };
}

/** A move played backwards: it leaves the way it came (each curve reversed, so an out-curve becomes the matching
 * in-curve). An exit is its entrance reversed. */
export function reverse(m: MoveLike): Move {
  const mm = toMove(m);
  const L = mm.frames;
  return {
    frames: L,
    tracks: mapTracks(mm, (t) => {
      const n = t.keys.length;
      const eases = Array.isArray(t.ease) ? t.ease : Array(Math.max(0, n - 1)).fill(t.ease);
      const keys = t.keys.map(([f, v]) => [L - f, v] as [number, number | string]).reverse();
      const rev = eases.map((e: Curve) => (e === "linear" || e === "" ? "linear" : `reverse(${e})`)).reverse();
      return { ...t, keys, ease: rev };
    }),
  };
}

function mapTracks(m: Move, f: (t: TrackSpec) => TrackSpec): Record<string, TrackSpec | TrackSpec[]> {
  const out: Record<string, TrackSpec | TrackSpec[]> = {};
  for (const [k, v] of Object.entries(m.tracks)) out[k] = Array.isArray(v) ? v.map(f) : f(v);
  return out;
}
