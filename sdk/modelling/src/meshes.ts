// Meshes of solids.
import { Solid } from "@raea/render/solid";
import { writeGlb, type GlbPrimitive } from "@raea/asset/glb";
import { DEG, V3, add3, cross3, dot3, len3, mul3, norm3, sub3 } from "./all";

// ---------------------------------------------------------------- meshes of solids

interface MeshData {
  v: Float32Array;
  t: Uint32Array;
  /** smooth vertex normals (area-weighted) */
  n: Float32Array;
  /** face normals */
  fn: Float32Array;
  center: V3;
  radius: number;
  min: V3;
  max: V3;
}

const meshes = new Map<number, MeshData>();

export function meshOf(s: Solid): MeshData {
  const hit = meshes.get(s.id);
  if (hit) return hit;
  const { verts: v, tris: t } = s.mesh();
  const n = new Float32Array(v.length);
  const fn = new Float32Array(t.length);
  for (let i = 0; i < t.length; i += 3) {
    const a = t[i] * 3, b = t[i + 1] * 3, c = t[i + 2] * 3;
    const e1: V3 = [v[b] - v[a], v[b + 1] - v[a + 1], v[b + 2] - v[a + 2]];
    const e2: V3 = [v[c] - v[a], v[c + 1] - v[a + 1], v[c + 2] - v[a + 2]];
    const f = cross3(e1, e2);
    const u = norm3(f);
    fn[i] = u[0]; fn[i + 1] = u[1]; fn[i + 2] = u[2];
    for (const k of [a, b, c]) {
      n[k] += f[0]; n[k + 1] += f[1]; n[k + 2] += f[2];
    }
  }
  const min: V3 = [Infinity, Infinity, Infinity], max: V3 = [-Infinity, -Infinity, -Infinity];
  for (let i = 0; i < v.length; i += 3) {
    const u = norm3([n[i], n[i + 1], n[i + 2]]);
    n[i] = u[0]; n[i + 1] = u[1]; n[i + 2] = u[2];
    for (let k = 0; k < 3; k++) {
      min[k] = Math.min(min[k], v[i + k]);
      max[k] = Math.max(max[k], v[i + k]);
    }
  }
  const center = mul3(add3(min, max), 0.5);
  const md: MeshData = { v, t, n, fn, center, radius: Math.max(1e-3, len3(sub3(max, min)) / 2), min, max };
  meshes.set(s.id, md);
  return md;
}

/** the surface with normals split where faces turn more than `crease` degrees: positions, normals, indices, and
 * each new vertex's source vertex */
export function surfaceOf(m: MeshData, crease = 40) {
  const cos = Math.cos(crease * DEG);
  const faces: number[][] = Array.from({ length: m.v.length / 3 }, () => []);
  for (let f = 0; f < m.t.length / 3; f++) for (let k = 0; k < 3; k++) faces[m.t[f * 3 + k]].push(f);
  const pos: number[] = [], nor: number[] = [], idx: number[] = [], src: number[] = [];
  const seen = new Map<string, number>();
  for (let f = 0; f < m.t.length / 3; f++) {
    const nf: V3 = [m.fn[f * 3], m.fn[f * 3 + 1], m.fn[f * 3 + 2]];
    for (let k = 0; k < 3; k++) {
      const vi = m.t[f * 3 + k];
      let s: V3 = [0, 0, 0];
      for (const g of faces[vi]) {
        const ng: V3 = [m.fn[g * 3], m.fn[g * 3 + 1], m.fn[g * 3 + 2]];
        if (dot3(nf, ng) >= cos) s = add3(s, ng);
      }
      const u = norm3(s);
      const key = `${vi}:${Math.round(u[0] * 50)},${Math.round(u[1] * 50)},${Math.round(u[2] * 50)}`;
      let at = seen.get(key);
      if (at === undefined) {
        at = src.length;
        seen.set(key, at);
        pos.push(m.v[vi * 3], m.v[vi * 3 + 1], m.v[vi * 3 + 2]);
        nor.push(...u);
        src.push(vi);
      }
      idx.push(at);
    }
  }
  return { pos, nor, idx, src };
}

/** the wireframe's edges (pairs of vertex indices): every triangle edge but those between faces flat to each other */
function edgesOf(m: MeshData, flat = 1.5): number[] {
  const cos = Math.cos(flat * DEG);
  const first = new Map<number, number>();
  const out: number[] = [];
  const nv = m.v.length / 3;
  for (let f = 0; f < m.t.length / 3; f++) {
    for (let k = 0; k < 3; k++) {
      const a = m.t[f * 3 + k], b = m.t[f * 3 + ((k + 1) % 3)];
      const key = Math.min(a, b) * nv + Math.max(a, b);
      const g = first.get(key);
      if (g === undefined) first.set(key, f);
      else {
        const d = m.fn[f * 3] * m.fn[g * 3] + m.fn[f * 3 + 1] * m.fn[g * 3 + 1] + m.fn[f * 3 + 2] * m.fn[g * 3 + 2];
        if (d < cos) out.push(a, b);
        first.delete(key);
      }
    }
  }
  for (const key of first.keys()) out.push(Math.floor(key / nv), key % nv);
  return out;
}

export const WIRE = "#1c1c1c";

const SELECT = "#ffaa33";

/** the wireframe of a solid as a line-list .glb, lifted off the surface along its normals */
const wires = new Map<string, string>();

export function wireGlb(s: Solid, color = WIRE, extra?: { joints: Uint16Array; weights: Float32Array }): GlbPrimitive {
  const m = meshOf(s);
  const lift = m.radius * 0.0035;
  const e = edgesOf(m);
  const used = new Map<number, number>();
  const pos: number[] = [], idx: number[] = [], jo: number[] = [], we: number[] = [];
  for (const vi of e) {
    let at = used.get(vi);
    if (at === undefined) {
      at = used.size;
      used.set(vi, at);
      pos.push(m.v[vi * 3] + m.n[vi * 3] * lift, m.v[vi * 3 + 1] + m.n[vi * 3 + 1] * lift, m.v[vi * 3 + 2] + m.n[vi * 3 + 2] * lift);
      if (extra) for (let k = 0; k < 4; k++) { jo.push(extra.joints[vi * 4 + k]); we.push(extra.weights[vi * 4 + k]); }
    }
    idx.push(at);
  }
  return { positions: pos, indices: idx, mode: "lines", material: { color, unlit: true }, ...(extra ? { joints: jo, weights: we } : {}) };
}

export function wireOf(s: Solid, color = WIRE): string {
  const key = `${s.id}:${color}`;
  let src = wires.get(key);
  if (!src) wires.set(key, (src = writeGlb({ primitives: [wireGlb(s, color)] })));
  return src;
}

/** an inside-out copy pushed out along the normals: drawn behind the part, it shows only as a rim (an outline) */
export function hullPrim(s: Solid, width: number, color: string, extra?: { joints: Uint16Array; weights: Float32Array }): GlbPrimitive {
  const m = meshOf(s);
  const pos = new Float32Array(m.v.length);
  for (let i = 0; i < m.v.length; i++) pos[i] = m.v[i] + m.n[i] * width;
  const idx = new Uint32Array(m.t.length);
  for (let i = 0; i < m.t.length; i += 3) {
    idx[i] = m.t[i]; idx[i + 1] = m.t[i + 2]; idx[i + 2] = m.t[i + 1];
  }
  return { positions: pos, indices: idx, material: { color, unlit: true }, ...(extra ? { joints: extra.joints, weights: extra.weights } : {}) };
}

const hulls = new Map<string, string>();

export function hullOf(s: Solid, color = SELECT): string {
  const key = `${s.id}:${color}`;
  let src = hulls.get(key);
  if (!src) hulls.set(key, (src = writeGlb({ primitives: [hullPrim(s, meshOf(s).radius * 0.022, color)] })));
  return src;
}

/** vertex dots: a small octahedron at each vertex (up to 2400 of them) */
const dotsMade = new Map<number, string | null>();

export function dotsOf(s: Solid): string | null {
  if (dotsMade.has(s.id)) return dotsMade.get(s.id)!;
  const m = meshOf(s);
  const nv = m.v.length / 3;
  if (nv > 2400) {
    dotsMade.set(s.id, null);
    return null;
  }
  const r = m.radius * 0.012, lift = m.radius * 0.004;
  const pos: number[] = [], idx: number[] = [];
  const O: V3[] = [[r, 0, 0], [-r, 0, 0], [0, r, 0], [0, -r, 0], [0, 0, r], [0, 0, -r]];
  const F = [[0, 2, 4], [2, 1, 4], [1, 3, 4], [3, 0, 4], [2, 0, 5], [1, 2, 5], [3, 1, 5], [0, 3, 5]];
  for (let i = 0; i < nv; i++) {
    const c: V3 = [m.v[i * 3] + m.n[i * 3] * lift, m.v[i * 3 + 1] + m.n[i * 3 + 1] * lift, m.v[i * 3 + 2] + m.n[i * 3 + 2] * lift];
    const base = pos.length / 3;
    for (const o of O) pos.push(c[0] + o[0], c[1] + o[1], c[2] + o[2]);
    for (const f of F) idx.push(base + f[0], base + f[1], base + f[2]);
  }
  const src = writeGlb({ primitives: [{ positions: pos, indices: idx, material: { color: "#111111", unlit: true } }] });
  dotsMade.set(s.id, src);
  return src;
}
