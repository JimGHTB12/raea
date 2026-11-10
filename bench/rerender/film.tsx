// RR0's film (docs/rerender.feature): six 5 s shots at 1920x1080, 30 fps (900 frames), each with its own look and
// camera, on one procedural set (no media files, so it renders the same on macOS, Windows and Linux). Two forms:
//   "film"            one composition, one camera that jumps at the cuts and a look that changes at every cut: what
//                     RR1 makes render correctly (before RR1 a render holds the first frame's look for all six)
//   "shot-1".."shot-6" each shot as its own composition, as agents split films today; bench.sh renders these as
//                     chunks and joins them, the baseline the overhaul is measured against
// Shot 4 (noir) is the one change.sh edits, as a note would.
import { Composition, registerRoot } from "raea";
import { FPS, Film, SHOT, SHOTS, one } from "./shots";

registerRoot(() => (
  <>
    <Composition id="film" component={Film} width={1920} height={1080} fps={FPS} durationInFrames={SHOT * 6} />
    {SHOTS.map((_, k) => <Composition key={k} id={`shot-${k + 1}`} component={one(k)} width={1920} height={1080} fps={FPS} durationInFrames={SHOT} />)}
  </>
));
