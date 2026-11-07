// Bevy v0.20.0 examples/stress_tests/many_lights.rs, written in raea's TSX the way an author would (docs/control.feature
// FC0a), run by tools/bevy-parity beside the Rust example: an inside-out white icosphere (9 subdivisions) of radius 50, 100 000 point lights
// on it (range 0.3, 1000 lumens) along a Fibonacci spiral, a deep pink cube five metres across at its top, and the
// camera at the centre turning 0.15 rad a frame-second about Z then X. The example draws each light's hue at random
// from an unseeded rng; bench/bevy-parity's build.rs makes the n-th the golden angle times n, as here (SEEDED).
// Frame n here is the example's update n.
import { Composition, registerRoot, useCurrentFrame } from "raea";

import { Transform, Quat } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const LIGHT_RADIUS = 0.3;
const LIGHT_INTENSITY = 1000;
const RADIUS = 50;
const N_LIGHTS = 100_000;
const GOLDEN = 0.5 * (1 + Math.sqrt(5));
const EPSILON = 0.36;

/** hsl(h, 1, 0.5) as hex, as Bevy's Color::hsl makes it (sRGB) */
function hsl(h: number): string {
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    return 0.5 - 0.5 * Math.max(-1, Math.min(k - 3, 9 - k, 1));
  };
  return "#" + [f(0), f(8), f(4)].map((c) => Math.round(c * 255).toString(16).padStart(2, "0")).join("");
}

const lights = Array.from({ length: N_LIGHTS }, (_, i) => {
  const theta = Math.PI * 2 * (i / GOLDEN);
  // the example takes acos in f32
  const phi = Math.acos(Math.fround(1 - (2 * (i + EPSILON)) / (N_LIGHTS - 1 + 2 * EPSILON)));
  const p: [number, number, number] = [Math.cos(theta) * Math.sin(phi) * RADIUS, Math.sin(theta) * Math.sin(phi) * RADIUS, Math.cos(phi) * RADIUS];
  return { p, color: hsl((i * 137.50776405003785) % 360) };
});

// the example's move_camera system: only the camera reads the clock
function Rig() {
  const f = useCurrentFrame();
  // rotate_z then rotate_x each update, about the world's axes: q = (Rx(d) * Rz(d))^f (it turns by the update's delta_secs, none on the first: f turns at frame f; the f + 1 form matched one frame early, SSIM 0.99996)
  const d = 0.15 / 60;
  const step = Quat.mul(Quat.fromRotationX(d), Quat.fromRotationZ(d));
  let q = Quat.IDENTITY;
  for (let k = 0; k < f; k++) q = Quat.mul(step, q);
  return <Camera3d transform={{ translation: [0, 0, 0], rotation: q }} />;
}

// the example's setup: spawned once, never touched again
const Scene = () => (
  <>
    <Mesh3d transform={{ translation: [0, 0, 0], scale: [-1, -1, -1] }}>
      <Sphere radius={RADIUS} ico={9} />
      <StandardMaterial base_color="#ffffff" />
    </Mesh3d>
    {lights.map((l, i) => (
      <PointLight key={i} transform={Transform.fromXyz(l.p[0], l.p[1], l.p[2])} range={LIGHT_RADIUS} intensity={LIGHT_INTENSITY} color={l.color} />
    ))}
    <Mesh3d transform={{ translation: [0, RADIUS, 0], scale: [5, 5, 5] }}>
      <Cuboid half_size={[0.5, 0.5, 0.5]} />
      <StandardMaterial base_color="#ff1493" />
    </Mesh3d>
  </>
);

const Shot = () => (
  <>
    <Rig />
    <Scene />
  </>
);

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={121} fps={60} width={1920} height={1080} />);
