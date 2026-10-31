// The library's features and the interfaces it answers to.
import type { Vec3 } from "raea/3d";
import { type Skeleton } from "raea/character";
import { type Clip } from "../clip";
import { type StyleKey } from "../style";
import { PhaseGoal } from "./all";

export const FUTURE = [10, 20, 30];

export const NF = 27;

/** how many motion features a library frame has, and STATE measurements (clips.ts lays its rows out by them) */
export const FEATURE_COUNT = NF;

export interface MotionLibrary {
  skel: Skeleton;
  fps: number;
  /** how many frames it has (its clips', then their mirror images') */
  size: number;
  /** each frame's clip (an index into `clips`) and its frame in that clip */
  frameClip: Int32Array;
  frameF: Int32Array;
  /** the clips' frame ranges, and the clip each plays (mirrored or not) */
  clips: { name: string; start: number; end: number; mirrored: boolean; clip: Clip }[];
  /** features, normalised and weighted (frames x NF): the pose's for every frame, the route's NaN where the clip
   * doesn't run on a second */
  features: Float32Array;
  /** whether a search may jump to each frame: one of the walking clips', a second from its end */
  jump: Uint8Array;
  scale: Float32Array;
  mean: Float32Array;
  /** each frame's ground frame: x, z and the yaw it faces */
  ground: Float32Array;
  /** whether each foot (L, R) stands on the floor at each frame */
  contact: Uint8Array;
  /** each frame's style: the STATE measurements of the second of motion around it (frames x STATE.length), raw, and
   * their mean and spread over the library */
  state: Float32Array;
  stateMean: Float32Array;
  stateSd: Float32Array;
  /** the sitting action: its stretches (library frames), the clip's seat (the hips' place on it) and the yaw it faces
   * there, and where the body stands before sitting, in the seat's frame (x its left, z ahead) */
  sit?: { down: [number, number]; sat: [number, number]; up: [number, number]; seat: Vec3; yaw: number; stand: [number, number] };
  /** a search to use instead of scanning every frame (clips.ts: the kernel's scoring, HDC, a CPU index): it
   * names the frames worth costing, and perform costs them exactly */
  search?: SearchBackend;
  /** a database library's per-frame annotations (clips.ts): tag bits (2 words a frame) and the tag names */
  db?: { tags: Uint32Array; tagIndex: Map<string, number> };
  /** the frames in or coming to a crouch, precomputed (a database's); worked out from the poses otherwise */
  crouch?: Uint8Array;
  /** frames x 2: each frame's swing clearance (swingClear) of the left hand and the right, m, precomputed (a
   * database's); worked out from the poses otherwise */
  clear?: Float32Array;
  /** frames x 6: each frame's left and right hand in its ground frame (x to the left, y up from the floor, z ahead), m
   * (a database's: an upper body's search matches its hands to limbs.upper.hands) */
  hands?: Float32Array;
}

/** What a search is asked: the query's normalised features and perform's cost terms (nearest's). */
export interface SearchQuery {
  z: Float32Array;
  skip?: [number, number];
  want?: Float32Array;
  ws: number;
  pen?: Float32Array;
  phase?: PhaseGoal;
  mask?: Uint8Array;
  tags?: TagCond;
}

/** A search backend: the library frames worth costing exactly for a query (the brute force costs them all). */
export interface SearchBackend {
  name: string;
  candidates(lib: MotionLibrary, q: SearchQuery): ArrayLike<number>;
}

/** Tag conditions as bits (a database library's): wanted (all of), forbidden (none of), and a cost per tag present. */
export interface TagCond { want: [number, number]; forbid: [number, number]; cost?: Float32Array }

/** Tag conditions by name, on a step or a whole spec. */
export interface TagSpec { want?: string[]; forbid?: string[]; cost?: Record<string, number> }

/** the style measurements a library matches by state: how the body moves, not where it goes */
export const STATE: StyleKey[] = ["cadence", "stride", "stepWidth", "armSwingL", "armSwingR", "handHeight", "handSpread", "elbowFlex", "shoulderRaise",
  "trunkPitch", "trunkRoll", "headPitch", "kneeBend", "kneeLift", "crouch", "bounce", "sway", "asymSwing", "asymStance", "energy", "jerk"];

export const NS = STATE.length;

export const STATE_COUNT = NS;

/** a state measurement in a mirror: its left and right swapped, its signed leans and asymmetries flipped */
export const MIRROR_STATE = STATE.map((k) => (k === "armSwingL" ? STATE.indexOf("armSwingR") : k === "armSwingR" ? STATE.indexOf("armSwingL") : STATE.indexOf(k)));

export const FLIP_STATE = STATE.map((k) => (k === "trunkRoll" || k === "asymSwing" || k === "asymStance" ? -1 : 1));
