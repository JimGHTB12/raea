// bench/frame-panic's 2D page: a box crossing the frame over 150 frames, drawn by the page engine alone
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
function Page() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#0f172a" }}>
      <div style={{ position: "absolute", left: 40 + f * 7, top: 300, width: 160, height: 120, borderRadius: 16, background: "#38bdf8" }} />
      <div style={{ position: "absolute", left: 60, top: 80, color: "#f8fafc", fontSize: 48 }}>frame {f}</div>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="page" component={Page} width={1280} height={720} fps={30} durationInFrames={150} />);
