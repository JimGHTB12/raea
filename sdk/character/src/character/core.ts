// The skeleton's types: joints, skeletons, poses.
import { type Vec3 } from "@raea/render/3d";
import { Side, add, qmul, qrot, slerp, sub, withFingers } from "./all";

export type Quat = [number, number, number, number];

export interface Joint {
  name: string;
  /** the parent's index, -1 for the root (Hips) */
  parent: number;
  /** where the joint is at rest, and where its bone ends (the next joint's head, mostly): metres, the body's frame */
  head: Vec3;
  tail: Vec3;
  /** a finger's joint (after the capture's 31): the capture's joint its skin rode before it had its own (a phalanx 1
   * the finger base, 2 and 3 the finger, the thumb's the thumb). Its turn is on top of that chain's (see `pose`) */
  follow?: number;
  /** a finger's joint: its thickness (the median of its skin's distance from the bone), metres */
  radius?: number;
}

/** A body's skeleton, as character.py writes it (skeleton.json). The body's frame: y up, facing +z, x its left, feet on
 * y = 0. */
export interface Skeleton {
  name: string;
  height: number;
  body?: Record<string, number>;
  joints: Joint[];
  /** between the eyes; how far the hip joints sit over a seat (the back of the thigh under them) */
  landmarks: { eyes: Vec3; sitDrop?: number };
  /** where each joint stands with the body at rest (Anny's own A-pose) and its turn from there to the T-pose */
  bind?: { turns: Quat[]; heads: Vec3[] };
  /** the skin's own joints (character.py binds the skin with Anny's default rig's weights): each at its head at rest,
   * carried by the joint `drive`, turned as `from` slerped `u` of the way to `to` */
  skin?: SkinJoint[];
  /** samples of the skin's surface (`surface`): each a rest vertex of body.glb, its offset `o` from skin joint j[0]'s
   * head, its normal `n` at rest, and its skin joints `j` and weights `w` as the glb has them */
  surface?: { part: string; side: Side | ""; j: number[]; w: number[]; o: Vec3; n: Vec3 }[];
}

export interface SkinJoint {
  name: string;
  head: Vec3;
  drive: number;
  from: number;
  to: number;
  u: number;
}

/** The skin's joints at a pose (the skeleton's own when it has no skin of its own): each's place and world turn. */
export function skinJoints(skel: Skeleton, w: Posed): { name: string; pos: Vec3; rot: Quat }[] {
  w = withFingers(skel, w);
  if (!skel.skin || !skel.bind) return skel.joints.map((j, i) => ({ name: j.name, pos: w.pos[i], rot: w.rot[i] }));
  const { turns, heads } = skel.bind;
  // each joint's turn from the body at rest: from rest to the T-pose, then the pose's from there
  const fromRest = w.rot.map((r, i) => qmul(r, turns[i]));
  return skel.skin.map((s) => ({
    name: s.name,
    pos: add(w.pos[s.drive], qrot(fromRest[s.drive], sub(s.head, heads[s.drive]))),
    rot: s.from === s.to ? fromRest[s.from] : slerp(fromRest[s.from], fromRest[s.to], s.u),
  }));
}

/** A pose: where the root (Hips) is in the body's frame, and each joint's turn from rest relative to its parent (by
 * joint index; a missing one is no turn). */
export interface Pose {
  root: Vec3;
  turns: Quat[];
}

/** A pose drawn: each joint's place and turn in the body's frame. */
export interface Posed {
  pos: Vec3[];
  rot: Quat[];
}
