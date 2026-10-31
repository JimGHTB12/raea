// gap: G5
// The camera inside a red box, cull_mode={null}: the box's inner (back) faces are drawn, so red fills the frame. One-sided,
// they are culled and the frame stays black.
// probe: 32 32 r>200 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { StandardMaterial } from "raea/pbr";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 0).lookingAt([0, 0, -1])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d name="room"><Cuboid half_size={[2, 2, 2]} /><StandardMaterial base_color="#ff0000" unlit double_sided cull_mode={null} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
