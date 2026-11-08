// bench/background-clip-text/check.sh's scrolled case: two scrollers of "One Two Three Four" (48px, a 60px line, 120px
// tall), each scrolled down a line. The first is plain pink text: Chrome 148 shows "Two" and "Three" (#46: raea showed
// "Three" and "Four", the text scrolled twice). The second clips a gradient to its text: Chrome masks it with the text
// as at scrollTop 0, so it shows "One" and "Two" (#37).
import { Composition, registerRoot } from "raea";
const box = { height: 120, overflow: "hidden" } as const;
const clip = { ...box, background: "linear-gradient(90deg,#f0c,#0cf)", backgroundClip: "text", color: "transparent" } as const;
function Scrolled() {
  return (
    <div style={{ width: 640, height: 240, background: "#111", font: "bold 48px sans-serif", lineHeight: "60px" }}>
      <div style={{ ...box, color: "#f0c" }} scrollTop={60}>One<br />Two<br />Three<br />Four</div>
      <div style={clip} scrollTop={60}>One<br />Two<br />Three<br />Four</div>
    </div>
  );
}
registerRoot(() => <Composition id="scrolled" component={Scrolled} width={640} height={240} fps={30} durationInFrames={1} />);
