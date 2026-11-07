// three's clipping planes (a material's clippingPlanes with the renderer's localClippingEnabled, the renderer's own
// clippingPlanes, a WebGPU <ClippingGroup>'s) done on the mesh: each triangle cut at the planes and what lies beyond
// dropped, its normals, uvs and colours carried to the new corners. three discards those fragments, so the surface
// left is the same, open where it was cut (three's is too: it has no caps).
//
// clipIntersection (three: clip only what lies beyond every plane) keeps the parts inside any one plane: each plane's
// side cut out of what the earlier ones didn't keep, so nothing is drawn twice.
import * as THREE from "three";
import type { Arrays } from "./geometry";

export interface Clip {
  /** world-space planes: what's on a plane's negative side is cut away */
  planes: THREE.Plane[];
  /** three's clipIntersection */
  intersection: boolean;
}

interface Vert {
  p: number[];
  n?: number[];
  uv?: number[];
  c?: number[];
}

const lerp = (a: number[] | undefined, b: number[] | undefined, t: number) => (a && b ? a.map((x, i) => x + (b[i] - x) * t) : undefined);
const mix = (a: Vert, b: Vert, t: number): Vert => ({ p: lerp(a.p, b.p, t)!, n: lerp(a.n, b.n, t), uv: lerp(a.uv, b.uv, t), c: lerp(a.c, b.c, t) });

/** a polygon cut by a plane (local space): what's on its positive side, or (keepBack) its negative side */
function cutBy(poly: Vert[], pl: THREE.Plane, keepBack = false): Vert[] {
  const out: Vert[] = [];
  const side = (v: Vert) => (pl.normal.x * v.p[0] + pl.normal.y * v.p[1] + pl.normal.z * v.p[2] + pl.constant) * (keepBack ? -1 : 1);
  for (let i = 0; i < poly.length; i++) {
    const a = poly[i], b = poly[(i + 1) % poly.length];
    const da = side(a), db = side(b);
    if (da >= 0) out.push(a);
    if ((da >= 0) !== (db >= 0)) out.push(mix(a, b, da / (da - db)));
  }
  return out;
}

/** the mesh's arrays clipped by `clip`'s planes (given in world space; `world` is the mesh's matrixWorld) */
export function clipArrays(a: Arrays, clip: Clip, world: THREE.Matrix4): Arrays | undefined {
  if (!clip.planes.length) return a;
  // the planes in the mesh's own space
  const inv = new THREE.Matrix4().copy(world).invert();
  const normalOf = new THREE.Matrix3().getNormalMatrix(inv);
  const planes = clip.planes.map((p) => p.clone().applyMatrix4(inv, normalOf));
  const nv = a.positions.length / 3;
  const vert = (i: number): Vert => ({
    p: a.positions.slice(i * 3, i * 3 + 3),
    n: a.normals?.slice(i * 3, i * 3 + 3),
    uv: a.uvs?.slice(i * 2, i * 2 + 2),
    c: a.colors?.slice(i * 4, i * 4 + 4),
  });
  const idx = a.indices ?? Array.from({ length: nv }, (_, i) => i);
  const out: Vert[] = [];
  const emit = (poly: Vert[]) => {
    for (let k = 1; k + 1 < poly.length; k++) out.push(poly[0], poly[k], poly[k + 1]);
  };
  for (let t = 0; t + 2 < idx.length; t += 3) {
    const tri = [vert(idx[t]), vert(idx[t + 1]), vert(idx[t + 2])];
    if (!clip.intersection) {
      let poly = tri;
      for (const pl of planes) {
        poly = cutBy(poly, pl);
        if (poly.length < 3) break;
      }
      if (poly.length >= 3) emit(poly);
      continue;
    }
    // inside any plane: plane k's side, less what planes before it kept
    for (let k = 0; k < planes.length; k++) {
      let poly = cutBy(tri, planes[k]);
      for (let j = 0; j < k && poly.length >= 3; j++) poly = cutBy(poly, planes[j], true);
      if (poly.length >= 3) emit(poly);
    }
  }
  if (!out.length) return undefined;
  const flat = (k: keyof Vert) => (out[0][k] ? out.flatMap((v) => v[k]!) : undefined);
  return { ...a, positions: flat("p")!, normals: flat("n"), uvs: flat("uv"), colors: flat("c"), indices: undefined } as Arrays;
}

/** the planes that clip a mesh with material `m` under `gl` and the clipping groups around it */
export function clipOf(m: THREE.Material, gl: THREE.WebGLRenderer | undefined, groups: Clip[]): Clip | undefined {
  const planes: THREE.Plane[] = [];
  let intersection = false;
  const g = gl as unknown as { clippingPlanes?: THREE.Plane[]; localClippingEnabled?: boolean } | undefined;
  if (g?.clippingPlanes?.length) planes.push(...g.clippingPlanes);
  if (m.clippingPlanes?.length && (g?.localClippingEnabled || (m as { isNodeMaterial?: boolean }).isNodeMaterial)) {
    planes.push(...m.clippingPlanes);
    intersection = m.clipIntersection;
  }
  for (const c of groups) {
    planes.push(...c.planes);
    intersection ||= c.intersection;
  }
  return planes.length ? { planes, intersection } : undefined;
}
