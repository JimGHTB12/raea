// gap: G3
// The mouse wheel and the buttons other than the primary in useInput(), from a scripted input timeline (wheel.json:
// the wheel turned 100 px down on frames 1 and 2, the right button held on frame 3). The game folds the wheel into a
// square's x (16 px per 100) and paints it green while the right button is held: at frame 4 (a state folds the frames before it) it's at x 36..48, green.
// frame: 4
// input: wheel.json
// probe: 42 32 g>150 r<100
// probe: 10 32 r<30 g<30
import { AbsoluteFill, Composition, registerRoot, useGameState } from "raea";

function Shot() {
  const s = useGameState({ x: 4, right: false }, (s, input) => ({ x: s.x + ((input.wheel?.y ?? 0) / 100) * 16, right: (input.buttons ?? []).includes("Right") }));
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <div style={{ position: "absolute", left: s.x, top: 26, width: 12, height: 12, background: s.right ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={5} fps={30} width={64} height={64} />);
