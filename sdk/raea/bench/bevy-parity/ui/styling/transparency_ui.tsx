// Bevy v0.20.0 examples/ui/styling/transparency_ui.rs in raea's TSX (docs/control.feature FC0a): on a black clear, a
// full-window row spacing two 150 x 65 buttons around (green srgb(0.1, 0.5, 0.1) and purple srgb(0.5, 0.1, 0.5)), each
// centring "Button N" in fonts/FiraSans-Bold.ttf at 33 px in white at alpha 0.2. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const FONT = staticFile("fonts/FiraSans-Bold.ttf");
const BUTTONS = [new Srgba(0.1, 0.5, 0.1), new Srgba(0.5, 0.1, 0.5)];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="#000000" />
      <Node width={percent(100)} height={percent(100)} align_items="Center" justify_content="SpaceAround">
        {BUTTONS.map((c, i) => (
          <Node key={i} width={px(150)} height={px(65)} justify_content="Center" align_items="Center" background_color={c}>
            <Text font={FONT} font_size={{ Px: 33 }} color={new Srgba(1, 1, 1, 0.2)}>{`Button ${i + 1}`}</Text>
          </Node>
        ))}
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
