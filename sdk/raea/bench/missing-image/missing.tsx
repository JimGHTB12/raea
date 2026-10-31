// The missing-image check's missing case: an <Img> of media/missing.png (not there) filling the frame (check.sh).
import { AbsoluteFill, Composition, Img, registerRoot, staticFile } from "raea";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#2d4a6b" }}>
      <Img src={staticFile("missing.png")} style={{ width: "100%", height: "100%" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={320} height={180} />);
