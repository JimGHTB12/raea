// The library: its frames' features, mirrored, as a table.
import type { Vec3 } from "raea/3d";
import { captured, joint, pose, qrot, qyaw, type Pose, type Quat, type Skeleton } from "raea/character";
import { framePose, type Clip } from "../clip";
import { fromBase64, toBase64 } from "@raea/video/codecs";
import { styleOf, type StyleVector } from "../style";
import { FLIP_STATE, FUTURE, MIRROR_STATE, MotionLibrary, NF, NS, STATE, scale, sub3 } from "./all";

// ---------------------------------------------------------------- the library

/** the yaw a pose faces: the hips' forward on the ground */
function facing(rot: Quat): number {
  const f = qrot(rot, [0, 0, 1]);
  return Math.atan2(f[0], f[2]);
}

const SWAP = (n: string) => n.startsWith("Left") ? "Right" + n.slice(4) : n.startsWith("Right") ? "Left" + n.slice(5) : n[0] === "L" && /^L[A-Z]/.test(n) ? "R" + n.slice(1) : n[0] === "R" && /^R[A-Z]/.test(n) ? "L" + n.slice(1) : n;

/** A pose seen in a mirror across the body's x = 0: left and right swapped. */
export function mirror(skel: Skeleton, p: Pose): Pose {
  // (as many turns as the pose has, the capture's joints at least: a missing one is unturned)
  const turns = skel.joints.slice(0, Math.max(captured(skel), p.turns.length)).map((j) => {
    const q = p.turns[joint(skel, SWAP(j.name))] ?? [0, 0, 0, 1];
    return [q[0], -q[1], -q[2], q[3]] as Quat;
  });
  return { root: [-p.root[0], p.root[1], p.root[2]], turns };
}

/** The pose a library frame plays (as captured: the root on the clip's own ground). */
export function libraryPose(lib: MotionLibrary, k: number): Pose {
  const r = lib.clips[lib.frameClip[k]], p = framePose(r.clip, lib.frameF[k]);
  return r.mirrored ? mirror(lib.skel, p) : p;
}

export interface LibraryOptions {
  /** the clips to walk with (by name; all by default) */
  locomotion?: string[];
  /** feature weights: the feet's places, their speeds, the hips' speed, the route's places and its facings */
  weights?: { feet?: number; feetVel?: number; hips?: number; route?: number; facing?: number };
  /** leave out the mirror images */
  noMirror?: boolean;
  /** clips that play whole as actions, never matched into: sitting down (`down`, frames of the clip), sat (`sat`) and
   * standing up (`up`) */
  sit?: { clip: string; down: [number, number]; sat: [number, number]; up: [number, number] };
  /** clips that play whole as actions in `act` steps (a run's jump over a hurdle, a duck under a bar), never
   * matched into */
  actions?: string[];
  /** the per-frame tables `libraryTable` wrote for these clips (one, or one per clip file): loaded instead of worked
   * out */
  table?: LibraryTable | LibraryTable[];
}

/** A library's per-frame table, worked out once and stored (`writeLibraryTable`): for each of its clips' frames (not
 * the mirror images, which follow from them) the features before normalising, the ground frame, the feet's contact
 * and the style, each column quantised to 16 bits between its least and most, base64. */
export interface LibraryTable {
  skeleton: string;
  fps: number;
  clips: { name: string; frames: number }[];
  /** columns: NF features, 3 ground, 2 contact, then the STATE measurements */
  columns: number;
  lo: number[];
  hi: number[];
  data: string;
}

const libs = new WeakMap<Map<string, Clip>, Map<string, MotionLibrary>>();

/** Each clip frame's raw row (features before normalising, NaN where the clip doesn't run on a second; ground x, z,
 * yaw; contacts L, R; the STATE measurements): what a table or a motion database stores. */
export function libraryRows(skel: Skeleton, list: { name: string; c: Clip }[]): Float32Array {
  return rawRows(skel, list.map((x) => ({ ...x, acting: false })));
}

/** each unmirrored clip frame's raw row: features (NaN near the end), ground, contact, state */
function rawRows(skel: Skeleton, list: { name: string; c: Clip; acting: boolean }[]): Float32Array {
  const C = NF + 5 + NS;
  const total = list.reduce((a, x) => a + x.c.frames, 0);
  const rows = new Float32Array(total * C).fill(NaN);
  const L = ["LeftFoot", "RightFoot", "LeftToeBase", "RightToeBase"].map((x) => joint(skel, x));
  const ankleRest = skel.joints[L[0]].head[1], toeRest = skel.joints[L[2]].head[1];
  let base = 0;
  for (const { c } of list) {
    const n = c.frames, fps = c.fps;
    const ground = new Float32Array(n * 3), feet: Vec3[][] = [], hips: Vec3[] = [];
    for (let f = 0; f < n; f++) {
      const p = framePose(c, f), w = pose(skel, p);
      ground[f * 3] = p.root[0]; ground[f * 3 + 1] = p.root[2]; ground[f * 3 + 2] = facing(w.rot[0]);
      feet.push(L.map((i) => w.pos[i]));
      hips.push(p.root);
    }
    const local = (k: number, p: Vec3): Vec3 => qrot(qyaw(-ground[k * 3 + 2]), [p[0] - ground[k * 3], p[1], p[2] - ground[k * 3 + 1]]);
    const localDir = (k: number, d: Vec3): Vec3 => qrot(qyaw(-ground[k * 3 + 2]), d);
    // the style of each second, every half second, given to the frames nearest its middle
    const win = Math.round(fps), hop = Math.max(1, Math.round(fps / 2));
    const styles: { mid: number; v: StyleVector }[] = [];
    for (let f = 0; f + win <= Math.max(win, n); f += hop) styles.push({ mid: f + win / 2, v: styleOf(skel, c, f, f + win) });
    for (let k = 0; k < n; k++) {
      const row = rows.subarray((base + k) * C, (base + k + 1) * C);
      const a = Math.max(0, k - 1), b = Math.min(n - 1, k + 1);
      // a foot stands while its ankle or its ball is low and still
      for (let s = 0; s < 2; s++) {
        const speed = (q: number) => Math.hypot(...sub3(feet[b][q], feet[a][q])) * fps / Math.max(1, b - a);
        const ankle = feet[k][s][1] < ankleRest + 0.03 && speed(s) < 0.25;
        const ball = feet[k][2 + s][1] < toeRest + 0.025 && speed(2 + s) < 0.25;
        row[NF + 3 + s] = ankle || ball ? 1 : 0;
      }
      row[NF] = ground[k * 3]; row[NF + 1] = ground[k * 3 + 1]; row[NF + 2] = ground[k * 3 + 2];
      let best = styles[0];
      for (const st of styles) if (Math.abs(st.mid - k) < Math.abs(best.mid - k)) best = st;
      STATE.forEach((key, i) => (row[NF + 5 + i] = best.v[key]));
      // features in the frame's ground frame: the pose's always (a query reads them from any frame, an action's
      // last too), the route's where the clip runs on far enough
      const v = row.subarray(0, NF), b1 = Math.min(n - 1, k + 1);
      let o = 0;
      for (const q of [0, 1]) v.set(local(k, feet[k][q]), (o += 3) - 3);
      for (const q of [0, 1]) v.set(localDir(k, scale(sub3(feet[b1][q], feet[a][q]), fps / Math.max(1, b1 - a))), (o += 3) - 3);
      v.set(localDir(k, scale(sub3(hips[b1], hips[a]), fps / Math.max(1, b1 - a))), (o += 3) - 3);
      if (k >= n - FUTURE[FUTURE.length - 1]) continue;
      for (const t of FUTURE) {
        const p = local(k, [ground[(k + t) * 3], 0, ground[(k + t) * 3 + 1]]);
        v[o++] = p[0];
        v[o++] = p[2];
      }
      for (const t of FUTURE) {
        const y = ground[(k + t) * 3 + 2] - ground[k * 3 + 2];
        v[o++] = Math.sin(y);
        v[o++] = Math.cos(y);
      }
    }
    base += n;
  }
  return rows;
}

/** a raw row seen in a mirror: left and right swapped, x and yaws negated */
function mirrorRow(r: Float32Array, out: Float32Array) {
  out.set(r);
  // the feet's places and speeds: left and right swapped, x negated
  for (const g of [0, 6]) for (let a = 0; a < 3; a++) { out[g + a] = r[g + 3 + a]; out[g + 3 + a] = r[g + a]; }
  for (const i of [0, 3, 6, 9, 12, 15, 17, 19, 21, 23, 25]) out[i] = -out[i];
  out[NF] = -r[NF]; out[NF + 2] = -r[NF + 2];
  out[NF + 3] = r[NF + 4]; out[NF + 4] = r[NF + 3];
  for (let i = 0; i < NS; i++) out[NF + 5 + i] = r[NF + 5 + MIRROR_STATE[i]] * FLIP_STATE[i];
}

/** The library a character matches against, built once per clip set and options. */
export function motionLibrary(skel: Skeleton, clips: Map<string, Clip>, o: LibraryOptions = {}): MotionLibrary {
  const { table, ...rest } = o;
  const key = JSON.stringify([skel.name, rest]);
  let byKey = libs.get(clips);
  if (!byKey) libs.set(clips, (byKey = new Map()));
  const hit = byKey.get(key);
  if (hit) return hit;
  const names = [...(o.locomotion ?? [...clips.keys()])];
  if (o.sit && !names.includes(o.sit.clip)) names.push(o.sit.clip);
  for (const a of o.actions ?? []) if (!names.includes(a)) names.push(a);
  const acting = new Set([...(o.sit ? [o.sit.clip] : []), ...(o.actions ?? [])]);
  const list = names.map((name) => {
    const c = clips.get(name);
    if (!c) throw new Error(`motionLibrary: no clip ${name}`);
    return { name, c, acting: acting.has(name) };
  });
  const fps = list[0].c.fps;
  for (const x of list) if (x.c.fps !== fps) throw new Error(`motionLibrary: ${x.name} is ${x.c.fps} fps, the rest ${fps}`);
  const C = NF + 5 + NS;
  let raw0: Float32Array;
  if (table) {
    raw0 = readTable(table, list);
  } else raw0 = rawRows(skel, list);
  const n0 = raw0.length / C, mirrors = o.noMirror ? 1 : 2, n = n0 * mirrors;
  const raw = new Float32Array(n * C);
  raw.set(raw0);
  if (mirrors === 2) for (let k = 0; k < n0; k++) mirrorRow(raw0.subarray(k * C, (k + 1) * C), raw.subarray((n0 + k) * C, (n0 + k + 1) * C));
  const ranges: MotionLibrary["clips"] = [];
  const frameClip = new Int32Array(n), frameF = new Int32Array(n);
  let at = 0;
  for (let m = 0; m < mirrors; m++) for (const x of list) {
    for (let f = 0; f < x.c.frames; f++) { frameClip[at + f] = ranges.length; frameF[at + f] = f; }
    ranges.push({ name: x.name, start: at, end: at + x.c.frames, mirrored: m === 1, clip: x.c });
    at += x.c.frames;
  }
  const ground = new Float32Array(n * 3), contact = new Uint8Array(n * 2), state = new Float32Array(n * NS), jump = new Uint8Array(n);
  for (const r of ranges) if (!acting.has(r.name)) for (let k = r.start; k < r.end; k++) jump[k] = Number.isNaN(raw[k * C + 15]) ? 0 : 1;
  for (let k = 0; k < n; k++) {
    ground.set(raw.subarray(k * C + NF, k * C + NF + 3), k * 3);
    contact[k * 2] = raw[k * C + NF + 3]; contact[k * 2 + 1] = raw[k * C + NF + 4];
    state.set(raw.subarray(k * C + NF + 5, (k + 1) * C), k * NS);
  }
  const w = { feet: 0.75, feetVel: 1, hips: 1, route: 1, facing: 1.5, ...o.weights };
  const groups: [number, number, number][] = [[0, 6, w.feet], [6, 12, w.feetVel], [12, 15, w.hips], [15, 21, w.route], [21, 27, w.facing]];
  const mean = new Float32Array(NF), sd = new Float32Array(NF), scl = new Float32Array(NF);
  // over the frames a search may jump to
  let m = 0;
  for (let k = 0; k < n; k++) if (jump[k]) { m++; for (let d = 0; d < NF; d++) mean[d] += raw[k * C + d]; }
  for (let d = 0; d < NF; d++) mean[d] /= m || 1;
  for (let k = 0; k < n; k++) if (jump[k]) for (let d = 0; d < NF; d++) sd[d] += (raw[k * C + d] - mean[d]) ** 2;
  for (const [a, b, wt] of groups) {
    // one spread per group (so a group's dimensions keep their proportions), and its weight
    let s = 0;
    for (let d = a; d < b; d++) s += Math.sqrt(sd[d] / (m || 1));
    s = s / (b - a) || 1;
    for (let d = a; d < b; d++) scl[d] = wt / s;
  }
  const features = new Float32Array(n * NF);
  for (let k = 0; k < n; k++) for (let d = 0; d < NF; d++) features[k * NF + d] = (raw[k * C + d] - mean[d]) * scl[d];
  const stateMean = new Float32Array(NS), stateSd = new Float32Array(NS);
  for (let k = 0; k < n; k++) for (let i = 0; i < NS; i++) stateMean[i] += state[k * NS + i] / n;
  for (let k = 0; k < n; k++) for (let i = 0; i < NS; i++) stateSd[i] += (state[k * NS + i] - stateMean[i]) ** 2 / n;
  for (let i = 0; i < NS; i++) stateSd[i] = Math.sqrt(stateSd[i]) || 1;
  const lib: MotionLibrary = { skel, fps, size: n, frameClip, frameF, clips: ranges, features, jump, scale: scl, mean, ground, contact, state, stateMean, stateSd };
  if (o.sit) {
    const r = ranges.find((x) => x.name === o.sit!.clip && !x.mirrored)!;
    const atF = (f: number) => r.start + Math.min(Math.max(0, f), r.end - r.start - 1);
    const s = o.sit;
    // the clip's seat: under the sat hips by the body's drop, on the ground frame the clip faces while sat
    const mid = atF(Math.round((s.sat[0] + s.sat[1]) / 2));
    const hips = libraryPose(lib, mid).root;
    const yaw = ground[mid * 3 + 2];
    const start = libraryPose(lib, atF(s.down[0])).root;
    // how far before the seat it stands to sit (along the way it faces), for the walk there
    const back = qrot(qyaw(-yaw), [start[0] - hips[0], 0, start[2] - hips[2]]);
    lib.sit = { down: [atF(s.down[0]), atF(s.down[1])], sat: [atF(s.sat[0]), atF(s.sat[1])], up: [atF(s.up[0]), atF(s.up[1])], seat: [hips[0], hips[1] - (skel.landmarks.sitDrop ?? 0.075), hips[2]], yaw, stand: [back[0], back[2]] };
  }
  byKey.set(key, lib);
  return lib;
}

/** The per-frame table of a set of clips (features, ground, contact, style), to store and load with the clips. */
export function libraryTable(skel: Skeleton, clips: Map<string, Clip>, names: string[] = [...clips.keys()]): LibraryTable {
  const list = names.map((name) => ({ name, c: clips.get(name)!, acting: false }));
  const rows = rawRows(skel, list), C = NF + 5 + NS, n = rows.length / C;
  const lo = new Array(C).fill(Infinity), hi = new Array(C).fill(-Infinity);
  for (let k = 0; k < n; k++) for (let d = 0; d < C; d++) { const v = rows[k * C + d]; if (!Number.isNaN(v)) { lo[d] = Math.min(lo[d], v); hi[d] = Math.max(hi[d], v); } }
  for (let d = 0; d < C; d++) if (!(hi[d] > lo[d])) { lo[d] = Number.isFinite(lo[d]) ? lo[d] : 0; hi[d] = lo[d] + 1; }
  // 0 is NaN; 1..65535 the column's range
  const q = new Uint16Array(n * C);
  for (let k = 0; k < n * C; k++) { const d = k % C, v = rows[k]; q[k] = Number.isNaN(v) ? 0 : 1 + Math.round(((v - lo[d]) / (hi[d] - lo[d])) * 65534); }
  return { skeleton: skel.name, fps: list[0].c.fps, clips: list.map((x) => ({ name: x.name, frames: x.c.frames })), columns: C, lo, hi, data: toBase64(new Uint8Array(q.buffer)) };
}

const tableData = new WeakMap<LibraryTable, Uint16Array>();

function readTable(tables: LibraryTable | LibraryTable[], list: { name: string; c: Clip }[]): Float32Array {
  const C = NF + 5 + NS;
  const where = new Map<string, { t: LibraryTable; at: number; frames: number }>();
  for (const t of Array.isArray(tables) ? tables : [tables]) {
    if (t.columns !== C) throw new Error(`motionLibrary: a table has ${t.columns} columns, this library ${C}; write it again`);
    let at = 0;
    for (const c of t.clips) { where.set(c.name, { t, at, frames: c.frames }); at += c.frames; }
  }
  const total = list.reduce((a, x) => a + x.c.frames, 0), out = new Float32Array(total * C);
  let base = 0;
  for (const x of list) {
    const w = where.get(x.name);
    if (!w) throw new Error(`motionLibrary: no table has the clip ${x.name}`);
    if (w.frames !== x.c.frames) throw new Error(`motionLibrary: the table's ${x.name} has ${w.frames} frames, the clip ${x.c.frames}`);
    let q = tableData.get(w.t);
    if (!q) { const b = fromBase64(w.t.data); q = new Uint16Array(b.buffer, b.byteOffset, b.byteLength / 2); tableData.set(w.t, q); }
    const { lo, hi } = w.t;
    for (let k = 0; k < x.c.frames * C; k++) {
      const d = k % C, v = q[w.at * C + k];
      out[base * C + k] = v === 0 ? NaN : lo[d] + ((v - 1) / 65534) * (hi[d] - lo[d]);
    }
    base += x.c.frames;
  }
  return out;
}
