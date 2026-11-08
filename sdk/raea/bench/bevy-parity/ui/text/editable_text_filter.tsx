// Bevy v0.20.0 examples/ui/text/editable_text_filter.rs in raea's TSX (docs/control.feature FC0a): a full-window node
// centring one TextInput: 240 px wide, a 2 px SLATE_300 border, 8 px padding on DARK_SLATE_GRAY, its EditableText empty
// at 32 px, taking at most 8 characters and only hex digits (EditableTextFilter), and focused (AutoFocus), so its caret
// (TextCursorStyle's default SLATE_700) is drawn, in the first half of its 1 s blink at frame 10. Typing needs input.
// Bevy's Camera2d over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { EditableText, Node, UiRect, percent, px } from "raea/ui";

// bevy_color's palettes: tailwind SLATE_300 #cbd5e1, css DARK_SLATE_GRAY #2f4f4f
const SLATE_300 = new Srgba(0xcb / 255, 0xd5 / 255, 0xe1 / 255);
const DARK_SLATE_GRAY = new Srgba(0x2f / 255, 0x4f / 255, 0x4f / 255);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center">
        <EditableText width={px(240)} border={UiRect.all(px(2))} padding={UiRect.all(px(8))} background_color={DARK_SLATE_GRAY} border_color={SLATE_300}
          max_characters={8} filter={(c) => /^[0-9a-fA-F]$/.test(c)} font_size={{ Px: 32 }} auto_focus />
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
