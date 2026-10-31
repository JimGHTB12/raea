// gap: G3
// Touches in useInput(), from a scripted input timeline (touch.json: two fingers on frame 1, as a live host marks
// them). A square is drawn under each finger: green at both.
// frame: 1
// input: touch.json
// probe: 16 16 g>150 r<100
// probe: 48 48 g>150 r<100
// probe: 48 16 r<30 g<30
import { AbsoluteFill, Composition, registerRoot, useInput } from "raea";

function Shot() {
  const input = useInput();
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      {(input.touches ?? []).map((t) => <div key={t.id} style={{ position: "absolute", left: t.x - 6, top: t.y - 6, width: 12, height: 12, background: "#00ff00" }} />)}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
