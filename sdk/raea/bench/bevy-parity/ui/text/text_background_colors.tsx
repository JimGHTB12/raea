// Bevy v0.20.0 examples/ui/text/text_background_colors.rs in raea's TSX (docs/control.feature FC0a): a full-window
// node centring a Text with a 5 px dark cyan border and 10 px of padding, justified Center, whose 21 spans spell
// "Text\nBackground\nColors!" in black at 100 px, each on a TextBackgroundColor from the palette in turn (red, green,
// blue, yellow, purple). The palette cycles once a second (its first order until then, at frames 5 and 10); the
// hover observers have no pointer. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>.
// In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, TextSpan, UiRect, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

// bevy::color::palettes::css RED, GREEN, BLUE, YELLOW, PURPLE and DARK_CYAN
const PALETTE = [new Srgba(1, 0, 0), new Srgba(0, 0.5019608, 0), new Srgba(0, 0, 1), new Srgba(1, 1, 0), new Srgba(0.5019608, 0, 0.5019608)];
const DARK_CYAN = new Srgba(0, 0.545, 0.545);
const MESSAGE = ["T", "e", "x", "t\n", "B", "a", "c", "k", "g", "r", "o", "u", "n", "d\n", "C", "o", "l", "o", "r", "s", "!"];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center">
        <Text border={UiRect.all(px(5))} padding={UiRect.all(px(10))} justify="Center" border_color={DARK_CYAN}>
          {MESSAGE.map((s, i) => <TextSpan key={i} color={new Srgba(0, 0, 0)} font_size={{ Px: 100 }} text_background_color={PALETTE[i % PALETTE.length]}>{s}</TextSpan>)}
        </Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
