// Bevy v0.20.0 examples/ui/widgets/virtual_keyboard.rs in raea's TSX (docs/control.feature FC0a): under feathers' dark
// UiTheme, an absolute full-width row (25 px padding, centred) holding an EditableText 80% wide with a 5 px white
// border, 5 px padding and 10 px corners, five 25 px lines high, unwrapped, with AutoFocus, so its white caret (5 px,
// cursor_width 0.2 of 25 px) shows at its start for the first half second; and, pinned to the bottom, a navy panel
// (5 px white border, 5 px padding and rows, 10 px corners, 25 px margin) of a "virtual keyboard" Text over a
// VirtualKeyboard of six rows of keys. Each key is a FeathersButton (24 px high, 8 px sides, 4 px corners, BUTTON_BG)
// growing to share its row, its caption a plain Text, which feathers' font doesn't reach (it propagates only to a
// ThemedText): Bevy's default font, 20 px white. Keys and typing work only under input. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { BorderRadius, EditableText, Node, Text, UiRect, percent, px } from "raea/ui";
import { UiTheme, VirtualKeyboard, create_dark_theme } from "raea/feathers";
import { Srgba } from "raea/color";

const WHITE = new Srgba(1, 1, 1);
const NAVY = new Srgba(0, 0, 0.5019608);

const KEYS = [
  ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", ".", ","],
  ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
  ["A", "S", "D", "F", "G", "H", "J", "K", "L", "'"],
  ["Z", "X", "C", "V", "B", "N", "M", "-", "/"],
  ["space", "enter", "backspace"],
  ["left", "right", "up", "down", "home", "end"],
];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <UiTheme theme={create_dark_theme()}>
        <Node width={percent(100)} padding={UiRect.all(px(25))} position_type="Absolute" align_items="Center" justify_content="Center">
          <EditableText width={percent(80)} border={UiRect.all(px(5))} padding={UiRect.all(px(5))} flex_grow={0} border_radius={BorderRadius.all(px(10))} border_color={WHITE} visible_lines={5} allow_newlines linebreak="NoWrap" cursor_style={{ color: WHITE }} font_size={{ Px: 25 }} auto_focus />
        </Node>
        <Node width={percent(100)} bottom={px(0)} flex_direction="Column" position_type="Absolute" align_items="Center" justify_content="End">
          <Node flex_direction="Column" border={UiRect.all(px(5))} row_gap={px(5)} padding={UiRect.all(px(5))} align_items="Center" margin={UiRect.all(px(25))} border_radius={BorderRadius.all(px(10))} background_color={NAVY} border_color={WHITE}>
            <Text>virtual keyboard</Text>
            <VirtualKeyboard keys={KEYS} />
          </Node>
        </Node>
      </UiTheme>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
