// A three.js BufferGeometry as raea's <BufferMesh> arrays: positions, indices, normals, uvs and vertex colours, read
// through each attribute's accessors (so interleaved, normalized and typed buffers all read the same). Kept per
// geometry and its attributes' versions, so the same arrays (the same objects) go to raea every frame and raea's own
// shape cache, keyed by the positions array, writes the shape once. A skinned or morphed mesh is posed on the CPU each
// frame (three's getVertexPosition applies its morph targets and bones), its normals made again from the posed shape.
import * as THREE from "three";

export interface Arrays {
  positions: number[];
  indices?: number[];
  normals?: number[];
  uvs?: number[];
  colors?: number[];
  /** a three shader's own attributes, per vertex (ThreeShaderMaterial) */
  attributes?: Record<string, { size: number; values: number[] }>;
}

type Attr = THREE.BufferAttribute | THREE.InterleavedBufferAttribute;

function read(a: Attr | undefined, per: number, flipV = false): number[] | undefined {
  if (!a) return undefined;
  const out = new Array<number>(a.count * per);
  for (let i = 0; i < a.count; i++) {
    out[i * per] = a.getX(i);
    if (per > 1) out[i * per + 1] = flipV ? 1 - a.getY(i) : a.getY(i);
    if (per > 2) out[i * per + 2] = a.getZ(i);
    if (per > 3) out[i * per + 3] = a.itemSize > 3 ? a.getW(i) : 1;
  }
  return out;
}

/** the triangles' vertex indices, of one group (a range of a multi-material mesh) or the draw range */
function triangles(g: THREE.BufferGeometry, start: number, count: number): number[] | undefined {
  const n = g.attributes.position.count;
  const total = g.index ? g.index.count : n;
  const from = Math.max(0, start), to = Math.min(total, start + count);
  if (from === 0 && to === total && !g.index) return undefined;
  const out: number[] = [];
  for (let i = from; i + 3 <= to; i += 3) {
    for (let k = 0; k < 3; k++) out.push(g.index ? g.index.getX(i + k) : i + k);
  }
  return out;
}

const versions = (g: THREE.BufferGeometry) =>
  [g.index?.version ?? -1, ...Object.values(g.attributes).map((a) => ("version" in a ? a.version : (a as THREE.InterleavedBufferAttribute).data.version)), g.drawRange.start, g.drawRange.count].join(",");

const kept = new WeakMap<THREE.BufferGeometry, Map<string, { v: string; arrays: Arrays }>>();

export interface ReadOptions {
  /** the range of triangles (a geometry group), by index; the draw range by default */
  start?: number;
  count?: number;
  /** v turned upside down: three's texture coordinates count v from the bottom of a picture (flipY), Bevy's from the top */
  flipV: boolean;
  /** the vertex colours, when the material uses them */
  colors: boolean;
  /** each triangle wound the other way: a mesh whose transform mirrors it (a negative scale), which three draws
   *  with its front face flipped and Bevy culls as it's wound */
  mirror?: boolean;
}

/** the triangles wound the other way (indices made for a geometry without them) */
export function rewound(a: Arrays): Arrays {
  const idx = a.indices ?? Array.from({ length: a.positions.length / 3 }, (_, i) => i);
  const out = idx.slice();
  for (let i = 0; i + 2 < out.length; i += 3) [out[i + 1], out[i + 2]] = [out[i + 2], out[i + 1]];
  return { ...a, indices: out };
}

/** A geometry's arrays for raea, the same objects while it hasn't changed. */
export function arraysOf(g: THREE.BufferGeometry, o: ReadOptions): Arrays | undefined {
  const pos = g.attributes.position;
  if (!pos || pos.count < 3) return undefined;
  const start = Math.max(o.start ?? 0, g.drawRange.start);
  const end = Math.min(o.count === undefined ? Infinity : (o.start ?? 0) + o.count, g.drawRange.count === Infinity ? Infinity : g.drawRange.start + g.drawRange.count);
  const key = `${start}:${end}:${o.flipV}:${o.colors}:${!!o.mirror}`;
  const v = versions(g);
  let m = kept.get(g);
  if (!m) kept.set(g, (m = new Map()));
  const had = m.get(key);
  if (had && had.v === v) return had.arrays;
  const indices = triangles(g, start, end - start);
  const arrays: Arrays = {
    positions: read(pos, 3)!,
    indices,
    normals: read(g.attributes.normal, 3),
    uvs: read(g.attributes.uv, 2, o.flipV),
    colors: o.colors ? read(g.attributes.color, 4) : undefined,
  };
  if (indices && indices.length < 3) return undefined;
  const out = o.mirror ? rewound(arrays) : arrays;
  m.set(key, { v, arrays: out });
  return out;
}

/** whether a mesh's shape moves from frame to frame: bones, or morph targets with any weight */
export function posed(mesh: THREE.Mesh): boolean {
  return (mesh as THREE.SkinnedMesh).isSkinnedMesh === true || (mesh.morphTargetInfluences?.some((w) => w !== 0) ?? false);
}

/** A skinned or morphed mesh's shape as it stands this frame, in the mesh's own space: new arrays each frame. */
export function posedArrays(mesh: THREE.Mesh, o: ReadOptions): Arrays | undefined {
  const base = arraysOf(mesh.geometry, o);
  if (!base) return undefined;
  const g = mesh.geometry;
  if ((mesh as THREE.SkinnedMesh).isSkinnedMesh) (mesh as THREE.SkinnedMesh).skeleton.update();
  const v = new THREE.Vector3();
  const n = g.attributes.position.count;
  const positions = new Array<number>(n * 3);
  for (let i = 0; i < n; i++) {
    mesh.getVertexPosition(i, v);
    positions[i * 3] = v.x;
    positions[i * 3 + 1] = v.y;
    positions[i * 3 + 2] = v.z;
  }
  // the posed shape's own normals, smooth over shared vertices as the geometry's were
  const shaped = new THREE.BufferGeometry();
  shaped.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
  if (g.index) shaped.setIndex(g.index);
  shaped.computeVertexNormals();
  const normals = base.normals ? read(shaped.attributes.normal, 3) : undefined;
  shaped.dispose();
  return { ...base, positions, normals };
}
