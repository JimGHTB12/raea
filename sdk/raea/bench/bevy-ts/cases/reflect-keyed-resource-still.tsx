// gap: FC1c
// A reflected resource keyed per frame (docs/control.feature FC1c): Bevy's GlobalAmbientLight, given as a
// <BevyResource>, at brightness 0 before frame 10 and 20000 from it, the only light on a white cube. At frame 15 the
// cube is lit (still: a render stepped from frame 0, which kept only frame 0's resource before FC1c).
// frame: 15
// probe: 32 32 r>200 g>200 b>200
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { BevyResource, Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Ambient() {
  const f = useCurrentFrame();
  return <BevyResource type="bevy_light::ambient_light::GlobalAmbientLight" value={{ color: { Srgba: { red: 1, green: 1, blue: 1, alpha: 1 } }, brightness: f < 10 ? 0 : 20000, affects_lightmapped_meshes: true }} />;
}
function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.fromXyz(0, 0, 0)}><Cuboid half_size={[0.6, 0.6, 0.6]} /><StandardMaterial base_color="#ffffff" /></Mesh3d>
        <Ambient />
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={30} fps={30} width={64} height={64} />);
