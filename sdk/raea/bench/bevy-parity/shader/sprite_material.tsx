// Bevy v0.20.0 examples/shader/sprite_material.rs in raea's TSX (docs/control.feature FC0a, FC2a): Bevy's bird
// (branding/bevy_bird_dark.png) as a Sprite in the middle with the example's SpriteMaterial<DissolveMaterial>, its
// assets/shaders/sprite_material.wesl as written (simplex noise against `amount`, a burn edge 0.05 wide in
// LinearRgba(1, 0.66, 0)), and "Space to dissolve" in Bevy's default font at the top left. Space (the input script,
// bench/bevy-parity/input/sprite_material.json) starts it dissolving, and again appearing: dissolve_on_input and
// update_dissolve as the example runs them, `amount` the progress added up by each update's delta, in f32. A Camera2d
// on Bevy's default clear colour, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile, useGameState, useInput } from "raea";
import { Canvas3D, Sprite } from "raea/3d";
import { Material } from "@raea/render/bevy3d"; // raea/pbr once scene-loop lands (its pbr.ts commit)

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const DT = Math.fround(1 / 60); // Time::delta_secs of a 1/60 s update, an f32

type State = { kind: "Visible" | "Dissolving" | "Appearing" | "Hidden"; progress: number; amount: number; space: boolean };
const START: State = { kind: "Visible", progress: 0, amount: 0, space: false };

/** One update: dissolve_on_input on Space's press, then update_dissolve (chained, so it sees the new state). */
function step(s: State, input: { keys: string[] }): State {
  const space = input.keys.includes("Space");
  let { kind, progress, amount } = s;
  if (space && !s.space) {
    if (kind === "Visible") [kind, progress] = ["Dissolving", 0];
    else if (kind === "Hidden") [kind, progress] = ["Appearing", 0];
    else [kind, progress] = [kind === "Dissolving" ? "Appearing" : "Dissolving", Math.fround(1 - progress)];
  }
  if (kind === "Dissolving" || kind === "Appearing") {
    progress = Math.fround(progress + DT);
    const done = progress >= 1;
    if (done) progress = 1;
    amount = kind === "Dissolving" ? progress : Math.fround(1 - progress);
    if (done) kind = kind === "Dissolving" ? "Hidden" : "Visible";
  }
  return { kind, progress, amount, space };
}

function Shot() {
  // Bevy's picture of frame f is drawn after update f, which read frame f's keys: the fold (updates before f) and f's own
  const s = step(useGameState(START, step), useInput());
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("branding/bevy_bird_dark.png")} x={640} y={360}>
          <Material fragment={staticFile("shaders/sprite_material.wesl")} uniforms={{ material: { burn_edge_color: [1, 0.66, 0, 1], amount: s.amount, burn_edge: 0.05 } }} />
        </Sprite>
      </Canvas3D>
      <div style={{ position: "absolute", top: 0, left: 0, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>Space to dissolve</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={131} fps={60} width={1280} height={720} />);
