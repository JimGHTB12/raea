// gap: control
// The receive-shadow case's scene with the ground receiving: its shadow is where the probes say (passes today).
// probe: 12 32 r<90
// probe: 50 32 r>120
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 10, 0.01).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <DirectionalLight transform={Transform.fromXyz(-4, 6, 0).lookingAt([0, 0, 0])} illuminance={16000} shadow_maps_enabled />
        <Mesh3d name="ground"><Cuboid half_size={[6, 0.05, 6]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={1} /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(0, 3, 0)} name="cube"><Cuboid half_size={[0.6, 0.6, 0.6]} /><StandardMaterial base_color="#202020" /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
