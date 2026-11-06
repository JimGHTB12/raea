// The joints' names: MakeHuman's game_engine rig (Unreal's mannequin names), raea's one character skeleton
// (docs/skeleton.feature SK0a). Every part of the SDK names joints through these, never with string literals.

export type Side = "L" | "R";

export const SIDES: readonly Side[] = ["L", "R"];

/** the suffix a side's joints carry */
const sx = (s: Side) => (s === "L" ? "l" : "r");

export const FINGERS = ["thumb", "index", "middle", "ring", "pinky"] as const;

export type Finger = (typeof FINGERS)[number];

/** Joint names, by role. A body's joints are the trunk, then each arm and leg; the fingers come after all of them. */
export const B = {
  pelvis: "pelvis",
  /** the spine from the pelvis up: spine_01 (the waist), spine_02, spine_03 (the chest, which the clavicles and neck hang from) */
  spine: ["spine_01", "spine_02", "spine_03"] as const,
  chest: "spine_03",
  neck: "neck_01",
  head: "head",
  clavicle: (s: Side) => `clavicle_${sx(s)}`,
  upperarm: (s: Side) => `upperarm_${sx(s)}`,
  lowerarm: (s: Side) => `lowerarm_${sx(s)}`,
  hand: (s: Side) => `hand_${sx(s)}`,
  thigh: (s: Side) => `thigh_${sx(s)}`,
  calf: (s: Side) => `calf_${sx(s)}`,
  foot: (s: Side) => `foot_${sx(s)}`,
  /** the ball of the foot, where the toes bend */
  ball: (s: Side) => `ball_${sx(s)}`,
  /** a finger's phalanx k (1 at the knuckle, 3 at the tip; the thumb's 1 is its metacarpal) */
  finger: (f: Finger, k: 1 | 2 | 3, s: Side) => `${f}_0${k}_${sx(s)}`,
  arm: (s: Side) => [B.upperarm(s), B.lowerarm(s), B.hand(s)],
  leg: (s: Side) => [B.thigh(s), B.calf(s), B.foot(s)],
  /** both feet's ankles then both balls: L foot, R foot, L ball, R ball */
  feet: () => [B.foot("L"), B.foot("R"), B.ball("L"), B.ball("R")],
};

/** The body's joints in skeleton order (parents first): what a clip turns. */
export const BODY_JOINTS: readonly string[] = [
  B.pelvis, ...B.spine, B.neck, B.head,
  ...SIDES.flatMap((s) => [B.clavicle(s), B.upperarm(s), B.lowerarm(s), B.hand(s)]),
  ...SIDES.flatMap((s) => [B.thigh(s), B.calf(s), B.foot(s), B.ball(s)]),
];

/** The fingers' joints in skeleton order, after the body's: each side's thumb, index, middle, ring, pinky, 1 to 3. */
export const FINGER_JOINTS: readonly string[] = SIDES.flatMap((s) => FINGERS.flatMap((f) => ([1, 2, 3] as const).map((k) => B.finger(f, k, s))));

/** a joint's side from its name ("" on the midline) */
export function sideOf(name: string): Side | "" {
  return name.endsWith("_l") ? "L" : name.endsWith("_r") ? "R" : "";
}

/** a joint's name on the other side (itself on the midline) */
export function mirrorName(name: string): string {
  return name.endsWith("_l") ? name.slice(0, -1) + "r" : name.endsWith("_r") ? name.slice(0, -1) + "l" : name;
}

/** a joint's role, its name without the side: "thigh", "index_02", "spine_01" */
export function roleOf(name: string): string {
  return name.replace(/_[lr]$/, "");
}

export const isFinger = (name: string): boolean => FINGERS.some((f) => name.startsWith(f + "_"));
