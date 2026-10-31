// Building a database from clips.
import { framePose, type Clip } from "../clip";
import { bodyScale, joint, pose, qrot, qyaw, type Skeleton } from "raea/character";
import { FEATURE_COUNT as NF, STATE_COUNT as NS, libraryRows, levelClip, swingClear } from "../matching";
import { DbClip, FLAG, FS, HLEVELS, HSTEP, HW, MF_SIGN, MF_SRC, MS_SIGN, MS_SRC, QF, SS, TAGS, TAG_INDEX, Tag, hdcEncode, q16, tagsOf, writeDb } from "./all";

// ---------------------------------------------------------------- building

export interface BuildClip { name: string; source: string; file: string; label: string; move?: string; clip: Clip }

/** The database of a body's clips (each stood on a level floor first, as play stands it). */
export function buildDb(skel: Skeleton, clips: Iterable<BuildClip>, log: (s: string) => void = () => {}): Uint8Array {
  const C = NF + 5 + NS, metas: DbClip[] = [], rowParts: Float32Array[] = [], hipParts: Uint16Array[] = [], spdParts: Uint16Array[] = [], movParts: Uint8Array[] = [], crParts: Uint8Array[] = [], rootParts: Float32Array[] = [];
  let n0 = 0, fps = 0;
  const t0 = performance.now();
  const handJ = ["LeftHand", "RightHand"].map((x) => joint(skel, x)), handParts: Int16Array[] = [], clearParts: Int16Array[] = [];
  for (const b of clips) {
    const c = levelClip(skel, b.clip);
    if (!fps) fps = c.fps;
    if (c.fps !== fps) { log(`skip ${b.name}: ${c.fps} fps, the rest ${fps}`); continue; }
    if (c.frames < 2) continue;
    const rows = libraryRows(skel, [{ name: b.name, c }]);
    rowParts.push(rows);
    const n = c.frames, hip = new Uint16Array(n), spd = new Uint16Array(n), mov = new Uint8Array(n), cr = new Uint8Array(n);
    const ys = Float32Array.from({ length: n }, (_, f) => c.root[f * 3 + 1]), med = Float32Array.from(ys).sort()[n >> 1];
    for (let f = 0; f < n; f++) {
      hip[f] = Math.max(0, Math.min(65535, Math.round((ys[f] / med) * 1000)));
      if (ys[f] < 0.92 * med) for (let j = Math.max(0, f - 12); j <= f; j++) cr[j] = 1;
      const a = Math.max(0, f - 15), e = Math.min(n - 1, f + 15), G = (k: number) => [rows[k * C + NF], rows[k * C + NF + 1]];
      const v = e > a ? (Math.hypot(G(e)[0] - G(a)[0], G(e)[1] - G(a)[1]) * fps) / (e - a) : 0;
      spd[f] = Math.min(65535, Math.round(v * 100));
      mov[f] = v > 0.15 ? 1 : 0;
    }
    // the hands in the frame's ground frame (as the feet's features are), mm
    // and each hand's swing clearance, mm
    const U = new Int16Array(n * 6), K = new Int16Array(n * 2);
    for (let f = 0; f < n; f++) {
      const w = pose(skel, framePose(c, f)), gx = rows[f * C + NF], gz = rows[f * C + NF + 1], gy = rows[f * C + NF + 2];
      (["L", "R"] as const).forEach((sd, s) => { K[f * 2 + s] = Math.max(-32767, Math.min(32767, Math.round(swingClear(skel, w, sd) * 1000))); });
      handJ.forEach((j, s) => {
        const p = qrot(qyaw(-gy), [w.pos[j][0] - gx, w.pos[j][1], w.pos[j][2] - gz]);
        for (let d = 0; d < 3; d++) U[f * 6 + s * 3 + d] = Math.max(-32767, Math.min(32767, Math.round(p[d] * 1000)));
      });
    }
    handParts.push(U); clearParts.push(K);
    rootParts.push(ys);
    hipParts.push(hip); spdParts.push(spd); movParts.push(mov); crParts.push(cr);
    metas.push({ name: b.name, source: b.source, file: b.file, label: b.label, move: b.move, tags: tagsOf(b.label, b.source, b.move), start: n0, frames: n });
    n0 += n;
    if (metas.length % 100 === 0) log(`${metas.length} clips, ${n0} frames, ${((performance.now() - t0) / 1000).toFixed(0)} s`);
  }
  const raw = new Float32Array(n0 * C);
  { let at = 0; for (const r of rowParts) { raw.set(r, at); at += r.length; } }
  const cat8 = (ps: Uint8Array[]) => { const o = new Uint8Array(n0); let a = 0; for (const p of ps) { o.set(p, a); a += p.length; } return o; };
  const cat16 = (ps: Uint16Array[]) => { const o = new Uint16Array(n0); let a = 0; for (const p of ps) { o.set(p, a); a += p.length; } return o; };
  const hip = cat16(hipParts), speed = cat16(spdParts), moving = cat8(movParts), crouch = cat8(crParts);
  // flags; "ok" (matchable): a route second here and 6 frames on, in the same clip
  const flags = new Uint8Array(n0), clipOf = new Int32Array(n0);
  metas.forEach((m, i) => clipOf.fill(i, m.start, m.start + m.frames));
  for (let k = 0; k < n0; k++) {
    const route = !Number.isNaN(raw[k * C + 15]);
    flags[k] = (route ? FLAG.route : 0) | (raw[k * C + NF + 3] ? FLAG.contactL : 0) | (raw[k * C + NF + 4] ? FLAG.contactR : 0) | (crouch[k] ? FLAG.crouch : 0) | (moving[k] ? FLAG.moving : 0);
  }
  for (let k = 0; k < n0; k++) if (flags[k] & FLAG.route && k + 6 < n0 && clipOf[k + 6] === clipOf[k] && flags[k + 6] & FLAG.route) flags[k] |= FLAG.ok;
  // normalising over the matchable frames and their mirror images (motionLibrary's, with its groups and weights),
  // worked out so a mirror image's columns are exactly the frame's permuted and negated
  const sum = new Float64Array(NF), sum2 = new Float64Array(NF);
  let m = 0;
  for (let k = 0; k < n0; k++) if (flags[k] & FLAG.route) { m++; for (let d = 0; d < NF; d++) { const v = raw[k * C + d]; sum[d] += v; sum2[d] += v * v; } }
  const mean = Array.from({ length: NF }, (_, d) => (sum[d] + MF_SIGN[d] * sum[MF_SRC[d]]) / (2 * m));
  const sd = Array.from({ length: NF }, (_, d) => Math.sqrt(Math.max(0, (sum2[d] + sum2[MF_SRC[d]]) / (2 * m) - mean[d] * mean[d])));
  const w = { feet: 0.75, feetVel: 1, hips: 1, route: 1, facing: 1.5 };
  const groups: [number, number, number][] = [[0, 6, w.feet], [6, 12, w.feetVel], [12, 15, w.hips], [15, 21, w.route], [21, 27, w.facing]];
  const scale = new Array<number>(NF).fill(1);
  for (const [a, b, wt] of groups) { let s = 0; for (let d = a; d < b; d++) s += sd[d]; s = s / (b - a) || 1; for (let d = a; d < b; d++) scale[d] = wt / s; }
  const ssum = new Float64Array(NS), ssum2 = new Float64Array(NS);
  for (let k = 0; k < n0; k++) for (let i = 0; i < NS; i++) { const v = raw[k * C + NF + 5 + i]; ssum[i] += v; ssum2[i] += v * v; }
  const stateMean = Array.from({ length: NS }, (_, i) => (ssum[i] + MS_SIGN[i] * ssum[MS_SRC[i]]) / (2 * n0));
  const stateSd = Array.from({ length: NS }, (_, i) => Math.sqrt(Math.max(0, (ssum2[i] + ssum2[MS_SRC[i]]) / (2 * n0) - stateMean[i] ** 2)) || 1);
  const F = new Int16Array(n0 * FS), S = new Int16Array(n0 * SS), G = new Float32Array(n0 * 3), T = new Uint32Array(n0 * 2), H = new Uint32Array(n0 * HW);
  for (let k = 0; k < n0; k++) {
    for (let d = 0; d < NF; d++) F[k * FS + d] = q16((raw[k * C + d] - mean[d]) * scale[d]);
    for (let i = 0; i < NS; i++) S[k * SS + i] = q16((raw[k * C + NF + 5 + i] - stateMean[i]) / stateSd[i]);
    G[k * 3] = raw[k * C + NF]; G[k * 3 + 1] = raw[k * C + NF + 1]; G[k * 3 + 2] = raw[k * C + NF + 2];
  }
  const ankleRest = skel.joints.find((j) => j.name === "LeftFoot")!.head[1], restHips = skel.joints[0].head[1], hz = bodyScale(skel);
  const hipY = new Float32Array(n0);
  { let a = 0; for (const b of rootParts) { hipY.set(b, a); a += b.length; } }
  const clipBits = metas.map((mt) => { const b: [number, number] = [0, 0]; for (const t of mt.tags) { const i = TAG_INDEX.get(t)!; b[i >> 5] |= 1 << (i & 31); } return b; });
  const bit = (t: Tag) => 1 << (TAG_INDEX.get(t)! & 31), hiWord = (t: Tag) => TAG_INDEX.get(t)! >= 32;
  for (let k = 0; k < n0; k++) {
    let [a, b] = clipBits[clipOf[k]];
    const meas: Tag[] = [moving[k] ? "moving" : "still"];
    if (speed[k] > 200) meas.push("fast");
    if (crouch[k]) meas.push("crouched");
    // stood on something: a planted foot's ankle 6 cm over its rest height off the floor, or the hips 8% over standing
    // (a ladder, a stair, a rock: "clean" and "Unspecified" captures say nothing of it)
    const up = (s: 0 | 1) => raw[k * C + NF + 3 + s] && raw[k * C + 1 + 3 * s] > ankleRest + 0.06 * hz;
    if (up(0) || up(1) || hipY[k] > 1.08 * restHips) meas.push("raised");
    for (const t of meas) { if (hiWord(t)) b |= bit(t); else a |= bit(t); }
    T[k * 2] = a >>> 0; T[k * 2 + 1] = b >>> 0;
    hdcEncode(F, k * FS, T[k * 2], T[k * 2 + 1], H, k * HW);
  }
  log(`${metas.length} clips, ${n0} frames in ${((performance.now() - t0) / 1000).toFixed(0)} s`);
  const U = new Int16Array(n0 * 6);
  { let at = 0; for (const u of handParts) { U.set(u, at); at += u.length; } }
  const K = new Int16Array(n0 * 2);
  { let at = 0; for (const u of clearParts) { K.set(u, at); at += u.length; } }
  return writeDb({ header: { version: 3, skeleton: skel.name, fps, frames: n0, nf: NF, ns: NS, qf: QF, mean, scale, stateMean, stateSd, tags: [...TAGS], hdc: { words: HW, levels: HLEVELS, step: HSTEP }, mirror: { f: MF_SRC, fs: MF_SIGN, s: MS_SRC, ss: MS_SIGN }, clips: metas }, F, S, G, flags, hip, speed, T, H, U, K });
}
