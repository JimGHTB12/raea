// The nearest frame: by features, by the hips' height, by the hands.
import type { Vec3 } from "raea/3d";
import { bodyScale } from "raea/character";
import { AHEAD, MotionLibrary, NF, NS, PhaseGoal, S, TagCond, phaseCost, tagCost } from "./all";

/** The library frame nearest a query (raw features `q`), by perform's cost: the feature distance, the state's, the
 * clip's pen, the phase goal's and the tags'; frames skipped, masked or failing the tags never. Every frame is costed
 * (the reference), or only those the library's search backend names. */
export function nearest(lib: MotionLibrary, q: Float32Array, skip?: [number, number], want?: Float32Array, ws = 1, pen?: Float32Array, phase?: PhaseGoal, mask?: Uint8Array, tags?: TagCond, swing?: Float32Array): { k: number; d: number } {
  const z = new Float32Array(NF);
  for (let d = 0; d < NF; d++) z[d] = (q[d] - lib.mean[d]) * lib.scale[d];
  const tc = tags && lib.db ? tags : undefined;
  const cands = lib.search ? lib.search.candidates(lib, { z, skip, want, ws, pen, phase, mask, tags: tc }) : undefined;
  const r = nearestZ(lib, z, cands, skip, want, ws, pen, phase, mask, tc, swing);
  // (no frame clear of a thing held: the clearance let go; then no frame with what the tags ask: the tags, rather
  // than no frame at all)
  if (r.k < 0 && swing) return nearest(lib, q, skip, want, ws, pen, phase, mask, tags);
  if (r.k < 0 && tc) return nearest(lib, q, skip, want, ws, pen, phase, mask);
  return r;
}

/** nearest for a normalised query, over the candidates given (every frame without) */
export function nearestZ(lib: MotionLibrary, z: Float32Array, cands: ArrayLike<number> | undefined, skip?: [number, number], want?: Float32Array, ws = 1, pen?: Float32Array, phase?: PhaseGoal, mask?: Uint8Array, tc?: TagCond, swing?: Float32Array): { k: number; d: number } {
  const n = cands ? cands.length : lib.size, F = lib.features, S = lib.state;
  const dims: number[] = [];
  if (want) for (let i = 0; i < NS; i++) if (!Number.isNaN(want[i])) dims.push(i);
  // the state's weight spread over the dimensions it names, so naming more doesn't outweigh the route
  const wd = dims.length ? (ws * ws) / dims.length : 0;
  let best = -1, bd = Infinity;
  for (let c = 0; c < n; c++) {
    const k = cands ? cands[c] : c;
    if ((skip && k >= skip[0] && k < skip[1]) || (mask && mask[k])) continue;
    const o = k * NF;
    // (a frame with a few frames' play after it: one at a clip's last matchable frame runs out a frame on, and two such
    // were switched between every frame)
    if (!lib.jump[k] || lib.jump[k + AHEAD] !== 1 || lib.frameClip[k + AHEAD] !== lib.frameClip[k]) continue;
    let s = pen ? pen[lib.frameClip[k]] : 0;
    if (swing) s += swing[k];
    if (tc) s += tagCost(lib, k, tc);
    if (phase) s += phaseCost(lib, k, phase);
    // (over candidates in any order, a tie goes to the lower frame, as the scan in order gives it)
    const tie = !!cands;
    for (let i = 0; i < dims.length && (s < bd || (tie && s === bd)); i++) {
      const j = dims[i], e = (S[k * NS + j] - lib.stateMean[j]) / lib.stateSd[j] - want![j];
      s += wd * e * e;
    }
    for (let d = 0; d < NF && (s < bd || (tie && s === bd)); d++) {
      const e = F[o + d] - z[d];
      s += e * e;
    }
    if (s < bd || (tie && s === bd && k < best)) { bd = s; best = k; }
  }
  return { k: best, d: bd };
}

/** the upper body's search (limbs.upper): nearestZ's cost, with each named hand's distance from where it should be
 * (in a tenth of the body's size: 5 cm off costs as much as a feature a whole spread off, by HAND_W) */
const HAND_W = 4;

export function nearestHands(lib: MotionLibrary, z: Float32Array, hands: Partial<Record<"L" | "R", Vec3>>, skip?: [number, number], want?: Float32Array, ws = 1, pen?: Float32Array, tc?: TagCond, swing?: Float32Array): { k: number; d: number } {
  const H = lib.hands!, F = lib.features, S = lib.state, u = 0.1 * bodyScale(lib.skel);
  const goals = (["L", "R"] as const).flatMap((s, i) => (hands[s] ? [[i * 3, hands[s]!] as const] : []));
  const dims: number[] = [];
  if (want) for (let i = 0; i < NS; i++) if (!Number.isNaN(want[i])) dims.push(i);
  const wd = dims.length ? (ws * ws) / dims.length : 0;
  let best = -1, bd = Infinity;
  for (let k = 0; k < lib.size; k++) {
    if (skip && k >= skip[0] && k < skip[1]) continue;
    if (!lib.jump[k] || lib.jump[k + AHEAD] !== 1 || lib.frameClip[k + AHEAD] !== lib.frameClip[k]) continue;
    let s = pen ? pen[lib.frameClip[k]] : 0;
    if (swing) s += swing[k];
    if (tc) s += tagCost(lib, k, tc);
    for (let g = 0; g < goals.length && s < bd; g++) {
      const [o, p] = goals[g];
      for (let d = 0; d < 3; d++) { const e = (H[k * 6 + o + d] - p[d]) / u; s += HAND_W * e * e; }
    }
    for (let i = 0; i < dims.length && s < bd; i++) {
      const j = dims[i], e = (S[k * NS + j] - lib.stateMean[j]) / lib.stateSd[j] - want![j];
      s += wd * e * e;
    }
    for (let d = 0; d < NF && s < bd; d++) { const e = F[k * NF + d] - z[d]; s += e * e; }
    if (s < bd) { bd = s; best = k; }
  }
  return { k: best, d: bd };
}

/** an upper body's hands' cost at frame k (as nearestHands counts it) */
export function handCost(lib: MotionLibrary, k: number, hands: Partial<Record<"L" | "R", Vec3>>): number {
  const H = lib.hands!, u = 0.1 * bodyScale(lib.skel);
  let s = 0;
  (["L", "R"] as const).forEach((sd, i) => { const p = hands[sd]; if (p) for (let d = 0; d < 3; d++) { const e = (H[k * 6 + i * 3 + d] - p[d]) / u; s += HAND_W * e * e; } });
  return s;
}
