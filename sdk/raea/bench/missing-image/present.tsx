// The missing-image check's present case: an <Img> of media/red.png filling the frame (check.sh).
import { AbsoluteFill, Composition, Img, registerRoot, staticFile } from "raea";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#2d4a6b" }}>
      <Img src={staticFile("red.png")} style={{ width: "100%", height: "100%" }} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={320} height={180} />);
