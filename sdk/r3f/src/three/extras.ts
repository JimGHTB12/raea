// raea's parts that three.js draws by the nearest thing it has (kinds.ts says what differs): the sky, fire, grass,
// gaussian splats (their centres), 2D sprites and shapes over the 3D, rich text, a background plate and a page screen.
import * as THREE from "three";
import { Sky } from "three/examples/jsm/objects/Sky.js";
import { texture } from "./materials";
import { place } from "./things";
import { vec } from "./lights";

type Props = Record<string, unknown>;

/** seeded numbers: the same each time */
export function rng(seed: number) {
  let s = seed >>> 0 || 1;
  return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296);
}

// ---- Atmosphere: three's Sky, its sun where the first DirectionalLight shines from ----
export function sky(had: Sky | undefined, sunFrom: THREE.Vector3 | undefined, p: Props): Sky {
  const s = had ?? new Sky();
  s.scale.setScalar(4.5e5);
  const u = s.material.uniforms;
  u.turbidity.value = 2;
  u.rayleigh.value = 1;
  u.mieCoefficient.value = 0.005;
  u.mieDirectionalG.value = 0.8;
  u.sunPosition.value.copy(sunFrom ?? new THREE.Vector3(0, 1, 0.3)).normalize();
  void p;
  return s;
}

// ---- Fire: additive soft sprites rising, flickering with `time` and `seed` ----
let flame: THREE.DataTexture | undefined;
function flameTexture() {
  if (flame) return flame;
  const n = 64, d = new Uint8Array(n * n * 4);
  for (let y = 0; y < n; y++)
    for (let x = 0; x < n; x++) {
      const dx = (x + 0.5) / n - 0.5, dy = (y + 0.5) / n - 0.5;
      const a = Math.max(0, 1 - Math.hypot(dx, dy) * 2) ** 2;
      d.set([255, 255, 255, Math.round(a * 255)], (y * n + x) * 4);
    }
  flame = new THREE.DataTexture(d, n, n);
  flame.needsUpdate = true;
  return flame;
}

export function fire(had: THREE.Group | undefined, p: Props, seconds: number): THREE.Group {
  const g = had ?? new THREE.Group();
  const count = 24;
  if (!had) {
    const mat = new THREE.SpriteMaterial({ map: flameTexture(), blending: THREE.AdditiveBlending, depthWrite: false, transparent: true });
    for (let i = 0; i < count; i++) g.add(new THREE.Sprite(mat.clone()));
  }
  place(g, p);
  const t = (typeof p.time === "number" ? p.time : seconds) * ((p.speed as number) ?? 1);
  const r = rng((p.seed as number) ?? 1);
  const colour = new THREE.Color((p.color as string) ?? "#ff8a2a");
  const k = ((p.intensity as number) ?? 1) * ((p.opacity as number) ?? 1);
  g.children.forEach((c, i) => {
    const s = c as THREE.Sprite;
    const phase = (t * (0.8 + r() * 0.6) + i / count) % 1;
    const x = (r() - 0.5) * 0.5 * (1 - phase), z = (r() - 0.5) * 0.5 * (1 - phase);
    s.position.set(x, phase * 1.2, z);
    const size = 0.6 * (1 - phase) + 0.15;
    s.scale.set(size, size * 1.4, 1);
    const m = s.material as THREE.SpriteMaterial;
    m.color.copy(colour).lerp(new THREE.Color("#fff2b0"), 1 - phase).multiplyScalar(k);
    m.opacity = (1 - phase) * 0.7;
  });
  return g;
}

// ---- Grass: blades scattered over the sampled ground, one instanced mesh ----
export function grass(spec: Props): THREE.InstancedMesh | undefined {
  const s = spec.spec as { area: number[]; grid: [number, number]; height: number[]; density: number[]; tints?: [string, string][]; bladeHeight?: number; width?: [number, number]; seed?: number } | undefined;
  if (!s) return undefined;
  const [x0, z0, x1, z1] = s.area, [nx, nz] = s.grid;
  const at = (gx: number, gz: number) => {
    const i = Math.min(nx - 2, Math.max(0, Math.floor(gx))), k = Math.min(nz - 2, Math.max(0, Math.floor(gz)));
    const fx = gx - i, fz = gz - k;
    const v = (a: number[], ii: number, kk: number) => a[kk * nx + ii];
    const lerp = (a: number[]) => (v(a, i, k) * (1 - fx) + v(a, i + 1, k) * fx) * (1 - fz) + (v(a, i, k + 1) * (1 - fx) + v(a, i + 1, k + 1) * fx) * fz;
    return { h: lerp(s.height), d: lerp(s.density) };
  };
  const r = rng(s.seed ?? 7);
  const area = (x1 - x0) * (z1 - z0);
  const blades: [number, number, number, number][] = [];
  const tries = Math.min(60000, Math.round(area * Math.max(...s.density)));
  for (let i = 0; i < tries; i++) {
    const u = r(), w = r();
    const x = x0 + u * (x1 - x0), z = z0 + w * (z1 - z0);
    const g = at(u * (nx - 1), w * (nz - 1));
    if (r() * Math.max(...s.density) <= g.d) blades.push([x, g.h, z, r()]);
  }
  if (!blades.length) return undefined;
  const h = s.bladeHeight ?? 0.35, [w0] = s.width ?? [0.03, 0.01];
  const geo = new THREE.BufferGeometry();
  geo.setAttribute("position", new THREE.Float32BufferAttribute([-w0, 0, 0, w0, 0, 0, 0, h, 0], 3));
  geo.computeVertexNormals();
  const [base, tip] = s.tints?.[0] ?? ["#3f6b2a", "#9cc25a"];
  const mat = new THREE.MeshStandardMaterial({ color: new THREE.Color(base).lerp(new THREE.Color(tip), 0.5), side: THREE.DoubleSide, roughness: 0.9 });
  const m = new THREE.InstancedMesh(geo, mat, blades.length);
  const o = new THREE.Object3D();
  blades.forEach(([x, y, z, k], i) => {
    o.position.set(x, y, z);
    o.rotation.set((k - 0.5) * 0.4, k * Math.PI * 2, 0);
    o.scale.setScalar(0.7 + k * 0.6);
    o.updateMatrix();
    m.setMatrixAt(i, o.matrix);
  });
  m.receiveShadow = true;
  return m;
}

// ---- Splat: the splats' centres in their colours (a .ply of 3DGS, or antimatter's .splat) ----
export async function splatPoints(src: string): Promise<THREE.Points> {
  const buf = await (await fetch(src)).arrayBuffer();
  const positions: number[] = [], colors: number[] = [];
  const bytes = new Uint8Array(buf);
  const head = new TextDecoder().decode(bytes.subarray(0, Math.min(bytes.length, 4096)));
  if (head.startsWith("ply")) {
    const end = head.indexOf("end_header\n") + "end_header\n".length;
    const props = [...head.slice(0, end).matchAll(/property (\w+) (\w+)/g)].map((m) => ({ type: m[1], name: m[2] }));
    const size = (t: string) => (t === "double" ? 8 : t === "float" || t === "int" || t === "uint" ? 4 : t === "short" || t === "ushort" ? 2 : 1);
    const stride = props.reduce((n, p) => n + size(p.type), 0);
    const off: Record<string, number> = {};
    props.reduce((n, p) => ((off[p.name] = n), n + size(p.type)), 0);
    const count = Number(/element vertex (\d+)/.exec(head)?.[1] ?? 0);
    const dv = new DataView(buf, end);
    const SH = 0.28209479;
    for (let i = 0; i < count; i++) {
      const b = i * stride, f = (k: string) => dv.getFloat32(b + off[k], true);
      positions.push(f("x"), f("y"), f("z"));
      if ("f_dc_0" in off) colors.push(0.5 + SH * f("f_dc_0"), 0.5 + SH * f("f_dc_1"), 0.5 + SH * f("f_dc_2"));
      else colors.push(1, 1, 1);
    }
  } else {
    const dv = new DataView(buf);
    for (let b = 0; b + 32 <= buf.byteLength; b += 32) {
      positions.push(dv.getFloat32(b, true), dv.getFloat32(b + 4, true), dv.getFloat32(b + 8, true));
      colors.push(bytes[b + 24] / 255, bytes[b + 25] / 255, bytes[b + 26] / 255);
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(positions, 3));
  g.setAttribute("color", new THREE.Float32BufferAttribute(colors.map((c) => Math.min(1, Math.max(0, c))), 3));
  return new THREE.Points(g, new THREE.PointsMaterial({ size: 0.01, vertexColors: true }));
}

// ---- 2D over the 3D (Sprite, Shape2D): a flat card just in front of the camera, sized in pixels ----
export function flat(had: THREE.Mesh | undefined, kind: string, p: Props): THREE.Mesh {
  let geo: THREE.BufferGeometry;
  const [w, h] = (p.size as [number, number] | undefined) ?? [(p.width as number) ?? 100, (p.height as number) ?? 100];
  const shape = (p.shape as string) ?? "rect";
  if (kind === "Shape2D" && shape !== "rect") {
    const sides = shape === "triangle" ? 3 : shape === "polygon" ? ((p.sides as number) ?? 6) : 64;
    geo = new THREE.CircleGeometry(0.5, sides);
    geo.scale(w, h, 1);
  } else geo = new THREE.PlaneGeometry(w, h);
  const mat = new THREE.MeshBasicMaterial({ color: (p.color as string) ?? "#ffffff", transparent: true, opacity: (p.opacity as number) ?? 1, depthTest: false, toneMapped: false });
  if (kind === "Sprite" && typeof p.src === "string") mat.map = texture(p.src, true);
  if (had) {
    had.geometry.dispose();
    (had.material as THREE.Material).dispose();
  }
  const m = had ?? new THREE.Mesh();
  m.geometry = geo;
  m.material = mat;
  m.renderOrder = 1000 + ((p.z as number) ?? 0);
  m.userData.flat = { x: (p.x as number) ?? 0, y: (p.y as number) ?? 0, rotation: (p.rotation as number) ?? 0, scale: p.scale ?? 1, flipX: p.flipX, flipY: p.flipY };
  return m;
}

/** the 2D cards placed in front of the camera, a pixel of the drawing a pixel of the card */
export function placeFlat(m: THREE.Mesh, camera: THREE.Camera, width: number, height: number) {
  const f = m.userData.flat as { x: number; y: number; rotation: number; scale: number | [number, number]; flipX?: boolean; flipY?: boolean };
  const persp = camera instanceof THREE.PerspectiveCamera;
  const d = persp ? camera.near * 1.5 : 1;
  const worldPerPx = persp ? (2 * d * Math.tan((camera.fov * Math.PI) / 360)) / height : ((camera as THREE.OrthographicCamera).top - (camera as THREE.OrthographicCamera).bottom) / height;
  const [sx, sy] = Array.isArray(f.scale) ? f.scale : [f.scale, f.scale];
  const q = camera.getWorldQuaternion(new THREE.Quaternion());
  const off = new THREE.Vector3((f.x - width / 2) * worldPerPx, (height / 2 - f.y) * worldPerPx, -d).applyQuaternion(q);
  m.position.copy(camera.getWorldPosition(new THREE.Vector3())).add(off);
  m.quaternion.copy(q).multiply(new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(0, 0, 1), -f.rotation));
  m.scale.set(sx * worldPerPx * (f.flipX ? -1 : 1), sy * worldPerPx * (f.flipY ? -1 : 1), 1);
}

// ---- RichText: its text drawn to a canvas on a plane (in a browser; elsewhere a blank card) ----
export function richText(had: THREE.Mesh | undefined, p: Props): THREE.Mesh {
  const spans = (p.spans as { text?: string }[] | undefined)?.map((s) => s.text ?? "").join("") ?? (p.text as string) ?? "";
  const size = (p.size as number) ?? 0.3;
  const doc = (globalThis as { document?: Document }).document;
  const mat = new THREE.MeshBasicMaterial({ transparent: true, side: THREE.DoubleSide, toneMapped: false });
  let aspect = Math.max(1, spans.length * 0.55);
  if (doc?.createElement) {
    const c = doc.createElement("canvas");
    const ctx = c.getContext("2d");
    if (ctx) {
      const px = 128;
      ctx.font = `${(p.weight as number) ?? 400} ${px}px ${(p.font as string) ?? "sans-serif"}`;
      c.width = Math.ceil(ctx.measureText(spans).width) + 8;
      c.height = Math.ceil(px * 1.3);
      ctx.font = `${(p.weight as number) ?? 400} ${px}px ${(p.font as string) ?? "sans-serif"}`;
      ctx.fillStyle = (p.color as string) ?? "#ffffff";
      ctx.textBaseline = "middle";
      ctx.fillText(spans, 4, c.height / 2);
      mat.map = new THREE.CanvasTexture(c);
      mat.map.colorSpace = THREE.SRGBColorSpace;
      aspect = c.width / c.height;
    }
  }
  const m = had ?? new THREE.Mesh();
  m.geometry?.dispose();
  m.geometry = new THREE.PlaneGeometry(size * 1.3 * aspect, size * 1.3);
  m.material = mat;
  place(m, p);
  return m;
}

// ---- Plate: footage behind the scene (a video in a browser) ----
const videos = new Map<string, THREE.VideoTexture>();
export function plateTexture(src: string, time: number): THREE.Texture | undefined {
  const doc = (globalThis as { document?: Document }).document;
  if (!doc?.createElement) return undefined;
  let t = videos.get(src);
  if (!t) {
    const v = doc.createElement("video");
    Object.assign(v, { src, muted: true, playsInline: true, crossOrigin: "anonymous", preload: "auto" });
    t = new THREE.VideoTexture(v);
    t.colorSpace = THREE.SRGBColorSpace;
    videos.set(src, t);
  }
  const v = t.image as HTMLVideoElement;
  if (Math.abs(v.currentTime - time) > 1 / 120) v.currentTime = time;
  return t;
}

// ---- Screen: the card a page is shown on ----
export function screenCard(had: THREE.Mesh | undefined, p: Props): THREE.Mesh {
  const [w, h] = [(p.width as number) ?? 1, (p.height as number) ?? 0.5625];
  const m = had ?? new THREE.Mesh(new THREE.PlaneGeometry(1, 1), new THREE.MeshBasicMaterial({ color: "#101418", toneMapped: false }));
  m.geometry.dispose();
  m.geometry = new THREE.PlaneGeometry(w, h);
  place(m, p);
  void vec;
  return m;
}
