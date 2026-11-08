// #60: a class giving transform-box and transform-origin (from a <style> inside the <svg>) with the transform inline
// (left), against all three inline (right). Both turn 30deg about their centres, so the two are the same picture.
// bench/svg-css-transform/check.sh
import { AbsoluteFill, Composition, registerRoot } from "raea";
function O() {
  return (
    <AbsoluteFill style={{ background: "#fff" }}>
      <svg width={900} height={300}>
        <style>{`.box { transform-box: fill-box; transform-origin: center; } .k { stroke: #000; stroke-width: 6; fill: none; }`}</style>
        <rect className="box k" x={50} y={50} width={200} height={200} style={{ transform: "rotate(30deg)" }} />
        <rect className="k" x={650} y={50} width={200} height={200} style={{ transformBox: "fill-box", transformOrigin: "center", transform: "rotate(30deg)" }} />
      </svg>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="o" component={O} width={900} height={300} fps={30} durationInFrames={1} />);
