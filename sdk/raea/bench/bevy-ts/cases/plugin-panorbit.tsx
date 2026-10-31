// gap: PL11
// bevy_panorbit_camera through raea/plugins/panorbit (docs/plugins.feature PL11): a camera placed at (0, 0, -5) looking
// away from everything, given a PanOrbitCamera with focus at the origin, yaw a quarter turn, no pitch and radius 5.
// The plugin puts it at (5, 0, 0) looking back at the focus: a red cube at the origin fills the middle and a green one
// at z = -1.5 stands to its right. Without the plugin the frame is black; yaw 0 would hide the green behind the red.
// build: plugins
// frame: 1
// probe: 32 32 r>150 g<80 b<80
// probe: 54 32 g>150 r<80 b<80
// probe: 8 32 r<30 g<30 b<30
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { PanOrbitCamera } from "raea/plugins/panorbit";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, -5).lookingAt([0, 0, -10])} projection={{ Perspective: { fov: 0.785398 } }}>
          <PanOrbitCamera value={{ focus: [0, 0, 0], yaw: Math.PI / 2, pitch: 0, radius: 5 } as PanOrbitCamera} />
        </Camera3d>
        <ToneMapping mode="none" />
        <Mesh3d name="red">
          <Cuboid half_size={[0.5, 0.5, 0.5]} />
          <StandardMaterial base_color="#ff0000" unlit />
        </Mesh3d>
        <Mesh3d name="green" transform={Transform.fromXyz(0, 0, -1.5)}>
          <Cuboid half_size={[0.4, 0.4, 0.4]} />
          <StandardMaterial base_color="#00ff00" unlit />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
