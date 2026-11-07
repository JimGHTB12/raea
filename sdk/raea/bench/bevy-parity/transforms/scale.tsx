// Bevy v0.20.0 examples/transforms/scale.rs in raea's TSX (docs/control.feature FC0a): a white 1 m cube turned 45 degrees about Y, growing along one axis
// at 2 a second to 5, shrinking back to 1, then the next axis (X, then Z, then Y), the camera at (0, 10, 20), a default
// DirectionalLight from (3, 3, 3), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

type V = [number, number, number];

/** change_scale_direction then scale_cube, over the updates up to frame f (no delta on the first) */
function scaleAt(f: number): V {
  let s: V = [1, 1, 1];
  let d: V = [1, 0, 0];
  for (let k = 0; k <= f; k++) {
    if (Math.max(...s) > 5) {
      d = d.map((c) => -c) as V;
      s = s.map(Math.floor) as V;
    }
    if (Math.min(...s) < 1) {
      d = d.map((c) => -c) as V;
      s = s.map(Math.ceil) as V;
      d = [d[2], d[0], d[1]];
    }
    const dt = k === 0 ? 0 : 1 / 60;
    s = s.map((c, i) => Math.fround(c + Math.fround(Math.fround(d[i] * 2) * dt))) as V;
  }
  return s;
}

function Shot() {
  return (
    <>
      <Mesh3d transform={{ rotation: Quat.fromRotationY(Math.PI / 4), scale: scaleAt(useCurrentFrame()) }}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 10, 20).lookingAt([0, 0, 0])} />
      <DirectionalLight transform={Transform.fromXyz(3, 3, 3).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={301} fps={60} width={1280} height={720} />);
