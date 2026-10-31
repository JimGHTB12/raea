// A pick's or throw's capture clip, a grip's turn, and what interact gives back.
import type { Vec3 } from "raea/3d";
import { len, palmAt, pose, qrot, qyaw, sub, type Pose, type Quat, type Side } from "raea/character";
import { libraryPose, type MotionLibrary } from "../matching";
import { type Handling, type Thing } from "../things";
import { G, ItemClip, wrap } from "./all";

/** A capture for a pick or a throw measured in the library: the act to play (the mirror image if the capture's own
 * hand, the one that travels furthest, isn't `hand`), its frames, its key (a pick's: the hand lowest; a throw's: the
 * hand fastest), and from the hips at the key, turned to its way (where it goes, if it steps on 20 cm or more: a throw
 * steps toward where it throws; else the way its hips face at its first frame), the palm there, where the hips stand
 * at its first and last frames and the way they face then. */
interface ClipMove { act: string; from: number; to: number; key: number; palm: Vec3; start: Vec3; end: Vec3; startYaw: number; endYaw: number }

export function clipMove(lib: MotionLibrary, c: ItemClip, kind: "pick" | "throw", hand: Side): ClipMove {
  const name = typeof c === "string" ? c : c.name, skel = lib.skel;
  const own = lib.clips.find((r) => r.name === name && !r.mirrored);
  if (!own) throw new Error(`planItems: no action clip ${name} (motionLibrary's actions option)`);
  const n = own.end - own.start, from = typeof c === "string" ? 0 : c.from ?? 0, to = Math.min(n - 1, typeof c === "string" ? n - 1 : c.to ?? n - 1);
  const palms = (r: typeof own, s: Side) => Array.from({ length: to - from + 1 }, (_, j) => palmAt(skel, pose(skel, libraryPose(lib, r.start + from + j)), s));
  const travel = (ps: Vec3[]) => ps.reduce((acc, q, j) => acc + (j ? len(sub(q, ps[j - 1])) : 0), 0);
  const mine: Side = travel(palms(own, "L")) > travel(palms(own, "R")) ? "L" : "R";
  const r = mine === hand ? own : lib.clips.find((x) => x.name === name && x.mirrored)!;
  const ps = palms(r, hand);
  let key = 0;
  for (let j = 1; j < ps.length - 1; j++) {
    const better = kind === "pick" ? ps[j][1] < ps[key][1] : len(sub(ps[j + 1], ps[j - 1])) > (key ? len(sub(ps[key + 1], ps[key - 1])) : 0);
    if (better) key = j;
  }
  const k = r.start + from + key, G = lib.ground, a = (r.start + from) * 3, b = (r.start + to) * 3;
  const y0 = Math.hypot(G[b] - G[a], G[b + 1] - G[a + 1]) >= 0.2 ? Math.atan2(G[b] - G[a], G[b + 1] - G[a + 1]) : G[a + 2];
  const local = (p: Vec3): Vec3 => qrot(qyaw(-y0), [p[0] - G[k * 3], p[1], p[2] - G[k * 3 + 1]]);
  const at = (f: number): Vec3 => local([G[(r.start + f) * 3], 0, G[(r.start + f) * 3 + 1]]);
  return {
    act: name + (r.mirrored ? "~" : ""), from, to, key: from + key, palm: local(ps[key]), start: at(from), end: at(to),
    startYaw: wrap(G[a + 2] - y0), endYaw: wrap(G[b + 2] - y0),
  };
}

/** The thing's turn in the grip's frame: a box (and a round thing) squared to the body at the nearest quarter turn
 * from how it stands, a pole across the body for two hands and along the way for one, a thing by its handle as it
 * stands. */
export function gripTurn(it: Thing, how: Handling, hand: Side, face: number): number {
  const rel = wrap((it.yaw ?? 0) - face), q = Math.PI / 2;
  // a handle (already toward the hand: planItems checks) is carried as it faces, turning it round would wring the wrist
  if (how.grip === "handle") return rel;
  if (how.grip === "pole") {
    // its length (z) across (odd quarters) or along (even)
    const k = Math.round(rel / q), odd = ((k % 2) + 2) % 2 === 1;
    return (how.hands === 2) === odd ? k * q : (k + (rel > k * q ? 1 : -1)) * q;
  }
  return Math.round(rel / q) * q;
}

export interface Interaction {
  frames: number;
  pose(frame: number): Pose;
  /** where a thing is at a frame: its bottom's middle and its turn */
  item(id: string, frame: number): { at: Vec3; rot: Quat };
  /** whether a thing is in hand at a frame */
  held(id: string, frame: number): boolean;
  /** how far the palm furthest from its goal is from it at a frame, metres, while the hands are fully on their goals
   * (0 otherwise): what the body can't reach */
  miss(frame: number): number;
}

/** A thing's state at a frame */
export interface Track { at: Vec3; rot: Quat; held: boolean }

/** A thing let go: falling, flying, bouncing or rolling, its bottom's middle at `at` */
export interface Flight { at: Vec3; vel: Vec3; rot: Quat; spin: Vec3; landed: boolean; settle: number; bounces: number }
