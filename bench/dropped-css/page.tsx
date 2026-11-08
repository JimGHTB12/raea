// #34: outline text, gradient-filled text, and a font face whose file isn't there (the reporter's page), its outline
// moving so its style attribute is parsed again on every frame. bench/dropped-css/check.sh
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
const css = `@font-face { font-family: "Brand"; src: url("/media/fonts/not-here.woff2"); }`;
const big = { position: "absolute" as const, left: 20, fontSize: 64, fontWeight: 900, fontFamily: "Brand, Arial" };
function Clip() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#ffffff" }}>
      <style>{css}</style>
      <div style={{ ...big, top: 30 + f, color: "transparent", WebkitTextStroke: "2px #111111" } as any}>OUTLINE</div>
      <div style={{ ...big, top: 150, color: "transparent", background: "linear-gradient(90deg, #e8174a, #2244ff)", backgroundClip: "text", WebkitBackgroundClip: "text" } as any}>GRADIENT</div>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="s" component={Clip} width={640} height={300} fps={30} durationInFrames={30} />);
