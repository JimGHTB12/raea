// The App: its plugins and its root. A video registers its Composition here; its shots play in Sequences.
import { AbsoluteFill, Composition, Sequence, registerRoot } from "raea";
import { Opening } from "./shots/opening";
import { Title } from "./title";

const FPS = 30;

function Main() {
  return (
    <AbsoluteFill style={{ background: "#101418" }}>
      <Sequence from={0} durationInFrames={FPS * 3}>
        <Opening />
      </Sequence>
      <Title text="feature-layout" from={FPS / 2} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Main} width={1080} height={1920} fps={FPS} durationInFrames={FPS * 3} />);
