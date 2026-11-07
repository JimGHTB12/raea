// Bevy v0.20.0 examples/3d/parenting.rs in raea's TSX (docs/control.feature FC0a): a 2 m cube of srgb(0.8, 0.7, 0.6)
// at (0, 0, 1) turning about X at 3 rad/s, with a child cube 3 m along its own Z, under a default PointLight at
// (4, 5, -4), the camera at (5, 10, 10) looking at the origin, in Bevy's default 1280 x 720 window.
import { Composition, registerRoot, useCurrentFrame } from "raea";
import { Entity } from "raea/3d";
import { Quat, Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Cuboid, Mesh3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { PointLight } from "raea/light";

function Cube({ z }: { z: number }) {
  return (
    <Mesh3d transform={Transform.fromXyz(0, 0, z)}>
      <Cuboid half_size={[1, 1, 1]} />
      <StandardMaterial base_color="rgb(204, 178.5, 153)" />
    </Mesh3d>
  );
}

function Shot() {
  // rotator_system: rotate_x by 3 x each update's delta_secs (none on the first)
  const a = (3 * useCurrentFrame()) / 60;
  return (
    <>
      <Entity transform={{ translation: [0, 0, 1], rotation: Quat.fromRotationX(a) }}>
        <Cube z={0} />
        <Cube z={3} />
      </Entity>
      <PointLight transform={Transform.fromXyz(4, 5, -4)} />
      <Camera3d transform={Transform.fromXyz(5, 10, 10).lookingAt([0, 0, 0])} />
    </>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
