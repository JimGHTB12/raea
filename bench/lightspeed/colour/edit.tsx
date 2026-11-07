// COLOUR: flat patches of known sRGB values, the colour-accuracy fixture of lightspeed's Q3
// (bench/lightspeed/export.sh): primaries and secondaries at full and half strength, a grey ramp, skin tones and the
// near-black and near-white steps where range and matrix mistakes show first. Rendered, encoded and decoded back, each
// patch's centre must come back as the value it was painted with.
import { AbsoluteFill, Composition, registerRoot } from "raea";

export const PATCHES = [
  "#ff0000", "#00ff00", "#0000ff", "#00ffff", "#ff00ff", "#ffff00",
  "#800000", "#008000", "#000080", "#008080", "#800080", "#808000",
  "#000000", "#101010", "#404040", "#808080", "#c0c0c0", "#ffffff",
  "#050505", "#f5f5f5", "#e0ac69", "#8d5524", "#c68642", "#ffdbac",
];
const COLS = 6;
const W = 1920;
const H = 1080;

function Patches() {
  const w = W / COLS;
  const h = H / (PATCHES.length / COLS);
  return (
    <AbsoluteFill style={{ background: "#000000" }}>
      {PATCHES.map((c, i) => (
        <div key={i} style={{ position: "absolute", left: (i % COLS) * w, top: Math.floor(i / COLS) * h, width: w, height: h, background: c }} />
      ))}
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="colour" component={Patches} width={W} height={H} fps={30} durationInFrames={30} />);
