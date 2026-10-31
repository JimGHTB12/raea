// gap: G3
// Live play's wire for input beyond keys and the pointer: `play --offscreen` given marked keys from frame 2 on
// (live-input.json: the wheel, the right button, a gamepad's South, a finger), as play's window marks winit's events.
// Each quarter lights green once the game has seen its kind: wheel top left, right button top right, gamepad bottom
// left, touch bottom right.
// play: live-input.json 0.4
// probe: 16 16 g>150 r<100
// probe: 48 16 g>150 r<100
// probe: 16 48 g>150 r<100
// probe: 48 48 g>150 r<100
import { AbsoluteFill, Composition, registerRoot, useGameState } from "raea";

function Shot() {
  const s = useGameState({ wheel: false, right: false, pad: false, touch: false }, (s, i) => ({
    wheel: s.wheel || (i.wheel?.y ?? 0) > 0,
    right: s.right || (i.buttons ?? []).includes("Right"),
    pad: s.pad || (i.gamepads?.[0]?.buttons ?? []).includes("South"),
    touch: s.touch || (i.touches ?? []).length > 0,
  }));
  const q = (on: boolean, left: number, top: number) => <div style={{ position: "absolute", left, top, width: 32, height: 32, background: on ? "#00ff00" : "#ff0000" }} />;
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      {q(s.wheel, 0, 0)}
      {q(s.right, 32, 0)}
      {q(s.pad, 0, 32)}
      {q(s.touch, 32, 32)}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={300} fps={30} width={64} height={64} />);
