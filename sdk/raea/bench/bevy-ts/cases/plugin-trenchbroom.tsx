// gap: PL4
// bevy_trenchbroom through raea/plugins/trenchbroom (docs/plugins.feature PL4): plugin-trenchbroom.map, a TrenchBroom
// (Valve 220) map whose worldspawn is one cube brush 80 units on a side (about 2 m at bevy_trenchbroom's 39.37 units a
// metre), loaded as Bevy's WorldAssetRoot from the map's #Scene and lit from the camera's side: a lit square in the
// middle of the frame, black in the corners. Its texture name has no file under the asset root's textures/, so it
// draws with bevy_trenchbroom's missing-texture checker (grey and magenta); the map loading and drawing is the check.
// build: plugins
// frame: 20
// probe: 32 32 r>60 g>60 b>60
// probe: 2 2 r<20 g<20 b<20
// probe: 61 61 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Canvas3D, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Canvas3D>
        <Camera3d transform={Transform.fromXyz(0, 0, 6).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <DirectionalLight transform={Transform.fromXyz(0.2, 0.3, 1).lookingAt([0, 0, 0])} illuminance={10000} />
        <Entity>
          <BevyComponent type="bevy_world_serialization::components::WorldAssetRoot" value="plugin-trenchbroom.map#Scene" />
        </Entity>
      </Canvas3D>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={21} fps={30} width={64} height={64} />);
