// Bevy v0.20.0 examples/ui/widgets/button.rs in raea's TSX (docs/control.feature FC0a): a full-window node centring
// one button, 150 x 65 with a 5 px black border, fully rounded (BorderRadius::MAX), in 0.15 grey, holding "Button"
// in FiraSans-Bold 33 px, 0.9 grey, with TextShadow's default. Its hover and press styles see no pointer here.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, Node, Text, TextShadow, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="Center">
        <Node width={px(150)} height={px(65)} border={UiRect.all(px(5))} justify_content="Center" align_items="Center" border_radius={BorderRadius.MAX} border_color={new Srgba(0, 0, 0)} background_color={new Srgba(0.15, 0.15, 0.15)}>
          <Text font={FONT} font_size={{ Px: 33 }} color={new Srgba(0.9, 0.9, 0.9)} shadow={TextShadow.default()}>Button</Text>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
