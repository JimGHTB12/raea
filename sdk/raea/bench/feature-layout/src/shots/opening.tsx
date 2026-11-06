// A shot: a scene component, one time range of the cut (main.tsx places it in a Sequence).
import { AbsoluteFill, Img, interpolate, staticFile, useCurrentFrame } from "raea";

export function Opening() {
  const f = useCurrentFrame();
  const x = interpolate(f, [0, 60], [-20, 20], { extrapolateRight: "clamp" });
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center" }}>
      <div style={{ width: "30%", aspectRatio: "1", borderRadius: "50%", background: "#ff7a1a", transform: `translateX(${x}%)` }} />
      {/* the fixture's own: a file in assets/, read by staticFile through the project's folder (S5a) */}
      <Img src={staticFile("mark.svg")} style={{ position: "absolute", left: "8%", top: "8%", width: "12%" }} />
    </AbsoluteFill>
  );
}
