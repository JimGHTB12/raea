// The Phrases check's early case (check.sh): a Phrases started at frame 0 whose first line is said at frame 60: refused, saying to start it at frame 60.
import { Composition, registerRoot } from "raea";
import { Ad, Phrases } from "./components/reel";

function Piece() {
  return (
    <Ad>
      <Phrases from={0} seconds={2} text="then *name* / your rock" />
    </Ad>
  );
}
registerRoot(() => <Composition id="piece" component={Piece} width={1080} height={1920} fps={30} durationInFrames={200} />);
