// A layered Canvas3D over a red page with clearColor="#000000" (layered-clear.test.tsx): the 3D clears black and its
// half-transparent white square blends over that in linear light (sRGB 188, as Bevy blends over its ClearColor), and
// the page's red is hidden.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Square } from "./square";

const Shot = () => (
  <AbsoluteFill style={{ background: "#ff0000" }}>
    <Square clearColor="#000000" />
  </AbsoluteFill>
);

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={3} fps={30} width={160} height={90} />);
