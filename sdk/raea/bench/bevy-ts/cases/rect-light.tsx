// gap: G13
// A white floor lit by one rectangular area light above it (Bevy's RectLight), nothing else: lit in the middle.
// probe: 32 32 r>60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { RectLight } from "raea/light";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 6, 0.01).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <RectLight transform={Transform.fromXyz(0, 2, 0).lookingAt([0, 0, 0])} width={2} height={2} intensity={900000} />
        <Mesh3d name="floor"><Cuboid half_size={[4, 0.05, 4]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={1} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
