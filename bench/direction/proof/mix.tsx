// DR4's proof, the mix as asked: a beep placed at 1 s, whose sound starts at once. checks/cue.ts holds it on the render.
import { AbsoluteFill, Audio, Composition, Sequence, registerRoot, useCurrentFrame } from "raea";

function Mix() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: f < 30 ? "#203040" : "#e0e0e0" }}>
      <Sequence from={30} name="beep" layout="none">
        <Audio src="media/beep.wav" />
      </Sequence>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="mix" component={Mix} width={540} height={960} fps={30} durationInFrames={60} />);
