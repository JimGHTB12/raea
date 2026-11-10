// DR4's proof, the ad's cheat: the caption's box is set to end above the band (70 px tall at 640 px, 74% down), but
// its words wrap past it to about 85%. A check of the element's box passes it; checks/caption.ts reads the words'
// own box and must fail it.
import { AbsoluteFill, Composition, registerRoot } from "raea";

function Ad() {
  return (
    <AbsoluteFill style={{ background: "#f4e9d8" }}>
      <div style={{ position: "absolute", left: 60, top: 640, width: 380, height: 70, fontSize: 44, fontWeight: 700, color: "#222" }}>
        Fresh bread every morning, baked in the shop since six
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="ad-cheat" component={Ad} width={540} height={960} fps={30} durationInFrames={90} />);
