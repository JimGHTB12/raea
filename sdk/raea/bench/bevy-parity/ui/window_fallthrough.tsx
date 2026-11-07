// Bevy v0.20.0 examples/ui/window_fallthrough.rs in raea's TSX (docs/control.feature FC0a): a transparent window
// (ClearColor(Color::NONE)) holding "Hit 'P' then scroll/click around!" in FiraSans-Bold 83 px, absolute 5 px from
// the bottom and 10 px from the right. Its pass-through toggle sees no key here. Bevy's Camera2d over that clear
// colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Text, px } from "raea/ui";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgba(0, 0, 0, 0)" />
      <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 83 }} position_type="Absolute" bottom={px(5)} right={px(10)}>Hit 'P' then scroll/click around!</Text>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
