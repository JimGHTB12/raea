// Bevy v0.20.0 examples/ui/text/multiple_text_inputs.rs in raea's TSX (docs/control.feature FC0a): a full-window grid
// (columns 160 and 3 x 320 px, 8 px gaps, its content centred) titled "Multiple Text Inputs Example" (FiraMono-Medium
// 24 px, across all four, 16 px below). Two sections, each headed by four 14 px labels (-4 px below): the six Justify
// modes, then the three TextReadWriteModes. Each row: its name in a 4 px white box; a TextInput (EditableText
// "Initial text N", 4 px SLATE_300 border, 4 px padding on DARK_GREY, the row's justify); the "value" box (its text from the start; 4 px white
// border, 4 px padding on DARK_SLATE_BLUE, clipped across to its content box) around a DARK_SLATE_GRAY Text; and
// the "submission" box, its Text empty. Both sections' first input has AutoFocus: the second section's is added
// last, so it holds the focus, and update_row_border_colors dims every other input's border (SLATE_300.darker(0.75)).
// Its caret sits at its text's end (EditableText::new queues TextEnd), in the first half of its 1 s blink at frame
// 10. "Press Enter to submit" closes the grid (16 px above). Typing, Enter and Tab need input. Bevy's Camera2d over the
// default clear colour is the layered <Canvas3D>, in Bevy's default 1280 x 720 window.
import { AbsoluteFill, Composition, registerRoot, staticFile } from "raea";
import { Canvas3D } from "raea/3d";
import { Srgba } from "raea/color";
import { EditableText, GridPlacement, GridTrack, Node, Overflow, OverflowClipMargin, RepeatedGridTrack, Text, UiRect, percent, px } from "raea/ui";

const FONT = staticFile("fonts/FiraMono-Medium.ttf");
const SIZE = { Px: 24 };
const WHITE = new Srgba(1, 1, 1);
// bevy_color's palettes: tailwind SLATE_300 #cbd5e1; css DARK_GREY #a9a9a9, DARK_SLATE_BLUE #483d8b, DARK_SLATE_GRAY #2f4f4f
const SLATE_300 = new Srgba(0xcb / 255, 0xd5 / 255, 0xe1 / 255);
const DARK_GREY = new Srgba(0xa9 / 255, 0xa9 / 255, 0xa9 / 255);
const DARK_SLATE_BLUE = new Srgba(0x48 / 255, 0x3d / 255, 0x8b / 255);
const DARK_SLATE_GRAY = new Srgba(0x2f / 255, 0x4f / 255, 0x4f / 255);
/** the row that has the input focus: the second section's first (its AutoFocus is added last) */
const FOCUSED = 100;

type Justify = "Left" | "Center" | "Right" | "Justified" | "Start" | "End";
type Mode = "Editable" | "ReadOnly" | "Static";

function labels(names: string[]) {
  return names.map((n) => <Text key={n} font={FONT} font_size={{ Px: 14 }} justify_self="Center" margin={UiRect.bottom(px(-4))}>{n}</Text>);
}

/** an output box: "value" shows its input's text from the start (TextEditChange is triggered as EditableText::new sets it), "submission" stays empty until Enter */
function Output({ text }: { text: string }) {
  return (
    <Node border={UiRect.all(px(4))} padding={UiRect.all(px(4))} overflow={Overflow.clip_x()} overflow_clip_margin={OverflowClipMargin.content_box()} background_color={DARK_SLATE_BLUE} border_color={WHITE}>
      <Text font={FONT} font_size={SIZE} linebreak="NoWrap" {...(text !== "" ? { background_color: DARK_SLATE_GRAY } : {})}>{text}</Text>
    </Node>
  );
}

function Row({ row, name, justify, mode }: { row: number; name: string; justify?: Justify; mode?: Mode }) {
  const border = row === FOCUSED ? SLATE_300 : SLATE_300.darker(0.75);
  return [
    <Node key={`${row}n`} border={UiRect.all(px(4))} justify_content="Center" align_items="Center" border_color={WHITE}>
      <Text font={FONT} font_size={SIZE}>{name}</Text>
    </Node>,
    <EditableText key={`${row}i`} border={UiRect.all(px(4))} padding={UiRect.all(px(4))} background_color={DARK_GREY} border_color={border} font={FONT} font_size={SIZE}
      {...(justify ? { justify, linebreak: "NoWrap" as const } : {})} {...(mode ? { read_write_mode: mode } : {})} auto_focus={row === FOCUSED}>{`Initial text ${row % 100}`}</EditableText>,
    <Output key={`${row}v`} text={`Initial text ${row % 100}`} />,
    <Output key={`${row}s`} text="" />,
  ];
}

const JUSTIFY: Justify[] = ["Left", "Center", "Right", "Justified", "Start", "End"];
const MODES: Mode[] = ["Editable", "ReadOnly", "Static"];

function Shot() {
  return (
    <AbsoluteFill>
      <Canvas3D layered clearColor="rgb(43, 44, 47)" />
      <Node width={percent(100)} height={percent(100)} display="Grid" justify_content="Center" align_content="Center" row_gap={px(8)} column_gap={px(8)}
        grid_template_columns={[GridTrack.px(160), RepeatedGridTrack.px(3, 320)]}>
        <Text font={FONT} font_size={SIZE} color={WHITE} grid_column={GridPlacement.span(4)} justify_self="Center" margin={UiRect.bottom(px(16))}>Multiple Text Inputs Example</Text>
        {labels(["Justify", "EditableText", "value", "submission"])}
        {JUSTIFY.map((j, row) => <Row key={j} row={row} name={j} justify={j} />)}
        {labels(["ReadWrite", "EditableText", "value", "submission"])}
        {MODES.map((m, row) => <Row key={m} row={row + 100} name={m} mode={m} />)}
        <Text font={FONT} font_size={SIZE} grid_column={GridPlacement.span(4)} justify_self="Center" margin={UiRect.top(px(16))}>Press Enter to submit</Text>
      </Node>
    </AbsoluteFill>
  );
}

registerRoot(() => <Composition id="main" component={Shot} durationInFrames={11} fps={60} width={1280} height={720} />);
