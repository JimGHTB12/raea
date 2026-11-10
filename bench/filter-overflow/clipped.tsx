// 60 small filtered boxes, each around a clipping box of its size holding a child that covers the frame from any of
// them (the last 30 clipping boxes positioned with a z-index, so the filtered box's stacking context paints them):
// 60 grey 60 px tiles on white. A filter's layer reaches over what its element draws, and stops at a clip in
// the way: made from the element's overflow, which reaches through clips, each of these layers would be the size of
// the frame, and 30 of them are more than Vello's image atlas holds, of either kind. bench/filter-overflow/check.sh
import { AbsoluteFill, Composition, registerRoot } from "raea";
function O() {
  return (
    <AbsoluteFill style={{ background: "#fff" }}>
      {Array.from({ length: 60 }, (_, i) => (
        <div key={i} style={{ position: "absolute", left: 60 + (i % 10) * 180, top: 60 + Math.floor(i / 10) * 170, width: 60, height: 60, filter: "grayscale(1)" }}>
          <div style={{ width: 60, height: 60, overflow: "hidden", ...(i < 30 ? {} : { position: "relative", zIndex: 1 }) }}>
            <div style={{ position: "relative", left: -2000, top: -1500, width: 4000, height: 3000, background: "#008000" }} />
          </div>
        </div>
      ))}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="o" component={O} width={1920} height={1080} fps={30} durationInFrames={1} />);
