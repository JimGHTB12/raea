// DR4's proof, the edit as asked: two pieces of one take, cut between its words (media/take.words.json: "one" ends
// at 0.95 s, "two" runs 1.05 to 1.90, "three" starts at 2.05). checks/cut.ts holds it.
import { AbsoluteFill, Composition, Sequence, Video, registerRoot } from "raea";

function Edit() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <Sequence from={0} durationInFrames={30} name="a">
        <Video src="media/take.mp4" startFrom={0} endAt={30} style={{ width: 540, height: 960 }} />
      </Sequence>
      <Sequence from={30} durationInFrames={30} name="b">
        <Video src="media/take.mp4" startFrom={60} endAt={90} style={{ width: 540, height: 960 }} />
      </Sequence>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="cut" component={Edit} width={540} height={960} fps={30} durationInFrames={60} />);
