// gap: G3
// A gamepad in useInput(), from a scripted input timeline (gamepad.json: the left stick right on frames 1 to 3, South
// held on frame 3, given once as fields and once as a live host's marked keys). The game moves a square 10 px a frame
// by the stick and paints it green while South is held: at frame 4 (a state folds the frames before it) it's at x 34..46, green.
// frame: 4
// input: gamepad.json
// probe: 40 32 g>150 r<100
// probe: 10 32 r<30 g<30
import { AbsoluteFill, Composition, registerRoot, useGameState } from "raea";

function Shot() {
  const s = useGameState({ x: 4, south: false }, (s, input) => {
    const pad = input.gamepads?.[0];
    return { x: s.x + (pad?.axes.LeftStickX ?? 0) * 10, south: (pad?.buttons ?? []).includes("South") };
  });
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <div style={{ position: "absolute", left: s.x, top: 26, width: 12, height: 12, background: s.south ? "#00ff00" : "#ff0000" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={5} fps={30} width={64} height={64} />);
