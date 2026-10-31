// Motion clips for raea/character: BVH read, retargeted onto a character's skeleton, stored compactly and
// played back at any time (slerped between frames). CMU's motion capture (mocap.cs.cmu.edu, free for any use; cgspeed's
// BVH conversion) is on the same 31 joints as a character, and a character's skin is bound in CMU's T-pose, so a clip's
// turns play nearly as they are: each bone takes a small correction for what's left between the two rests (CMU's legs
// splayed, its spine straighter), and the hips' travel is scaled by the legs' length.
import { fromBase64, toBase64 } from "@raea/video/codecs";
import { captured, joint, pose, qaxis, qfromTo, qinv, qmul, qnorm, slerp, sub, type Pose, type Quat, type Skeleton } from "raea/character";
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
}

/** A BVH frame's turns (from its rest, relative to the parent) and its root's place, in BVH units. */
function bvhFrame(b: Bvh, values: number[]): { root: Vec3; turns: Quat[] } {
  const DEG = Math.PI / 180;
  let c = 0;
  let root: Vec3 = [0, 0, 0];
  const turns = b.joints.map((j) => {
    let q: Quat = [0, 0, 0, 1];
    for (const ch of j.channels) {
      const v = values[c++];
      if (ch === "Xposition") root = [v, root[1], root[2]];
      else if (ch === "Yposition") root = [root[0], v, root[2]];
      else if (ch === "Zposition") root = [root[0], root[1], v];
      // the channels in their order, each about the frame the ones before left (R = R1·R2·R3)
      else q = qmul(q, qaxis(ch[0] === "X" ? [1, 0, 0] : ch[0] === "Y" ? [0, 1, 0] : [0, 0, 1], v * DEG));
    }
    return q;
  });
  return { root, turns };
}

/** the child whose head a joint's bone points at (a joint with several follows its main line) */
const MAIN: Record<string, string> = { Hips: "LowerBack", Spine1: "Neck", LeftHand: "LeftFingerBase", RightHand: "RightFingerBase" };
/** the bones that branch off a parent's pivot to a joint beside the chain: the hip joints and the shoulders */
const BRANCH = /^([LR]HipJoint|(Left|Right)Shoulder)$/;
/** the body's axis: the back, neck and head */
const AXIS = /^(LowerBack|Spine1?|Neck1?|Head)$/;
/** the neck and head, held about the clip's mean */
const CARRIAGE = /^(Neck1?|Head)$/;

/** A BVH retargeted onto a skeleton, resampled to `fps`, from `from` to `to` seconds. Its joints are found by the
 * skeleton's names, or by `map` (the skeleton's name to the BVH's; a joint left out of a map, or missing from the BVH,
 * holds still against its parent: fingers a BVH hasn't). `carriage`: the neck and head move about this (a clip's
 * `carriage` from the same actor) rather than the clip's own average. The BVH's up is y and its subject faces +z at rest (cgspeed's
 * CMU files and 100STYLE's do); its units are scaled by the legs. */
export function retarget(skel: Skeleton, b: Bvh, o: { name: string; fps?: number; from?: number; to?: number; map?: Record<string, string>; carriage?: Record<string, Quat> }): Clip {
  const fps = o.fps ?? 30;
  // (the capture's joints: the fingers' own come after them and no BVH turns them)
  const J = skel.joints.slice(0, captured(skel));
  // cgspeed's files call the fingers' joint HandIndex1
  const alias = (n: string) => n.replace("HandFinger1", "HandIndex1");
  const src = J.map((j) => {
    if (o.map) return o.map[j.name] ? b.joints.findIndex((x) => x.name === o.map![j.name]) : -1;
    const i = b.joints.findIndex((x) => x.name === j.name || x.name === alias(j.name));
    if (i < 0) throw new Error(`retarget ${o.name}: the BVH has no joint ${j.name}`);
    return i;
  });
  if (src[0] < 0) throw new Error(`retarget ${o.name}: the BVH has no joint for the root ${J[0].name}`);
  if (o.map) for (const [t, n] of Object.entries(o.map)) if (!b.joints.some((x) => x.name === n)) throw new Error(`retarget ${o.name}: the map sends ${t} to ${n}, which the BVH hasn't`);
  // the BVH's joints at rest, in the world (its rest frames are the world's)
  const restAt: Vec3[] = [];
  b.joints.forEach((j, i) => { restAt[i] = j.parent < 0 ? [0, 0, 0] : [restAt[j.parent][0] + j.offset[0], restAt[j.parent][1] + j.offset[1], restAt[j.parent][2] + j.offset[2]]; });
  // each bone at rest in both: towards its main child's head, looking past children at the same place (cgspeed's
  // hand, its finger base and its thumb are all at the wrist), or its end site
  const bone = (i: number) => {
    const child = (j: number) => (MAIN[J[j].name] ? joint(skel, MAIN[J[j].name]) : J.findIndex((x) => x.parent === j));
    let c = child(i), s: Vec3 | undefined;
    for (let j = i; ; ) {
      const cj = child(j);
      if (cj < 0) { s = b.joints[src[i]].end; break; }
      if (src[cj] >= 0) {
        const d = sub(restAt[src[cj]], restAt[src[i]]);
        if (Math.hypot(...d) > 1e-6) { s = d; break; }
      }
      j = cj;
    }
    const t = c >= 0 ? sub(J[c].head, J[i].head) : sub(J[i].tail, J[i].head);
    if (Math.hypot(...t) < 1e-6) c = -1;
    return { s, t: c >= 0 ? t : sub(J[i].tail, J[i].head) };
  };
  // the correction a joint's turn takes so the target's bone lies along the source's: both stand in a T-pose at rest,
  // but CMU's legs are splayed. Small, so a shortest turn keeps each bone's roll (bound in Anny's A-pose the arms took
  // 48-degree corrections that twisted them and broke the shoulders). Not for the root, nor the back, neck and head:
  // CMU's are a straight stick leaning back, and laid along it Anny's spine lost its curve and her neck, 20 degrees
  // forward at rest, walked 9 back, the head pushed back over it and the chin into the throat. They keep Anny's build
  // and take the capture's turns from rest as they are. Nor the bones that branch off a parent's pivot to a joint
  // beside it (the hip joints off the pelvis, the shoulders off the chest): those differ in build, and turning them
  // moves the joint they lead to (corrected, Anny's hip joints went 6 cm down and forward and the whole body rode
  // up). Those take their parent's correction, so they turn with it as the body is built
  const fix: Quat[] = [];
  J.forEach((j, i) => {
    if (i === 0 || AXIS.test(j.name)) fix[i] = [0, 0, 0, 1];
    else if (BRANCH.test(j.name) || src[i] < 0) fix[i] = fix[j.parent];
    else {
      const { s, t } = bone(i);
      fix[i] = s && Math.hypot(...s) > 1e-6 && Math.hypot(...t) > 1e-6 ? qfromTo(t, s) : [0, 0, 0, 1];
    }
  });
  // the legs' length in both, for the hips' travel
  const legOf = (pts: (n: string) => Vec3) => Math.hypot(...sub(pts("LeftUpLeg"), pts("LeftLeg"))) + Math.hypot(...sub(pts("LeftLeg"), pts("LeftFoot")));
  const k = legOf((n) => J[joint(skel, n)].head) / legOf((n) => restAt[src[joint(skel, n)]]);
  const dur = (b.frames.length - 1) * b.frameTime;
  // (cgspeed's files open on a T-pose frame: left out)
  const t0 = Math.max(b.frameTime, o.from ?? 0), t1 = Math.min(dur, o.to ?? dur);
  const n = Math.max(1, Math.floor((t1 - t0) * fps) + 1);
  const root = new Float32Array(n * 3), turns = new Float32Array(n * J.length * 4);
  for (let f = 0; f < n; f++) {
    const at = (t0 + f / fps) / b.frameTime;
    const a = Math.min(b.frames.length - 1, Math.floor(at)), c = Math.min(b.frames.length - 1, a + 1), u = at - a;
    const A = bvhFrame(b, b.frames[a]), B = bvhFrame(b, b.frames[c]);
    // the source's world turns, then the target's: G_t = G_s · fix, and its turn relative to the parent's G_t
    const gs: Quat[] = [], gt: Quat[] = [];
    b.joints.forEach((j, i) => {
      const q = slerp(A.turns[i], B.turns[i], u);
      gs[i] = j.parent < 0 ? q : qmul(gs[j.parent], q);
    });
    for (let i = 0; i < J.length; i++) {
      const p = J[i].parent;
      // a joint the BVH hasn't holds still against its parent
      gt[i] = src[i] >= 0 ? qnorm(qmul(gs[src[i]], fix[i])) : p < 0 ? [0, 0, 0, 1] : gt[p];
      const l = p < 0 ? gt[i] : qmul(qinv(gt[p]), gt[i]);
      turns.set(l, (f * J.length + i) * 4);
    }
    const r: Vec3 = [(A.root[0] + (B.root[0] - A.root[0]) * u) * k, (A.root[1] + (B.root[1] - A.root[1]) * u) * k, (A.root[2] + (B.root[2] - A.root[2]) * u) * k];
    root.set(r, f * 3);
  }
  // the neck and head carry themselves as Anny does, moving about it as the capture moves about its average: CMU's
  // subjects hold the neck 10 degrees back and the head 16 down on a clip's mean, a chin pulled into the throat. The
  // average is the clip's own, or `carriage` (another clip's, its actor at their most neutral) so a style's own
  // carriage (an old man's hanging head) stays
  const carriage: Record<string, Quat> = {};
  for (let i = 0; i < J.length; i++) {
    if (!CARRIAGE.test(J[i].name)) continue;
    let mq = o.carriage?.[J[i].name];
    if (!mq) {
      const m = [0, 0, 0, 0];
      for (let f = 0; f < n; f++) {
        const o4 = (f * J.length + i) * 4, sgn = turns[o4 + 3] < 0 ? -1 : 1;
        for (let a = 0; a < 4; a++) m[a] += sgn * turns[o4 + a];
      }
      mq = qnorm(m as Quat);
    }
    carriage[J[i].name] = mq;
    const off = qinv(mq);
    for (let f = 0; f < n; f++) {
      const o4 = (f * J.length + i) * 4;
      turns.set(qmul(off, Array.from(turns.subarray(o4, o4 + 4)) as Quat), o4);
    }
  }
  // feet on the floor: the clip lifted so its lowest toe, over the clip, is at the toes' height at rest
  const clip: Clip = { name: o.name, fps, frames: n, root, turns, carriage };
  const toes = ["LeftToeBase", "RightToeBase"].map((x) => joint(skel, x));
  let low = Infinity;
  for (let f = 0; f < n; f++) {
    const w = pose(skel, clipFrame(clip, f, J.length));
    for (const t of toes) low = Math.min(low, w.pos[t][1]);
  }
  const lift = J[toes[0]].head[1] - low;
  for (let f = 0; f < n; f++) root[f * 3 + 1] += lift;
  return clip;
}

function clipFrame(c: Clip, f: number, nj: number): Pose {
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
  clips: { name: string; fps: number; frames: number; root: string; turns: string }[];
}

export function writeClips(skel: Skeleton, clips: Clip[]): ClipFile {
  return {
    skeleton: skel.name,
    joints: skel.joints.slice(0, captured(skel)).map((j) => j.name),
    clips: clips.map((c) => {
      const q = new Int16Array(c.turns.length);
      for (let i = 0; i < q.length; i++) q[i] = Math.round(Math.max(-1, Math.min(1, c.turns[i])) * 32767);
      return { name: c.name, fps: c.fps, frames: c.frames, root: toBase64(new Uint8Array(c.root.buffer.slice(0))), turns: toBase64(new Uint8Array(q.buffer)) };
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
    m.set(c.name, { name: c.name, fps: c.fps, frames: c.frames, root, turns });
  }
  decoded.set(file, m);
  return m;
}


