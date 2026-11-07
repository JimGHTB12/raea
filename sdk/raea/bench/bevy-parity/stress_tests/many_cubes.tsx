// Bevy v0.20.0 examples/stress_tests/many_cubes.rs, written in raea's TSX the way an author would (docs/control.feature
// FC0a), run by tools/bevy-parity beside the Rust example with `--benchmark --instance-count N`: the sphere layout, one
// mesh (a cube of half-size H, the first of the example's seeded radii) and one white material shared by every cube,
// a camera at the centre turning 0.15 rad a frame-second about Z then X, an inside-out white box around it all and
// Bevy's default DirectionalLight (10 000 lux) facing (0, -1, -1). Frame n here is the example's update n.
import { Composition, registerRoot, useCurrentFrame } from "raea";

import { Transform, Quat } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

// the run's size (tools/bevy-parity writes N and H into a copy; these are its defaults)
const N = 10_000;
const H = 0.362; // the example's first seeded radius, as the probe reads its bounds (0.3620)
const RADIUS = 200 * 2.5;
const GOLDEN = 0.5 * (1 + Math.sqrt(5));
const EPS = 1e-10;

const places: [number, number, number][] = Array.from({ length: N }, (_, i) => {
  const theta = Math.PI * 2 * (i / GOLDEN);
  const phi = Math.acos(1 - (2 * (i + EPS)) / (N - 1 + 2 * EPS));
  return [Math.cos(theta) * Math.sin(phi) * RADIUS, Math.sin(theta) * Math.sin(phi) * RADIUS, Math.cos(phi) * RADIUS];
});

// the example's move_camera system: only the camera reads the clock, as only that system touches it each update
function Rig() {
  const f = useCurrentFrame();
  // rotate_z then rotate_x each update, about the world's axes: q = (Rx(d) * Rz(d))^(f + 1) (--benchmark turns a fixed 1/60 s each update, the first too)
  const d = 0.15 / 60;
  const step = Quat.mul(Quat.fromRotationX(d), Quat.fromRotationZ(d));
  let q = Quat.IDENTITY;
  for (let k = 0; k <= f; k++) q = Quat.mul(step, q);
  return <Camera3d transform={{ translation: [0, 0, 0], rotation: q }} />;
}

// the example's setup: spawned once, never touched again
const Cubes = () => (
  <>
    {places.map((p, i) => (
      <Mesh3d key={i} transform={Transform.fromXyz(p[0], p[1], p[2]).lookingAt([0, 0, 0])}>
        <Cuboid half_size={[H, H, H]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
    ))}
  </>
);

function Shot() {
  return (
    <>
      <Rig />
      <DirectionalLight transform={Transform.fromXyz(0, 0, 0).lookingAt([0, -1, -1])} />
      <Cubes />
      <Mesh3d transform={{ translation: [0, 0, 0], scale: [-1, -1, -1] }}>
        <Cuboid half_size={[RADIUS * 1.1, RADIUS * 1.1, RADIUS * 1.1]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1920} height={1080} />);
