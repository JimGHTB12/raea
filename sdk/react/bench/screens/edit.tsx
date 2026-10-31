// Free-standing flat <Screen>s, checked by check.sh: each frame's page is on its screen at its own colours (no tone
// curve), a see-through background (#ffffff00) shows the screen behind it, and both hold under DepthOfField and a
// tone curve. Frames:
//   0  one red page, "hello" in white, <ToneMapping mode="none" />
//   1  frame 0 under the camera's default tone curve (only checked for being the page, not its exact colours)
//   2  a blue page behind a see-through one carrying a white card on its left half, no tone curve
//   3  frame 2 with DepthOfField focused on the screens
import { AbsoluteFill, Composition, registerRoot, useCurrentFrame } from "raea";
import { DepthOfField, PerspectiveCamera, Screen, ToneMapping } from "raea/3d";

function Scene() {
  const f = useCurrentFrame();
  return (
    <AbsoluteFill style={{ background: "#000" }}>
      <>
        <PerspectiveCamera position={[0, 0, 4]} lookAt={[0, 0, 0]} fov={40} focus={4} />
        {f !== 1 && <ToneMapping mode="none" />}
        {f === 3 && <DepthOfField aperture={2.8} />}
        {f <= 1 ? (
          <Screen name="red" crt={false} resolution={[640, 360]} size={[2]} background="#ffffff">
            <AbsoluteFill style={{ background: "#ff3030", color: "#fff", fontSize: 80, padding: 20 }}>hello</AbsoluteFill>
          </Screen>
        ) : (
          <>
            <Screen name="back" crt={false} resolution={[640, 360]} size={[2]} position={[0, 0, -0.01]} background="#2040ff" />
            <Screen name="front" crt={false} resolution={[640, 360]} size={[2]} position={[0, 0, 0]} background="#ffffff00">
              <div style={{ position: "absolute", left: 40, top: 40, width: 240, height: 280, background: "#ffffff", borderRadius: 16 }} />
            </Screen>
          </>
        )}
      </>
    </AbsoluteFill>
  );
}
registerRoot(() => <Composition id="screens" component={Scene} width={960} height={540} fps={30} durationInFrames={4} />);
