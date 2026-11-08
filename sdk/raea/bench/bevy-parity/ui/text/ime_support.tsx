// Bevy v0.20.0 examples/ui/text/ime_support.rs in raea's TSX (docs/control.feature FC0a): a column (24 px padding, 16 px
// gaps) of the instructions at 20 px, wrapping to the window; a TextInput 400 x 250 with a 3 px SLATE_300 border and 8 px
// padding on DARK_GREY, its EditableText empty in the system sans-serif at 32 px, taking newlines, not wrapping; and
// "Your text here!" in the system sans-serif at 32 px. Nothing has focus (no AutoFocus), so no caret is drawn. IME
// composition, Ctrl+Enter and Tab need input. Bevy's Camera2d over the default clear colour is the layered <Canvas3D>,
// in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { EditableText, FontSource, Node, Text, UiRect, px } from "raea/ui";

// bevy_color's palettes: tailwind SLATE_300 #cbd5e1, css DARK_GREY #a9a9a9
const SLATE_300 = new Srgba(0xcb / 255, 0xd5 / 255, 0xe1 / 255);
const DARK_GREY = new Srgba(0xa9 / 255, 0xa9 / 255, 0xa9 / 255);

const INSTRUCTIONS = "Type using your IME, then press Ctrl+Enter to submit. Your system default sans-serif font will be used, so make sure you have fonts installed that support the characters you want to input!";

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node flex_direction="Column" padding={UiRect.all(px(24))} row_gap={px(16)}>
        <Text font_size={{ Px: 20 }}>{INSTRUCTIONS}</Text>
        <EditableText width={px(400)} height={px(250)} border={UiRect.all(px(3))} padding={UiRect.all(px(8))} border_color={SLATE_300} background_color={DARK_GREY}
          font={FontSource.sans_serif()} font_size={{ Px: 32 }} allow_newlines linebreak="NoWrap" />
        <Text font={FontSource.sans_serif()} font_size={{ Px: 32 }}>Your text here!</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
