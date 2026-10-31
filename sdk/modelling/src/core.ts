// A build's types: transforms, views, poses, steps, bones and the Modeller.
import { Solid, type CrossSection } from "@raea/render/solid";

export type V3 = [number, number, number];

/** A transform: scaled, then turned (degrees, Manifold's order: x, then y, then z), about `pivot` (the origin), then moved. */
export interface Xf {
  move?: V3;
  rotate?: V3;
  scale?: number | V3;
  pivot?: V3;
}

/** A camera hint for a step: yaw and pitch in degrees about the part (0, 0 looks along -z, from the front), zoom > 1 nearer. */
export interface View {
  yaw?: number;
  pitch?: number;
  zoom?: number;
}

export interface StepBase {
  beats: number;
  view?: View;
}

/** A pose: each bone's turn (degrees, x y z) about its head in its parent's frame, and `$move`, the root's shift. */
export type Pose = Record<string, V3> & { $move?: V3 };

export type Op =
  | { kind: "add"; object: string; after: Solid; from: V3; at: V3 }
  | { kind: "transform"; object: string; before: Solid; after: Solid; xf: Xf; gizmo: "move" | "rotate" | "scale" }
  | { kind: "boolean"; object: string; fn: "minus" | "union" | "intersect"; before: Solid; tool: Solid; after: Solid }
  | { kind: "mirror"; object: string; before: Solid; copy: Solid; after: Solid; axis: 0 | 1 | 2 }
  | { kind: "extrude"; object: string; before: Solid | null; profile: CrossSection; local: Solid; after: Solid; place: Xf }
  | { kind: "revolve"; object: string; before: Solid | null; stages: Solid[]; after: Solid; place: Xf; profile: CrossSection }
  | { kind: "sculpt"; object: string; before: Solid | null; stages: Solid[]; after: Solid }
  | { kind: "smooth"; object: string; before: Solid; after: Solid }
  | { kind: "array"; object: string; before: Solid; copies: Solid[]; after: Solid; template: string | null }
  | { kind: "bone"; bones: string[] }
  | { kind: "weights"; bones: string[] }
  | { kind: "pose"; fn: (u: number) => Pose }
  | { kind: "finish" }
  | { kind: "animate"; fn: (beat: number) => Pose }
  | { kind: "wait" };

export type Step = Op & StepBase & { start: number; index: number };

export interface Bone {
  name: string;
  head: V3;
  tail: V3;
  parent: string | null;
}

export interface BuildLook {
  /** the viewport's object grey */
  clay?: string;
  /** the finished look's backdrop (a flat colour) and its outline colour */
  background?: string;
  line?: string;
}

export interface Built {
  steps: Step[];
  /** beats, start to end */
  duration: number;
  /** every object's final solid and colour */
  objects: Map<string, { solid: Solid; color: string }>;
  bones: Bone[];
  /** the build's objects as they stand at `at` (beats) */
  stateAt(at: number): Map<string, Solid>;
  /** the rig's pose at `at` */
  poseAt(at: number): Pose;
  /** weights: per object, per vertex, four [bone index, weight] pairs (computed on first use) */
  weights(): Map<string, { joints: Uint16Array; weights: Float32Array }>;
}

export interface Modeller {
  /** a solid made at the origin, popped in at the 3D cursor (`at` moves the cursor there first) */
  add(object: string, solid: Solid, o?: { at?: V3 } & Partial<StepBase>): void;
  /** moved, turned or scaled with a gizmo (which one from the transform, or `gizmo`) */
  transform(object: string, xf: Xf, o?: { gizmo?: "move" | "rotate" | "scale" } & Partial<StepBase>): void;
  move(object: string, by: V3, o?: Partial<StepBase>): void;
  rotate(object: string, degrees: V3, o?: { pivot?: V3 } & Partial<StepBase>): void;
  scale(object: string, by: number | V3, o?: { pivot?: V3 } & Partial<StepBase>): void;
  /** the tool shown as a red ghost sweeping into place, then cut away */
  minus(object: string, tool: Solid, o?: Partial<StepBase>): void;
  /** the tool as a green ghost, then joined */
  union(object: string, tool: Solid, o?: Partial<StepBase>): void;
  intersect(object: string, tool: Solid, o?: Partial<StepBase>): void;
  /** mirrored across the plane through the origin across `axis`, joined to itself: the plane shown, the copy unfolding */
  mirror(object: string, axis: "x" | "y" | "z", o?: Partial<StepBase>): void;
  /** a profile (x-y) extruded along z by `height`, grown from the profile, then placed; joined to the object if it exists */
  extrude(object: string, profile: CrossSection, height: number, o?: { place?: Xf; twist?: number; scaleTop?: [number, number] } & Partial<StepBase>): void;
  /** a profile (x >= 0) revolved about z, swept round, then placed */
  revolve(object: string, profile: CrossSection, o?: { place?: Xf; segments?: number } & Partial<StepBase>): void;
  /** an SDF surfaced by Manifold's level set, inflating from `from` (a negative level, shrunk) to `level` (0) */
  sculpt(object: string, sdf: (x: number, y: number, z: number) => number, o: { min: V3; max: V3; resolution?: number; level?: number; from?: number; stages?: number } & Partial<StepBase>): void;
  /** rounded (Manifold's smooth then refine): the wireframe's density changes */
  smooth(object: string, o?: { angle?: number; smoothness?: number; refine?: number } & Partial<StepBase>): void;
  refine(object: string, n: number, o?: Partial<StepBase>): void;
  /** copies of `from`'s solid, each transformed, popping in one after another into `object` (from is consumed) */
  array(object: string, from: string, copies: Xf[], o?: Partial<StepBase>): void;
  /** the finished look's colour for an object */
  color(object: string, hex: string): void;
  /** a bone, grown from head to tail */
  bone(name: string, head: V3, tail: V3, o?: { parent?: string } & Partial<StepBase>): void;
  /** several bones in one step, popping in one after another */
  bones(list: { name: string; head: V3; tail: V3; parent?: string }[], o?: Partial<StepBase>): void;
  /** automatic weights, shown as a heatmap for each of `bones` in turn (blue none, red all) */
  weights(o?: { bones?: string[] } & Partial<StepBase>): void;
  /** the rig posed: `fn(u)` over the step (u 0..1); the pose holds after it */
  pose(fn: (u: number) => Pose, o?: Partial<StepBase>): void;
  /** the viewport gives way to the finished look (an instant, on the step's start) */
  finish(o?: Partial<StepBase>): void;
  /** the finished rig animated: `fn(beat)`, beats since the step began */
  animate(fn: (beat: number) => Pose, o?: Partial<StepBase>): void;
  /** nothing new for a while (the camera keeps turning) */
  wait(beats: number): void;
}
