// Vectors and quaternions.
import { type Vec3 } from "@raea/render/3d";
import { Quat } from "./all";

// ---------------------------------------------------------------- vectors and quaternions

export const I: Quat = [0, 0, 0, 1];

export const add = (a: Vec3, b: Vec3): Vec3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];

export const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

export const scale3 = (a: Vec3, k: number): Vec3 => [a[0] * k, a[1] * k, a[2] * k];

export const dot = (a: Vec3, b: Vec3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

export const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];

export const len = (a: Vec3) => Math.hypot(a[0], a[1], a[2]);

export const norm = (a: Vec3): Vec3 => {
  const l = len(a);
  return l > 1e-12 ? scale3(a, 1 / l) : [0, 0, 0];
};

export function qmul(a: Quat, b: Quat): Quat {
  const [ax, ay, az, aw] = a, [bx, by, bz, bw] = b;
  return [aw * bx + ax * bw + ay * bz - az * by, aw * by - ax * bz + ay * bw + az * bx, aw * bz + ax * by - ay * bx + az * bw, aw * bw - ax * bx - ay * by - az * bz];
}

export const qinv = (q: Quat): Quat => [-q[0], -q[1], -q[2], q[3]];

export function qnorm(q: Quat): Quat {
  const l = Math.hypot(q[0], q[1], q[2], q[3]) || 1;
  return [q[0] / l, q[1] / l, q[2] / l, q[3] / l];
}

export function qrot(q: Quat, v: Vec3): Vec3 {
  const u: Vec3 = [q[0], q[1], q[2]];
  const t = scale3(cross(u, v), 2);
  return add(add(v, scale3(t, q[3])), cross(u, t));
}

export function qaxis(axis: Vec3, angle: number): Quat {
  const a = norm(axis), s = Math.sin(angle / 2);
  return [a[0] * s, a[1] * s, a[2] * s, Math.cos(angle / 2)];
}

/** the shortest turn taking direction a to direction b */
export function qfromTo(a: Vec3, b: Vec3): Quat {
  const u = norm(a), v = norm(b);
  const d = dot(u, v);
  if (d < -0.999999) {
    const ax = Math.abs(u[0]) < 0.9 ? cross(u, [1, 0, 0]) : cross(u, [0, 1, 0]);
    return qaxis(ax, Math.PI);
  }
  const c = cross(u, v);
  return qnorm([c[0], c[1], c[2], 1 + d]);
}

export function slerp(a: Quat, b: Quat, t: number): Quat {
  let [bx, by, bz, bw] = b;
  let d = a[0] * bx + a[1] * by + a[2] * bz + a[3] * bw;
  if (d < 0) { bx = -bx; by = -by; bz = -bz; bw = -bw; d = -d; }
  if (d > 0.9995) return qnorm([a[0] + (bx - a[0]) * t, a[1] + (by - a[1]) * t, a[2] + (bz - a[2]) * t, a[3] + (bw - a[3]) * t]);
  const th = Math.acos(d), s = Math.sin(th);
  const ka = Math.sin((1 - t) * th) / s, kb = Math.sin(t * th) / s;
  return [a[0] * ka + bx * kb, a[1] * ka + by * kb, a[2] * ka + bz * kb, a[3] * ka + bw * kb];
}

/** the turn whose x, y, z axes are the given (orthonormal) columns */
export function qbasis(x: Vec3, y: Vec3, z: Vec3): Quat {
  const t = x[0] + y[1] + z[2];
  let q: Quat;
  if (t > 0) {
    const s = Math.sqrt(t + 1) * 2;
    q = [(y[2] - z[1]) / s, (z[0] - x[2]) / s, (x[1] - y[0]) / s, s / 4];
  } else if (x[0] > y[1] && x[0] > z[2]) {
    const s = Math.sqrt(1 + x[0] - y[1] - z[2]) * 2;
    q = [s / 4, (y[0] + x[1]) / s, (z[0] + x[2]) / s, (y[2] - z[1]) / s];
  } else if (y[1] > z[2]) {
    const s = Math.sqrt(1 + y[1] - x[0] - z[2]) * 2;
    q = [(y[0] + x[1]) / s, s / 4, (z[1] + y[2]) / s, (z[0] - x[2]) / s];
  } else {
    const s = Math.sqrt(1 + z[2] - x[0] - y[1]) * 2;
    q = [(z[0] + x[2]) / s, (z[1] + y[2]) / s, s / 4, (x[1] - y[0]) / s];
  }
  return qnorm(q);
}

/** Euler XYZ (radians) as Model's rotation takes it (R = Rx·Ry·Rz) */
export function eulerXYZ(q: Quat): Vec3 {
  const [x, y, z, w] = q;
  const m00 = 1 - 2 * (y * y + z * z), m01 = 2 * (x * y - z * w), m02 = 2 * (x * z + y * w);
  const m11 = 1 - 2 * (x * x + z * z), m12 = 2 * (y * z - x * w);
  const m21 = 2 * (y * z + x * w), m22 = 1 - 2 * (x * x + y * y);
  const ey = Math.asin(Math.max(-1, Math.min(1, m02)));
  if (Math.abs(m02) < 0.9999999) return [Math.atan2(-m12, m22), ey, Math.atan2(-m01, m00)];
  return [Math.atan2(m21, m11), ey, 0];
}

/** a turn about y (yaw), as a quaternion */
export const qyaw = (yaw: number): Quat => [0, Math.sin(yaw / 2), 0, Math.cos(yaw / 2)];
