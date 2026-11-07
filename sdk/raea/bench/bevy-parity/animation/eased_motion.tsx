// Bevy v0.20.0 examples/animation/eased_motion.rs in raea's TSX (docs/control.feature FC0a): an orange 2 m cube whose
// AnimationClip (repeating) moves it from (-6, 2, 0) to (6, 2, 0) on CubicInOut over 3 s and back (ping_pong), and
// turns it a quarter about Y on ElasticInOut over 4 s (held after, the clip sampled clamped), on a 50 m silver plane
// under a shadow-casting PointLight of 10 000 000 lm and range 100 at (8, 16, 8), the camera at (0, 6, 12) looking at
// (0, 1.5, 0), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

const f = Math.fround;
// bevy_curve's easing functions, in f32
const cubicInOut = (t: number) => (t < 0.5 ? f(4 * t * t * t) : f(1 - f((-2 * t + 2) ** 3) / 2));
function elasticInOut(t: number): number {
  const c = f((2 * Math.PI) / 4.5);
  return t < 0.5 ? f(-f(2 ** f(20 * t - 10)) * f(Math.sin(f(f(t * 20 - 11.125) * c))) / 2) : f(f(2 ** f(-20 * t + 10)) * f(Math.sin(f(f(t * 20 - 11.125) * c))) / 2 + 1);
}

function Shot() {
  // the player's seek time: each update's delta of 16_666_667 ns (none on the first), repeating over the clip's 6 s
  const time = f(f(useCurrentFrame() * 16_666_667e-9) % 6);
  // translation: [0, 3] there, [3, 6] back (ping_pong of the curve reparametrized onto [0, 3])
  const u = time <= 3 ? f(time / 3) : f((6 - time) / 3);
  const x = f(-6 + f(12 * cubicInOut(u)));
  // rotation: Quat's unbounded interpolation from IDENTITY to a quarter about Y, its domain [0, 4] sampled clamped
  const turn = f(elasticInOut(f(Math.min(time, 4) / 4)) * f(Math.PI / 2));
  return (
    <>
      <Mesh3d transform={{ translation: [x, 2, 0], rotation: Quat.fromRotationY(turn) }}>
        <Cuboid half_size={[1, 1, 1]} />
        <StandardMaterial base_color="#ffa500" />
      </Mesh3d>
      <PointLight shadow_maps_enabled intensity={10_000_000} range={100} transform={Transform.fromXyz(8, 16, 8)} />
      <Mesh3d>
        <Plane3d half_size={[25, 25]} />
        <StandardMaterial base_color="#c0c0c0" />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 6, 12).lookingAt([0, 1.5, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={301} fps={60} width={1280} height={720} />);
