// Bevy v0.20.0 examples/3d/mesh_ray_cast.rs in raea's TSX (docs/control.feature FC0a): a 1 m box of six inward-facing
// 1 m planes (css GRAY at alpha 0.01, so Blend), a default DirectionalLight turned from_euler(XYZ, -0.1, 0.2, 0), on
// black under TonyMcMapface and Bloom::NATURAL, the camera at (1.5, 1.5, 1.5) looking at the origin, and the laser:
// from a point circling inside the box after 4 s (t = cos((elapsed - 4) * 0.03) * PI), a ray bounced 64 times off the
// walls MeshRayCast finds (its default culls back faces, so from inside only the wall ahead counts: the nearest plane
// along each axis it heads for), drawn as a gradient line strip from HDR Color::srgb(30, 0, 0) through each hit at
// BLACK.mix(RED, 1 + 10 * (1 - i / 64)) (Color::BLACK is linear, so linear red at that strength), a 0.005 m gizmo
// sphere at twice it at each hit, and a white 0.1 m gizmo sphere at the start. A gizmo sphere is three great circles of
// 32 segments (bevy_gizmos circles.rs). No cursor in the harness's window, so no green ray from the pointer.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Bloom, Canvas3D, ToneMapping } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Srgba, LinearRgba } from "raea/color";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";
import { Gizmo, type GizmoColor } from "raea/gizmos";

type V3 = [number, number, number];
const f = Math.fround;
const PI = Math.PI;

/** the six walls: (translation, rotation as Quat::from_scaled_axis) */
const WALLS: [V3, [number, number, number, number]][] = [
  [[0, 0.5, 0], Quat.fromRotationX(PI)],
  [[0, -0.5, 0], Quat.IDENTITY],
  [[0.5, 0, 0], Quat.fromRotationZ(PI / 2)],
  [[-0.5, 0, 0], Quat.fromRotationZ(-PI / 2)],
  [[0, 0, 0.5], Quat.fromRotationX(-PI / 2)],
  [[0, 0, -0.5], Quat.fromRotationX(PI / 2)],
];

const norm = (v: V3): V3 => { const l = Math.hypot(...v); return [v[0] / l, v[1] / l, v[2] / l]; };

/** bounce_ray: the strip's points and colours, and each hit for its sphere */
function laser(origin: V3, dir: V3): { points: V3[]; colours: GizmoColor[]; hits: [V3, number][] } {
  const points: V3[] = [origin];
  const colours: GizmoColor[] = [new Srgba(30, 0, 0, 1)];
  const hits: [V3, number][] = [];
  let [o, d] = [origin, dir];
  for (let i = 0; i < 64; i++) {
    // the wall ahead on each axis faces the ray (the rest are culled back faces); the nearest is the hit
    let best = Infinity;
    let axis = -1;
    for (let k = 0; k < 3; k++) {
      if (d[k] === 0) continue;
      const t = ((d[k]! > 0 ? 0.5 : -0.5) - o[k]!) / d[k]!;
      if (t > 0 && t < best) [best, axis] = [t, k];
    }
    if (axis < 0) break;
    const hit: V3 = [o[0] + d[0] * best, o[1] + d[1] * best, o[2] + d[2] * best];
    const brightness = 1 + 10 * (1 - i / 64);
    points.push(hit);
    colours.push(new LinearRgba(brightness, 0, 0, 1));
    hits.push([hit, brightness * 2]);
    // the wall's normal faces inwards, against the ray: reflect it, and start a hair along the new direction
    const n: V3 = [0, 0, 0];
    n[axis] = d[axis]! > 0 ? -1 : 1;
    const dn = d[0] * n[0] + d[1] * n[1] + d[2] * n[2];
    d = norm([d[0] - 2 * dn * n[0], d[1] - 2 * dn * n[1], d[2] - 2 * dn * n[2]]);
    o = [hit[0] + d[0] * 1e-6, hit[1] + d[1] * 1e-6, hit[2] + d[2] * 1e-6];
  }
  return { points, colours, hits };
}

/** gizmos.sphere: a great circle about each axis, Quat::from_rotation_arc(Z, axis) turning the XY circle onto it */
const ARCS: [number, number, number, number][] = [[0, Math.SQRT1_2, 0, Math.SQRT1_2], [-Math.SQRT1_2, 0, 0, Math.SQRT1_2], [0, 0, 0, 1]];
function sphere(at: V3, r: number): (V3 | null)[] {
  const out: (V3 | null)[] = [];
  for (const q of ARCS) {
    for (let i = 0; i <= 32; i++) {
      const a = f(f(i * f(2 * PI)) / 32);
      const p = Quat.mulVec3(q, [Math.sin(a) * r, Math.cos(a) * r, 0]);
      out.push([at[0] + p[0], at[1] + p[1], at[2] + p[2]]);
    }
    out.push(null);
  }
  return out;
}

function Shot() {
  const elapsed = f(useCurrentFrame() * 16_666_667e-9);
  const t = f(f(Math.cos(f(Math.max(f(elapsed - 4), 0) * f(0.03)))) * f(PI));
  const pos: V3 = [f(Math.sin(t)) * 0.5, f(f(Math.cos(f(3 * t))) * 0.5) * 0.5, f(Math.cos(t)) * 0.5];
  const { points, colours, hits } = laser(pos, norm([-pos[0], -pos[1], -pos[2]]));
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(0, 0, 0)">
        <Camera3d transform={Transform.fromXyz(1.5, 1.5, 1.5).lookingAt([0, 0, 0])} />
        <ToneMapping mode="tony" />
        <Bloom intensity={1} threshold={0} />
        {WALLS.map(([translation, rotation], i) => (
          <Mesh3d key={`w${i}`} transform={{ translation, rotation }}>
            <Plane3d half_size={[0.5, 0.5]} />
            <StandardMaterial base_color={new Srgba(0.5, 0.5, 0.5, 0.01)} alpha_mode="Blend" />
          </Mesh3d>
        ))}
        <DirectionalLight transform={{ translation: [0, 0, 0], rotation: Quat.fromEuler("XYZ", -0.1, 0.2, 0) }} />
        <Gizmo strip={sphere(pos, 0.1)} color="#ffffff" />
        <Gizmo strip={points} color={colours} />
        {hits.map(([at, k], i) => (
          <Gizmo key={`h${i}`} strip={sphere(at, 0.005)} color={new LinearRgba(k, 0, 0, 1)} />
        ))}
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={901} fps={60} width={1280} height={720} />);
