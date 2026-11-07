// Bevy v0.20.0 examples/3d/rect_light.rs in raea's TSX (docs/control.feature FC0a): a 20 m metallic floor (roughness
// 0.6), a white 1 m sphere at (0, 1, 0), two RectLights (warm 2 x 1 m at 100 000 lm from (1, 3, 1) looking at (0, 1, 0);
// cool 1.5 x 4 m at 800 000 lm at (-2, 1.5, -3) turned half about Y), each with its light gizmo (bevy_light's
// rect_light_gizmo: its rectangle and a 0.6 m arrow along its -Z, in the light's colour), the camera at (-8, 5, 8)
// looking at (0, 1, 0), and the controls text in Bevy's default font at 18 px, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Quat, Transform, type TransformBuilt } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { RectLight } from "raea/light";
import { Gizmo } from "raea/gizmos";

type V3 = [number, number, number];
const add = (a: V3, b: V3): V3 => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const scale = (a: V3, k: number): V3 => [a[0] * k, a[1] * k, a[2] * k];
const norm = (a: V3): V3 => scale(a, 1 / Math.hypot(...a));

/** glam's Quat::from_rotation_arc(X, to): the shortest turn taking +X onto `to` (both unit) */
function arcFromX(to: V3): [number, number, number, number] {
  const dot = to[0];
  if (dot < -1 + 2e-7) return [0, 1, 0, 0]; // opposite: half a turn about any axis at right angles (glam's any_orthonormal)
  // cross(X, to) = (0, -to.z, to.y); glam: from_xyzw(c.x, c.y, c.z, 1 + dot).normalize()
  const q: [number, number, number, number] = [0, -to[2], to[1], 1 + dot];
  const n = Math.hypot(...q);
  return q.map((x) => x / n) as [number, number, number, number];
}

/** bevy_light's rect_light_gizmo: gizmos.rect in the light's plane, then gizmos.arrow 0.6 m along -Z, tip 0.2 */
function rectGizmo(t: TransformBuilt, width: number, height: number): [V3, V3][] {
  const at = (x: number, y: number) => add(t.translation, Quat.mulVec3(t.rotation, [x, y, 0]));
  const [hx, hy] = [width / 2, height / 2];
  const corners = [at(-hx, hy), at(hx, hy), at(hx, -hy), at(-hx, -hy)];
  const loop = corners.map((c, i) => [c, corners[(i + 1) % 4]!] as [V3, V3]);
  const end = add(t.translation, scale(Quat.mulVec3(t.rotation, [0, 0, -1]), 0.6));
  const arc = arcFromX(norm(add(end, scale(t.translation, -1))));
  const tips = ([[-1, 1, 0], [-1, 0, 1], [-1, -1, 0], [-1, 0, -1]] as V3[]).map((v) => [end, add(end, Quat.mulVec3(arc, scale(norm(v), 0.2)))] as [V3, V3]);
  return [...loop, [t.translation, end], ...tips];
}

const WARM = Transform.fromXyz(1, 3, 1).lookingAt([0, 1, 0]);
const COOL = Transform.fromXyz(-2, 1.5, -3).withRotation(Quat.fromRotationY(Math.PI));
const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Mesh3d>
        <Plane3d half_size={[10, 10]} />
        <StandardMaterial base_color="#ffffff" metallic={1} perceptual_roughness={0.6} />
      </Mesh3d>
      <Mesh3d transform={Transform.fromXyz(0, 1, 0)}>
        <Sphere radius={1} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <RectLight color="rgb(255, 77, 51)" intensity={100_000} width={2} height={1} range={20} transform={WARM} />
      <RectLight color="rgb(128, 179, 255)" intensity={800_000} width={1.5} height={4} range={20} transform={COOL} />
      <Gizmo lines={rectGizmo(WARM, 2, 1)} color="#ff4d33" />
      <Gizmo lines={rectGizmo(COOL, 1.5, 4)} color="#80b3ff" />
      <Camera3d transform={Transform.fromXyz(-8, 5, 8).lookingAt([0, 1, 0])} />
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 18, lineHeight: 1.2, color: "rgb(230, 230, 230)", whiteSpace: "pre" }}>{"Controls\nArrow Up/Down: Adjust floor roughness\nG: Toggle light gizmos\n\nRoughness: 0.60"}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
