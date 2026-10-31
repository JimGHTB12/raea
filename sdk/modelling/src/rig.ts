// The rig: skinning, clips, posed bones.
import { writeGlb, type GlbPrimitive, type GlbJoint } from "@raea/asset/glb";
import { Bone, BuildLook, Built, M3, Pose, V3, WIRE, add3, hullPrim, meshOf, mm3, mv3, poseRot, quatOf, sub3, surfaceOf, weightColor, wireGlb } from "./all";

// ---------------------------------------------------------------- the rig: skinning, clips, posed bones

/** joint world matrices (3x3 rotation + translation) for a pose: J = J_parent * T(head - parent head) * R */
export function jointWorlds(bones: Bone[], pose: Pose): { r: M3; t: V3 }[] {
  const out: { r: M3; t: V3 }[] = [];
  const idx = new Map(bones.map((b, i) => [b.name, i]));
  bones.forEach((b, i) => {
    const R = poseRot(pose[b.name] ?? [0, 0, 0]);
    if (!b.parent) {
      out[i] = { r: R, t: add3(b.head, pose.$move ?? [0, 0, 0]) };
    } else {
      const P = out[idx.get(b.parent)!];
      const pb = bones[idx.get(b.parent)!];
      out[i] = { r: mm3(P.r, R), t: add3(P.t, mv3(P.r, sub3(b.head, pb.head))) };
    }
  });
  return out;
}

interface RigFiles {
  viewport: string;
  final: string;
  /** clip time 0 is this build beat */
  from: number;
  to: number;
}

const rigs = new WeakMap<Built, RigFiles>();

const weightMaps = new WeakMap<Built, Map<string, string>>();

export function sortedBones(b: Built): Bone[] {
  // parents before children (glTF joint order in glb.ts needs it)
  const out: Bone[] = [];
  const add = (x: Bone) => {
    if (out.includes(x)) return;
    if (x.parent) add(b.bones.find((y) => y.name === x.parent)!);
    out.push(x);
  };
  b.bones.forEach(add);
  return out;
}

export function rigOf(b: Built, look: Required<BuildLook>): RigFiles {
  const hit = rigs.get(b);
  if (hit) return hit;
  const bones = sortedBones(b);
  const order = bones.map((x) => b.bones.indexOf(x));
  const remap = new Uint16Array(b.bones.length);
  order.forEach((orig, k) => (remap[orig] = k));
  const W = b.weights();
  const joints: GlbJoint[] = bones.map((x) => ({ name: x.name, parent: x.parent ? bones.findIndex((y) => y.name === x.parent) : -1, head: x.head }));
  const first = b.steps.find((s) => s.kind === "pose" || s.kind === "animate" || s.kind === "finish");
  const from = first ? first.start : b.duration;
  const to = Math.max(from + 1, b.duration);
  const SPB = 16;
  const times: number[] = [];
  const rots: number[][] = bones.map(() => []);
  const moves: number[] = [];
  const rootIdx = bones.findIndex((x) => !x.parent);
  for (let i = 0; i <= Math.ceil((to - from) * SPB); i++) {
    const at = from + i / SPB;
    times.push(i / SPB);
    const pose = b.poseAt(at);
    bones.forEach((x, k) => rots[k].push(...quatOf(poseRot(pose[x.name] ?? [0, 0, 0]))));
    moves.push(...(pose.$move ?? [0, 0, 0]));
  }
  const clips = [{ name: "rig", times, rotations: rots, translations: rootIdx >= 0 ? { [rootIdx]: moves } : {} }];
  const vp: GlbPrimitive[] = [], fin: GlbPrimitive[] = [];
  for (const [name, { solid, color }] of b.objects) {
    const m = meshOf(solid);
    const w = W.get(name)!;
    const jw = { joints: w.joints.map((j) => remap[j]) as Uint16Array, weights: w.weights };
    const surf = surfaceOf(m);
    const J: number[] = [], Wt: number[] = [];
    for (const s of surf.src) for (let k = 0; k < 4; k++) { J.push(jw.joints[s * 4 + k]); Wt.push(jw.weights[s * 4 + k]); }
    vp.push({ positions: surf.pos, normals: surf.nor, indices: surf.idx, joints: J, weights: Wt, material: { color: look.clay, roughness: 0.6 } });
    vp.push(wireGlb(solid, WIRE, jw));
    fin.push({ positions: surf.pos, normals: surf.nor, indices: surf.idx, joints: J, weights: Wt, material: { color, roughness: 0.45 } });
    fin.push(hullPrim(solid, Math.max(0.012, m.radius * 0.03), look.line, jw));
  }
  const files: RigFiles = { viewport: writeGlb({ primitives: vp, joints, clips }), final: writeGlb({ primitives: fin, joints, clips }), from, to };
  rigs.set(b, files);
  return files;
}

export function weightMapOf(b: Built, bone: string): string {
  let byBone = weightMaps.get(b);
  if (!byBone) weightMaps.set(b, (byBone = new Map()));
  const hit = byBone.get(bone);
  if (hit) return hit;
  const k = b.bones.findIndex((x) => x.name === bone);
  const W = b.weights();
  const prims: GlbPrimitive[] = [];
  for (const [name, { solid }] of b.objects) {
    const m = meshOf(solid);
    const w = W.get(name)!;
    const surf = surfaceOf(m);
    const colors: number[] = [];
    for (const s of surf.src) {
      let x = 0;
      for (let j = 0; j < 4; j++) if (w.joints[s * 4 + j] === k) x += w.weights[s * 4 + j];
      // the colour ramp is in sRGB: hand glTF linear values
      colors.push(...(weightColor(x).map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4)) as V3), 1);
    }
    prims.push({ positions: surf.pos, normals: surf.nor, indices: surf.idx, colors, material: { color: "#ffffff", roughness: 0.7 } });
    prims.push(wireGlb(solid, "#101010"));
  }
  const src = writeGlb({ primitives: prims });
  byBone.set(bone, src);
  return src;
}
