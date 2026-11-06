// gap: PL5
// bevy_aseprite_ultra through raea/plugins/aseprite (docs/plugins.feature PL5): AseAnimation plays plugin-aseprite
// .aseprite (4 x 4, one layer, two frames of 500 ms: red, then green; made by hand from the file format) on a 32 px
// sprite in the middle of the 2D overlay (Draws2d brings its camera up). By frame 20 (0.67 s) the animation has moved
// on to the green frame, with the frame's corners black.
// build: plugins
// frame: 20
// probe: 32 32 g>150 r<80 b<80
// probe: 2 2 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { BevyComponent, Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { AseAnimation } from "raea/plugins/aseprite";

const play = { tag: null, speed: 1, playing: true, repeat: "Loop", direction: null, queue: [], hold_relative_frame: false, relative_group: 0, new_relative_group: 0 } as const;

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[32, -32, 0]}>
          <BevyComponent type="raea_sys_sprite::Draws2d" value={{}} />
          <BevyComponent type="bevy_sprite::sprite::Sprite" value={{ custom_size: [32, 32] }} />
          <AseAnimation value={{ aseprite: "plugin-aseprite.aseprite", animation: { ...play, queue: [] } }} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={24} fps={30} width={64} height={64} />);
