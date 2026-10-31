// The frame-purity check's composition (check.sh): a number drawn when the module loads, three drawn while the frame
// renders, and the frame's Date, as text, so a frame's pixels say what Math.random and Date gave it.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";

const loaded = Math.random();

function Shot() {
  const f = useCurrentFrame();
  const drawn = [Math.random(), Math.random(), Math.random()];
  return (
    <AbsoluteFill style={{ background: "#fff", color: "#000", fontFamily: "monospace", fontSize: 18, padding: 12 }}>
      <div>frame {f}</div>
      <div>loaded {loaded.toFixed(8)}</div>
      {drawn.map((n, i) => <div key={i}>drawn {n.toFixed(8)}</div>)}
      <div>now {new Date().toISOString()}</div>
      <div data-pure={JSON.stringify({ f, loaded, drawn, now: Date.now() })} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="pure" component={Shot} durationInFrames={30} fps={30} width={480} height={200} />);
