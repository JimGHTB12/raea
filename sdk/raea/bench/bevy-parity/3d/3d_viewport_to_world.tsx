// Bevy v0.20.0 examples/3d/3d_viewport_to_world.rs in raea's TSX (docs/control.feature FC0a): a 20 m plane of
// srgb(0.3, 0.5, 0.3), a default DirectionalLight from (1, 1, 1) at the origin, and the camera at (15, 5, 15) looking at
// the origin, in Bevy's default 1280 x 720 window. The cursor's circle gizmo needs a cursor in the window, and the
// harness's window has none, so neither side draws it.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { Mesh3d, Plane3d } from "raea/mesh";
import { StandardMaterial } from "raea/pbr";
import { DirectionalLight } from "raea/light";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Mesh3d transform={Transform.IDENTITY}>
          <Plane3d half_size={[10, 10]} />
          <StandardMaterial base_color="rgb(76.5, 127.5, 76.5)" />
        </Mesh3d>
        <DirectionalLight transform={Transform.fromXyz(1, 1, 1).lookingAt([0, 0, 0])} />
        <Camera3d transform={Transform.fromXyz(15, 5, 15).lookingAt([0, 0, 0])} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
