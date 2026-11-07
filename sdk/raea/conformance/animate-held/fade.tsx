// A white box that fades in over frames 0-6 with <Animate enter="fade"> (animate-held.test.tsx): React renders it once
// and never again, so the engine samples it each frame (runtime/react animate.rs) though its props don't change.
import { Animate, Composition, registerRoot } from "raea";

const Shot = () => (
  <div style={{ position: "absolute", inset: 0, background: "#000000" }}>
    <Animate enter="fade" style={{ position: "absolute", inset: 0, background: "#ffffff" }} />
  </div>
);

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={20} fps={30} width={160} height={90} />);
