// Bevy v0.20.0 examples/animation/animated_transform.rs in raea's TSX (docs/control.feature FC0a): an AnimationClip on
// three named entities, played on repeat: the planet (a default ico-5 sphere of srgb(0.8, 0.7, 0.6)) moved round a
// 2 m square over 4 s, its orbit_controller turned a quarter about Y each second, and the satellite (a 0.5 m cube of
// srgb(0.3, 0.9, 0.3) 1.5 m out) pulsed between scale 0.8 and 1.2 every half second and turned with the controller.
// UnevenSampleAutoCurve: Vec3 keys lerped, Quat keys slerped. GlobalAmbientLight 150, a PointLight of 500 000 at
// (0, 2.5, 0) and the camera at (-2, 2.5, 5) looking at the origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Entity } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Sphere } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { GlobalAmbientLight, PointLight } from "raea/light";

type V3 = [number, number, number];

// the player's seek time at update f: f steps of 1/60 s, summed in f32, on repeat over the clip's 4 s
function seekAt(f: number): number {
  let t = 0;
  for (let k = 1; k <= f; k++) t = Math.fround(t + Math.fround(16_666_667e-9));
  return t % 4;
}

// an UnevenSampleAutoCurve at t: the keys either side of it, mixed
function sample<T>(times: number[], keys: T[], t: number, mix: (a: T, b: T, s: number) => T): T {
  const i = Math.min(times.findIndex((x) => x > t), times.length - 1) - 1;
  if (i < 0) return keys[0];
  return mix(keys[i], keys[i + 1], (t - times[i]) / (times[i + 1] - times[i]));
}
const lerp3 = (a: V3, b: V3, s: number): V3 => [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s];
const quarters = [0, 1, 2, 3, 0].map((k) => (k === 0 ? Quat.IDENTITY : Quat.fromAxisAngle([0, 1, 0], (Math.PI / 2) * k)));

function Shot() {
  const t = seekAt(useCurrentFrame());
  const second = [0, 1, 2, 3, 4];
  const planet = sample<V3>(second, [[1, 0, 1], [-1, 0, 1], [-1, 0, -1], [1, 0, -1], [1, 0, 1]], t, lerp3);
  const turn = sample(second, quarters, t, Quat.slerp);
  const s = sample<V3>([0, 0.5, 1, 1.5, 2, 2.5, 3, 3.5, 4], [0.8, 1.2, 0.8, 1.2, 0.8, 1.2, 0.8, 1.2, 0.8].map((k) => [k, k, k] as V3), t, lerp3);
  return (
    <>
      <GlobalAmbientLight brightness={150} />
      <Camera3d transform={Transform.fromXyz(-2, 2.5, 5).lookingAt([0, 0, 0])} />
      <PointLight intensity={500_000} transform={Transform.fromXyz(0, 2.5, 0)} />
      <Entity transform={{ translation: planet }}>
        <Mesh3d>
          <Sphere radius={0.5} ico={5} />
          <StandardMaterial base_color="rgb(204, 178.5, 153)" />
        </Mesh3d>
        <Entity transform={{ rotation: turn }}>
          <Entity transform={{ translation: [1.5, 0, 0], rotation: turn, scale: s }}>
            <Mesh3d>
              <Cuboid half_size={[0.25, 0.25, 0.25]} />
              <StandardMaterial base_color="rgb(76.5, 229.5, 76.5)" />
            </Mesh3d>
          </Entity>
        </Entity>
      </Entity>
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={166} fps={60} width={1280} height={720} />);
