// The Phrases check's cue-less case (check.sh): a Phrases with no cue after a Hook. It starts where its first line is
// said (frame 60), so the Hook has the band until then. Before, it started at frame 0 with the Hook, which was cut to
// one frame, and the band stood empty until the Phrases' first line.
import { Composition, registerRoot } from "raea";
import { Ad, Hook, Phrases } from "./components/reel";

function Piece() {
  return (
    <Ad>
      <Hook say="pick a stone" until="some moss" text="pick a / *stone*" />
      <Phrases text="then *name* / your rock / and keep it" />
    </Ad>
  );
}
registerRoot(() => <Composition id="piece" component={Piece} width={1080} height={1920} fps={30} durationInFrames={200} />);
