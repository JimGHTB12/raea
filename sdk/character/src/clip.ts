// Motion clips for raea/character: BVH read, retargeted onto a character's skeleton (retarget.ts), stored compactly
// and played back at any time (slerped between frames).
import { fromBase64, toBase64 } from "@raea/video/codecs";
import { captured, qnorm, slerp, type Pose, type Quat, type Skeleton } from "raea/character";
import type { Vec3 } from "raea/3d";

export interface Bvh {
  joints: { name: string; parent: number; offset: Vec3; channels: string[]; end?: Vec3 }[];
  frameTime: number;
  /** each frame's channel values, in the joints' order */
  frames: number[][];
}

/** A BVH file's hierarchy and motion. */
export function readBvh(text: string): Bvh {
  const tok = text.split(/\s+/).filter(Boolean);
  let k = 0;
  const joints: Bvh["joints"] = [];
  const stack: number[] = [];
  while (tok[k] !== "MOTION") {
    const t = tok[k++];
    if (t === "ROOT" || t === "JOINT") {
      joints.push({ name: tok[k++], parent: stack.length ? stack[stack.length - 1] : -1, offset: [0, 0, 0], channels: [] });
    } else if (t === "End") {
      k++; // Site
      k++; // {
      k++; // OFFSET
      joints[stack[stack.length - 1]].end = [+tok[k++], +tok[k++], +tok[k++]];
      k++; // }
    } else if (t === "{") {
      stack.push(joints.length - 1);
    } else if (t === "}") {
      stack.pop();
    } else if (t === "OFFSET") {
      joints[joints.length - 1].offset = [+tok[k++], +tok[k++], +tok[k++]];
    } else if (t === "CHANNELS") {
      const n = +tok[k++];
      joints[joints.length - 1].channels = tok.slice(k, k + n);
      k += n;
    } else if (t !== "HIERARCHY") {
      throw new Error(`readBvh: unexpected ${t}`);
    }
  }
  k++; // MOTION
  k++; // Frames:
  const n = +tok[k++];
  k += 2; // Frame Time:
  const frameTime = +tok[k++];
  const width = joints.reduce((a, j) => a + j.channels.length, 0);
  const frames: number[][] = [];
  for (let f = 0; f < n; f++, k += width) frames.push(tok.slice(k, k + width).map(Number));
  return { joints, frameTime, frames };
}

/** A clip on a skeleton: per frame the root's place and every joint's turn (Pose's terms). */
export interface Clip {
  name: string;
  fps: number;
  frames: number;
  /** frames x 3 */
  root: Float32Array;
  /** frames x joints x 4 */
  turns: Float32Array;
  /** the neck's and head's mean turns the retarget took out (its actor's carriage), for retargeting the actor's other
   * clips about the same */
  carriage?: Record<string, Quat>;
  /** the body's hip height over the capture's actor's (the retarget scaled the travel by it): what the cadence is
   * scaled by in play, as Froude's dynamic similarity has it (matching/perform.ts) */
  scale?: number;
}

export function clipFrame(c: Clip, f: number, nj: number): Pose {
  const turns: Quat[] = [];
  for (let i = 0; i < nj; i++) {
    const o = (f * nj + i) * 4;
    turns.push([c.turns[o], c.turns[o + 1], c.turns[o + 2], c.turns[o + 3]]);
  }
  return { root: [c.root[f * 3], c.root[f * 3 + 1], c.root[f * 3 + 2]], turns };
}

/** A clip's pose at `t` seconds (looped, or held at its ends), slerped between its frames. */
export function clipPose(c: Clip, t: number, o: { loop?: boolean } = {}): Pose {
  const nj = c.turns.length / (c.frames * 4);
  let x = t * c.fps;
  x = o.loop ? ((x % c.frames) + c.frames) % c.frames : Math.min(Math.max(x, 0), c.frames - 1);
  const a = Math.floor(x), b = o.loop ? (a + 1) % c.frames : Math.min(a + 1, c.frames - 1), u = x - a;
  const A = clipFrame(c, a, nj), B = clipFrame(c, b, nj);
  return {
    root: [A.root[0] + (B.root[0] - A.root[0]) * u, A.root[1] + (B.root[1] - A.root[1]) * u, A.root[2] + (B.root[2] - A.root[2]) * u],
    turns: A.turns.map((q, i) => slerp(q, B.turns[i], u)),
  };
}

/** A pose's frame of a clip (no blending). */
export function framePose(c: Clip, f: number): Pose {
  return clipFrame(c, Math.min(Math.max(0, f), c.frames - 1), c.turns.length / (c.frames * 4));
}

/** A set of clips for one skeleton as JSON: the root's places as float32 and the turns as int16 (a part in 32767),
 * base64. */
export interface ClipFile {
  skeleton: string;
  joints: string[];
  clips: { name: string; fps: number; frames: number; root: string; turns: string; scale?: number }[];
}

export function writeClips(skel: Skeleton, clips: Clip[]): ClipFile {
  return {
    skeleton: skel.name,
    joints: skel.joints.slice(0, captured(skel)).map((j) => j.name),
    clips: clips.map((c) => {
      const q = new Int16Array(c.turns.length);
      for (let i = 0; i < q.length; i++) q[i] = Math.round(Math.max(-1, Math.min(1, c.turns[i])) * 32767);
      return { name: c.name, fps: c.fps, frames: c.frames, root: toBase64(new Uint8Array(c.root.buffer.slice(0))), turns: toBase64(new Uint8Array(q.buffer)), ...(c.scale ? { scale: Math.round(c.scale * 1e4) / 1e4 } : {}) };
    }),
  };
}

const decoded = new WeakMap<ClipFile, Map<string, Clip>>();
/** The clips of a file, by name (decoded once per file). */
export function readClips(file: ClipFile): Map<string, Clip> {
  let m = decoded.get(file);
  if (m) return m;
  m = new Map();
  for (const c of file.clips) {
    const rb = fromBase64(c.root), tb = fromBase64(c.turns);
    const root = new Float32Array(rb.buffer.slice(rb.byteOffset, rb.byteOffset + rb.byteLength));
    const q = new Int16Array(tb.buffer.slice(tb.byteOffset, tb.byteOffset + tb.byteLength));
    const turns = new Float32Array(q.length);
    for (let i = 0; i < q.length; i += 4) {
      const v = qnorm([q[i] / 32767, q[i + 1] / 32767, q[i + 2] / 32767, q[i + 3] / 32767]);
      turns.set(v, i);
    }
    m.set(c.name, { name: c.name, fps: c.fps, frames: c.frames, root, turns, ...(c.scale ? { scale: c.scale } : {}) });
  }
  decoded.set(file, m);
  return m;
}

export { retarget, retargetIK, SHRUG, type RetargetOptions } from "./retarget";
