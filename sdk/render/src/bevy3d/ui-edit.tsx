// bevy_text's EditableText and TextCursorStyle on a bevy_ui Node (bevy_ui text_input_layout.rs), drawn by the page
// (docs/control.feature FC0a). The node measures as Bevy's TextInputMeasure does: visible_lines lines of its line
// height high (rounded up), visible_width "0"s wide (CSS's ch is the advance of "0", as Bevy measures it), else the
// width it's given. Its text is drawn as a Text's, and the caret, when it has focus, is parley's cursor_geometry: a
// bar cursor_width x the font size wide from the caret's x, its line's block high, at the text's end
// (EditableText::new queues TextEnd). Bevy blinks it from the moment focus lands: shown for the first half of each
// cursor_blink_period, here from frame 0 (AutoFocus). Typing, selection and scrolling need input.
import type { CSSProperties, ReactNode } from "react";
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import { Node, Text, textCss, type NodeProps, type TextStyleProps, type TextProps } from "./ui";
import { LinearRgba, Srgba } from "./colors";
import { over } from "./ui-val";

type Color = Srgba | LinearRgba | string;

/** bevy_text's TextCursorStyle (color: SLATE_700 by default). @bevy bevy_text::TextCursorStyle */
export interface TextCursorStyle {
  color?: Color;
  selection_color?: Color;
  unfocused_selection_color?: Color;
  selected_text_color?: Color;
  selection_radius?: number;
}

/** tailwind's SLATE_700, TextCursorStyle's default caret */
const SLATE_700 = new Srgba(0.2, 0.25490196, 0.33333334, 1);

export interface EditableTextProps extends NodeProps, TextStyleProps {
  /** the text it starts with (EditableText::new) */
  children?: string;
  /** visible_lines: its height in lines (1 by default; null leaves the height to the layout) */
  visible_lines?: number | null;
  /** visible_width: its width in "0"s (None by default: the width it's given) */
  visible_width?: number | null;
  /** allow_newlines (false by default): Enter adds a line, under input */
  allow_newlines?: boolean;
  /** cursor_width: the caret's width as a share of the font size (0.2 by default) */
  cursor_width?: number;
  /** cursor_blink_period in seconds (1 by default) */
  cursor_blink_period?: number;
  /** TextCursorStyle */
  cursor_style?: TextCursorStyle;
  /** AutoFocus: it has the input focus, so its caret is drawn */
  auto_focus?: boolean;
  /** TextLayout's justify and linebreak */
  justify?: TextProps["justify"];
  linebreak?: TextProps["linebreak"];
}

/** A text input's text, its caret drawn while it has focus. @bevy bevy_text::EditableText */
export function EditableText({ children = "", visible_lines = 1, visible_width = null, allow_newlines = false, cursor_width = 0.2, cursor_blink_period = 1, cursor_style, auto_focus = false, justify, linebreak, font, font_size, weight, style, font_width, color, line_height, font_variations, font_features, letter_spacing, ...node }: EditableTextProps) {
  if (typeof children !== "string") throw new TypeError(`EditableText: its text is one string, got ${JSON.stringify(children)}`);
  if (visible_lines !== null && !(visible_lines > 0 && Number.isFinite(visible_lines))) throw new RangeError(`EditableText: visible_lines is above 0 or null, got ${visible_lines}`);
  if (visible_width !== null && !(visible_width > 0 && Number.isFinite(visible_width))) throw new RangeError(`EditableText: visible_width is above 0 or null, got ${visible_width}`);
  if (!(cursor_width >= 0 && Number.isFinite(cursor_width))) throw new RangeError(`EditableText: cursor_width is 0 or more, got ${cursor_width}`);
  if (!(cursor_blink_period > 0 && Number.isFinite(cursor_blink_period))) throw new RangeError(`EditableText: cursor_blink_period is above 0 (seconds), got ${cursor_blink_period}`);
  void allow_newlines;
  const textStyle: TextStyleProps = { font, font_size, weight, style, font_width, color, line_height, font_variations, font_features, letter_spacing };
  const { css } = textCss("EditableText", textStyle);
  const sizePx = parseFloat(String(css.fontSize));
  const lh = css.lineHeight;
  const linePx = typeof lh === "number" ? lh * sizePx : parseFloat(String(lh));
  const exact = String(css.fontSize).endsWith("px") && Number.isFinite(linePx);
  // TextInputMeasure: lines x the line height, rounded up (Vec2::ceil)
  const line = exact ? `${linePx}px` : typeof lh === "number" ? `${lh}em` : String(lh);
  // the measured box: Bevy's content size (lines x the line height, rounded up; visible_width "0"s, which CSS's ch is
  // in the box's own font), else the width the node gives it (flex 1 from 0: Bevy's measure takes the available width)
  const box: CSSProperties = {
    fontFamily: css.fontFamily, fontSize: css.fontSize, fontWeight: css.fontWeight, fontStyle: css.fontStyle, fontStretch: css.fontStretch,
    overflow: "hidden", minWidth: 0, ...(visible_width === null ? { flex: "1 1 0px" } : { width: `${visible_width}ch`, flex: "0 0 auto" }),
  };
  if (visible_lines !== null) box.height = exact ? `${Math.ceil(visible_lines * linePx)}px` : `calc(${visible_lines} * ${line})`;
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const shown = auto_focus && (frame / fps) % cursor_blink_period < cursor_blink_period / 2;
  const caret = shown ? <Caret width={cursor_width * sizePx} height={line} color={cursor_style?.color ?? SLATE_700} /> : null;
  return (
    <Node {...node}>
      <div style={box}>
        <Text {...textStyle} justify={justify} linebreak={linebreak}>
          {children}
          {caret}
        </Text>
      </div>
    </Node>
  );
}

/** the caret: a bar at the caret's x, its line's block high, taking no room in the line */
function Caret({ width, height, color }: { width: number; height: string; color: Color }): ReactNode {
  const background = over("EditableText", "cursor_style.color", color, undefined).css;
  return <span data-raea-caret="" style={{ display: "inline-block", width: `${width}px`, height, marginRight: `${-width}px`, verticalAlign: "top", background }} />;
}
