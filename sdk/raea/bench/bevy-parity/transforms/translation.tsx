// Bevy v0.20.0 examples/transforms/translation.rs in raea's TSX (docs/control.feature FC0a): a white 1 m cube moving along its local X at 2 m/s, turning back
// past 5 m from where it started, the camera at (0, 10, 20) looking at it, a default DirectionalLight from (3, 3, 3),
// in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

/** move_cube over the updates up to frame f: turn when past 5 m, then move by speed x delta_secs (none on the first) */
function xAt(f: number): number {
  let x = 0;
  let speed = 2;
  for (let k = 0; k <= f; k++) {
    if (Math.abs(x) > 5) speed = -speed;
    x = Math.fround(x + Math.fround(speed * (k === 0 ? 0 : 1 / 60)));
  }
  return x;
}

function Shot() {
  return (
    <>
      <Mesh3d transform={Transform.fromXyz(xAt(useCurrentFrame()), 0, 0)}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 10, 20).lookingAt([0, 0, 0])} />
      <DirectionalLight transform={Transform.fromXyz(3, 3, 3).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={181} fps={60} width={1280} height={720} />);
