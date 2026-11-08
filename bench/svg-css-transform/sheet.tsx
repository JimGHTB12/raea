// #60: an SVG shape turned by a <style> inside its <svg> (red: transform-box, transform-origin and transform from a
// class), by the transform attribute (blue) and by the three set inline (green). Chrome turns all three 30deg about
// their centres. bench/svg-css-transform/check.sh
import { AbsoluteFill, Composition, registerRoot } from "raea";
function O() {
  return (
    <AbsoluteFill style={{ background: "#fff" }}>
      <svg width={900} height={300}>
        <style>{`.css { fill: none; stroke: #e0245e; stroke-width: 6; transform-box: fill-box; transform-origin: center; transform: rotate(30deg); }
          .attr { fill: none; stroke: #1d9bf0; stroke-width: 6; }`}</style>
        <rect className="css" x={50} y={50} width={200} height={200} />
        <rect className="attr" x={350} y={50} width={200} height={200} transform="rotate(30 450 150)" />
        <rect x={650} y={50} width={200} height={200} style={{ fill: "none", stroke: "#17bf63", strokeWidth: 6, transformBox: "fill-box", transformOrigin: "center", transform: "rotate(30deg)" }} />
      </svg>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="o" component={O} width={900} height={300} fps={30} durationInFrames={1} />);
