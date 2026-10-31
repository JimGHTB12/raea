// The Phrases check's unsaid case (check.sh): a Phrases whose lines aren't said at all: refused, saying so.
import { Composition, registerRoot } from "raea";
import { Ad, Phrases } from "./components/reel";

function Piece() {
  return (
    <Ad>
      <Phrases text="never said here" />
    </Ad>
  );
}
registerRoot(() => <Composition id="piece" component={Piece} width={1080} height={1920} fps={30} durationInFrames={200} />);
