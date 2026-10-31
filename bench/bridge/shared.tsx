// B3 H4's bench (docs/blitz-bevy.feature): a composition reading and writing a resource the App shares
// (bench/bridge/examples/shared_state.rs). React sets the score to 100 (<Resource value>); a Bevy system counts up from there;
// the text shows the World's value as it comes back.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { resource } from "raea/bevy";
import { Resource, useResource } from "raea/ecs";

const Score = resource<{ points: number }>("bench::Score");

function Shot() {
  const score = useResource(Score);
  return (
    <AbsoluteFill>
      <Resource resource={Score} value={{ points: 100 }} />
      <div id="score">{score ? `score ${score.points}` : "score none"}</div>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="shared" component={Shot} width={320} height={180} fps={30} durationInFrames={600} />);
