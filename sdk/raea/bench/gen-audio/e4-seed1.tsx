// The generated-audio check's e4-seed1 case (check.sh): one TokenSynth note, E4 with seed 1, played for a second.
import { AbsoluteFill, Audio, Composition, registerRoot } from "raea";
import * as Tone from "raea/tone";

const note = Object.values(Tone.tokensynth("a soft felt piano, warm and close", ["E4"], { seconds: 1, seed: 1 }))[0];

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#111" }}>
      <Audio src={note} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={30} fps={30} width={160} height={90} />);
