// Composition passes fixture: a painted, inked character in front of a crisp <Screen> page in its own pass ("ui"),
// under the film's whole stack (a paint <Look>, <Outline>, <Bloom>, a <Grade> with aberration and a vignette, and
// <DepthOfField>), each told to skip "ui". check.sh beside this checks that the page comes out at its own colours (no
// grain, rag, bands, ink, glow or fringes on it), that the character keeps its paint and ink where it stands over the
// page, that a render equals play, and that a frame rendered in a run equals the same frame rendered alone.
//   passes  the fixture
//   plain   the same scene with nothing skipped: the page painted like the rest (the check's contrast)
// The character is the kid (examples/character, media/ links to its media), turning a little each frame.
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { Bloom, DepthOfField, Grade, Look, Outline, Screen, ToneMapping } from "raea/3d";
import { Character, registerBody, type Skeleton } from "raea/character";
import kidJson from "./media/characters/kid/skeleton.json" with { type: "json" };
import { DirectionalLight } from "raea/light";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

registerBody(kidJson as unknown as Skeleton);

const FRAMES = 30;
export const PAGE = "#2a6df4";
export const CARD = "#ffd23f";

function Shot({ skip }: { skip?: string[] }) {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#f4efe6" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 1.1, 3.4).lookingAt([0, 0.9, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }}><DepthOfField focal_distance={2.4} aperture_f_stops={4} /></Camera3d>
        <ToneMapping mode="none" />
        <Look mode="paint" bands={3} shadow="#4a5590" mid="#c98a74" sky={["#f4efe6", "#f4efe6"]} rag={0.5} grain={0.1} skip={skip} />
        <Outline color="#1d1a24" width={2} skip={skip} />
        <Bloom intensity={0.4} skip={skip} />
        <Grade temperature={0.03} vignette={0.3} aberration={0.02} skip={skip} />
        <DirectionalLight transform={Transform.fromXyz(-3, 4, 3).lookingAt([0, 0, 0])} illuminance={11200} shadow_maps_enabled />
        <Screen name="site" pass="ui" crt={false} resolution={[960, 540]} size={[3.2]} position={[0.5, 1, -1]} background={PAGE}>
          <AbsoluteFill style={{ background: PAGE, color: "#ffffff", fontSize: 72, padding: 48, fontWeight: 700 }}>
            raea.io
            <div style={{ position: "absolute", right: 48, top: 48, width: 300, height: 300, background: CARD, borderRadius: 24 }} />
          </AbsoluteFill>
        </Screen>
        <Character name="kid" body="kid" position={[-0.45, 0, 0.6]} yaw={0.4 + f * 0.02} />
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => (
  <>
    <Composition id="passes" component={() => <Shot skip={["ui"]} />} width={640} height={360} fps={30} durationInFrames={FRAMES} />
    <Composition id="plain" component={() => <Shot />} width={640} height={360} fps={30} durationInFrames={FRAMES} />
  </>
));
