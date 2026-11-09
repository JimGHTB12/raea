// Picking in raea/3d: which part the pointer is over, worked out from the camera and the parts' bounds alone, so
// a game's step stays a pure function of its input (a render and a play pick the same part on the same frame). This is
// what bevy_picking's mesh backend answers in a Bevy app: the nearest hit along the pointer's ray, with where it hit.
//
//   const { pointer } = useInput();
//   const hit = pointer && pick(pointer, CAMERA, [{ name: "door", box: { min: [-1, 0, -0.1], max: [1, 2, 0.1] } }], { width, height });
//   if (hit?.name === "door" && pointer.down) ...
import type { Vec3 } from "./3d";

/** The camera as a scene sets it: a PerspectiveCamera's props (fov in degrees, vertical) or an OrthographicCamera's. */
export interface PickCamera {
  /** where it stands, metres (x, y, z) */
  position?: Vec3;
  /** the point it faces */
  lookAt?: Vec3;
  /** vertical field of view in degrees (50 by default); ignored when `height` is set */
  fov?: number;
  /** an orthographic camera's view height in metres */
  height?: number;
  /** which way is up for it (+Y) */
  up?: Vec3;
}

/** Something the pointer can hit: an axis-aligned box or a sphere, in world space. */
export type Pickable = { name: string; box: { min: Vec3; max: Vec3 } } | { name: string; sphere: { center: Vec3; radius: number } };

export interface Hit {
  /** the pickable it hit */
  name: string;
  /** metres from the camera (from its near plane's point, for an orthographic one) along the ray */
  distance: number;
  /** where the ray first meets it, in world space */
  point: Vec3;
}

const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const add = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const mul = (a: Vec3, k: number): Vec3 => [a[0] * k, a[1] * k, a[2] * k];
const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const norm = (a: Vec3): Vec3 => {
  const l = Math.hypot(a[0], a[1], a[2]);
  return l > 0 ? mul(a, 1 / l) : a;
};

/** The pointer's ray: from the camera through the pointer (x, y in the frame's pixels, y down), as the engine draws it
 * (Bevy's looking_at, with +Z for up when the camera looks straight up or down, as rdb does). */
export function pointerRay(pointer: { x: number; y: number }, camera: PickCamera, size: { width: number; height: number }): { origin: Vec3; direction: Vec3 } {
  const pos = camera.position ?? [0, 0, 5];
  const fwd = norm(sub(camera.lookAt ?? [0, 0, 0], pos));
  const upHint: Vec3 = camera.up ?? (Math.hypot(...cross(fwd, [0, 1, 0])) < 1e-3 ? [0, 0, 1] : [0, 1, 0]);
  const right = norm(cross(fwd, upHint));
  const up = cross(right, fwd);
  const aspect = size.width / size.height;
  const nx = (2 * pointer.x) / size.width - 1;
  const ny = 1 - (2 * pointer.y) / size.height;
  if (camera.height !== undefined) {
    const half = camera.height / 2;
    return { origin: add(pos, add(mul(right, nx * half * aspect), mul(up, ny * half))), direction: fwd };
  }
  const t = Math.tan((((camera.fov ?? 50) / 2) * Math.PI) / 180);
  return { origin: pos, direction: norm(add(fwd, add(mul(right, nx * t * aspect), mul(up, ny * t)))) };
}

function hitBox(o: Vec3, d: Vec3, min: Vec3, max: Vec3): number | null {
  let near = -Infinity;
  let far = Infinity;
  for (let k = 0; k < 3; k++) {
    if (Math.abs(d[k]) < 1e-12) {
      if (o[k] < min[k] || o[k] > max[k]) return null;
      continue;
    }
    const a = (min[k] - o[k]) / d[k];
    const b = (max[k] - o[k]) / d[k];
    near = Math.max(near, Math.min(a, b));
    far = Math.min(far, Math.max(a, b));
  }
  if (far < Math.max(near, 0)) return null;
  return near >= 0 ? near : far;
}

function hitSphere(o: Vec3, d: Vec3, c: Vec3, r: number): number | null {
  const oc = sub(o, c);
  const b = dot(oc, d);
  const disc = b * b - (dot(oc, oc) - r * r);
  if (disc < 0) return null;
  const s = Math.sqrt(disc);
  const t = -b - s >= 0 ? -b - s : -b + s;
  return t >= 0 ? t : null;
}

/** The nearest part under the pointer, or null. `size` is the frame's (useVideoConfig's width and height), whose
 * pixels the pointer is in. */
export function pick(pointer: { x: number; y: number }, camera: PickCamera, parts: readonly Pickable[], size: { width: number; height: number }): Hit | null {
  if (!(size.width > 0 && size.height > 0)) throw new RangeError(`pick: size is the frame's { width, height } in pixels, got ${JSON.stringify(size)}`);
  const { origin, direction } = pointerRay(pointer, camera, size);
  let best: Hit | null = null;
  for (const p of parts) {
    const t = "box" in p ? hitBox(origin, direction, p.box.min, p.box.max) : hitSphere(origin, direction, p.sphere.center, p.sphere.radius);
    if (t !== null && (best === null || t < best.distance)) best = { name: p.name, distance: t, point: add(origin, mul(direction, t)) };
  }
  return best;
}
