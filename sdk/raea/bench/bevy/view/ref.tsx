// view's scene as a frame of its own at its box's size (400 x 300): what view's box shows.
import { AbsoluteFill, Composition, interpolate, registerRoot, useCurrentFrame } from "raea";
import { Scene } from "./scene";

function Shot() {
  const t = interpolate(useCurrentFrame(), [0, 29], [0, 1.2]);
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Scene t={t} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="view-ref" component={Shot} width={400} height={300} fps={30} durationInFrames={30} />);
