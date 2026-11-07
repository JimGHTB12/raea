// Bevy v0.20.0 examples/camera/pan_camera_controller.rs in raea's TSX (docs/control.feature FC0a): a Camera2d with
// Bevy's PanCamera (no input, so it holds still), Bevy's bird as a Sprite in the middle, and the camera's controls as
// Bevy UI text (PanCamera's Display: a leading newline, then five lines) at top -16, left 12, in Bevy's default font
// at its default 20 px, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D, Sprite } from "raea/3d";

const FONT = `@font-face { font-family: "Fira Mono"; src: url("${staticFile("fonts/FiraMono-Medium.ttf")}"); }`;
const CONTROLS = `
PanCamera Controls:
  Move Up / Down    - KeyW / KeyS
  Move Left / Right - KeyA / KeyD
  Rotate CCW / CW   - KeyQ / KeyE
  Zoom              - Mouse Scroll + Equal / Minus
`;

function Shot() {
  return (
    <AbsoluteFill>
      <style>{FONT}</style>
      <Canvas3D layered clearColor="rgb(43, 44, 47)">
        <Sprite src={staticFile("branding/bevy_bird_dark.png")} x={640} y={360} />
      </Canvas3D>
      <div style={{ position: "absolute", top: -16, left: 12, fontFamily: "Fira Mono", fontSize: 20, lineHeight: 1.2, color: "#fff", whiteSpace: "pre" }}>{CONTROLS}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
