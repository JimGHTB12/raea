// gap: FC1c
// A StandardMaterial field past the row columns' four, animated (docs/control.feature FC1c): a black cube glowing red
// whose emissive_exposure_weight runs from 0 at frame 0 to 1 at frame 20, set by reflection on its own material each
// frame. Played live (0.85 s, frame 25, past the track's end) it is grey, not red: the emissive under the camera's exposure leaves only the black base's specular.
// play: live-world.json 0.85
// probe: 32 32 r<80
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { StandardMaterial } from "raea/pbr";
import { Cuboid, Mesh3d } from "raea/mesh";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";
import { track } from "raea";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D layered>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} />
        <Mesh3d transform={Transform.fromXyz(0, 0, 0)}>
          <Cuboid half_size={[0.8, 0.8, 0.8]} />
          <StandardMaterial base_color="#000000" emissive="#ff0000" animate={{ emissive_exposure_weight: track({ 0: 0, 20: 1 }) }} />
        </Mesh3d>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={30} fps={30} width={64} height={64} />);
