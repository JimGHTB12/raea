// gap: PL7
// bevy_spritesheet_animation through raea/plugins/spritesheet (docs/plugins.feature PL7): SheetAnimation, raea's
// reflected component over the crate's Spritesheet, Animation and Sprite, plays a sheet made in code (two one-pixel
// cells, red then green, sampled nearest) at 500 ms a cell as a 32 px sprite in the middle of the 2D overlay. Frame 1 shows the red cell; by
// frame 20 (0.67 s) the animation has moved on to the green one, with the frame's corners black.
// build: plugins
// frame: 20
// probe: 32 32 g>150 r<80 b<80
// probe: 2 2 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Entity } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { SheetAnimation } from "raea/plugins/spritesheet";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <Entity position={[32, -32, 0]}>
          <SheetAnimation value={{ image: { width: 2, height: 1, pixels: ["#ff0000", "#00ff00"], nearest: true }, image_size: [2, 1], columns: 2, rows: 1, frames: [0, 1], frame_ms: 500, size: [32, 32] }} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={24} fps={30} width={64} height={64} />);
