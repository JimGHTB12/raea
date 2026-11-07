// Quat, Transform, Mesh3d and Camera3d.
import { Children, isValidElement, useId, type ReactNode } from "react";
import { meshLayers } from "../layers";
import { DepthOfField, dofOf, eulerOf, type DepthOfFieldProps, type Vec3 } from "../3d";
import { AmbientLight as AmbientLight3, Mesh, OrthographicCamera, PerspectiveCamera, type MeshProps } from "../three";
import { Transform as TransformComponent } from "@raea/app/bevy";
import { AmbientLight, ambientOf, type AmbientLightProps } from "./all";

/** A rotation as glam's Quat: [x, y, z, w]. */
export type Quat = [number, number, number, number];

const qmul = (a: Quat, b: Quat): Quat => [
  a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
  a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
  a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
  a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
];

const qdot = (a: Quat, b: Quat) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];

const axisAngle = (axis: Vec3, angle: number): Quat => {
  const n = Math.hypot(...axis) || 1;
  const s = Math.sin(angle / 2) / n;
  return [axis[0] * s, axis[1] * s, axis[2] * s, Math.cos(angle / 2)];
};

/** glam's EulerRot, the intrinsic orders: "YXZ" turns about Y, then the turned X, then the turned Z. */
export type EulerRot = "XYZ" | "XZY" | "YXZ" | "YZX" | "ZXY" | "ZYX";

const EULER_ROTS: readonly string[] = ["XYZ", "XZY", "YXZ", "YZX", "ZXY", "ZYX"];

const AXES: Record<string, Vec3> = { X: [1, 0, 0], Y: [0, 1, 0], Z: [0, 0, 1] };

/** glam's Quat constructors, by their Rust names in camelCase. */
export const Quat = {
  IDENTITY: [0, 0, 0, 1] as Quat,
  fromRotationX: (a: number) => axisAngle([1, 0, 0], a),
  fromRotationY: (a: number) => axisAngle([0, 1, 0], a),
  fromRotationZ: (a: number) => axisAngle([0, 0, 1], a),
  fromAxisAngle: axisAngle,
  /** Quat::from_euler: `fromEuler("XYZ", x, y, z)` is fromRotationX(x) * fromRotationY(y) * fromRotationZ(z), the order a
   *  3d.tsx rotation [x, y, z] is composed in (eulerOf inverts it) */
  fromEuler(order: EulerRot, a: number, b: number, c: number): Quat {
    if (!EULER_ROTS.includes(order)) throw new TypeError(`Quat.fromEuler: order is one of ${EULER_ROTS.join(", ")}, got ${JSON.stringify(order)}`);
    const [i, j, k] = order.split("").map((n) => AXES[n]);
    return qmul(qmul(axisAngle(i, a), axisAngle(j, b)), axisAngle(k, c));
  },
  mul: qmul,
  /** v turned by q */
  mulVec3(q: Quat, v: Vec3): Vec3 {
    const r = qmul(qmul(q, [v[0], v[1], v[2], 0]), [-q[0], -q[1], -q[2], q[3]]);
    return [r[0], r[1], r[2]];
  },
};

/** glam's Affine2 as its parts: scaled, then turned `angle` radians, then moved by `translation` (from_scale_angle_translation). */
export type Affine2 = { scale: [number, number]; angle: number; translation: [number, number] };

/** glam's Affine2 constructors, by their Rust names in camelCase. */
export const Affine2 = {
  IDENTITY: { scale: [1, 1], angle: 0, translation: [0, 0] } as Affine2,
  fromScale: (scale: [number, number]): Affine2 => ({ scale, angle: 0, translation: [0, 0] }),
  fromAngle: (angle: number): Affine2 => ({ scale: [1, 1], angle, translation: [0, 0] }),
  fromTranslation: (translation: [number, number]): Affine2 => ({ scale: [1, 1], angle: 0, translation }),
  fromScaleAngleTranslation: (scale: [number, number], angle: number, translation: [number, number]): Affine2 => ({ scale, angle, translation }),
};

/** The rotation that points -Z from `eye` at `target` with +Y toward `up`, as Bevy's Transform::looking_at. */
function lookingAt(eye: Vec3, target: Vec3, up: Vec3): Quat {
  const sub = (a: Vec3, b: Vec3): Vec3 => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
  const cross = (a: Vec3, b: Vec3): Vec3 => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  // Bevy's fallbacks (look_to): a target at the eye looks down -Z, a zero up is +Y, and an up along the view takes glam's
  // any_orthonormal_vector of up
  const norm = (a: Vec3, or: () => Vec3): Vec3 => {
    const n = Math.hypot(...a);
    return n > 0 && Number.isFinite(n) ? [a[0] / n, a[1] / n, a[2] / n] : or();
  };
  const back = norm(sub(eye, target), () => [0, 0, 1]);
  const up1 = norm(up, () => [0, 1, 0]);
  const right = norm(cross(up1, back), () => {
    const sign = up1[2] < 0 || Object.is(up1[2], -0) ? -1 : 1;
    const a = -1 / (sign + up1[2]);
    return [up1[0] * up1[1] * a, sign + up1[1] * up1[1] * a, -up1[1]];
  });
  const u = cross(back, right);
  // the rotation matrix's columns are right, u, back; its quaternion (Shepperd's method)
  const [m00, m10, m20, m01, m11, m21, m02, m12, m22] = [right[0], right[1], right[2], u[0], u[1], u[2], back[0], back[1], back[2]];
  const t = m00 + m11 + m22;
  if (t > 0) {
    const s = Math.sqrt(t + 1) * 2;
    return [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, s / 4];
  }
  if (m00 > m11 && m00 > m22) {
    const s = Math.sqrt(1 + m00 - m11 - m22) * 2;
    return [s / 4, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s];
  }
  if (m11 > m22) {
    const s = Math.sqrt(1 + m11 - m00 - m22) * 2;
    return [(m01 + m10) / s, s / 4, (m12 + m21) / s, (m02 - m20) / s];
  }
  const s = Math.sqrt(1 + m22 - m00 - m11) * 2;
  return [(m02 + m20) / s, (m12 + m21) / s, s / 4, (m10 - m01) / s];
}

/** A Transform's value, as Bevy reflects it. */
export interface TransformValue {
  translation?: Vec3;
  rotation?: Quat;
  scale?: Vec3;
}

/** A Transform built with Bevy's builders: each returns a new one. */
export interface TransformBuilt extends Required<TransformValue> {
  withTranslation(v: Vec3): TransformBuilt;
  withRotation(q: Quat): TransformBuilt;
  withScale(s: Vec3 | number): TransformBuilt;
  lookingAt(target: Vec3, up?: Vec3): TransformBuilt;
}

function built(translation: Vec3, rotation: Quat, scale: Vec3): TransformBuilt {
  return {
    translation,
    rotation,
    scale,
    withTranslation: (v) => built(v, rotation, scale),
    withRotation: (q) => built(translation, q, scale),
    withScale: (s) => built(translation, rotation, typeof s === "number" ? [s, s, s] : s),
    lookingAt: (target, up = [0, 1, 0]) => built(translation, lookingAt(translation, target, up), scale),
  };
}

/** Bevy's Transform: the component's handle for a program (`with(Transform, Transform.fromXyz(1, 0, 0))`) and its
 *  builders for a scene (`<Mesh3d transform={Transform.fromXyz(0, 1, 0)}>`). */
export const Transform = /* @__PURE__ */ Object.assign({}, TransformComponent, {
  IDENTITY: built([0, 0, 0], [0, 0, 0, 1], [1, 1, 1]),
  fromXyz: (x: number, y: number, z: number) => built([x, y, z], [0, 0, 0, 1], [1, 1, 1]),
  fromTranslation: (v: Vec3) => built(v, [0, 0, 0, 1], [1, 1, 1]),
  fromRotation: (q: Quat) => built([0, 0, 0], q, [1, 1, 1]),
  fromScale: (s: Vec3) => built([0, 0, 0], [0, 0, 0, 1], s),
});

// the rotation goes as Bevy's quaternion, end to end (docs/control.feature FC1d); a part whose `animate` turns it
// keeps the Euler angles its tracks patch
function place(t: TransformValue | undefined, animated = false): Pick<MeshProps, "position" | "rotation" | "quaternion" | "scale"> {
  if (!t) return {};
  return {
    ...(t.translation ? { position: t.translation } : {}),
    ...(t.rotation ? (animated ? { rotation: eulerOf(t.rotation) } : { quaternion: t.rotation }) : {}),
    ...(t.scale ? { scale: t.scale } : {}),
  };
}

export interface Mesh3dProps extends Omit<MeshProps, "position" | "rotation" | "scale"> {
  transform?: TransformValue;
  /** Bevy's Visibility: "Hidden" draws nothing; "Visible" and "Inherited" draw */
  visibility?: "Visible" | "Hidden" | "Inherited";
  children?: ReactNode;
}

/** Bevy's Mesh3d on an entity with its Transform: a shape (Cuboid, Sphere ...) and a material inside, and any page
 * element inside as a page layer in the world at it, or on it with layer="surface" (`meshLayers`).
 * @bevy bevy_mesh::components::Mesh3d
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::pbr_material::StandardMaterial>
 * castShadow={false} and receiveShadow={false} put Bevy's markers on it: @bevy bevy_light::NotShadowCaster
 * @bevy bevy_light::NotShadowReceiver
 * `layers` is its RenderLayers (a light's and a camera's too): @bevy bevy_camera::visibility::render_layers::RenderLayers
 * shadow="only" draws it with the shadow-only material (engine/render shadow_only.rs):
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<raea_render::engine::shadow_only::ShadowOnlyMaterial> */
export function Mesh3d({ transform, visibility, children, ...rest }: Mesh3dProps) {
  const id = useId();
  if (visibility === "Hidden") return null;
  const layers = meshLayers("Mesh3d", rest.name ?? `layer${id}`, children, true);
  const at = place(transform, rest.animate !== undefined);
  if (!layers.surface) return <Mesh {...rest} {...at}>{layers.children}</Mesh>;
  return (
    <>
      <Mesh {...rest} {...at} surface={layers.surface}>{layers.children}</Mesh>
      {layers.part}
    </>
  );
}

/** Bevy's Projection enum, as it reflects: fov in radians (Bevy's default PI/4). */
export type Projection =
  | { Perspective: { fov?: number; near?: number; far?: number } }
  | { Orthographic: { near?: number; far?: number; scaling_mode?: { FixedVertical: { viewport_height: number } } } };

export interface Camera3dProps {
  transform?: TransformValue;
  projection?: Projection;
  /** Bevy's RenderLayers on the camera: it draws only the parts on these layers (a Mesh3d's `layers`; [0] by default) */
  layers?: number[];
  children?: ReactNode;
}

/** Bevy's Camera3d with its Transform and Projection: it looks along its -Z, +Y up, as Bevy's cameras do.
 * @bevy bevy_camera::components::Camera3d
 * @bevy bevy_camera::camera::Camera
 * @bevy bevy_camera::projection::Projection */
export function Camera3d({ transform, projection = { Perspective: {} }, layers, children: kids }: Camera3dProps) {
  // Bevy's DepthOfField on the camera: its focal_distance is the camera's focus, the rest the scene's DepthOfField
  let dof: DepthOfFieldProps | undefined;
  // Bevy 0.20's AmbientLight on the camera: its own ambient, over GlobalAmbientLight (the scene's other ambient lights)
  const ambient: ReactNode[] = [];
  const rest = Children.toArray(kids).filter((c) => {
    if (isValidElement(c) && c.type === AmbientLight) {
      ambient.push(<AmbientLight3 key={ambient.length} {...ambientOf(c.props as AmbientLightProps)} camera />);
      return false;
    }
    if (!(isValidElement(c) && c.type === DepthOfField)) return true;
    if (dof) throw new TypeError("Camera3d: one DepthOfField");
    dof = c.props as DepthOfFieldProps;
    return false;
  });
  const focus = dof ? { focus: dof.focal_distance ?? 10 } : {};
  const children = dof || ambient.length ? rest : kids;
  // on the camera it is Bevy's component, so Bevy's f/1 unless the old aperture is given
  const bevy = dof && dof.aperture === undefined && dof.aperture_f_stops === undefined ? { aperture_f_stops: 1 } : {};
  const lens = dof ? <DepthOfField {...dofOf("Camera3d", { ...dof, ...bevy, focal_distance: undefined })} /> : null;
  const eye: Vec3 = transform?.translation ?? [0, 0, 0];
  const q = transform?.rotation ?? Quat.IDENTITY;
  const f = Quat.mulVec3(q, [0, 0, -1]);
  const lookAt: Vec3 = [eye[0] + f[0], eye[1] + f[1], eye[2] + f[2]];
  // `up` only for a camera that rolls: one that doesn't is placed as the old camera is (+Y up), and an `up` sends its
  // row down the engine's roll path (matchmove.rs sync_view), which draws a moving camera elsewhere
  const level = Math.abs(f[1]) < 1 - 1e-9 && Math.abs(qdot(q, lookingAt(eye, lookAt, [0, 1, 0]))) > 1 - 1e-9;
  const look = { position: eye, lookAt, ...(level ? {} : { up: Quat.mulVec3(q, [0, 1, 0]) }), ...(layers ? { layers } : {}) };
  if ("Orthographic" in projection) {
    const o = projection.Orthographic;
    const height = o.scaling_mode?.FixedVertical.viewport_height;
    if (dof) throw new TypeError("Camera3d: DepthOfField needs a Perspective projection");
    return (
      <>
        <OrthographicCamera {...look} near={o.near} far={o.far} height={height}>{children}</OrthographicCamera>
        {ambient}
      </>
    );
  }
  const p = projection.Perspective;
  return (
    <>
      <PerspectiveCamera {...look} fov={((p.fov ?? Math.PI / 4) * 180) / Math.PI} near={p.near} far={p.far} {...focus}>{children}</PerspectiveCamera>
      {lens}
      {ambient}
    </>
  );
}
