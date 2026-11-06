// The conformance tests' stick body, and a BVH of it walking: a small stand-in on raea's game_engine joints
// (docs/skeleton.feature SK0a) in a T-pose, shared by character.test.ts, ragdoll.test.ts and the CharacterPlugin
// program (bench/bevy-ts/programs/character.ts).
import { BODY_JOINTS, FINGER_JOINTS, FINGERS, sub, type Skeleton } from "raea/character";
import type { Vec3 } from "../src/3d";

export const mirrorX = (v: Vec3): Vec3 => [-v[0], v[1], v[2]];

// the left side and the midline: [name, parent, head, tail?]; the right side is the left's mirror image
const LEFT: [string, string | null, Vec3, Vec3?][] = [
  ["pelvis", null, [0, 0.65, 0]], ["spine_01", "pelvis", [0, 0.7, 0]], ["spine_02", "spine_01", [0, 0.8, 0]],
  ["spine_03", "spine_02", [0, 0.92, 0]], ["neck_01", "spine_03", [0, 1.04, 0]], ["head", "neck_01", [0, 1.1, 0], [0, 1.2, 0]],
  ["clavicle_l", "spine_03", [0.02, 0.99, 0]], ["upperarm_l", "clavicle_l", [0.12, 0.98, 0]], ["lowerarm_l", "upperarm_l", [0.34, 0.98, 0]],
  ["hand_l", "lowerarm_l", [0.54, 0.98, 0], [0.6, 0.98, 0]],
  ["thigh_l", "pelvis", [0.08, 0.64, 0]], ["calf_l", "thigh_l", [0.09, 0.35, 0.01]], ["foot_l", "calf_l", [0.1, 0.06, 0]],
  ["ball_l", "foot_l", [0.1, 0.02, 0.1], [0.1, 0.02, 0.14]],
];
// each finger's knuckle (z across the hand: the thumb ahead, the pinky behind) and its phalanges' length
const KNUCKLE: Record<string, [Vec3, number]> = {
  thumb: [[0.55, 0.98, 0.03], 0.022], index: [[0.62, 0.98, 0.02], 0.03], middle: [[0.62, 0.98, 0], 0.032],
  ring: [[0.615, 0.98, -0.018], 0.03], pinky: [[0.605, 0.98, -0.034], 0.024],
};
for (const f of FINGERS) {
  const [k, l] = KNUCKLE[f], dir: Vec3 = f === "thumb" ? [0.6, 0, 0.8] : [1, 0, 0];
  for (const ph of [1, 2, 3]) {
    const at = (m: number): Vec3 => [k[0] + dir[0] * l * m, k[1], k[2] + dir[2] * l * m];
    LEFT.push([`${f}_0${ph}_l`, ph === 1 ? "hand_l" : `${f}_0${ph - 1}_l`, at(ph - 1), at(ph)]);
  }
}
const all = new Map<string, [string | null, Vec3, Vec3?]>();
for (const [n, p, h, t] of LEFT) {
  all.set(n, [p, h, t]);
  if (n.endsWith("_l")) all.set(n.slice(0, -1) + "r", [p && (p.endsWith("_l") ? p.slice(0, -1) + "r" : p), mirrorX(h), t && mirrorX(t)]);
}
export const names = [...BODY_JOINTS, ...FINGER_JOINTS];
export const skel: Skeleton = {
  name: "stick", height: 1.2, landmarks: { eyes: [0, 1.14, 0.08], sitDrop: 0.07 },
  joints: names.map((n) => {
    const [p, h, t] = all.get(n)!;
    const kid = names.find((c) => all.get(c)![0] === n && !/^(thumb|index|ring|pinky)_01|^thigh|^clavicle/.test(c));
    return { name: n, parent: p ? names.indexOf(p) : -1, head: h, tail: t ?? (kid ? all.get(kid)![1] : [h[0], h[1] + 0.05, h[2]]) };
  }),
};

// a BVH of the stick's own skeleton: walking forward along +z, the legs swinging
export function stickBvh(frames: number, fps = 120, still = false): string {
  const lines = ["HIERARCHY"];
  const write = (i: number, depth: number) => {
    const j = skel.joints[i], pad = "  ".repeat(depth);
    const off = j.parent < 0 ? [0, 0, 0] : sub(j.head, skel.joints[j.parent].head);
    lines.push(`${pad}${j.parent < 0 ? "ROOT" : "JOINT"} ${j.name}`, `${pad}{`, `${pad}  OFFSET ${off.join(" ")}`);
    lines.push(`${pad}  CHANNELS ${j.parent < 0 ? "6 Xposition Yposition Zposition " : "3 "}Zrotation Yrotation Xrotation`);
    const kids = skel.joints.map((k, ki) => [k, ki] as const).filter(([k]) => k.parent === i);
    if (!kids.length) lines.push(`${pad}  End Site`, `${pad}  {`, `${pad}    OFFSET ${sub(j.tail, j.head).join(" ")}`, `${pad}  }`);
    for (const [, ki] of kids) write(ki, depth + 1);
    lines.push(`${pad}}`);
  };
  write(0, 0);
  // (the joints in the BVH's own order, as it was written: depth first)
  const order: number[] = [];
  const walk = (i: number) => { order.push(i); skel.joints.forEach((k, ki) => k.parent === i && walk(ki)); };
  walk(0);
  lines.push("MOTION", `Frames: ${frames}`, `Frame Time: ${1 / fps}`);
  for (let f = 0; f < frames; f++) {
    const t = still ? 0 : f / fps, swing = 25 * Math.sin(t * 2 * Math.PI);
    const row: number[] = [];
    for (const i of order) {
      const j = skel.joints[i];
      if (j.parent < 0) row.push(0, 0.65, t * 1.1);
      const x = j.name === "thigh_l" ? swing : j.name === "thigh_r" ? -swing : 0;
      row.push(0, 0, x);
    }
    lines.push(row.join(" "));
  }
  return lines.join("\n");
}
