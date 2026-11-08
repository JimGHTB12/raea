// #26's composition: scene3d.tsx's scene over 360 frames, which bench/at-3d/check.sh draws four of with --at
import { Composition, registerRoot } from "raea";
import { Scene } from "./scene3d";
registerRoot(() => <Composition id="steps" component={Scene} width={1280} height={720} fps={60} durationInFrames={360} />);
