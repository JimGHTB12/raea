// A small glTF 2.0 binary writer, so TypeScript can hand the engine geometry Manifold doesn't make: line lists (a
// wireframe, a grid), open or inside-out triangles (an outline hull), per-vertex colours (a weight map) and skinned
// meshes with a joint hierarchy and a clip (a rig posed by `<Model animation time>`). The file's name is a hash of its
// bytes, so the same geometry is written once and the same frame always names the same file (frame-pure).
//
//   const src = writeGlb({ primitives: [{ positions, indices, mode: "lines", material: { color: "#202020", unlit: true } }] });
//   <Model src={src} />

import { native, hasNative } from "@raea/app/native";

export type Rgba = [number, number, number, number];

export interface GlbMaterial {
  /** hex (#rrggbb) or linear rgba 0..1 */
  color?: string | Rgba;
  /** drawn without light (KHR_materials_unlit) */
  unlit?: boolean;
  /** alpha-blended (the colour's alpha, times the vertex colours' alpha) */
  blend?: boolean;
  doubleSided?: boolean;
  roughness?: number;
  metalness?: number;
  emissive?: string;
}

export interface GlbPrimitive {
  /** x y z per vertex */
  positions: ArrayLike<number>;
  normals?: ArrayLike<number>;
  /** r g b a per vertex, linear 0..1 (multiplies the material's colour) */
  colors?: ArrayLike<number>;
  /** by threes for triangles, by twos for lines; none: every vertex in order */
  indices?: ArrayLike<number>;
  mode?: "triangles" | "lines";
  /** four joint indices per vertex (a skinned mesh) */
  joints?: ArrayLike<number>;
  /** four weights per vertex, summing to 1 */
  weights?: ArrayLike<number>;
  material?: GlbMaterial;
}

export interface GlbJoint {
  name: string;
  /** index of the parent joint, -1 for a root */
  parent: number;
  /** the joint's head in world space at rest (its node's rest translation is head - parent's head) */
  head: [number, number, number];
}

export interface GlbClip {
  name: string;
  /** sample times (any unit the Model's `time` uses) */
  times: ArrayLike<number>;
  /** per joint, a quaternion x y z w per sample: the joint's rotation in its parent's frame */
  rotations: ArrayLike<number>[];
  /** per root joint (by joint index), a translation offset x y z per sample, added to its rest translation */
  translations?: Record<number, ArrayLike<number>>;
}

export interface GlbScene {
  primitives: GlbPrimitive[];
  /** a skin: all primitives are skinned by these joints when it's given */
  joints?: GlbJoint[];
  clips?: GlbClip[];
}

/** sRGB hex to linear rgba */
export function linearRgba(c: string | Rgba, alpha = 1): Rgba {
  if (Array.isArray(c)) return c;
  const m = /^#?([0-9a-f]{6})$/i.exec(c.trim());
  if (!m) throw new TypeError(`glb: colours are #rrggbb, got ${JSON.stringify(c)}`);
  const n = parseInt(m[1], 16);
  const lin = (v: number) => {
    const s = v / 255;
    return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  };
  return [lin((n >> 16) & 255), lin((n >> 8) & 255), lin(n & 255), alpha];
}

const B64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
function base64(b: Uint8Array): string {
  let out = "";
  for (let i = 0; i < b.length; i += 3) {
    const n = (b[i] << 16) | ((b[i + 1] ?? 0) << 8) | (b[i + 2] ?? 0);
    out += B64[(n >> 18) & 63] + B64[(n >> 12) & 63] + (i + 1 < b.length ? B64[(n >> 6) & 63] : "=") + (i + 2 < b.length ? B64[n & 63] : "=");
  }
  return out;
}

/** FNV-1a over the bytes, two lanes: a 16-hex-digit name */
function hash(b: Uint8Array): string {
  let h1 = 0x811c9dc5, h2 = 0x01000193 ^ b.length;
  for (let i = 0; i < b.length; i++) {
    h1 = Math.imul(h1 ^ b[i], 0x01000193) >>> 0;
    h2 = Math.imul(h2 ^ b[i], 0x5bd1e995) >>> 0;
  }
  return h1.toString(16).padStart(8, "0") + h2.toString(16).padStart(8, "0");
}

/** The scene as glTF binary bytes. */
export function glbBytes(scene: GlbScene): Uint8Array {
  const chunks: Uint8Array[] = [];
  let byteLength = 0;
  const bufferViews: Record<string, unknown>[] = [];
  const accessors: Record<string, unknown>[] = [];
  const view = (bytes: Uint8Array, target?: number) => {
    while (byteLength % 4) {
      chunks.push(new Uint8Array(1));
      byteLength++;
    }
    bufferViews.push({ buffer: 0, byteOffset: byteLength, byteLength: bytes.length, ...(target ? { target } : {}) });
    chunks.push(bytes);
    byteLength += bytes.length;
    return bufferViews.length - 1;
  };
  const SIZES: Record<string, number> = { SCALAR: 1, VEC2: 2, VEC3: 3, VEC4: 4, MAT4: 16 };
  const floats = (data: ArrayLike<number>, type: string, target?: number, minmax = false) => {
    const f = Float32Array.from(data);
    const n = SIZES[type];
    const acc: Record<string, unknown> = { bufferView: view(new Uint8Array(f.buffer), target), componentType: 5126, count: f.length / n, type };
    if (minmax) {
      const min = new Array(n).fill(Infinity), max = new Array(n).fill(-Infinity);
      for (let i = 0; i < f.length; i++) {
        min[i % n] = Math.min(min[i % n], f[i]);
        max[i % n] = Math.max(max[i % n], f[i]);
      }
      acc.min = min;
      acc.max = max;
    }
    accessors.push(acc);
    return accessors.length - 1;
  };
  const ints = (data: ArrayLike<number>, type: string, short: boolean, target?: number) => {
    const a = short ? Uint16Array.from(data) : Uint32Array.from(data);
    accessors.push({ bufferView: view(new Uint8Array(a.buffer), target), componentType: short ? 5123 : 5125, count: a.length / SIZES[type], type });
    return accessors.length - 1;
  };

  let unlit = false;
  const materials = scene.primitives.map((p) => {
    const m = p.material ?? {};
    const c = linearRgba(m.color ?? "#cccccc");
    if (m.unlit) unlit = true;
    const out: Record<string, unknown> = {
      pbrMetallicRoughness: { baseColorFactor: c, metallicFactor: m.metalness ?? 0, roughnessFactor: m.roughness ?? 0.6 },
      doubleSided: !!m.doubleSided,
    };
    if (m.blend) out.alphaMode = "BLEND";
    if (m.unlit) out.extensions = { KHR_materials_unlit: {} };
    if (m.emissive) out.emissiveFactor = linearRgba(m.emissive).slice(0, 3);
    return out;
  });
  const primitives = scene.primitives.map((p, i) => {
    const attributes: Record<string, number> = { POSITION: floats(p.positions, "VEC3", 34962, true) };
    if (p.normals) attributes.NORMAL = floats(p.normals, "VEC3", 34962);
    if (p.colors) attributes.COLOR_0 = floats(p.colors, "VEC4", 34962);
    if (p.joints) attributes.JOINTS_0 = ints(p.joints, "VEC4", true, 34962);
    if (p.weights) attributes.WEIGHTS_0 = floats(p.weights, "VEC4", 34962);
    const out: Record<string, unknown> = { attributes, material: i, mode: p.mode === "lines" ? 1 : 4 };
    if (p.indices) out.indices = ints(p.indices, "SCALAR", false, 34963);
    return out;
  });
  const nodes: Record<string, unknown>[] = [{ name: "mesh", mesh: 0 }];
  const sceneNodes = [0];
  const json: Record<string, unknown> = { asset: { version: "2.0", generator: "raea glb.ts" }, scene: 0, materials, meshes: [{ primitives }] };
  if (scene.joints?.length) {
    const J = scene.joints;
    const base = nodes.length;
    J.forEach((j, k) => {
      const ph = j.parent >= 0 ? J[j.parent].head : [0, 0, 0];
      nodes.push({ name: j.name, translation: [j.head[0] - ph[0], j.head[1] - ph[1], j.head[2] - ph[2]] });
      if (j.parent >= 0) {
        if (j.parent >= k) throw new RangeError(`glb: joint ${j.name}'s parent must come before it`);
        const pn = nodes[base + j.parent] as { children?: number[] };
        (pn.children ??= []).push(base + k);
      } else sceneNodes.push(base + k);
    });
    // inverse bind: the joint's rest world is a translation to its head
    const ibm: number[] = [];
    for (const j of J) ibm.push(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, -j.head[0], -j.head[1], -j.head[2], 1);
    json.skins = [{ joints: J.map((_, k) => base + k), inverseBindMatrices: floats(ibm, "MAT4"), skeleton: sceneNodes[1] }];
    nodes[0].skin = 0;
    if (scene.clips?.length) {
      json.animations = scene.clips.map((c) => {
        const input = floats(c.times, "SCALAR", undefined, true);
        const samplers: Record<string, unknown>[] = [];
        const channels: Record<string, unknown>[] = [];
        c.rotations.forEach((q, k) => {
          samplers.push({ input, output: floats(q, "VEC4"), interpolation: "LINEAR" });
          channels.push({ sampler: samplers.length - 1, target: { node: base + k, path: "rotation" } });
        });
        for (const [k, t] of Object.entries(c.translations ?? {})) {
          const j = J[+k];
          const ph = j.parent >= 0 ? J[j.parent].head : [0, 0, 0];
          const rest = [j.head[0] - ph[0], j.head[1] - ph[1], j.head[2] - ph[2]];
          const out = Float32Array.from(t, (v, i) => v + rest[i % 3]);
          samplers.push({ input, output: floats(out, "VEC3"), interpolation: "LINEAR" });
          channels.push({ sampler: samplers.length - 1, target: { node: base + +k, path: "translation" } });
        }
        return { name: c.name, samplers, channels };
      });
    }
  }
  json.nodes = nodes;
  json.scenes = [{ nodes: sceneNodes }];
  if (unlit) json.extensionsUsed = ["KHR_materials_unlit"];
  while (byteLength % 4) {
    chunks.push(new Uint8Array(1));
    byteLength++;
  }
  json.buffers = [{ byteLength }];
  json.bufferViews = bufferViews;
  json.accessors = accessors;

  let text = JSON.stringify(json);
  while (text.length % 4) text += " ";
  const jsonBytes = new TextEncoder().encode(text);
  const total = 12 + 8 + jsonBytes.length + 8 + byteLength;
  const out = new Uint8Array(total);
  const dv = new DataView(out.buffer);
  dv.setUint32(0, 0x46546c67, true);
  dv.setUint32(4, 2, true);
  dv.setUint32(8, total, true);
  dv.setUint32(12, jsonBytes.length, true);
  dv.setUint32(16, 0x4e4f534a, true);
  out.set(jsonBytes, 20);
  let at = 20 + jsonBytes.length;
  dv.setUint32(at, byteLength, true);
  dv.setUint32(at + 4, 0x004e4942, true);
  at += 8;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  return out;
}

let tmpDir: string | null = null;
const written = new Set<string>();

/** The scene written as a .glb named by its content (once per process), its path: a `<Model src>`. Under plain bun
 * (no engine) nothing is written and the would-be path is still returned, so a composition can be checked. */
export function writeGlb(scene: GlbScene): string {
  const bytes = glbBytes(scene);
  if (tmpDir === null) {
    const t = hasNative() ? native("host.env", { name: "TMPDIR" }) : null;
    tmpDir = (t || "/tmp/").replace(/\/?$/, "/");
  }
  const path = `${tmpDir}raea-glb-${hash(bytes)}.glb`;
  if (!written.has(path) && hasNative()) native("media.write", { path, data: base64(bytes), truncate: true });
  written.add(path);
  return path;
}
