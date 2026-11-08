// Bevy v0.20.0 examples/ui/text/multiline_text_input.rs in raea's TSX (docs/control.feature FC0a): centred in the window,
// a column (aligned to its end, 10 px gaps) of: a grid of the multiline TextInput (450 px wide, a 2 px SLATE_300 border,
// 8 px padding on DARK_SLATE_GRAY, EditableText empty, 8 visible lines, newlines allowed, word-or-character wrapping,
// FiraMono-Medium 30 px, a white caret) beside its 10 px DARK_SLATE_GRAY scrollbar, its thumb hidden while nothing
// overflows; three rows of a 30 px label and a 100 px input justified to its end ("visible lines:" 8, "font size:" 30,
// "corner radius:" 0, each with its EditableTextFilter); and the closed menu's button, "Justify::Left" at 24 px, a 2 px
// SLATE_300 border and 8 px padding across. The multiline input has AutoFocus, so its caret is drawn at its start, in
// the first half of its 1 s blink at frame 10. Typing, scrolling, the settings and the menu need input. Bevy's Camera2d
// over the default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { EditableText, Node, RepeatedGridTrack, Text, UiRect, percent, px } from "raea/ui";

const FONT = staticFile("fonts/FiraMono-Medium.ttf");
const WHITE = new Srgba(1, 1, 1);
const BLACK = new Srgba(0, 0, 0);
// bevy_color's palettes: tailwind SLATE_300 #cbd5e1, css DARK_SLATE_GRAY #2f4f4f
const SLATE_300 = new Srgba(0xcb / 255, 0xd5 / 255, 0xe1 / 255);
const DARK_SLATE_GRAY = new Srgba(0x2f / 255, 0x4f / 255, 0x4f / 255);

/** a label and its 100 px setting */
function Setting({ name, value, filter }: { name: string; value: string; filter: (c: string) => boolean }) {
  return (
    <Node flex_direction="Row" column_gap={px(10)}>
      <Text font={FONT} font_size={{ Px: 30 }}>{name}</Text>
      <EditableText width={px(100)} border={UiRect.all(px(2))} background_color={DARK_SLATE_GRAY} border_color={SLATE_300} font={FONT} font_size={{ Px: 30 }} justify="End"
        filter={filter} cursor_style={{ color: WHITE, selected_text_color: BLACK }}>{value}</EditableText>
    </Node>
  );
}

const decimal = (c: string) => /^[0-9.]$/.test(c);

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} justify_content="Center" align_items="Center">
        <Node flex_direction="Column" align_items="End" row_gap={px(10)}>
          <Node display="Grid" grid_template_columns={[RepeatedGridTrack.auto(1), RepeatedGridTrack.auto(1)]} column_gap={px(4)}>
            <EditableText width={px(450)} border={UiRect.all(px(2))} padding={UiRect.all(px(8))} background_color={DARK_SLATE_GRAY} border_color={SLATE_300}
              visible_lines={8} allow_newlines linebreak="WordOrCharacter" font={FONT} font_size={{ Px: 30 }} cursor_style={{ color: WHITE, selected_text_color: BLACK }} auto_focus />
            {/* the scrollbar; its thumb is Visibility::Hidden until the text overflows */}
            <Node min_width={px(10)} background_color={DARK_SLATE_GRAY} />
          </Node>
          <Setting name="visible lines:" value="8" filter={decimal} />
          <Setting name="font size:" value="30" filter={(c) => /^[0-9]$/.test(c)} />
          <Setting name="corner radius:" value="0" filter={decimal} />
          <Node>
            <Node border={UiRect.all(px(2))} padding={UiRect.horizontal(px(8))} background_color={DARK_SLATE_GRAY} border_color={SLATE_300}>
              <Text font={FONT} font_size={{ Px: 24 }}>Justify::Left</Text>
            </Node>
          </Node>
        </Node>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
