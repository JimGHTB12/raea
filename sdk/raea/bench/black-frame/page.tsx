// The black-frame redraw's page case: a cream page full of text and shapes, all on the Blitz page (no 3D).
// check.sh renders it as is and with a GPU fault injected (RAEA_BLACK_FAULT).
import { AbsoluteFill, Composition, registerRoot } from "raea";

const words = "the quick brown fox jumps over the lazy dog while the render keeps every frame it draws".split(" ");

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#efe0b4", padding: 24, display: "flex", flexWrap: "wrap", gap: 8, fontFamily: "sans-serif" }}>
      {Array.from({ length: 60 }, (_, i) => (
        <div key={i} style={{ padding: "6px 10px", borderRadius: 8, background: i % 3 ? "#2d4a6b" : "#b5452e", color: "#fff", fontSize: 18 }}>
          {words[i % words.length]} {i}
        </div>
      ))}
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={640} height={360} />);
