// A three.js scene as raea's parts. The scene graph keeps its shape: an object with children is a <Group> at its own
// transform (raea composes them as three does, shear included), a mesh a <Mesh> with its <BufferMesh> and material.
// An InstancedMesh is <Instances> while its copies hold still and one mesh a copy when they move or each has its own
// colour; Points are one mesh of quads facing the camera; lines are a <Gizmo>; a Sprite is a quad facing the camera.
// Lights, the camera, fog and the environment come out in world space (lights.ts). Each part is named by its place in
// the tree (and its own name), so raea matches it across frames.
import * as THREE from "three";
import { arraysOf, posed, posedArrays, rewound, type Arrays } from "./geometry";
import { isTroikaText, textParts } from "./text";
import { clipArrays, clipOf, type Clip } from "./clip";
import { cameraOf, fogOf, lightsOf, skyOf, toneOf } from "./lights";
import { materialOf, type Part, type Props } from "./material";
import { isShader, ownAttributes, shaderPart } from "./shader";
import { composerEffects, postParts } from "./post";
import { classicOf } from "./tsl";

type V3 = [number, number, number];
type Q = [number, number, number, number];

/** an object's own transform, from its matrix (so matrixAutoUpdate={false} objects come out as three draws them) */
function place(o: THREE.Object3D): Props {
  const p = new THREE.Vector3(), q = new THREE.Quaternion(), s = new THREE.Vector3();
  o.matrix.decompose(p, q, s);
  const out: Props = { position: p.toArray() as V3, quaternion: q.toArray() as Q };
  out.scale = s.x === s.y && s.y === s.z && s.x > 0 ? s.x : (s.toArray() as V3);
  return out;
}

const hex = (c: THREE.Color) => "#" + c.getHexString();

const warned = new Set<string>();
function warnOnce(k: string, why: string) {
  if (warned.has(k)) return;
  warned.add(k);
  console.warn(`raea/r3f: ${k} ${why}`);
}

export interface Walk {
  camera: THREE.Camera;
  /** the renderer's shadow map is on (<Canvas shadows>) */
  shadows: boolean;
  /** lights and the camera, gathered in world space */
  world: Part[];
  /** the drawing's height in pixels (a point's size without attenuation is in pixels) */
  height: number;
  scene: THREE.Scene;
  /** R3F's clock this frame and its step, seconds (a node material's `time` and `deltaTime`) */
  time: number;
  delta: number;
  /** the renderer (its clipping planes) */
  gl?: THREE.WebGLRenderer;
  /** the clipping groups (three/webgpu's ClippingGroup) the walk is inside */
  clips: Clip[];
}

function flipped(m: THREE.Material | THREE.Material[]): boolean {
  // a node material's graph reads its uvs as three gives them, and flips where it samples a flipped picture (tsl.ts)
  const one = Array.isArray(m) ? m[0] : m;
  if ((one as { isNodeMaterial?: boolean } | undefined)?.isNodeMaterial && !classicOf(one)) return false;
  // a ShaderMaterial reads its uvs as three gives them; the engine flips where it samples a flipped picture
  if (one && isShader(one)) return false;
  const map = ((Array.isArray(m) ? m[0] : m) as THREE.MeshBasicMaterial | undefined)?.map;
  return map ? map.flipY : true;
}

function colouredByVertex(m: THREE.Material) {
  return m.vertexColors === true;
}

/** normals as colours (MeshNormalMaterial, in the camera's space as three draws them) */
function normalColours(a: Arrays, mesh: THREE.Object3D, camera: THREE.Camera): Arrays {
  if (!a.normals) return a;
  const toView = new THREE.Matrix3().getNormalMatrix(new THREE.Matrix4().multiplyMatrices(camera.matrixWorldInverse, mesh.matrixWorld));
  const v = new THREE.Vector3();
  const colors: number[] = [];
  for (let i = 0; i < a.normals.length; i += 3) {
    v.fromArray(a.normals, i).applyMatrix3(toView).normalize();
    // three writes the normal's 0.5 + 0.5n as the screen value (sRGB), raea's vertex colours are linear
    const c = new THREE.Color().setRGB(0.5 + 0.5 * v.x, 0.5 + 0.5 * v.y, 0.5 + 0.5 * v.z, THREE.SRGBColorSpace);
    colors.push(c.r, c.g, c.b, 1);
  }
  return { ...a, colors };
}

function meshParts(mesh: THREE.Mesh, name: string, w: Walk, at: Props): Part[] {
  const mats = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
  const g = mesh.geometry;
  const ranges = Array.isArray(mesh.material) && g.groups.length ? g.groups.map((r) => ({ start: r.start, count: r.count, material: mats[r.materialIndex ?? 0] })) : [{ start: undefined, count: undefined, material: mats[0] }];
  const out: Part[] = [];
  // a mirroring transform (drei's <ContactShadows> plane is scaled -1 in y): three flips its front face, Bevy doesn't
  mesh.updateWorldMatrix(true, false);
  const mirror = mesh.matrixWorld.determinant() < 0;
  ranges.forEach((r, i) => {
    if (!r.material || !r.material.visible) return;
    const o = { start: r.start, count: r.count, flipV: flipped(r.material), colors: colouredByVertex(r.material), mirror };
    let arrays = posed(mesh) ? posedArrays(mesh, { ...o, mirror: false }) : arraysOf(g, o);
    if (arrays && mirror && posed(mesh)) arrays = rewound(arrays);
    if (!arrays) return;
    const surface = materialOf(r.material, o.colors, { object: mesh, scene: w.scene, camera: w.camera, time: w.time, delta: w.delta });
    if (!surface.type) return;
    if (surface.normals) arrays = normalColours(arrays, mesh, w.camera);
    // clipping planes cut the triangles (clip.ts); a three shader's own attributes would lose their count
    const clip = clipOf(r.material, w.gl, w.clips);
    if (clip) {
      if ((arrays as { attributes?: unknown }).attributes) warnOnce("a clipped ShaderMaterial with attributes of its own", "is drawn whole");
      else {
        mesh.updateWorldMatrix(true, false);
        const cut = clipArrays(arrays, clip, mesh.matrixWorld);
        if (!cut) return;
        arrays = cut;
      }
    }
    // a three shader's own attributes (aRandom, aOffset...), numbered as the BufferMesh's vertices are
    if (surface.type === "ThreeShaderMaterial" && !posed(mesh) && arrays.positions.length / 3 === g.attributes.position?.count) {
      const own = attributesOf(g);
      if (own) arrays = { ...arrays, attributes: own } as Arrays;
    }
    const props: Props = { name: ranges.length > 1 ? `${name}#${i}` : name, ...at, castShadow: w.shadows && mesh.castShadow, receiveShadow: mesh.receiveShadow };
    if (surface.catchShadow) props.shadow = "catch";
    out.push({ type: "Mesh", props, children: [{ type: "BufferMesh", props: { ...arrays } }, surface] });
  });
  return out;
}

/** whose copies have moved since the first frame: those are drawn one mesh a copy */
const firstPlacement = new WeakMap<THREE.InstancedMesh, number>();

function instancedParts(m: THREE.InstancedMesh, name: string, w: Walk, at: Props): Part[] {
  const v = m.instanceMatrix.version;
  if (!firstPlacement.has(m)) firstPlacement.set(m, v);
  const moving = firstPlacement.get(m) !== v || m.instanceColor !== null || Array.isArray(m.material);
  const one = new THREE.Matrix4();
  const p = new THREE.Vector3(), q = new THREE.Quaternion(), s = new THREE.Vector3(), e = new THREE.Euler();
  const copies: { position: V3; quaternion: Q; rotation: V3; scale: V3 }[] = [];
  for (let i = 0; i < m.count; i++) {
    m.getMatrixAt(i, one);
    one.decompose(p, q, s);
    copies.push({ position: p.toArray() as V3, quaternion: q.toArray() as Q, rotation: e.setFromQuaternion(q, "XYZ").toArray().slice(0, 3) as V3, scale: s.toArray() as V3 });
  }
  if (!moving) {
    const material = m.material as THREE.Material;
    const arrays = arraysOf(m.geometry, { flipV: flipped(material), colors: colouredByVertex(material) });
    const surface = materialOf(material, false, { object: m, scene: w.scene, camera: w.camera, time: w.time, delta: w.delta });
    if (!arrays || !surface.type) return [];
    return [{ type: "Instances", props: { name, ...at, at: copies.map(({ position, rotation, scale }) => ({ position, rotation, scale })), shadow: w.shadows && m.castShadow, receiveShadow: m.receiveShadow }, children: [{ type: "BufferMesh", props: { ...arrays } }, surface] }];
  }
  const kids: Part[] = [];
  const c = new THREE.Color();
  copies.forEach((k, i) => {
    for (const part of meshParts(m, `${name}.${i}`, w, { position: k.position, quaternion: k.quaternion, scale: k.scale })) {
      if (m.instanceColor) {
        m.getColorAt(i, c);
        const surface = part.children![1];
        const base = new THREE.Color((surface.props.color as string) ?? "#ffffff");
        surface.props = { ...surface.props, color: hex(base.multiply(c)) };
      }
      kids.push(part);
    }
  });
  return [{ type: "Group", props: { name, ...at }, children: kids }];
}

/** a BatchedMesh: each of its instances a mesh of its own geometry's range, at its matrix, in its colour */
function batchedParts(b: THREE.BatchedMesh, name: string, w: Walk): Part[] {
  const info = (b as unknown as { _instanceInfo: { active: boolean; visible: boolean }[] })._instanceInfo;
  const material = b.material as THREE.Material;
  if (!material.visible) return [];
  const surface = materialOf(material, false);
  if (!surface.type) return [];
  const one = new THREE.Matrix4(), p = new THREE.Vector3(), q = new THREE.Quaternion(), s = new THREE.Vector3(), c = new THREE.Color();
  const colours = (b as unknown as { _colorsTexture: unknown })._colorsTexture !== null;
  const out: Part[] = [];
  info.forEach((k, i) => {
    if (!k.active || !k.visible) return;
    const range = b.getGeometryRangeAt(b.getGeometryIdAt(i));
    if (!range) return;
    const arrays = arraysOf(b.geometry, { start: range.start, count: range.count, flipV: flipped(material), colors: false });
    if (!arrays) return;
    b.getMatrixAt(i, one);
    one.decompose(p, q, s);
    let own = surface;
    if (colours) {
      b.getColorAt(i, c);
      own = { ...surface, props: { ...surface.props, color: hex(new THREE.Color((surface.props.color as string) ?? "#ffffff").multiply(c)) } };
    }
    out.push({ type: "Mesh", props: { name: `${name}.${i}`, position: p.toArray(), quaternion: q.toArray(), scale: s.toArray(), castShadow: w.shadows && b.castShadow, receiveShadow: b.receiveShadow }, children: [{ type: "BufferMesh", props: { ...arrays } }, own] });
  });
  return out;
}

/** a geometry's own attributes, read again only when one of them changes */
const attrCache = new WeakMap<THREE.BufferGeometry, { key: string; value: ReturnType<typeof ownAttributes> }>();
function attributesOf(g: THREE.BufferGeometry) {
  const key = Object.entries(g.attributes).map(([k, a]) => `${k}:${(a as THREE.BufferAttribute).version}`).join(",");
  const hit = attrCache.get(g);
  if (hit?.key === key) return hit.value;
  const value = ownAttributes(g);
  attrCache.set(g, { key, value });
  return value;
}

/** Points drawn by a ShaderMaterial: a quad at each point in the object's own space, its corners as `uv`, which the
 * engine spreads gl_PointSize pixels on screen (each attribute repeated at the quad's four corners) */
function shaderPointParts(pts: THREE.Points, mat: THREE.ShaderMaterial, name: string, at: Props): Part[] {
  const g = pts.geometry;
  const pos = g.attributes.position;
  const from = g.drawRange.start, to = Math.min(pos.count, g.drawRange.start + g.drawRange.count);
  const own = attributesOf(g);
  const colours = mat.vertexColors && g.attributes.color;
  const positions: number[] = [], uvs: number[] = [], colors: number[] = [], indices: number[] = [];
  const attributes: Record<string, { size: number; values: number[] }> = {};
  for (const [k, a] of Object.entries(own ?? {})) attributes[k] = { size: a.size, values: [] };
  for (let i = from; i < to; i++) {
    const base = positions.length / 3;
    for (const [u, v] of [[0, 0], [1, 0], [1, 1], [0, 1]]) {
      positions.push(pos.getX(i), pos.getY(i), pos.getZ(i));
      uvs.push(u, v);
      if (colours) colors.push(colours.getX(i), colours.getY(i), colours.getZ(i), colours.itemSize > 3 ? colours.getW(i) : 1);
      for (const [k, a] of Object.entries(own ?? {})) attributes[k].values.push(...a.values.slice(i * a.size, i * a.size + a.size));
    }
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }
  if (!indices.length) return [];
  const arrays = { positions, indices, uvs, ...(colours ? { colors } : {}), ...(own ? { attributes } : {}) };
  return [{ type: "Mesh", props: { name, ...at, castShadow: false, receiveShadow: false }, children: [{ type: "BufferMesh", props: arrays as unknown as Props }, shaderPart(mat, true)] }];
}

/** a quad facing the camera at each point: one mesh, its vertex colours the points' */
function pointParts(pts: THREE.Points, name: string, w: Walk, at: Props): Part[] {
  const g = pts.geometry, mat = pts.material as THREE.PointsMaterial;
  const pos = g.attributes.position;
  if (!pos || !mat.visible) return [];
  if (isShader(mat)) return shaderPointParts(pts, mat, name, at);
  const cam = w.camera;
  const right = new THREE.Vector3(1, 0, 0).applyQuaternion(cam.getWorldQuaternion(new THREE.Quaternion()));
  const up = new THREE.Vector3(0, 1, 0).applyQuaternion(cam.getWorldQuaternion(new THREE.Quaternion()));
  const camAt = cam.getWorldPosition(new THREE.Vector3());
  const persp = (cam as THREE.PerspectiveCamera).isPerspectiveCamera ? (cam as THREE.PerspectiveCamera) : undefined;
  const tanHalf = persp ? Math.tan((persp.fov * Math.PI) / 360) / persp.zoom : 1;
  const height = persp ? 0 : ((cam as THREE.OrthographicCamera).top - (cam as THREE.OrthographicCamera).bottom) / (cam as THREE.OrthographicCamera).zoom;
  const px = w.height;
  const colours = mat.vertexColors && g.attributes.color;
  const positions: number[] = [], uvs: number[] = [], colors: number[] = [], indices: number[] = [];
  const v = new THREE.Vector3();
  const from = g.drawRange.start, to = Math.min(pos.count, g.drawRange.start + g.drawRange.count);
  for (let i = from; i < to; i++) {
    v.fromBufferAttribute(pos, i).applyMatrix4(pts.matrixWorld);
    // three's point size: with attenuation, `size` scaled as a world-space object of size·tan(fov/2); without, pixels
    const dist = v.distanceTo(camAt);
    const side = persp ? (mat.sizeAttenuation ? mat.size * tanHalf : (mat.size * 2 * dist * tanHalf) / px) : mat.sizeAttenuation ? mat.size : (mat.size * height) / px;
    const h = side / 2;
    const base = positions.length / 3;
    for (const [sx, sy, u, vv] of [[-1, -1, 0, 1], [1, -1, 1, 1], [1, 1, 1, 0], [-1, 1, 0, 0]]) {
      positions.push(v.x + (right.x * sx + up.x * sy) * h, v.y + (right.y * sx + up.y * sy) * h, v.z + (right.z * sx + up.z * sy) * h);
      uvs.push(u, vv);
      if (colours) colors.push(colours.getX(i), colours.getY(i), colours.getZ(i), colours.itemSize > 3 ? colours.getW(i) : 1);
    }
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }
  if (!indices.length) return [];
  const surface = materialOf(mat, !!colours);
  const arrays: Arrays = { positions, indices, uvs, colors: colours ? colors : undefined };
  return [{ type: "Mesh", props: { name, castShadow: false, receiveShadow: false }, children: [{ type: "BufferMesh", props: arrays as unknown as Props }, surface] }];
}

/** lines as raea's gizmo lines, in world space */
function lineParts(l: THREE.Line, name: string): Part[] {
  const g = l.geometry, mat = l.material as THREE.LineBasicMaterial;
  const pos = g.attributes.position;
  if (!pos || !mat.visible) return [];
  const idx = g.index;
  const n = idx ? idx.count : pos.count;
  const point = (i: number) => new THREE.Vector3().fromBufferAttribute(pos, idx ? idx.getX(i) : i).applyMatrix4(l.matrixWorld).toArray() as V3;
  const colour = (i: number) => {
    const c = mat.vertexColors && g.attributes.color ? new THREE.Color().fromBufferAttribute(g.attributes.color as THREE.BufferAttribute, idx ? idx.getX(i) : i).multiply(mat.color) : mat.color;
    return hex(c);
  };
  const width = mat.linewidth > 0 ? mat.linewidth : 1;
  // a LineDashedMaterial: the dashes of each segment, by the line's distances (computeLineDistances) as three's shader
  // cuts them (a fragment is drawn where scale x distance mod (dash + gap) is under dash); without the distances three
  // draws it solid, as here
  const dashed = mat as unknown as THREE.LineDashedMaterial;
  const along = g.attributes.lineDistance;
  if (dashed.isLineDashedMaterial && along) {
    const at = (i: number) => along.getX(idx ? idx.getX(i) : i) * dashed.scale;
    const period = dashed.dashSize + dashed.gapSize;
    const lines: [V3, V3][] = [], cs: string[] = [];
    const cut = (i: number, j: number) => {
      const a = new THREE.Vector3(...point(i)), b = new THREE.Vector3(...point(j));
      const d0 = at(i), d1 = at(j);
      if (!(period > 0) || d1 === d0) return;
      const lo = Math.min(d0, d1), hi = Math.max(d0, d1);
      for (let start = Math.floor(lo / period) * period; start < hi; start += period) {
        const s0 = Math.max(lo, start), s1 = Math.min(hi, start + dashed.dashSize);
        if (s1 <= s0) continue;
        const p0 = a.clone().lerp(b, (s0 - d0) / (d1 - d0)), p1 = a.clone().lerp(b, (s1 - d0) / (d1 - d0));
        lines.push([p0.toArray() as V3, p1.toArray() as V3]);
        cs.push(colour(i), colour(j));
      }
    };
    if ((l as THREE.LineSegments).isLineSegments) for (let i = 0; i + 1 < n; i += 2) cut(i, i + 1);
    else {
      for (let i = 0; i + 1 < n; i++) cut(i, i + 1);
      if ((l as THREE.LineLoop).isLineLoop && n > 2) cut(n - 1, 0);
    }
    return lines.length ? [{ type: "Gizmo", props: { name, lines, color: mat.vertexColors ? cs : hex(mat.color), width, depthBias: 0 } }] : [];
  }
  if ((l as THREE.LineSegments).isLineSegments) {
    const lines: [V3, V3][] = [], cs: string[] = [];
    for (let i = 0; i + 1 < n; i += 2) {
      lines.push([point(i), point(i + 1)]);
      cs.push(colour(i), colour(i + 1));
    }
    return lines.length ? [{ type: "Gizmo", props: { name, lines, color: mat.vertexColors ? cs : hex(mat.color), width, depthBias: 0 } }] : [];
  }
  const strip: V3[] = [], cs: string[] = [];
  for (let i = 0; i < n; i++) {
    strip.push(point(i));
    cs.push(colour(i));
  }
  if ((l as THREE.LineLoop).isLineLoop && n > 2) {
    strip.push(point(0));
    cs.push(colour(0));
  }
  return strip.length > 1 ? [{ type: "Gizmo", props: { name, strip, color: mat.vertexColors ? cs : hex(mat.color), width, depthBias: 0 } }] : [];
}

/** a sprite: a quad at its place facing the camera, turned by its material's rotation, its centre where `center` says */
function spriteParts(s: THREE.Sprite, name: string, w: Walk): Part[] {
  const mat = s.material;
  if (!mat.visible) return [];
  const pos = s.getWorldPosition(new THREE.Vector3());
  const scale = s.getWorldScale(new THREE.Vector3());
  const q = w.camera.getWorldQuaternion(new THREE.Quaternion()).multiply(new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), mat.rotation));
  const [cx, cy] = [s.center.x, s.center.y];
  const positions = [-cx, -cy, 0, 1 - cx, -cy, 0, 1 - cx, 1 - cy, 0, -cx, 1 - cy, 0];
  const flip = mat.map ? mat.map.flipY : true;
  const uvs = flip ? [0, 1, 1, 1, 1, 0, 0, 0] : [0, 0, 1, 0, 1, 1, 0, 1];
  const surface = materialOf(mat, false);
  return [{ type: "Mesh", props: { name, position: pos.toArray(), quaternion: q.toArray(), scale: [scale.x, scale.y, 1], castShadow: false, receiveShadow: false }, children: [{ type: "BufferMesh", props: { positions, uvs, normals: [0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1], indices: [0, 1, 2, 0, 2, 3] } }, surface] }];
}

/** one object and what hangs from it */
function walk(o: THREE.Object3D, path: string, w: Walk): Part[] {
  if (!o.visible || !o.layers.test(w.camera.layers)) return [];
  const name = o.name ? `${path}:${o.name}` : path;
  if ((o as THREE.LOD).isLOD) (o as THREE.LOD).update(w.camera);
  if ((o as THREE.Light).isLight) {
    w.world.push(...lightsOf(o as THREE.Light, w.shadows, name));
  }
  // a ClippingGroup (three/webgpu) clips what's inside it
  const group = o as unknown as { isClippingGroup?: boolean; enabled?: boolean; clippingPlanes?: THREE.Plane[]; clipIntersection?: boolean };
  const clips = group.isClippingGroup && group.enabled !== false && group.clippingPlanes?.length;
  if (clips) w.clips.push({ planes: group.clippingPlanes!, intersection: !!group.clipIntersection });
  const kids = o.children.flatMap((c, i) => walk(c, `${path}/${i}`, w));
  if (clips) w.clips.pop();
  const at = place(o);
  let own: Part[] = [];
  // troika's Text (drei's <Text>) before the meshes: raea draws its words itself (text.ts)
  if (isTroikaText(o)) own = textParts(o, name);
  else if ((o as THREE.BatchedMesh).isBatchedMesh) own = batchedParts(o as THREE.BatchedMesh, name, w);
  else if ((o as THREE.InstancedMesh).isInstancedMesh) own = instancedParts(o as THREE.InstancedMesh, name, w, {});
  else if ((o as THREE.Mesh).isMesh) own = meshParts(o as THREE.Mesh, name, w, {});
  else if ((o as THREE.Points).isPoints) return [...pointParts(o as THREE.Points, name, w, at), ...kids.length ? [{ type: "Group", props: { name: `${name}/`, ...at }, children: kids }] : []];
  else if ((o as THREE.Line).isLine) return [...lineParts(o as THREE.Line, name), ...kids.length ? [{ type: "Group", props: { name: `${name}/`, ...at }, children: kids }] : []];
  else if ((o as THREE.Sprite).isSprite) return [...spriteParts(o as THREE.Sprite, name, w), ...kids.length ? [{ type: "Group", props: { name: `${name}/`, ...at }, children: kids }] : []];
  // a mesh alone keeps its transform on itself; with children (or a group) the transform goes on a group around both
  if (own.length === 1 && !kids.length && own[0].type === "Mesh" && !(o as THREE.BatchedMesh).isBatchedMesh) return [{ ...own[0], props: { ...own[0].props, ...at } }];
  const inside = [...own, ...kids];
  return inside.length ? [{ type: "Group", props: { name: `${name}/`, ...at }, children: inside }] : [];
}

export interface Converted {
  parts: Part[];
  /** the scene's background colour, the page's behind the 3D */
  background?: string;
}

/** R3F's scene, camera and renderer settings as raea's parts, this frame */
export function convert(scene: THREE.Scene, camera: THREE.Camera, gl: THREE.WebGLRenderer, height: number, time = 0, delta = 0): Converted {
  scene.updateMatrixWorld();
  camera.updateMatrixWorld();
  if (camera.parent === null) camera.matrixWorldInverse.copy(camera.matrixWorld).invert();
  const w: Walk = { camera, shadows: gl.shadowMap.enabled, world: [], height, scene, time, delta, gl, clips: [] };
  let drawn = scene.children.flatMap((c, i) => walk(c, `r3f/${i}`, w));
  // a scene moved, turned or scaled as a whole (rare, but three allows it)
  if (!scene.matrix.equals(new THREE.Matrix4()) && drawn.length) drawn = [{ type: "Group", props: { name: "r3f", ...place(scene) }, children: drawn }];
  // a node material runs only under three's WebGPURenderer, so a scene with one is drawn by it (whatever gl stands in)
  let webgpu = (gl as { isWebGPURenderer?: boolean }).isWebGPURenderer === true;
  if (!webgpu) scene.traverse((o) => {
    const m = (o as THREE.Mesh).material;
    if ((Array.isArray(m) ? m : m ? [m] : []).some((x) => (x as { isNodeMaterial?: boolean }).isNodeMaterial)) webgpu = true;
  });
  const sky = skyOf(scene, gl, webgpu);
  const cam = cameraOf(camera);
  const fog = fogOf(scene.fog as THREE.Fog | THREE.FogExp2 | null);
  // an <EffectComposer>'s effects (post.ts): its frame is tone-mapped only by a ToneMapping effect in it
  const composer = composerEffects(scene);
  const post = composer.found ? postParts(composer.effects) : { parts: [] as Part[], tone: undefined };
  const tone = composer.found ? post.tone ?? { type: "ToneMapping", props: { mode: "none" } } : toneOf(gl);
  return { parts: [...(cam ? [cam] : []), tone, ...(fog ? [fog] : []), ...sky.parts, ...post.parts, ...w.world, ...drawn], background: sky.background };
}
