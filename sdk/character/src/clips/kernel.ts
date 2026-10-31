// The kernel: queries scored over blocks of frames, the best K.
import { FEATURE_COUNT as NF, STATE_COUNT as NS, type MotionLibrary, type SearchQuery } from "../matching";
import { DbLibrary, FLAG, FS, HW, QF, SS, popcount, qround } from "./all";

// ---------------------------------------------------------------- the kernel's scoring

/** The query's words, as the kernel reads them */
export const QW = { mode: 0, skipA: 1, skipB: 2, maskBits: 3, want0: 4, want1: 5, forbid0: 6, forbid1: 7, wq: 8, phaseOn: 9, phaseTo: 10, phaseM: 11, pwq: 12, phaseFar: 13, hq: 14, dims: 15, q: 16, want: 43, cost: 64, hdc: 128, length: 136 } as const;

/** mode bits: the exact cost, the Hamming distance */
export const MODE = { exact: 1, hamming: 2, motion: 4 } as const;

export const SCORE_MAX = 0x7fffffff, SCORE_CAP = (1 << 30) - 1, BLOCK = 64;

export interface KernelQuery { words: Int32Array; pen: Int32Array }

/** A search query as the kernel's words: quantised, its weights in 1/256, each clip's pen in 1/QF² units (an action's,
 * or an infinite one, excluded) */
export function kernelQuery(lib: DbLibrary, q: SearchQuery, mode: number = MODE.exact, hdc?: { words: Uint32Array; weight: number }): KernelQuery {
  const W = new Int32Array(QW.length), U = QF * QF;
  W[QW.mode] = mode;
  W[QW.skipA] = q.skip ? q.skip[0] : 0; W[QW.skipB] = q.skip ? q.skip[1] : 0;
  W[QW.maskBits] = q.mask ? (q.mask === lib.crouch ? FLAG.crouch : (() => { throw new Error("kernelQuery: only the crouch mask is a flag"); })()) : 0;
  if (q.tags) { W[QW.want0] = q.tags.want[0] | 0; W[QW.want1] = q.tags.want[1] | 0; W[QW.forbid0] = q.tags.forbid[0] | 0; W[QW.forbid1] = q.tags.forbid[1] | 0; if (q.tags.cost) for (let t = 0; t < 64; t++) W[QW.cost + t] = Math.max(0, Math.min(1 << 24, Math.round(q.tags.cost[t] * U))); }
  let dims = 0, nd = 0;
  if (q.want) for (let i = 0; i < NS; i++) if (!Number.isNaN(q.want[i])) { dims |= 1 << i; nd++; W[QW.want + i] = Math.max(-32767, Math.min(32767, qround(q.want[i] * QF))); }
  W[QW.dims] = dims;
  W[QW.wq] = nd ? Math.min(1500, Math.round(((q.ws * q.ws) / nd) * 256)) : 0;
  for (let d = 0; d < NF; d++) W[QW.q + d] = Math.max(-32767, Math.min(32767, qround(q.z[d] * QF)));
  if (q.phase) { W[QW.phaseOn] = 1; W[QW.phaseTo] = q.phase.to; W[QW.phaseM] = q.phase.m; W[QW.pwq] = Math.min(512, Math.round(q.phase.w * 256)); W[QW.phaseFar] = Math.min(SCORE_CAP, Math.round(q.phase.w * 50 * U)); }
  if (hdc) { W[QW.hq] = Math.max(0, Math.min(1 << 22, Math.round(hdc.weight * U))); for (let w = 0; w < HW; w++) W[QW.hdc + w] = hdc.words[w] | 0; }
  const pen = new Int32Array(lib.clips.length);
  for (let c = 0; c < pen.length; c++) { const p = q.pen ? q.pen[c] : 0; pen[c] = lib.acting[c] || !(p < Infinity) ? -1 : Math.min(SCORE_CAP, Math.round(p * U)); }
  return { words: W, pen };
}

/** a saturating add, as the kernel's `cap&s+cap&t`: each term capped, then the sum */
const sat = (a: number, b: number) => Math.min(a + Math.min(b, SCORE_CAP), SCORE_CAP);

/** Frame k's integer score for a query (i32; was the kgpus's mdb_score.k kernel, term for term). Method B. */
export function kernelScore(lib: DbLibrary, Q: KernelQuery, k: number, H?: Uint32Array): number {
  const W = Q.words, { F, S, T, flags, clip } = lib.full, n = lib.size;
  const fl = flags[k];
  if (!(fl & FLAG.ok)) return SCORE_MAX;
  if (k >= W[QW.skipA] && k < W[QW.skipB]) return SCORE_MAX;
  if (fl & W[QW.maskBits]) return SCORE_MAX;
  const t0 = T[k * 2] | 0, t1 = T[k * 2 + 1] | 0;
  if ((t0 & W[QW.want0]) !== W[QW.want0] || (t1 & W[QW.want1]) !== W[QW.want1] || (t0 & W[QW.forbid0]) || (t1 & W[QW.forbid1])) return SCORE_MAX;
  const pen = Q.pen[clip[k]];
  if (pen < 0) return SCORE_MAX;
  let s = 0;
  const mode = W[QW.mode];
  if (mode & MODE.exact) {
    s = pen;
    let f = 0;
    const o = k * FS;
    for (let d = 0; d < NF; d++) { const e = Math.min(Math.abs(F[o + d] - W[QW.q + d]), 4095); f += e * e; }
    s = sat(s, f);
    const dims = W[QW.dims];
    if (dims) {
      let st = 0;
      for (let i = 0; i < NS; i++) if ((dims >> i) & 1) { const e = Math.min(Math.abs(S[k * SS + i] - W[QW.want + i]), 2047); st += e * e; }
      s = sat(s, Math.imul(st >> 6, W[QW.wq]) >> 2);
    }
    if (W[QW.phaseOn]) {
      const k2 = k + W[QW.phaseM];
      if (k2 >= n || clip[k2] !== clip[k]) s = sat(s, W[QW.phaseFar]);
      else {
        let p = 0;
        const a = k2 * FS, b = W[QW.phaseTo] * FS;
        for (let d = 0; d < 15; d++) { const e = Math.min(Math.abs(F[a + d] - F[b + d]), 4095); p += e * e; }
        s = sat(s, Math.imul(p >> 6, W[QW.pwq]) >> 2);
      }
    }
    let c = 0;
    for (let t = 0; t < 64; t++) if (((t < 32 ? t0 >>> t : t1 >>> (t - 32)) & 1)) c += W[QW.cost + t];
    s = sat(s, c);
  }
  if (mode & MODE.hamming && H) {
    let hm = 0;
    for (let w = 0; w < HW; w++) hm += popcount((H[k * HW + w] ^ W[QW.hdc + w]) >>> 0);
    s = sat(s, Math.imul(hm, W[QW.hq]));
  }
  return s;
}

/** kernels/mdb_block.k in TS: each BLOCK frames' least score */
export function blockMins(lib: DbLibrary, Q: KernelQuery, H?: Uint32Array): Int32Array {
  const nb = Math.ceil(lib.size / BLOCK), out = new Int32Array(nb);
  for (let b = 0; b < nb; b++) {
    let m = SCORE_MAX;
    for (let k = b * BLOCK, e = Math.min(lib.size, k + BLOCK); k < e; k++) { const s = kernelScore(lib, Q, k, H); if (s < m) m = s; }
    out[b] = m;
  }
  return out;
}

/** the K best of a score array (lowest first, ties by index), as indices; those at SCORE_MAX left out */
export function topK(scores: ArrayLike<number>, K: number): number[] {
  const best: number[] = [];
  for (let i = 0; i < scores.length; i++) {
    const s = scores[i];
    if (s === SCORE_MAX) continue;
    if (best.length === K && s >= scores[best[K - 1]]) continue;
    let j = best.length === K ? K - 1 : best.length;
    if (best.length < K) best.push(i);
    while (j > 0 && scores[best[j - 1]] > s) { best[j] = best[j - 1]; j--; }
    best[j] = i;
  }
  return best;
}

/** The K best of a score array (lowest first, ties by index), in linear time: a histogram of the scores' top bits
 * finds the K-th's bucket, and only what is below it is sorted. Those at SCORE_MAX are left out. */
export function topKFast(scores: Int32Array, K: number): Int32Array {
  const hist = new Int32Array(32769);
  for (let i = 0; i < scores.length; i++) if (scores[i] !== SCORE_MAX) hist[scores[i] >= 0 ? (scores[i] >>> 16) : 0]++;
  let acc = 0, cut = 32768;
  for (let b = 0; b < 32768; b++) { acc += hist[b]; if (acc >= K) { cut = b; break; } }
  const pick: number[] = [];
  for (let i = 0; i < scores.length; i++) { const s = scores[i]; if (s !== SCORE_MAX && (s >= 0 ? s >>> 16 : 0) <= cut) pick.push(i); }
  pick.sort((a, b) => scores[a] - scores[b] || a - b);
  return Int32Array.from(pick.slice(0, K));
}

/** the frames of the blocks given */
export function blockFrames(lib: MotionLibrary, blocks: number[]): Int32Array {
  const out: number[] = [];
  for (const b of blocks) for (let k = b * BLOCK, e = Math.min(lib.size, k + BLOCK); k < e; k++) out.push(k);
  return Int32Array.from(out);
}
