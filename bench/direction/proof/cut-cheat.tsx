// DR4's proof, the edit's cheat: the same two pieces with the same length of shot, the first cut out of the take in
// the middle of "two" (1.5 s) and the second into it in the middle of "three" (2.5 s). checks/cut.ts must fail it.
import { AbsoluteFill, Composition, Sequence, Video, registerRoot } from "raea";

function Edit() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Sequence from={0} durationInFrames={45} name="a">
        <Video src="media/take.mp4" startFrom={0} endAt={45} style={{ width: 540, height: 960 }} />
      </Sequence>
      <Sequence from={45} durationInFrames={15} name="b">
        <Video src="media/take.mp4" startFrom={75} endAt={90} style={{ width: 540, height: 960 }} />
      </Sequence>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="cut-cheat" component={Edit} width={540} height={960} fps={30} durationInFrames={60} />);
