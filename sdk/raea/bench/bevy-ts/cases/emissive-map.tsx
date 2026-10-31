// gap: G5
// A black, rough plane in the dark glowing only through its emissive map (a red picture): red in the middle.
// probe: 32 32 r>150 g<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Mesh3d, Plane3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { LinearRgba, Srgba } from "raea/color";
import { StandardMaterial } from "raea/pbr";

const RED = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='8' height='8'><rect width='8' height='8' fill='%23ff0000'/></svg>";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <Mesh3d name="card"><Plane3d normal={[0, 0, 1]} half_size={[1, 1]} /><StandardMaterial base_color="#000000" perceptual_roughness={1} emissive={LinearRgba.from(Srgba.hex("#ffffff")).mul(1)} emissive_texture={RED} /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
