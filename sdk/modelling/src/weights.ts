// Automatic weights.
import { Solid } from "@raea/render/solid";
import { Bone, V3, add3, clamp01, dot3, len3, meshOf, mul3, sub3 } from "./all";

// ---------------------------------------------------------------- automatic weights

function segDist(p: V3, a: V3, b: V3): number {
  const ab = sub3(b, a);
  const h = clamp01(dot3(sub3(p, a), ab) / Math.max(1e-12, dot3(ab, ab)));
  return len3(sub3(p, add3(a, mul3(ab, h))));
}

/** Weights by distance to each bone's segment, sharpened toward the nearest, then diffused over the mesh's edges (a
 * cheap stand-in for Blender's heat weighting: weights spread along the surface, not through the air), the four
 * largest kept and normalised. */
export function autoWeights(objects: Map<string, { solid: Solid }>, bones: Bone[], o: { sharpness?: number; iterations?: number } = {}) {
  const out = new Map<string, { joints: Uint16Array; weights: Float32Array }>();
  const nb = bones.length;
  if (!nb) return out;
  const p = o.sharpness ?? 8, iters = o.iterations ?? 12;
  for (const [name, { solid }] of objects) {
    const m = meshOf(solid);
    const nv = m.v.length / 3;
    let W = new Float32Array(nv * nb);
    for (let i = 0; i < nv; i++) {
      const v: V3 = [m.v[i * 3], m.v[i * 3 + 1], m.v[i * 3 + 2]];
      const d = bones.map((b) => segDist(v, b.head, b.tail) + 1e-4);
      const dmin = Math.min(...d);
      let sum = 0;
      for (let k = 0; k < nb; k++) sum += W[i * nb + k] = (dmin / d[k]) ** p;
      for (let k = 0; k < nb; k++) W[i * nb + k] /= sum;
    }
    // neighbours
    const nbrs: number[][] = Array.from({ length: nv }, () => []);
    for (let f = 0; f < m.t.length; f += 3) {
      const a = m.t[f], b = m.t[f + 1], c = m.t[f + 2];
      nbrs[a].push(b, c); nbrs[b].push(a, c); nbrs[c].push(a, b);
    }
    for (let it = 0; it < iters; it++) {
      const N = new Float32Array(W.length);
      for (let i = 0; i < nv; i++) {
        const ns = nbrs[i];
        for (let k = 0; k < nb; k++) {
          let s = 0;
          for (const j of ns) s += W[j * nb + k];
          N[i * nb + k] = 0.5 * W[i * nb + k] + (0.5 * s) / Math.max(1, ns.length);
        }
      }
      W = N;
    }
    const joints = new Uint16Array(nv * 4), weights = new Float32Array(nv * 4);
    for (let i = 0; i < nv; i++) {
      const order = Array.from({ length: nb }, (_, k) => k).sort((a, b) => W[i * nb + b] - W[i * nb + a]).slice(0, 4);
      let sum = 0;
      for (const k of order) sum += W[i * nb + k];
      order.forEach((k, j) => {
        joints[i * 4 + j] = k;
        weights[i * 4 + j] = W[i * nb + k] / sum;
      });
    }
    out.set(name, { joints, weights });
  }
  return out;
}

/** Blender's weight colours: blue (0), cyan, green, yellow, red (1) */
export function weightColor(w: number): [number, number, number] {
  const h = (1 - clamp01(w)) * 240;
  const x = 1 - Math.abs(((h / 60) % 2) - 1);
  const [r, g, b] = h < 60 ? [1, x, 0] : h < 120 ? [x, 1, 0] : h < 180 ? [0, 1, x] : [0, x, 1];
  return [r, g, b];
}
