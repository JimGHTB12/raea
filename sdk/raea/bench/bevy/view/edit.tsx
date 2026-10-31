// A camera inside a page element (docs/sdk.feature S3a): no Canvas3D; the Camera3d sits in a 400 x 300 box, and its
// view is drawn there at the box's aspect (ref.tsx draws the same scene as a 400 x 300 frame of its own: the box
// equals it). The page around the box is the page alone; a label after the box is over the view. Render equals its
// baked play (check.sh).
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Scene } from "./scene";

function Shot() {
  const t = interpolate(useCurrentFrame(), [0, 29], [0, 1.2]);
  return (
    <AbsoluteFill style={{ background: "#e8eef6" }}>
      <div style={{ position: "absolute", left: 120, top: 80, width: 400, height: 300, background: "#000" }}>
        <Scene t={t} />
      </div>
      <div style={{ position: "absolute", left: 140, top: 340, font: "800 24px Inter", color: "#fff" }}>VIEW</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="view" component={Shot} width={960} height={540} fps={30} durationInFrames={30} />);
