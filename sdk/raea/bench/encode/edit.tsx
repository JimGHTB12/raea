// Flat vector motion graphics at 4K (encode quality fixture): hard-edged SVG shapes and text that move every frame,
// so the encoder has to spend bits on sharp edges in motion. check.sh encodes it and compares frame 45 against
// the lossless PNG of the same frame (SSIM, and a crop of text edges).
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";

function Flat() {
  const f = useCurrentFrame();
  const x = 400 + f * 18;
  const spin = f * 3;
  return (
    <AbsoluteFill style={{ background: "#f4efe6" }}>
      <svg width={3840} height={2160} viewBox="0 0 3840 2160" style={{ position: "absolute", left: 0, top: 0 }}>
        <rect x={x} y={260} width={900} height={520} rx={60} fill="#1d3557" />
        <circle cx={3200 - f * 12} cy={700} r={320} fill="#e63946" />
        <g transform={`translate(1900 1500) rotate(${spin})`}>
          <polygon points="0,-380 90,-120 370,-120 145,45 230,310 0,150 -230,310 -145,45 -370,-120 -90,-120" fill="#ffb703" stroke="#023047" strokeWidth={14} />
        </g>
        {Array.from({ length: 24 }, (_, i) => (
          <line key={i} x1={0} y1={1900 + i * 10} x2={3840} y2={1900 + i * 10 - 200 + ((f * 7) % 200)} stroke={i % 2 ? "#023047" : "#8ecae6"} strokeWidth={3} />
        ))}
      </svg>
      <div style={{ position: "absolute", left: 200 + f * 4, top: 900, fontFamily: "Inter", fontWeight: 700, fontSize: 220, color: "#023047", letterSpacing: -4 }}>
        Render Anything
      </div>
      <div style={{ position: "absolute", left: 220, top: 1180 - f * 2, width: 1500, fontFamily: "Inter", fontSize: 44, lineHeight: 1.35, color: "#1d3557" }}>
        Small type is where an encoder gives up first: thin stems, counters and the hairline gaps between letters.
        The quick brown fox jumps over the lazy dog. 0123456789 — ilIl1| ¶ § @ & %.
      </div>
      <div style={{ position: "absolute", right: 160, top: 120, fontFamily: "JetBrains Mono", fontSize: 64, color: "#e63946" }}>
        {String(f).padStart(4, "0")}
      </div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="flat4k" component={Flat} width={3840} height={2160} fps={30} durationInFrames={90} />);
