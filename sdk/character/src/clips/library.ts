// The library a database reads as.
import { readClips, type Clip, type ClipFile } from "../clip";
import { pose, type Skeleton } from "raea/character";
import { FEATURE_COUNT as NF, STATE_COUNT as NS, levelClip, type MotionLibrary } from "../matching";
import { FLAG, FS, HW, MF_SIGN, MF_SRC, MS_SIGN, MS_SRC, MotionDb, NAN16, QF, SS, hdcEncode } from "./all";

// ---------------------------------------------------------------- the library

/** Where a database's clip files are read from (bun: the file system; raea's V8: a native read). */
export type ClipSource = (file: string) => ClipFile;

export interface DbLibraryOptions {
  /** a reader of the database's clip files (by their paths in the library's directory: clips-bun.ts's reads them) */
  read: ClipSource;
  /** clips that play whole as actions, never matched into */
  actions?: string[];
  /** the sitting action, as motionLibrary's */
  sit?: { clip: string; down: [number, number]; sat: [number, number]; up: [number, number] };
}

/** A database's library in memory: the arrays perform reads, mirror images included */
export interface DbLibrary extends MotionLibrary {
  mdb: MotionDb;
  /** the mirrored-in-full arrays the kernels score (frames x FS, x SS, x 2, flags), and each frame's clip */
  full: { F: Int16Array; S: Int16Array; T: Uint32Array; flags: Uint8Array; clip: Int32Array; H?: Uint32Array };
  /** each clip range's playing (1: an action, never matched into) */
  acting: Uint8Array;
}

/** A matchable library from a motion database: every frame, mirror images derived exactly, the clips read the first
 * time they play (stood level, as the features were measured). */
export function dbLibrary(skel: Skeleton, mdb: MotionDb, o: DbLibraryOptions): DbLibrary {
  const h = mdb.header, n0 = h.frames, n = n0 * 2;
  if (h.nf !== NF || h.ns !== NS) throw new Error(`dbLibrary: the database has ${h.nf}/${h.ns} columns, this library ${NF}/${NS}; build it again`);
  const read = o.read;
  const loaded = new Map<string, Clip>();
  const clipOfName = (name: string, file: string): Clip => {
    let c = loaded.get(name);
    if (c) return c;
    for (const [nm, cl] of readClips(read(file))) if (!loaded.has(nm)) loaded.set(nm, levelClip(skel, cl));
    c = loaded.get(name);
    if (!c) throw new Error(`dbLibrary: ${file} has no clip ${name}`);
    return c;
  };
  const F = new Int16Array(n * FS), S = new Int16Array(n * SS), T = new Uint32Array(n * 2), flags = new Uint8Array(n), clip = new Int32Array(n);
  F.set(mdb.F); S.set(mdb.S); T.set(mdb.T); T.set(mdb.T, n0 * 2); flags.set(mdb.flags);
  for (let k = 0; k < n0; k++) {
    const a = k * FS, b = (n0 + k) * FS;
    for (let d = 0; d < NF; d++) { const v = mdb.F[a + MF_SRC[d]]; F[b + d] = v === NAN16 ? NAN16 : MF_SIGN[d] * v; }
    const sa = k * SS, sb = (n0 + k) * SS;
    for (let i = 0; i < NS; i++) S[sb + i] = MS_SIGN[i] * mdb.S[sa + MS_SRC[i]];
    const f = mdb.flags[k];
    flags[n0 + k] = (f & ~(FLAG.contactL | FLAG.contactR)) | (f & FLAG.contactL ? FLAG.contactR : 0) | (f & FLAG.contactR ? FLAG.contactL : 0);
  }
  const nc = h.clips.length, acts = new Set([...(o.actions ?? []), ...(o.sit ? [o.sit.clip] : [])]);
  const clips: MotionLibrary["clips"] = [];
  const acting = new Uint8Array(nc * 2);
  for (let mi = 0; mi < 2; mi++) h.clips.forEach((c, i) => {
    const start = mi * n0 + c.start;
    clip.fill(mi * nc + i, start, start + c.frames);
    acting[mi * nc + i] = acts.has(c.name) ? 1 : 0;
    const r = { name: c.name, start, end: start + c.frames, mirrored: mi === 1 } as MotionLibrary["clips"][number];
    Object.defineProperty(r, "clip", { enumerable: true, get: () => clipOfName(c.name, c.file) });
    clips.push(r);
  });
  for (const a of acts) if (!h.clips.some((c) => c.name === a)) throw new Error(`dbLibrary: no clip ${a}`);
  // the float arrays perform reads
  const features = new Float32Array(n * NF), state = new Float32Array(n * NS), ground = new Float32Array(n * 3), contact = new Uint8Array(n * 2), jump = new Uint8Array(n), crouch = new Uint8Array(n);
  for (let k = 0; k < n; k++) {
    for (let d = 0; d < NF; d++) { const v = F[k * FS + d]; features[k * NF + d] = v === NAN16 ? NaN : v / QF; }
    for (let i = 0; i < NS; i++) state[k * NS + i] = h.stateMean[i] + h.stateSd[i] * (S[k * SS + i] / QF);
    const f = flags[k];
    contact[k * 2] = f & FLAG.contactL ? 1 : 0; contact[k * 2 + 1] = f & FLAG.contactR ? 1 : 0;
    jump[k] = f & FLAG.route && !acting[clip[k]] ? 1 : 0;
    crouch[k] = f & FLAG.crouch ? 1 : 0;
  }
  // the hands (m), a mirror image's the other hand's with x negated
  let hands: Float32Array | undefined;
  if (mdb.U) {
    hands = new Float32Array(n * 6);
    for (let k = 0; k < n0; k++) for (let s = 0; s < 2; s++) for (let d = 0; d < 3; d++) {
      const v = mdb.U[k * 6 + s * 3 + d] / 1000;
      hands[k * 6 + s * 3 + d] = v;
      hands[(n0 + k) * 6 + (1 - s) * 3 + d] = d === 0 ? -v : v;
    }
  }
  // the swing clearances (m), a mirror image's the other hand's
  let clear: Float32Array | undefined;
  if (mdb.K) {
    clear = new Float32Array(n * 2);
    for (let k = 0; k < n0; k++) for (let s = 0; s < 2; s++) { const v = mdb.K[k * 2 + s] / 1000; clear[k * 2 + s] = v; clear[(n0 + k) * 2 + 1 - s] = v; }
  }
  ground.set(mdb.G);
  for (let k = 0; k < n0; k++) { ground[(n0 + k) * 3] = -mdb.G[k * 3]; ground[(n0 + k) * 3 + 1] = mdb.G[k * 3 + 1]; ground[(n0 + k) * 3 + 2] = -mdb.G[k * 3 + 2]; }
  const frameClip = clip, frameF = new Int32Array(n);
  for (const r of clips) for (let k = r.start; k < r.end; k++) frameF[k] = k - r.start;
  const lib: DbLibrary = {
    skel, fps: h.fps, size: n, frameClip, frameF, clips, features, jump, scale: Float32Array.from(h.scale), mean: Float32Array.from(h.mean), ground, contact, state,
    stateMean: Float32Array.from(h.stateMean), stateSd: Float32Array.from(h.stateSd), db: { tags: T, tagIndex: new Map(h.tags.map((t, i) => [t, i])) }, crouch, mdb, full: { F, S, T, flags, clip }, acting,
    ...(hands ? { hands } : {}), ...(clear ? { clear } : {}),
  };
  if (o.sit) {
    const r = clips.find((x) => x.name === o.sit!.clip && !x.mirrored)!, s = o.sit;
    const atF = (f: number) => r.start + Math.min(Math.max(0, f), r.end - r.start - 1);
    const pose = (k: number) => { const c = r.clip, f = frameF[k]; return [c.root[f * 3], c.root[f * 3 + 1], c.root[f * 3 + 2]] as [number, number, number]; };
    const mid = atF(Math.round((s.sat[0] + s.sat[1]) / 2)), hips = pose(mid), yaw = ground[mid * 3 + 2], start = pose(atF(s.down[0]));
    const dx = start[0] - hips[0], dz = start[2] - hips[2], c = Math.cos(-yaw), si = Math.sin(-yaw);
    lib.sit = { down: [atF(s.down[0]), atF(s.down[1])], sat: [atF(s.sat[0]), atF(s.sat[1])], up: [atF(s.up[0]), atF(s.up[1])], seat: [hips[0], hips[1] - (skel.landmarks.sitDrop ?? 0.075), hips[2]], yaw, stand: [c * dx + si * dz, -si * dx + c * dz] };
  }
  return lib;
}

/** The mirror images' hypervectors (the file holds the frames'), worked out once (the kernel does this on load) */
export function fullHdc(lib: DbLibrary): Uint32Array {
  if (lib.full.H) return lib.full.H;
  const n0 = lib.mdb.header.frames, H = new Uint32Array(lib.size * HW);
  H.set(lib.mdb.H);
  for (let k = n0; k < lib.size; k++) hdcEncode(lib.full.F, k * FS, lib.full.T[k * 2], lib.full.T[k * 2 + 1], H, k * HW);
  return (lib.full.H = H);
}
