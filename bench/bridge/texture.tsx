// B3 H5's texture element (bench/bridge/run.sh --pixels, bench/bridge/examples/texture_slot): an
// <img data-raea-texture={7}> on a plain App's page shows the App's PageTextures 7, a camera of the App's own.
import { AbsoluteFill, Composition, registerRoot } from "raea";

import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#ff0000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 5).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
      </>
      <img data-raea-texture={7} style={{ position: "absolute", left: 100, top: 50, width: 200, height: 150 }} />
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="bridgetexture" component={Shot} width={640} height={360} fps={30} durationInFrames={60} />);
