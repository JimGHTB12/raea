// Bevy v0.20.0 examples/transforms/transform.rs in raea's TSX (docs/control.feature FC0a): a white 1 m cube starting at
// (0, 0, -10) turned a quarter about Y, moving forward at 2 m/s and turning 0.2 x delta of the way towards a looking_at
// of the yellow sphere (radius 3, ico 32) at the origin each update; the sphere's scale is 1 - 0.05 x the cube's
// distance from its start (0.1 at least). The camera at (0, 10, 20) looks at the origin, a default DirectionalLight from
// (3, 3, 3), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Quat, Transform, type TransformValue } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

type V3 = [number, number, number];
type Q = [number, number, number, number];
const f = Math.fround;
const START: V3 = [0, 0, -10];

/** glam's Quat::lerp: towards `b` (flipped to the near side), then normalized */
function lerp(a: Q, b: Q, s: number): Q {
  const dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
  const e = dot < 0 ? b.map((x) => -x) : b;
  const r = a.map((x, i) => f(f(x * f(1 - s)) + f(e[i]! * s))) as Q;
  const n = Math.hypot(...r);
  return r.map((x) => f(x / n)) as Q;
}

/** The cube's Transform and the sphere's scale after the updates up to frame `n` (none moves on the first: delta 0) */
function at(n: number): { cube: { translation: V3; rotation: Q }; size: number } {
  let t: V3 = START;
  let q = Quat.fromRotationY(Math.PI / 2) as Q;
  for (let k = 1; k <= n; k++) {
    const dt = f(16_666_667e-9);
    // move_cube: forward (-Z turned) x 2 x delta
    const fw = Quat.mulVec3(q, [0, 0, -1]);
    t = t.map((x, i) => f(x + f(f(fw[i]! * 2) * dt))) as V3;
    // rotate_cube: towards looking_at(the sphere, the cube's local Y) by 0.2 x delta
    const up = Quat.mulVec3(q, [0, 1, 0]);
    const look = Transform.fromTranslation(t).withRotation(q).lookingAt([0, 0, 0], up).rotation as Q;
    q = lerp(q, look, f(0.2 * dt));
  }
  const dist = Math.hypot(t[0] - START[0], t[1] - START[1], t[2] - START[2]);
  return { cube: { translation: t, rotation: q }, size: Math.max(f(1 - f(0.05 * dist)), 0.1) };
}

function Shot() {
  const { cube, size } = at(useCurrentFrame());
  const sphere: TransformValue = { scale: [size, size, size] };
  return (
    <>
      <Mesh3d transform={sphere}>
        <Sphere radius={3} ico={32} />
        <StandardMaterial base_color="#ffff00" />
      </Mesh3d>
      <Mesh3d transform={cube}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 10, 20).lookingAt([0, 0, 0])} />
      <DirectionalLight transform={Transform.fromXyz(3, 3, 3).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1280} height={720} />);
