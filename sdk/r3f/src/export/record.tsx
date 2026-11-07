// A raea composition's frames baked into what the source export writes (docs/r3f.feature R3F9): each frame rendered as
// raea renders it (react-dom/server into a page), drawn by the three.js host (Scene3d), and its scene read back object
// by object: what each object is (its first frame's), and per frame what moves (transform, visibility), what a
// material, light, instancing, model or camera changes to. A value the same on every frame is written once.
import type { ComponentType, ReactNode } from "react";
import * as THREE from "three";
import { Scene3d } from "../three/scene";
import type { ModelPart } from "../three/things";
import { backgroundUnder3d } from "../three/page";
import { standInImages } from "../input/texture";
import { page, reconcile, type El } from "./dom";

export interface Rec {
  id: number;
  parent: number;
  obj: THREE.Object3D;
  /** per frame, each only when it changes: position, quaternion, scale, visible, the material's settings (a list per
   * material), the light's settings, the instances' matrices, the model's state, the geometry's id */
  p: number[][]; q: number[][]; s: number[][]; v: boolean[];
  mat: string[]; light: string[]; inst: number[][]; model: string[]; geo: number[];
}

export interface Recording {
  fps: number; width: number; height: number; frames: number;
  recs: Rec[];
  geometries: THREE.BufferGeometry[];
  camera: { kind: "perspective" | "orthographic"; p: number[][]; q: number[][]; fov: number[]; height: number[]; near: number; far: number };
  settings: Scene3d["settings"];
  background?: string;
  fog?: { color: string; near?: number; far?: number; density?: number };
  environment?: Record<string, unknown>;
  /** the frame effects a frame (the lens and motion blur animate), and the camera's focus for depth of field */
  effects: Scene3d["effects"][];
  focus: number[];
}

export const r5 = (x: number) => Math.round(x * 1e5) / 1e5;
const arr = (a: ArrayLike<number>) => Array.from(a, r5);
const hex = (c: THREE.Color) => "#" + c.getHexString();

/** the settings of a material that differ from a fresh one of its class, as JSON (textures by url) */
export const MATERIAL_KEYS = [
  "color", "emissive", "emissiveIntensity", "roughness", "metalness", "opacity", "transparent", "alphaTest", "side", "flatShading",
  "vertexColors", "depthWrite", "depthTest", "colorWrite", "toneMapped", "wireframe", "fog", "clearcoat", "clearcoatRoughness",
  "transmission", "ior", "thickness", "attenuationDistance", "attenuationColor", "specularIntensity", "specularColor", "sheen",
  "sheenRoughness", "sheenColor", "iridescence", "anisotropy", "reflectivity", "envMapIntensity", "size", "sizeAttenuation",
  "rotation", "blending", "premultipliedAlpha", "map", "emissiveMap", "normalMap", "roughnessMap", "metalnessMap", "aoMap",
  "alphaMap", "bumpMap", "displacementMap", "gradientMap", "clearcoatMap", "transmissionMap", "lightMap",
];
export const SRGB_MAPS = new Set(["map", "emissiveMap", "specularColorMap", "sheenColorMap"]);

/** a texture's source, as a url a TextureLoader (or a <video>) takes */
export function textureUrl(t: THREE.Texture): string | undefined {
  if (typeof t.userData.src === "string") return t.userData.src;
  const img = t.image as { src?: string; currentSrc?: string; toDataURL?: () => string; data?: ArrayLike<number>; width?: number; height?: number } | undefined;
  if (!img) return undefined;
  if (typeof img.currentSrc === "string" && img.currentSrc) return img.currentSrc;
  if (typeof img.src === "string" && img.src) return img.src;
  try {
    return img.toDataURL?.();
  } catch {
    return undefined;
  }
}

function value(v: unknown): unknown {
  if (v instanceof THREE.Color) return hex(v);
  if (v instanceof THREE.Texture) {
    const url = textureUrl(v);
    if (!url) return undefined;
    const t: Record<string, unknown> = { tex: url };
    if (v.repeat.x !== 1 || v.repeat.y !== 1) t.repeat = arr(v.repeat.toArray());
    if (v.offset.x !== 0 || v.offset.y !== 0) t.offset = arr(v.offset.toArray());
    if (v.rotation) t.rotation = r5(v.rotation);
    if (v.wrapS !== THREE.ClampToEdgeWrapping || v.wrapT !== THREE.ClampToEdgeWrapping) t.wrap = [v.wrapS, v.wrapT];
    if (v.magFilter === THREE.NearestFilter) t.nearest = true;
    if (v.flipY === false) t.flipY = false;
    return t;
  }
  return typeof v === "number" ? r5(v) : v;
}

const fresh = new Map<string, Record<string, unknown>>();
export function materialOf(m: THREE.Material): { type: string; props: Record<string, unknown> } {
  const C = m.constructor as new () => THREE.Material;
  let base = fresh.get(m.type);
  if (!base) {
    base = new C() as unknown as Record<string, unknown>;
    fresh.set(m.type, base);
  }
  const props: Record<string, unknown> = {};
  const own = m as unknown as Record<string, unknown>;
  for (const k of MATERIAL_KEYS) {
    if (!(k in own)) continue;
    const v = value(own[k]), b = value(base[k]);
    if (v !== undefined && JSON.stringify(v) !== JSON.stringify(b)) props[k] = v;
  }
  // a shader three.js core makes for an addon (Sky): its uniforms
  if ((m as THREE.ShaderMaterial).isShaderMaterial) {
    const u: Record<string, unknown> = {};
    for (const [k, x] of Object.entries((m as THREE.ShaderMaterial).uniforms)) {
      const v = x.value as unknown;
      if (typeof v === "number" || typeof v === "boolean") u[k] = typeof v === "number" ? r5(v) : v;
      else if (v instanceof THREE.Vector3 || v instanceof THREE.Vector2 || v instanceof THREE.Vector4) u[k] = arr(v.toArray());
      else if (v instanceof THREE.Color) u[k] = hex(v);
    }
    props.uniforms = u;
  }
  return { type: m.type, props };
}

const LIGHT_KEYS = ["color", "intensity", "distance", "decay", "angle", "penumbra", "width", "height", "groundColor", "castShadow"];
function lightOf(l: THREE.Light): string {
  const own = l as unknown as Record<string, unknown>;
  const out: Record<string, unknown> = {};
  for (const k of LIGHT_KEYS) if (k in own) out[k] = value(own[k]);
  const t = (l as THREE.DirectionalLight).target;
  if (t) out.target = arr(t.getWorldPosition(new THREE.Vector3()).toArray());
  if (l.shadow && l.castShadow) {
    const c = l.shadow.camera as THREE.OrthographicCamera & THREE.PerspectiveCamera;
    const camera: Record<string, number> = { near: r5(c.near), far: r5(c.far) };
    if ((c as THREE.OrthographicCamera).isOrthographicCamera) Object.assign(camera, { left: r5(c.left), right: r5(c.right), top: r5(c.top), bottom: r5(c.bottom) });
    out.shadow = { mapSize: [l.shadow.mapSize.x, l.shadow.mapSize.y], bias: r5(l.shadow.bias), normalBias: r5(l.shadow.normalBias), camera };
  }
  return JSON.stringify(out);
}

/** bake `frames` frames of a composition's component from `from` */
export function record(component: ComponentType<Record<string, unknown>> | (() => ReactNode), opts: { width: number; height: number; fps: number; from?: number; frames?: number; props?: Record<string, unknown> }): Recording {
  standInImages();
  const { width, height, fps } = opts, from = opts.from ?? 0, frames = Math.max(1, opts.frames ?? 1);
  const C = component as ComponentType<Record<string, unknown>>;
  const host = new Scene3d();
  const recs = new Map<THREE.Object3D, Rec>();
  const geometries: THREE.BufferGeometry[] = [];
  const cam: Recording["camera"] = { kind: "perspective", p: [], q: [], fov: [], height: [], near: 0.1, far: 1000 };
  let background: string | undefined;
  const effects: Scene3d["effects"][] = [], focus: number[] = [];
  let tree: El | undefined;
  for (let i = 0; i < frames; i++) {
    const f = from + i;
    const fresh = page(<C {...(opts.props ?? {})} />, f, [width, height], fps);
    tree = tree ? reconcile(tree, fresh) : fresh;
    const el = tree as unknown as Element;
    host.sync(el, f / fps);
    host.fit(width, height);
    if (i === 0) background = backgroundUnder3d(el);
    host.scene.updateMatrixWorld(true);
    const seen = new Set<Rec>();
    const visit = (o: THREE.Object3D, parent: number) => {
      let r = recs.get(o);
      if (!r) {
        r = { id: recs.size, parent, obj: o, p: [], q: [], s: [], v: [], mat: [], light: [], inst: [], model: [], geo: [] };
        recs.set(o, r);
      }
      seen.add(r);
      r.p[i] = arr(o.position.toArray());
      r.q[i] = arr(o.quaternion.toArray());
      r.s[i] = arr(o.scale.toArray());
      r.v[i] = o.visible;
      const mesh = o as THREE.Mesh;
      if (mesh.geometry) {
        let g = geometries.indexOf(mesh.geometry);
        if (g < 0) g = geometries.push(mesh.geometry) - 1;
        r.geo[i] = g;
      }
      if (mesh.material) r.mat[i] = JSON.stringify((Array.isArray(mesh.material) ? mesh.material : [mesh.material]).map(materialOf));
      if ((o as THREE.Light).isLight) r.light[i] = lightOf(o as THREE.Light);
      if ((o as THREE.InstancedMesh).isInstancedMesh) r.inst[i] = arr((o as THREE.InstancedMesh).instanceMatrix.array as Float32Array);
      const model = o.userData.raeaModel as ModelPart | undefined;
      if (model) {
        r.model[i] = JSON.stringify(model.state);
        return; // its glTF's insides are the <Model>'s own
      }
      for (const c of o.children) visit(c, r.id);
    };
    for (const c of host.scene.children) visit(c, -1);
    for (const r of recs.values()) if (!seen.has(r)) r.v[i] = false;
    const c = host.camera;
    cam.p[i] = arr(c.position.toArray());
    cam.q[i] = arr(c.quaternion.toArray());
    if (c instanceof THREE.OrthographicCamera) {
      cam.kind = "orthographic";
      cam.height[i] = r5(c.top - c.bottom);
    } else cam.fov[i] = r5((c as THREE.PerspectiveCamera).fov);
    cam.near = (c as THREE.PerspectiveCamera).near;
    cam.far = (c as THREE.PerspectiveCamera).far;
    effects[i] = JSON.parse(JSON.stringify(host.effects)) as Scene3d["effects"];
    focus[i] = r5((c.userData.focus as number | undefined) ?? 10);
  }
  const fog = host.scene.fog;
  return {
    fps, width, height, frames,
    recs: [...recs.values()],
    geometries,
    camera: cam,
    settings: { ...host.settings },
    background: host.scene.background instanceof THREE.Color ? hex(host.scene.background) : background,
    fog: fog instanceof THREE.Fog ? { color: hex(fog.color), near: r5(fog.near), far: r5(fog.far) } : fog instanceof THREE.FogExp2 ? { color: hex(fog.color), density: r5(fog.density) } : undefined,
    environment: host.scene.userData.raeaEnvironment as Record<string, unknown> | undefined,
    effects,
    focus,
  };
}
