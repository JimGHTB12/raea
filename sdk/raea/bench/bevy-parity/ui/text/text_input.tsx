// Bevy v0.20.0 examples/ui/text/text_input.rs in raea's TSX (docs/control.feature FC0a): a column centred by auto
// margins (20 px padding, 16 px gaps) holding "Enter to submit text\nTab to switch inputs" in FiraSans-Bold 25 px, a
// row (16 px gap) of two TextInputs (EditableText 10 "0"s wide, no newlines, no wrap, 24 px, a 2 px SLATE_300 border on
// DARK_GREY) and the output: a 400 px box, 2 px SLATE_300 border, 8 px padding, its Text empty at 24 px. AutoFocus is
// on the inputs' row, not on an input, so neither input has focus and no caret is drawn. Typing, Tab and Enter need
// input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { EditableText, Node, Text, UiRect, auto, px } from "raea/ui";

// bevy_color's palettes: tailwind SLATE_300 #cbd5e1, css DARK_GREY #a9a9a9
const SLATE_300 = new Srgba(0xcb / 255, 0xd5 / 255, 0xe1 / 255);
const DARK_GREY = new Srgba(0xa9 / 255, 0xa9 / 255, 0xa9 / 255);

/** build_input_text */
function Input() {
  return (
    <EditableText border={UiRect.all(px(2))} border_color={SLATE_300} background_color={DARK_GREY} visible_width={10} linebreak="NoWrap" font_size={{ Px: 24 }} />
  );
}

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node align_items="Center" flex_direction="Column" padding={UiRect.all(px(20))} row_gap={px(16)} margin={UiRect.all(auto)}>
        <Text font={staticFile("fonts/FiraSans-Bold.ttf")} font_size={{ Px: 25 }}>{"Enter to submit text\nTab to switch inputs"}</Text>
        <Node column_gap={px(16)}>
          <Input />
          <Input />
        </Node>
        <Text width={px(400)} border={UiRect.all(px(2))} padding={UiRect.all(px(8))} border_color={SLATE_300} font_size={{ Px: 24 }} linebreak="WordOrCharacter">{""}</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
