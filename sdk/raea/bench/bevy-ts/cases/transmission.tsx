// gap: G5
// A clear glass slab (transmission 1, IOR 1.5, white) in front of a red card: the red shows through it.
// probe: 32 32 r>120 g<80
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d, Plane3d } from "raea/mesh";
import { AmbientLight, DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <AmbientLight brightness={200} />
        <DirectionalLight transform={Transform.fromXyz(1, 2, 3).lookingAt([0, 0, 0])} illuminance={8000} shadow_maps_enabled />
        <Mesh3d transform={Transform.fromXyz(0, 0, -1)} name="back"><Plane3d normal={[0, 0, 1]} half_size={[2, 2]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
        <Mesh3d name="glass"><Cuboid half_size={[1, 1, 0.1]} /><StandardMaterial base_color="#ffffff" perceptual_roughness={0} specular_transmission={1} ior={1.5} thickness={0.2} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
