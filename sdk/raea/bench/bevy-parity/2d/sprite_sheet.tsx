// Bevy v0.20.0 examples/2d/sprite_sheet.rs in raea's TSX (docs/control.feature FC0a): gabe's idle-and-run sheet
// (textures/rpg/chars/gabe/gabe-idle-run.png, seven 24 px cells in a row) as a Sprite scaled 6x, sampled nearest, its
// cell stepping from 1 to 6 and round again each time a 0.1 s repeating timer finishes, on Bevy's default clear colour,
// in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

/** animate_sprite over the updates up to frame f: the timer ticks by each delta (Bevy's Duration of 1/60 s, in whole
 *  nanoseconds; none on the first) and the cell steps once per finish */
function cellAt(f: number): number {
  const finishes = Math.floor((f * 16_666_667) / 100_000_000);
  return 1 + (finishes % 6);
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("textures/rpg/chars/gabe/gabe-idle-run.png")} grid={[7, 1]} frame={cellAt(useCurrentFrame())} x={640} y={360} scale={6} />
      </Canvas3D>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
