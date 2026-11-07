// Bevy v0.20.0 examples/transforms/3d_rotation.rs in raea's TSX (docs/control.feature FC0a): a white 1 m cube at the
// origin turning about Y at 0.3 turns a second, the camera at (0, 10, 20) looking at it, a default DirectionalLight from
// (3, 3, 3), in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

function Shot() {
  // rotate_cube: rotate_y by 0.3 x TAU x each update's delta_secs (none on the first)
  const a = (0.3 * 2 * Math.PI * useCurrentFrame()) / 60;
  return (
    <>
      <Mesh3d transform={{ rotation: Quat.fromRotationY(a) }}>
        <Cuboid half_size={[0.5, 0.5, 0.5]} />
        <StandardMaterial base_color="#ffffff" />
      </Mesh3d>
      <Camera3d transform={Transform.fromXyz(0, 10, 20).lookingAt([0, 0, 0])} />
      <DirectionalLight transform={Transform.fromXyz(3, 3, 3).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
