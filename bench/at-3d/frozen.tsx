// The frames bench/at-3d/check.sh draws from scene.tsx with --at, as a 4-frame composition: each frame k is the scene
// frozen at F[k], so its 3D rows come from stepping frames 0..4, the whole composition, not the sparse path
import { AbsoluteFill, Composition, Freeze, registerRoot, useCurrentFrame } from "raea";
import { Scene } from "./scene3d";
const F = [30, 150, 270, 330];
function Frozen() {
  const k = useCurrentFrame();
  return (
    <AbsoluteFill>
      <Freeze frame={F[k]}>
        <Scene />
      </Freeze>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="frozen" component={Frozen} width={1280} height={720} fps={60} durationInFrames={4} />);
