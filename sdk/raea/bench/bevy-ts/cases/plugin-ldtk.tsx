// gap: PL4
// bevy_ecs_ldtk through raea/plugins/ldtk (docs/plugins.feature PL4): plugin-ldtk.ldtk, one 64 x 64 px level with
// one Tiles layer of 32 px tiles from plugin-tiles.png (red, green / green, red as LDtk draws it, top row first),
// loaded by its LdtkProjectHandle with CurrentLevel 0. bevy_ecs_ldtk puts a level's bottom left at its world entity
// (+y up), so the world sits at y -64 on the 2D overlay (pixels from the frame's top left) to fill the frame.
// build: plugins
// frame: 20
// probe: 16 16 r>200 g<60 b<60
// probe: 48 16 g>200 r<60 b<60
// probe: 16 48 g>200 r<60 b<60
// probe: 48 48 r>200 g<60 b<60
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { CurrentLevel, LdtkProjectHandle } from "raea/plugins/ldtk";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[0, -64, 0]}>
          <LdtkProjectHandle value={{ handle: "plugin-ldtk.ldtk" }} />
          <CurrentLevel value={{ index: 0 }} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={21} fps={30} width={64} height={64} />);
