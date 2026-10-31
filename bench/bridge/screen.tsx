// B3 H5's <Screen> case (bench/bridge/run.sh --pixels): a flat screen in the 3D showing a nested composition, in a
// plain App's page (each screen a document of its own, copied onto the screen) against raea's render of the frame.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Screen, ToneMapping } from "raea/3d";
import { Transform } from "raea/transform";
import { Camera3d } from "raea/camera";

function Shot() {
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <Camera3d transform={Transform.fromXyz(0, 0, 4).lookingAt([0, 0, 0])} projection={{ Perspective: { fov: 0.698131700797732 } }} />
        <ToneMapping mode="none" />
        <Screen name="panel" crt={false} resolution={[640, 360]} size={[2]} background="#ffffff">
          <AbsoluteFill style={{ background: "#ff3030", color: "#fff", fontSize: 80, padding: 20 }}>
            hello
            <div style={{ position: "absolute", right: 40, bottom: 40, width: 160, height: 120, background: "#2040ff", borderRadius: 16 }} />
          </AbsoluteFill>
        </Screen>
      </>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="bridgescreen" component={Shot} width={1280} height={720} fps={30} durationInFrames={60} />);
