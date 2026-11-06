// gap: PL7
// bevy_rich_text3d through raea/plugins/text3d (docs/plugins.feature PL7): Label3d, raea's reflected component over
// Text3d, Text3dStyle, the mesh and an unlit material over the glyph atlas, in Fira Mono (bevy_text's subset, which
// the adapter loads so text draws the same on every machine). A white "H" whose em is 1.5 m (world_scale is the em's
// height in metres, as raea's own text sets it), 3 m from the camera: its stems and crossbar white, the gap between
// the stems above the crossbar and the frame's corners black.
// build: plugins
// frame: 1
// probe: 25 22 r>200 g>200 b>200
// probe: 38 40 r>200 g>200 b>200
// probe: 32 31 r>200 g>200 b>200
// probe: 32 22 r<20 g<20 b<20
// probe: 2 2 r<20 g<20 b<20
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Entity, ToneMapping } from "raea/3d";
import { Camera3d } from "raea/camera";
import { Transform } from "raea/transform";
import { Label3d } from "raea/plugins/text3d";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 3).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.785398 } }} />
        <ToneMapping mode="none" />
        <Entity position={[0, 0, 0]}>
          <Label3d value={{ text: "H", size: 128, world_scale: 1.5, color: [1, 1, 1, 1], font: "Fira Mono" }} />
        </Entity>
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="main" component={Shot} durationInFrames={2} fps={30} width={64} height={64} />);
