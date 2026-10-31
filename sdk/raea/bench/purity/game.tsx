// The frame-purity check's game (check.sh): each step of useGameState's fold draws a Math.random and keeps it, so a
// frame's state lists what every step before it drew. Seeded by the frame being drawn, step 5's draw would depend on
// which frame's render ran the fold (frame 20 alone runs steps 0-19, frame 30 alone 0-29); seeded by its own frame it
// doesn't.
import { AbsoluteFill, Composition, registerRoot, useGameState } from "raea";

const step = (drawn: number[]) => [...drawn, Math.random()];

function Game() {
  const drawn = useGameState<number[]>([], step);
  return (
    <AbsoluteFill style={{ background: "#fff", color: "#000", fontFamily: "monospace", fontSize: 18, padding: 12 }}>
      <div>steps {drawn.length}</div>
      <div>step 5 drew {drawn[5]?.toFixed(8)}</div>
      <div data-pure={JSON.stringify({ drawn })} />
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="game" component={Game} durationInFrames={40} fps={30} width={480} height={200} />);
