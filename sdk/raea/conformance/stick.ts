// The conformance tests' stick body, and a BVH of it walking: a small stand-in with CMU's joints' names (no Anny
// export needed), shared by character.test.ts and the CharacterPlugin program (bench/bevy-ts/programs/character.ts).
import { sub, type Skeleton } from "raea/character";
import type { Vec3 } from "../src/3d";

// a T-posed stick body: 1.2 m, facing +z, x its left
const J: [string, string | null, Vec3][] = [
  ["Hips", null, [0, 0.65, 0]], ["LHipJoint", "Hips", [0, 0.65, 0]], ["LeftUpLeg", "LHipJoint", [0.08, 0.64, 0]], ["LeftLeg", "LeftUpLeg", [0.09, 0.35, 0.01]],
  ["LeftFoot", "LeftLeg", [0.1, 0.06, 0]], ["LeftToeBase", "LeftFoot", [0.1, 0.02, 0.1]], ["LowerBack", "Hips", [0, 0.7, 0]], ["Spine", "LowerBack", [0, 0.8, 0]],
  ["Spine1", "Spine", [0, 0.92, 0]], ["LeftShoulder", "Spine1", [0.02, 0.99, 0]], ["LeftArm", "LeftShoulder", [0.12, 0.98, 0]], ["LeftForeArm", "LeftArm", [0.34, 0.98, 0]],
  ["LeftHand", "LeftForeArm", [0.54, 0.98, 0]], ["LThumb", "LeftHand", [0.55, 0.98, 0.03]], ["LeftFingerBase", "LeftHand", [0.56, 0.98, 0]], ["LeftHandFinger1", "LeftFingerBase", [0.62, 0.98, 0]],
  ["Neck", "Spine1", [0, 1.04, 0]], ["Neck1", "Neck", [0, 1.08, 0]], ["Head", "Neck1", [0, 1.1, 0]],
];
export const mirrorX = (v: Vec3): Vec3 => [-v[0], v[1], v[2]];
const named = [...J];
for (const [n, p, h] of J) {
  const m = n.replace(/^Left/, "Right").replace(/^LHip/, "RHip").replace(/^LThumb/, "RThumb");
  if (m !== n) named.push([m, p && p.replace(/^Left/, "Right").replace(/^LHip/, "RHip"), mirrorX(h)]);
}
export const names = named.map((j) => j[0]);
const tails: Record<string, Vec3> = { LeftToeBase: [0.1, 0.02, 0.14], RightToeBase: [-0.1, 0.02, 0.14], Head: [0, 1.2, 0], LeftHandFinger1: [0.66, 0.98, 0], RightHandFinger1: [-0.66, 0.98, 0], LThumb: [0.57, 0.98, 0.06], RThumb: [-0.57, 0.98, 0.06] };
export const skel: Skeleton = {
  name: "stick", height: 1.2, landmarks: { eyes: [0, 1.14, 0.08], sitDrop: 0.07 },
  joints: named.map(([n, p, h]) => {
    const kids = named.filter((x) => x[1] === n);
    return { name: n, parent: p ? names.indexOf(p) : -1, head: h, tail: tails[n] ?? (kids.length ? kids[0][2] : [h[0], h[1] + 0.05, h[2]]) };
  }),
};

// a BVH of the stick's own skeleton: walking forward along +z, the legs swinging
export function stickBvh(frames: number, fps = 120, still = false): string {
  const lines = ["HIERARCHY"];
  const write = (i: number, depth: number) => {
    const j = skel.joints[i], pad = "  ".repeat(depth);
    const off = j.parent < 0 ? [0, 0, 0] : sub(j.head, skel.joints[j.parent].head);
    lines.push(`${pad}${j.parent < 0 ? "ROOT" : "JOINT"} ${j.name.replace("HandFinger1", "HandIndex1")}`, `${pad}{`, `${pad}  OFFSET ${off.join(" ")}`);
    lines.push(`${pad}  CHANNELS ${j.parent < 0 ? "6 Xposition Yposition Zposition " : "3 "}Zrotation Yrotation Xrotation`);
    const kids = skel.joints.map((k, ki) => [k, ki] as const).filter(([k]) => k.parent === i);
    if (!kids.length) lines.push(`${pad}  End Site`, `${pad}  {`, `${pad}    OFFSET ${sub(j.tail, j.head).join(" ")}`, `${pad}  }`);
    for (const [, ki] of kids) write(ki, depth + 1);
    lines.push(`${pad}}`);
  };
  write(0, 0);
  lines.push("MOTION", `Frames: ${frames}`, `Frame Time: ${1 / fps}`);
  for (let f = 0; f < frames; f++) {
    const t = still ? 0 : f / fps, swing = 25 * Math.sin(t * 2 * Math.PI);
    const row: number[] = [];
    skel.joints.forEach((j) => {
      if (j.parent < 0) row.push(0, 0.65, t * 1.1);
      const x = j.name === "LeftUpLeg" ? swing : j.name === "RightUpLeg" ? -swing : 0;
      row.push(0, 0, x);
    });
    lines.push(row.join(" "));
  }
  return lines.join("\n");
}
