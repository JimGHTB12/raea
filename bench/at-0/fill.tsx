// bench/at-0: a 160 x 160 composition of one #246 fill, drawn at frame 0 with --at (check.sh).
import { AbsoluteFill, Composition, registerRoot } from "raea";

const Fill = () => <AbsoluteFill style={{ backgroundColor: "#246" }} />;

registerRoot(() => <Composition id="main" component={Fill} durationInFrames={10} fps={30} width={160} height={160} />);
