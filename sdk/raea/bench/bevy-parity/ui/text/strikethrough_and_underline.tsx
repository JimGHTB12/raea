// Bevy v0.20.0 examples/ui/text/strikethrough_and_underline.rs in raea's TSX (docs/control.feature FC0a): a struck
// "struck\nstruck" in FiraSans-Bold at 67 px on black, absolute 5 px from the bottom right, and a full-window column
// of Texts and TextSpans with Strikethrough, Underline, their colours and TextBackgroundColor (a Text's own lines and
// background are its string's, not its spans'). Text in Bevy's default font at 20 px unless sized. Bevy's Camera2d over
// the default clear colour is the layered <Canvas3D>. In Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Node, Text, TextSpan, percent, px } from "raea/ui";
import { Srgba } from "raea/color";

const BOLD = staticFile("fonts/FiraSans-Bold.ttf");
// bevy::color::palettes::css (basic) RED, GREEN, YELLOW, NAVY; Color::BLACK and WHITE
const RED = new Srgba(1, 0, 0);
const GREEN = new Srgba(0, 0.5019608, 0);
const YELLOW = new Srgba(1, 1, 0);
const NAVY = new Srgba(0, 0, 0.5019608);
const BLACK = new Srgba(0, 0, 0);
const WHITE = new Srgba(1, 1, 1);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Text strikethrough font={BOLD} font_size={{ Px: 67 }} justify="Center" position_type="Absolute" bottom={px(5)} right={px(5)} text_background_color={BLACK}>{"struck\nstruck"}</Text>
      <Node flex_direction="Column" width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center">
        <Text strikethrough strikethrough_color={RED} text_background_color={GREEN}>{"struck\nstruckstruck\nstruckstuckstruck"}</Text>
        <Text underline>underline</Text>
        <Text strikethrough text_background_color={GREEN}>
          struck
          <TextSpan underline>underline</TextSpan>
          <TextSpan strikethrough>struck</TextSpan>
        </Text>
        <Text strikethrough font_size={{ Px: 67 }}>struck struck</Text>
        <Text strikethrough font={BOLD} font_size={{ Px: 67 }} background_color={NAVY}>{"2struck\nstruck"}</Text>
        <Text>
          {""}
          <TextSpan strikethrough font_size={{ Px: 15 }} color={RED} text_background_color={BLACK}>struck</TextSpan>
          <TextSpan underline underline_color={YELLOW} font_size={{ Px: 30 }} color={RED} text_background_color={GREEN}>{"\nunderline"}</TextSpan>
          <TextSpan strikethrough font_size={{ Px: 50 }} color={RED} text_background_color={NAVY}>{"\nstruck"}</TextSpan>
          <TextSpan strikethrough underline font_size={{ Px: 70 }} color={RED} text_background_color={NAVY} strikethrough_color={WHITE} underline_color={WHITE}>underlined and struck</TextSpan>
        </Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
