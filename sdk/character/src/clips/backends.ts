// Search backends: kernel, HDC, coarse, IVF, recording, exact.
import { FEATURE_COUNT as NF, STATE_COUNT as NS, nearestZ, phaseCost, type MotionLibrary, type SearchBackend, type SearchQuery } from "../matching";
import { BLOCK, DbLibrary, FLAG, FS, HW, MODE, QF, SCORE_MAX, blockFrames, blockMins, fullHdc, hdcEncode, kernelQuery, kernelScore, qround, topK, topKFast } from "./all";

// ---------------------------------------------------------------- backends

/** Method B on the CPU in TS: the kernel's integer scoring, the K best blocks reranked (slow: the kernel's reference) */
export function kernelSearch(K = 16): SearchBackend {
  return { name: `kernel-ts K=${K}`, candidates: (lib, q) => blockFrames(lib, topK(blockMins(lib as DbLibrary, kernelQuery(lib as DbLibrary, q)), K)) };
}

/** The query's hypervector: its quantised features bound as a frame's, bundled with the wanted tags (D) or not (C) */
export function queryHdc(q: SearchQuery, bindTags: boolean): Uint32Array {
  const qq = new Int16Array(NF), out = new Uint32Array(HW);
  for (let d = 0; d < NF; d++) qq[d] = Math.max(-32767, Math.min(32767, qround(q.z[d] * QF)));
  hdcEncode(qq, 0, bindTags && q.tags ? q.tags.want[0] : 0, bindTags && q.tags ? q.tags.want[1] : 0, out, 0, bindTags);
  return out;
}

/** Methods C and D: Hamming over the hypervectors picks the K best frames (with the masks and, unless `bound`, the tags
 * as bit filters: D binds them into the vectors instead, and filters nothing by them), reranked exactly. The frames'
 * vectors bind their tags (the file's); `motionOnly` re-encodes them without (C). */
export function hdcSearch(K = 1024, o: { bound?: boolean; motionOnly?: boolean } = {}): SearchBackend {
  let Hm: Uint32Array | undefined;
  return {
    name: `hdc${o.bound ? "+tags-bound" : o.motionOnly ? "-motion" : ""} K=${K}`,
    candidates: (l, q) => {
      const lib = l as DbLibrary;
      let H = fullHdc(lib);
      if (o.motionOnly) { if (!Hm) { Hm = new Uint32Array(lib.size * HW); for (let k = 0; k < lib.size; k++) hdcEncode(lib.full.F, k * FS, 0, 0, Hm, k * HW, false); } H = Hm; }
      const qs = o.bound ? { ...q, tags: q.tags && { want: [0, 0] as [number, number], forbid: q.tags.forbid, cost: q.tags.cost } } : q;
      const Q = kernelQuery(lib, qs, MODE.hamming, { words: queryHdc(q, !!o.bound), weight: 1 / (QF * QF) });
      const sc = new Int32Array(lib.size);
      for (let k = 0; k < lib.size; k++) sc[k] = kernelScore(lib, Q, k, H);
      return topKFast(sc, K);
    },
  };
}

/** Method E (no GPU): a coarse scan of every `step`th frame by the exact cost, then the BLOCK frames about each of the
 * K best coarse frames reranked */
export function coarseSearch(step = 8, K = 32): SearchBackend {
  return {
    name: `coarse /${step} K=${K}`,
    candidates: (lib, q) => {
      const sub: number[] = [];
      for (let k = 0; k < lib.size; k += step) sub.push(k);
      // the coarse frames' costs: nearestZ over each alone is slow; score them in one pass
      const cand = Int32Array.from(sub), sc = Array.from(cand, (k) => { const c = exactCost(lib, q, k); return c === Infinity ? SCORE_MAX : c; });
      const order = topK(sc, K), out = new Set<number>();
      for (const i of order) { const c = cand[i]; for (let k = Math.max(0, c - BLOCK / 2); k < Math.min(lib.size, c + BLOCK / 2); k++) out.add(k); }
      return Int32Array.from(out);
    },
  };
}

/** Method E': IVF over k-means clusters of the matchable frames' features: the query's `probe` nearest clusters'
 * frames, reranked */
export function ivfSearch(lib: DbLibrary, clusters = 1024, probe = 8, iters = 6): SearchBackend {
  const n = lib.size, F = lib.features, ok: number[] = [];
  for (let k = 0; k < n; k++) if (lib.full.flags[k] & FLAG.ok) ok.push(k);
  // k-means on a sample, then every frame assigned
  let seed = 1;
  const rnd = () => ((seed = Math.imul(seed, 1664525) + 1013904223 | 0) >>> 0) / 4294967296;
  const C = new Float32Array(clusters * NF);
  for (let c = 0; c < clusters; c++) { const k = ok[Math.floor(rnd() * ok.length)]; C.set(F.subarray(k * NF, k * NF + NF), c * NF); }
  const sample = Array.from({ length: Math.min(ok.length, clusters * 64) }, () => ok[Math.floor(rnd() * ok.length)]);
  const nearestC = (k: number) => { let b = 0, bd = Infinity; for (let c = 0; c < clusters; c++) { let s = 0; for (let d = 0; d < NF && s < bd; d++) { const e = F[k * NF + d] - C[c * NF + d]; s += e * e; } if (s < bd) { bd = s; b = c; } } return b; };
  for (let it = 0; it < iters; it++) {
    const acc = new Float64Array(clusters * NF), cnt = new Int32Array(clusters);
    for (const k of sample) { const c = nearestC(k); cnt[c]++; for (let d = 0; d < NF; d++) acc[c * NF + d] += F[k * NF + d]; }
    for (let c = 0; c < clusters; c++) if (cnt[c]) for (let d = 0; d < NF; d++) C[c * NF + d] = acc[c * NF + d] / cnt[c];
  }
  const lists: number[][] = Array.from({ length: clusters }, () => []);
  for (const k of ok) lists[nearestC(k)].push(k);
  const probeWith = (p: number) => (_l: MotionLibrary, q: SearchQuery) => {
    const d2 = new Float64Array(clusters);
    for (let c = 0; c < clusters; c++) { let s = 0; for (let d = 0; d < NF; d++) { const e = q.z[d] - C[c * NF + d]; s += e * e; } d2[c] = s; }
    const order = topK(Array.from(d2), p);
    return Int32Array.from(order.flatMap((c) => lists[c]));
  };
  return { name: `ivf ${clusters}/${probe}`, candidates: probeWith(probe), probeWith } as SearchBackend;
}

/** A backend that records every query it is asked (for the recall bench) and passes them on (to the brute force, by
 * default) */
export function recording(into: SearchQuery[], inner?: SearchBackend): SearchBackend {
  return {
    name: `recording(${inner?.name ?? "js"})`,
    candidates: (lib, q) => {
      into.push({ ...q, z: Float32Array.from(q.z), want: q.want && Float32Array.from(q.want), pen: q.pen && Float32Array.from(q.pen) });
      return inner ? inner.candidates(lib, q) : allFrames(lib);
    },
  };
}

const allCache = new WeakMap<MotionLibrary, Int32Array>();

function allFrames(lib: MotionLibrary): Int32Array { let a = allCache.get(lib); if (!a) allCache.set(lib, (a = Int32Array.from({ length: lib.size }, (_, i) => i))); return a; }

/** A frame's exact cost for a query, perform's (nearestZ's terms, in full): Infinity where it can't be chosen. */
export function exactCost(lib: MotionLibrary, q: SearchQuery, k: number): number {
  if ((q.skip && k >= q.skip[0] && k < q.skip[1]) || (q.mask && q.mask[k])) return Infinity;
  if (!lib.jump[k] || lib.jump[k + 6] !== 1 || lib.frameClip[k + 6] !== lib.frameClip[k]) return Infinity;
  let s = q.pen ? q.pen[lib.frameClip[k]] : 0;
  if (q.tags) {
    const T = lib.db!.tags, a = T[k * 2], b = T[k * 2 + 1], tc = q.tags;
    if (((a & tc.want[0]) >>> 0) !== tc.want[0] >>> 0 || ((b & tc.want[1]) >>> 0) !== tc.want[1] >>> 0 || (a & tc.forbid[0]) || (b & tc.forbid[1])) return Infinity;
    if (tc.cost) for (let t = 0; t < 64; t++) if (tc.cost[t] && ((t < 32 ? a >>> t : b >>> (t - 32)) & 1)) s += tc.cost[t];
  }
  if (q.phase) s += phaseCost(lib, k, q.phase);
  if (q.want) {
    let m = 0;
    for (let i = 0; i < NS; i++) if (!Number.isNaN(q.want[i])) m++;
    const wd = m ? (q.ws * q.ws) / m : 0;
    for (let i = 0; i < NS; i++) if (!Number.isNaN(q.want[i])) { const e = (lib.state[k * NS + i] - lib.stateMean[i]) / lib.stateSd[i] - q.want[i]; s += wd * e * e; }
  }
  for (let d = 0; d < NF; d++) { const e = lib.features[k * NF + d] - q.z[d]; s += e * e; }
  return s;
}

/** the exact best over the candidates (the reference over all frames: nearestZ without candidates) */
export function rerank(lib: MotionLibrary, q: SearchQuery, cands?: ArrayLike<number>) {
  return nearestZ(lib, q.z, cands, q.skip, q.want, q.ws, q.pen, q.phase, q.mask, q.tags);
}
