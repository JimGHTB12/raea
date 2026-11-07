// Bevy v0.20.0 examples/2d/sprite_animation.rs in raea's TSX (docs/control.feature FC0a): gabe's sheet twice, 6x and
// sampled nearest, at x -70 and 70, each stepping from cell 1 to 6 on a Once timer of 1/fps s (10 and 20 fps), back
// to 1 at the end and stopping there (no arrow key starts them again), and two lines of instructions in Bevy's default
// font at top 12, left 12, on Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useCurrentFrame } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const SHEET = staticFile("textures/rpg/chars/gabe/gabe-idle-run.png");
const DELTA = 16_666_667; // Bevy's 1/60 s Duration, in nanoseconds

/** execute_animations up to frame f: a Once timer of Duration::from_secs_f32(1 / fps) ticked by each delta (none on
 *  the first); each finish steps the cell and starts a new timer, but the one finishing on the last cell goes back to
 *  the first and starts none */
function cellAt(f: number, fps: number): number {
  const dur = Math.floor(Math.fround(1 / fps) * 1e9);
  let idx = 1, el = 0, running = true;
  for (let k = 0; k <= f; k++) {
    if (!running) break;
    el += k === 0 ? 0 : DELTA;
    if (el < dur) continue;
    if (idx === 6) {
      idx = 1;
      running = false;
    } else {
      idx += 1;
      el = 0;
    }
  }
  return idx;
}

function Shot() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={SHEET} grid={[7, 1]} frame={cellAt(f, 10)} x={640 - 70} y={360} scale={6} />
        <Sprite src={SHEET} grid={[7, 1]} frame={cellAt(f, 20)} x={640 + 70} y={360} scale={6} />
      </Canvas3D>
      <div style={{ position: "absolute", top: 12, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{"Left Arrow: Animate Left Sprite\nRight Arrow: Animate Right Sprite"}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={61} fps={60} width={1280} height={720} />);
