// Vectors and matrices.
import { type ReactNode } from "react";

import { Solid } from "@raea/render/solid";
import { V3, Xf } from "./all";
import { Quat, Transform } from "raea/transform";
import { Entity } from "raea/3d";

// ---------------------------------------------------------------- vectors and matrices

export const add3 = (a: V3, b: V3): V3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];

export const sub3 = (a: V3, b: V3): V3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];

export const mul3 = (a: V3, k: number): V3 => [a[0] * k, a[1] * k, a[2] * k];

export const dot3 = (a: V3, b: V3) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];

export const len3 = (a: V3) => Math.hypot(a[0], a[1], a[2]);

export const norm3 = (a: V3): V3 => {
  const l = len3(a);
  return l > 1e-12 ? mul3(a, 1 / l) : [0, 1, 0];
};

export const cross3 = (a: V3, b: V3): V3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];

export const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

export const lerp3 = (a: V3, b: V3, t: number): V3 => [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];

export const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);

export const smooth = (x: number) => {
  const t = clamp01(x);
  return t * t * (3 - 2 * t);
};

export const easeOut = (x: number) => 1 - (1 - clamp01(x)) ** 3;

/** 0 -> 1 with a springy overshoot */
export const backOut = (x: number) => {
  const t = clamp01(x), c = 2.2;
  return 1 + (c + 1) * (t - 1) ** 3 + c * (t - 1) ** 2;
};

export const span = (u: number, a: number, b: number) => clamp01((u - a) / (b - a));

export const DEG = Math.PI / 180;

/** 3x3 rotation matrices, row-major [m00 m01 m02 m10 ...] */
export type M3 = number[];

const rx = (a: number): M3 => [1, 0, 0, 0, Math.cos(a), -Math.sin(a), 0, Math.sin(a), Math.cos(a)];

const ry = (a: number): M3 => [Math.cos(a), 0, Math.sin(a), 0, 1, 0, -Math.sin(a), 0, Math.cos(a)];

const rz = (a: number): M3 => [Math.cos(a), -Math.sin(a), 0, Math.sin(a), Math.cos(a), 0, 0, 0, 1];

export function mm3(a: M3, b: M3): M3 {
  const o = new Array(9).fill(0);
  for (let r = 0; r < 3; r++) for (let c = 0; c < 3; c++) for (let k = 0; k < 3; k++) o[r * 3 + c] += a[r * 3 + k] * b[k * 3 + c];
  return o;
}

export const mv3 = (m: M3, v: V3): V3 => [m[0] * v[0] + m[1] * v[1] + m[2] * v[2], m[3] * v[0] + m[4] * v[1] + m[5] * v[2], m[6] * v[0] + m[7] * v[1] + m[8] * v[2]];

/** Euler XYZ (three.js's intrinsic order, which <Group rotation> takes) of a rotation matrix */
export function eulerXYZ(m: M3): V3 {
  const m13 = Math.max(-1, Math.min(1, m[2]));
  const y = Math.asin(m13);
  if (Math.abs(m13) < 0.9999999) return [Math.atan2(-m[5], m[8]), y, Math.atan2(-m[1], m[0])];
  return [Math.atan2(m[7], m[4]), y, 0];
}

/** rotation from Euler degrees in the pose's order (x, then y, then z about the parent's axes: Rz Ry Rx) */
export const poseRot = (d: V3): M3 => mm3(rz(d[2] * DEG), mm3(ry(d[1] * DEG), rx(d[0] * DEG)));

export function quatOf(m: M3): [number, number, number, number] {
  const t = m[0] + m[4] + m[8];
  let x, y, z, w;
  if (t > 0) {
    const s = Math.sqrt(t + 1) * 2;
    w = s / 4; x = (m[7] - m[5]) / s; y = (m[2] - m[6]) / s; z = (m[3] - m[1]) / s;
  } else if (m[0] > m[4] && m[0] > m[8]) {
    const s = Math.sqrt(1 + m[0] - m[4] - m[8]) * 2;
    w = (m[7] - m[5]) / s; x = s / 4; y = (m[1] + m[3]) / s; z = (m[2] + m[6]) / s;
  } else if (m[4] > m[8]) {
    const s = Math.sqrt(1 + m[4] - m[0] - m[8]) * 2;
    w = (m[2] - m[6]) / s; x = (m[1] + m[3]) / s; y = s / 4; z = (m[5] + m[7]) / s;
  } else {
    const s = Math.sqrt(1 + m[8] - m[0] - m[4]) * 2;
    w = (m[3] - m[1]) / s; x = (m[2] + m[6]) / s; y = (m[5] + m[7]) / s; z = s / 4;
  }
  return [x, y, z, w];
}

/** the rotation taking +y to d */
export function alignY(d: V3): M3 {
  const n = norm3(d);
  const c = n[1];
  if (c > 0.999999) return [1, 0, 0, 0, 1, 0, 0, 0, 1];
  if (c < -0.999999) return [1, 0, 0, 0, -1, 0, 0, 0, -1];
  const ax = norm3(cross3([0, 1, 0], n)), s = Math.sqrt(1 - c * c), t = 1 - c;
  const [x, y, z] = ax;
  return [t * x * x + c, t * x * y - s * z, t * x * z + s * y, t * x * y + s * z, t * y * y + c, t * y * z - s * x, t * x * z - s * y, t * y * z + s * x, t * z * z + c];
}

/** a solid transformed as Xf says (Manifold) */
export function applyXf(s: Solid, xf: Xf): Solid {
  const p = xf.pivot ?? [0, 0, 0];
  let o = p[0] || p[1] || p[2] ? s.move(-p[0], -p[1], -p[2]) : s;
  if (xf.scale !== undefined) o = typeof xf.scale === "number" ? o.scale(xf.scale) : o.scale(xf.scale[0], xf.scale[1], xf.scale[2]);
  if (xf.rotate) o = o.rotate(xf.rotate[0], xf.rotate[1], xf.rotate[2]);
  if (p[0] || p[1] || p[2]) o = o.move(p[0], p[1], p[2]);
  if (xf.move) o = o.move(xf.move[0], xf.move[1], xf.move[2]);
  return o;
}

export function xfPoint(v: V3, xf: Xf): V3 {
  const p = xf.pivot ?? [0, 0, 0];
  let o = sub3(v, p);
  if (xf.scale !== undefined) o = typeof xf.scale === "number" ? mul3(o, xf.scale) : [o[0] * xf.scale[0], o[1] * xf.scale[1], o[2] * xf.scale[2]];
  if (xf.rotate) o = mv3(poseRot(xf.rotate), o);
  o = add3(o, p);
  return xf.move ? add3(o, xf.move) : o;
}

/** The same transform as Groups, `t` of the way (0 none, 1 all), around children. */
export function XfGroup({ xf, t = 1, children }: { xf: Xf; t?: number; children: ReactNode }) {
  const p = xf.pivot ?? [0, 0, 0];
  const r = xf.rotate ?? [0, 0, 0];
  const sc: V3 = xf.scale === undefined ? [1, 1, 1] : typeof xf.scale === "number" ? [xf.scale, xf.scale, xf.scale] : xf.scale;
  const s: V3 = [lerp(1, sc[0], t), lerp(1, sc[1], t), lerp(1, sc[2], t)].map((v) => Math.max(1e-3, v)) as V3;
  const mv = xf.move ? mul3(xf.move, t) : [0, 0, 0];
  return (
    <Entity transform={Transform.fromTranslation(add3(p, mv as V3))}>
      <Entity transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", 0, 0, r[2] * DEG * t))}>
        <Entity transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", 0, r[1] * DEG * t, 0))}>
          <Entity transform={Transform.IDENTITY.withRotation(Quat.fromEuler("XYZ", r[0] * DEG * t, 0, 0))}>
            <Entity transform={Transform.IDENTITY.withScale(s)}>
              <Entity transform={Transform.fromTranslation(mul3(p, -1))}>{children}</Entity>
            </Entity>
          </Entity>
        </Entity>
      </Entity>
    </Entity>
  );
}
