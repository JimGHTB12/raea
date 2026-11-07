// The same layered Canvas3D without clearColor (layered-clear.test.tsx): the 3D clears transparent, the page's red
// shows where the 3D has nothing.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Square } from "./square";

const Shot = () => (
  <AbsoluteFill style={{ background: "#ff0000" }}>
    <Square />
  </AbsoluteFill>
);

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={3} fps={30} width={160} height={90} />);
