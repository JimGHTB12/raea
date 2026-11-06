// The skeleton's types: joints, skeletons, poses.
import { type Vec3 } from "@raea/render/3d";
import { Side, withFingers } from "./all";

export type Quat = [number, number, number, number];

export interface Joint {
  name: string;
  /** the parent's index, -1 for the root (the pelvis) */
  parent: number;
  /** where the joint is at rest, and where its bone ends (the next joint's head, mostly): metres, the body's frame */
  head: Vec3;
  tail: Vec3;
  /** a finger's joint: its thickness (the median of its skin's distance from the bone), metres */
  radius?: number;
}

/** A body's skeleton, as projects/psx/export.py writes it (skeleton.json): MakeHuman's game_engine joints (bones.ts). The body's frame: y up, facing +z, x its left, feet on
 * y = 0. */
export interface Skeleton {
  name: string;
  height: number;
  body?: Record<string, number>;
  joints: Joint[];
  /** between the eyes; how far the hip joints sit over a seat (the back of the thigh under them) */
  landmarks: { eyes: Vec3; sitDrop?: number };
  /** samples of the skin's surface: each a rest vertex of body.glb, its offset `o` from joint j[0]'s head, its normal
   * `n` at rest, and its joints `j` and weights `w` as the glb skins it */
  surface?: { part: string; side: Side | ""; j: number[]; w: number[]; o: Vec3; n: Vec3 }[];
}

/** The joints at a pose as the skin follows them (each body.glb bone is the joint of its name): each's place and
 * world turn, the fingers' too. */
export function skinJoints(skel: Skeleton, w: Posed): { name: string; pos: Vec3; rot: Quat }[] {
  w = withFingers(skel, w);
  return skel.joints.map((j, i) => ({ name: j.name, pos: w.pos[i], rot: w.rot[i] }));
}

/** A pose: where the root (the pelvis) is in the body's frame, and each joint's turn from rest relative to its parent (by
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
