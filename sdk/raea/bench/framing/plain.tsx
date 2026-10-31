// The framing check's plain case: the stock talking take (engine/cli/conformance/fixtures/face.mp4, linked into media/ by
// check.sh) filling a 9:16 frame.
import { AbsoluteFill, Composition, registerRoot, staticFile, Video } from "raea";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Video src={staticFile("face.mp4")} style={{ width: "100%", height: "100%", objectFit: "cover" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={60} fps={30} width={360} height={640} />);
