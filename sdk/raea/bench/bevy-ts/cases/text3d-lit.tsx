// gap: P19
// Text in the 3D world takes light and is occluded (rendered): a white "O" lit only by a red light from the front
// reads red, shaded like a mesh (a white sphere in its place reads 242,87,73 under this light, the O 232,76,66; drawn
// flat it would be white); a blue box in front of its right half hides it (depth-tested like any mesh). A large "O" so
// its ring crosses the probes.
// probe: 18 32 r>180 g<120 b<120
// probe: 50 32 b>120 r<40
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D, RichText } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { AmbientLight, PointLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <PointLight color="#ff0000" transform={Transform.fromXyz(0, 0, 3)} intensity={540000} />
        <AmbientLight brightness={0} />
        <RichText name="o" size={2.6} weight={900} roughness={1}>O</RichText>
        <Mesh3d transform={Transform.fromXyz(1, 0, 1)} name="cover"><Cuboid half_size={[1, 1.5, 0.05]} /><StandardMaterial base_color="#0000ff" unlit /></Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
