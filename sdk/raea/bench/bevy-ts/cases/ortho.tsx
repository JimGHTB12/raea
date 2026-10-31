// gap: G2
// An orthographic camera 4 m tall: two 1 m cubes, one 6 m further away, are the same size on screen (16 px a metre),
// so the far one's right half shows at x 52. A perspective camera would draw it half as wide, ending near x 40.
// probe: 52 32 r>200 g<60
// probe: 12 32 r>200 g<60
// probe: 32 8 r<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, OrthographicCamera } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Transform } from "raea/transform";
import { Cuboid, Mesh3d } from "raea/mesh";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <OrthographicCamera position={[0, 0, 10]} lookAt={[0, 0, 0]} height={4} />
        <Mesh3d transform={Transform.fromXyz(-1, 0, 0)} name="near"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
        <Mesh3d transform={Transform.fromXyz(1, 0, -6)} name="far"><Cuboid half_size={[0.5, 0.5, 0.5]} /><StandardMaterial base_color="#ff0000" unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
