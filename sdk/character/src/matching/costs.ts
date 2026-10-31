// What a frame costs: states, phases, a crouch, an arm's swing past held things, tags.
import type { Vec3 } from "raea/3d";
import { bodyScale, joint, palmAt, palmFacing, pose, surface, type Posed, type Quat, type Skeleton } from "raea/character";
import { STYLE_KEYS, type StyleVector } from "../style";
import { STATES } from "../effects";
import { MotionLibrary, NF, NS, STATE, Step, TagCond, TagSpec, add3, libraryPose, movingAt } from "./all";

export const sub3 = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

export const scale = (a: Vec3, k: number): Vec3 => [a[0] * k, a[1] * k, a[2] * k];

/** A state to move in: a style's name (100STYLE's: "Old", "LimpLeft") or a state's (`STATES`: "sad", "tired"), a mix
 * of them by weight ({ Depressed: 0.6, LimpLeft: 0.8 }), or some STATE measurements (a style's own numbers). */
export type MotionState = string | Record<string, number> | Partial<StyleVector>;

/** a state as a mix of style labels by weight, or null for measurements */
export function stateMix(s: MotionState): Record<string, number> | null {
  if (typeof s === "string") return STATES[s] ?? { [s]: 1 };
  const keys = Object.keys(s);
  if (keys.length && keys.every((k) => (STYLE_KEYS as string[]).includes(k))) return null;
  const out: Record<string, number> = {};
  for (const [k, w] of Object.entries(s as Record<string, number>)) for (const [l, v] of Object.entries(STATES[k] ?? { [k]: 1 })) out[l] = (out[l] ?? 0) + v * w;
  return out;
}

/** a clip's style label: 100STYLE's clips are STYLE_MOVE; CMU's (c07_01) are unlabelled, taken as neutral */
export const labelOf = (name: string) => (/^c\d+_\d+$/.test(name) ? "" : name.split("_")[0]);

/** A state as the library's z-scores (NaN where it doesn't say). */
export function stateQuery(lib: MotionLibrary, s: MotionState, moving?: boolean): Float32Array | undefined {
  const z = new Float32Array(NS).fill(NaN);
  const mix = stateMix(s);
  if (!mix) {
    const v = s as Partial<StyleVector>;
    STATE.forEach((k, i) => { if (v[k] !== undefined) z[i] = (v[k]! - lib.stateMean[i]) / lib.stateSd[i]; });
    return z;
  }
  // a label's mean state (its clips', not their mirror images'), as z-scores: over its moving frames for a walk and its
  // still ones for a wait (a style's walk and stand averaged together is neither, and looks like some other style's walk)
  const meanOf = (label: string): Float64Array | null => {
    const sum = new Float64Array(NS);
    let m = 0;
    for (const r of lib.clips) {
      if (r.mirrored || labelOf(r.name) !== label) continue;
      for (let k = r.start; k < r.end; k++) { if (moving !== undefined && movingAt(lib, k, r) !== moving) continue; m++; for (let i = 0; i < NS; i++) sum[i] += lib.state[k * NS + i]; }
    }
    if (!m) return null;
    for (let i = 0; i < NS; i++) sum[i] = (sum[i] / m - lib.stateMean[i]) / lib.stateSd[i];
    return sum;
  };
  // neutral plus each label's difference from it, by weight; labels the library hasn't are left to the effects
  const neutral = meanOf("Neutral") ?? new Float64Array(NS);
  const acc = Float64Array.from(neutral);
  let any = false;
  for (const [l, w] of Object.entries(mix)) {
    const m = meanOf(l);
    if (!m) continue;
    any = true;
    for (let i = 0; i < NS; i++) acc[i] += w * (m[i] - neutral[i]);
  }
  // none of them: Neutral's own state, so the frames under the effects are neutral ones (if the library has any)
  if (!any && !meanOf("Neutral")) return undefined;
  for (let i = 0; i < NS; i++) z[i] = acc[i];
  return z;
}

/** Each library clip's cost for a state, by its style's label: none for a style the state is all of, more the less of the
 * state it is (the state's styles the library has, by weight; Neutral if it has none of them). Where the library has a
 * state's real frames, these pick them; undefined for a library without the labels. */
export function labelCost(lib: MotionLibrary, s: MotionState, ws: number): Float32Array | undefined {
  const mix = stateMix(s);
  if (!mix) return undefined;
  const has = new Set(lib.clips.map((r) => labelOf(r.name)));
  let own = Object.entries(mix).filter(([l, w]) => w > 0 && has.has(l));
  if (!own.length) { if (!has.has("Neutral")) return undefined; own = [["Neutral", 1]]; }
  const total = own.reduce((a, [, w]) => a + w, 0), share = new Map(own.map(([l, w]) => [l, w / total]));
  return Float32Array.from(lib.clips, (r) => { const x = share.get(labelOf(r.name)); return x === undefined ? Infinity : ((ws * ws) / 2) * (1 - x); });
}

/** how far a library frame's style is from a wanted state (z-scores, NaN where it doesn't say), weighted */
export function stateCost(lib: MotionLibrary, k: number, want: Float32Array | undefined, ws: number): number {
  if (!want) return 0;
  let s = 0, m = 0;
  for (let i = 0; i < NS; i++) {
    if (Number.isNaN(want[i])) continue;
    const e = (lib.state[k * NS + i] - lib.stateMean[i]) / lib.stateSd[i] - want[i];
    s += e * e; m++;
  }
  return m ? (ws * ws * s) / m : 0;
}

/** an action ahead: library frame `to` is to play `m` library frames from now (perform steers the walk into its phase) */
export interface PhaseGoal { to: number; m: number; w: number }

/** how far frame k, played m frames on, is from the goal's frame in pose (the feet's places and speeds, the hips'
 * speed): a walk arriving at an action with its feet as the action starts */
export function phaseCost(lib: MotionLibrary, k: number, g: PhaseGoal): number {
  const k2 = k + g.m;
  if (k2 >= lib.clips[lib.frameClip[k]].end) return g.w * 50;
  const F = lib.features, a = k2 * NF, b = g.to * NF;
  let s = 0;
  for (let d = 0; d < 15; d++) { const e = F[a + d] - F[b + d]; s += e * e; }
  return g.w * s;
}

export const AHEAD = 6;

/** the library frames in a capture's crouch or coming to one within 12 frames: the hips under 0.92 of their clip's
 * median height. A stand isn't matched to them (127_05's ready-to-run, the hips at 0.69, was played as a
 * wait's last frames before a walk: a bow 40 cm down in the middle of looking at a thing in hand) */
const crouchedOf = new WeakMap<MotionLibrary, Uint8Array>();

export function crouched(lib: MotionLibrary): Uint8Array {
  if (lib.crouch) return lib.crouch;
  let m = crouchedOf.get(lib);
  if (m) return m;
  m = new Uint8Array(lib.size);
  for (const c of lib.clips) {
    const ys = Float32Array.from({ length: c.end - c.start }, (_, i) => libraryPose(lib, c.start + i).root[1]);
    const med = Float32Array.from(ys).sort()[ys.length >> 1];
    for (let i = 0; i < ys.length; i++) if (ys[i] < 0.92 * med) for (let j = Math.max(0, i - 12); j <= i; j++) m[c.start + j] = 1;
  }
  crouchedOf.set(lib, m);
  return m;
}

/** Swing clearance: how far a thing held in a hand clears the body's thigh on its side and the waist, as the arm
 * swings. The thing's middle is taken off the palm as a small thing rides it (HELD_AT, about a mug's or a can's half
 * size), and the thigh and the waist as capsules about their bones, as thick as the skin is at rest. */
const HELD_AT = 0.04;

const capsulesOf = new WeakMap<Skeleton, { thigh: [number, number, number][]; waist: [number, number, number] }>();

const segDist = (p: Vec3, a: Vec3, b: Vec3) => {
  const ab0 = b[0] - a[0], ab1 = b[1] - a[1], ab2 = b[2] - a[2], ap0 = p[0] - a[0], ap1 = p[1] - a[1], ap2 = p[2] - a[2];
  const t = Math.max(0, Math.min(1, (ab0 * ap0 + ab1 * ap1 + ab2 * ap2) / Math.max(1e-12, ab0 * ab0 + ab1 * ab1 + ab2 * ab2)));
  return Math.hypot(ap0 - t * ab0, ap1 - t * ab1, ap2 - t * ab2);
};

function swingCapsules(skel: Skeleton) {
  let c = capsulesOf.get(skel);
  if (c) return c;
  const J = (n: string) => joint(skel, n);
  // [from joint, to joint, radius]: the skin's 90th percentile off the bone at rest (a guess by the body's size without a skin)
  const rest = pose(skel, { root: skel.joints[0].head, turns: skel.joints.map(() => [0, 0, 0, 1] as Quat) });
  const fit = (a: number, b: number, re: RegExp, side: string | undefined, guess: number) => {
    const d = surface(skel, rest, re, side as "L" | "R" | undefined).map((x) => segDist(x.at, rest.pos[a], rest.pos[b])).sort((x, y) => x - y);
    return d.length ? d[Math.floor(0.9 * (d.length - 1))] : guess * bodyScale(skel);
  };
  const thigh = (["L", "R"] as const).map((s) => { const a = J(s === "L" ? "LeftUpLeg" : "RightUpLeg"), b = J(s === "L" ? "LeftLeg" : "RightLeg"); return [a, b, fit(a, b, /^thigh$/, s, 0.075)] as [number, number, number]; });
  const wa = J("Hips"), wb = J("Spine");
  c = { thigh, waist: [wa, wb, fit(wa, wb, /^(waist|hips)$/, undefined, 0.12)] };
  capsulesOf.set(skel, c);
  return c;
}

/** a posed hand's swing clearance: its held thing's middle (off the palm) to the skin of its thigh and of the waist, m */
export function swingClear(skel: Skeleton, w: Posed, s: "L" | "R"): number {
  const c = swingCapsules(skel), p = add3(palmAt(skel, w, s), palmFacing(skel, w, s).map((v) => v * HELD_AT) as Vec3);
  const [a, b, r] = c.thigh[s === "L" ? 0 : 1], [wa, wb, wr] = c.waist;
  return Math.min(segDist(p, w.pos[a], w.pos[b]) - r, segDist(p, w.pos[wa], w.pos[wb]) - wr);
}

/** Each library frame's least swing clearance per hand (frames x 2, left then right, m) over the next half second of
 * its clip: a thing held in that hand from there clears the thigh and the waist by this much. From the database's
 * per-frame clearances (lib.clear), or the poses. Worked out once per library. */
const SWING_AHEAD = 0.5;

const swingOf = new WeakMap<MotionLibrary, Float32Array>();

export function swingClearance(lib: MotionLibrary): Float32Array {
  let m = swingOf.get(lib);
  if (m) return m;
  let inst = lib.clear;
  if (!inst) {
    inst = new Float32Array(lib.size * 2);
    for (let k = 0; k < lib.size; k++) { const w = pose(lib.skel, libraryPose(lib, k)); inst[k * 2] = swingClear(lib.skel, w, "L"); inst[k * 2 + 1] = swingClear(lib.skel, w, "R"); }
  }
  const H = Math.max(1, Math.round(SWING_AHEAD * lib.fps));
  m = new Float32Array(lib.size * 2);
  // (a running minimum over the window ahead, each clip on its own: a deque would be quicker, the window is 15 frames)
  for (const r of lib.clips) for (let k = r.start; k < r.end; k++) for (let s = 0; s < 2; s++) {
    let v = Infinity;
    for (let j = k; j < Math.min(r.end, k + H); j++) v = Math.min(v, inst[j * 2 + s]);
    m[k * 2 + s] = v;
  }
  swingOf.set(lib, m);
  return m;
}

/** the cost of each frame for a step holding things (`need`: the room each hand's thing needs, its half size and a
 * margin): Infinity where a hand's swing comes closer than that, 0 elsewhere (a mask nearest adds as a cost, so it
 * passes to any search). A cost by how far short (the shortfall in cm squared, half a feature's spread a centimetre)
 * did worse: the man's ball-look failed (11 frames) where the mask passed it (8). */
const swingCosts = new WeakMap<MotionLibrary, Map<string, Float32Array>>();

export function swingCost(lib: MotionLibrary, need: Partial<Record<"L" | "R", number>>): Float32Array | undefined {
  const sides = (["L", "R"] as const).filter((s) => need[s] !== undefined);
  if (!sides.length) return undefined;
  const key = JSON.stringify(sides.map((s) => [s, need[s]]));
  let byKey = swingCosts.get(lib);
  if (!byKey) swingCosts.set(lib, (byKey = new Map()));
  let c = byKey.get(key);
  if (c) return c;
  const C = swingClearance(lib);
  c = new Float32Array(lib.size);
  for (let k = 0; k < lib.size; k++) for (const s of sides) if (C[k * 2 + (s === "L" ? 0 : 1)] < need[s]!) c[k] = Infinity;
  byKey.set(key, c);
  return c;
}

/** A step's tag conditions as bits, on a database library (undefined on any other, or when it says none): its own and
 * the spec's, and its kind's defaults. A walk under a run's pace forbids `fast` (a run's frames, with their crouched
 * starts and dips) and, with no state asked, the styled walks (a zombie's, a drunk's); every walk forbids what no walk
 * plays as (dancing, fighting, lying, falling, climbing, the playground). */
export function stepTags(lib: MotionLibrary, st: Step | undefined, all?: TagSpec): TagCond | undefined {
  if (!lib.db) return undefined;
  const want: string[] = [...(all?.want ?? [])], forbid: string[] = [...(all?.forbid ?? [])], cost: Record<string, number> = { ...all?.cost };
  if (st && "walk" in st) {
    forbid.push("dance", "fight", "lie", "fall", "climb", "playground", "stairs", "uneven", "raised", "jump", "hop", "sport");
    if ((st.speed ?? 0) < 1.8 * bodyScale(lib.skel)) forbid.push("fast", "crouched");
    if (!st.state) forbid.push("styled");
    // (a walk at walking pace from walks, unless it asks for frames of its own: a walk took arm swings and strides
    // from pantomimes and chores, the legs a long stride into a mug carried at the side)
    if ((st.speed ?? 0) < 1.8 * bodyScale(lib.skel) && !st.tags?.want?.length && !all?.want?.length) want.push("walk");
  }
  if (st && "wait" in st) forbid.push("dance", "fight", "lie", "fall", "fast", "raised", "climb", "playground", "stairs");
  if (st && ("walk" in st || "wait" in st) && st.tags) { want.push(...(st.tags.want ?? [])); forbid.push(...(st.tags.forbid ?? [])); Object.assign(cost, st.tags.cost); }
  // (a tag wanted outweighs a default forbidding it)
  const wanted = new Set(want);
  const bits = (names: string[]): [number, number] => {
    const b: [number, number] = [0, 0];
    for (const n of names) { const t = lib.db!.tagIndex.get(n); if (t === undefined) throw new Error(`perform: no tag ${n} (clips.ts TAGS)`); b[t >> 5] = (b[t >> 5] | (1 << (t & 31))) >>> 0; }
    return b;
  };
  const tc: TagCond = { want: bits(want), forbid: bits(forbid.filter((f) => !wanted.has(f))) };
  if (Object.keys(cost).length) { tc.cost = new Float32Array(64); for (const [n, v] of Object.entries(cost)) { const t = lib.db.tagIndex.get(n); if (t === undefined) throw new Error(`perform: no tag ${n}`); tc.cost[t] = v; } }
  return tc.want[0] || tc.want[1] || tc.forbid[0] || tc.forbid[1] || tc.cost ? tc : undefined;
}

/** whether frame k passes the tag conditions, and their cost there (Infinity where it doesn't) */
export function tagCost(lib: MotionLibrary, k: number, tc: TagCond): number {
  const T = lib.db!.tags, a = T[k * 2], b = T[k * 2 + 1];
  if (((a & tc.want[0]) >>> 0) !== tc.want[0] >>> 0 || ((b & tc.want[1]) >>> 0) !== tc.want[1] >>> 0 || (a & tc.forbid[0]) || (b & tc.forbid[1])) return Infinity;
  let s = 0;
  if (tc.cost) for (let t = 0; t < 64; t++) if (tc.cost[t] && ((t < 32 ? a >>> t : b >>> (t - 32)) & 1)) s += tc.cost[t];
  return s;
}
