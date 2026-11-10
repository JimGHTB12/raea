// DR4's proof, the vertical ad as asked: its caption well above the platform's button band (from 80% down).
// checks/caption.ts holds it.
import { AbsoluteFill, Composition, registerRoot } from "raea";

function Ad() {
  return (
    <AbsoluteFill style={{ background: "#f4e9d8" }}>
      <div style={{ position: "absolute", left: 60, top: 560, width: 380, fontSize: 44, fontWeight: 700, color: "#222" }}>Fresh bread, every morning</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="ad" component={Ad} width={540} height={960} fps={30} durationInFrames={90} />);
