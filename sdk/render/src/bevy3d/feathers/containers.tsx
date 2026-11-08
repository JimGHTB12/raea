// bevy_feathers' containers (pane.rs, subpane.rs, group.rs, flex_spacer.rs; docs/control.feature FC0a): the editor's
// panes, sub-panes and groups, each a header over a body, coloured from the UiTheme on the surface level each sets
// (Propagate<ThemeContext>: the node itself and everything inside), and handing feathers' font and a text colour to
// the ThemedText inside. As Bevy's, each is a function taking its children; `node` patches its Node, as a bsn! patch.
import type { ReactElement, ReactNode } from "react";
import { Node, type NodeProps } from "../ui";
import { BorderRadius, UiRect, px } from "../ui-val";
import { fonts, size } from "./constants";
import { to_border_radius } from "./controls";
import type { SurfaceLevel } from "./dark_theme";
import { InheritableText, ThemeContext, useThemeColor } from "./theme";
import { tokens, type ThemeToken } from "./tokens";

const FONT = { font: fonts.REGULAR, font_size: size.MEDIUM_FONT, weight: 400 } as const;

/** a Node coloured from the theme on its own surface level */
function Themed({ level, bg, border, text, font, node, children }: { level?: SurfaceLevel; bg?: ThemeToken; border?: ThemeToken; text?: ThemeToken; font?: boolean; node: NodeProps; children?: ReactNode }) {
  const inner = <Painted bg={bg} border={border} text={text} font={font} node={node}>{children}</Painted>;
  return level ? <ThemeContext level={level}>{inner}</ThemeContext> : inner;
}

function Painted({ bg, border, text, font, node, children }: { bg?: ThemeToken; border?: ThemeToken; text?: ThemeToken; font?: boolean; node: NodeProps; children?: ReactNode }) {
  const background_color = useThemeColor(bg ?? tokens.WINDOW_BG);
  const border_color = useThemeColor(border ?? tokens.WINDOW_BG);
  const color = useThemeColor(text ?? tokens.WINDOW_BG);
  const body = text || font ? <InheritableText {...(font ? FONT : {})} color={text ? color : undefined}>{children}</InheritableText> : children;
  return (
    <Node {...(bg ? { background_color } : {})} {...(border ? { border_color } : {})} {...node}>
      {body}
    </Node>
  );
}

const TOP = (w: number) => UiRect.new(px(w), px(w), px(w), px(0));
const BOTTOM = (w: number) => UiRect.new(px(w), px(w), px(0), px(w));

/** A pane: a column of a pane_header over a pane_body. @bevy bevy_feathers::containers::pane */
export function pane(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return <Node display="Flex" flex_direction="Column" align_items="Stretch" {...node}>{children}</Node>;
}

/** A pane's header: a row of tools, 6 px padding and gaps, at least HEADER_HEIGHT, its top corners 4 px, a 1 px
 *  border on three sides, on the Base level. @bevy bevy_feathers::containers::pane_header */
export function pane_header(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed level="Base" bg={tokens.PANE_HEADER_BG} border={tokens.PANE_HEADER_BORDER} text={tokens.PANE_HEADER_TEXT} font node={{ display: "Flex", flex_direction: "Row", align_items: "Center", justify_content: "SpaceBetween", padding: UiRect.all(px(6)), border: TOP(1), min_height: size.HEADER_HEIGHT, column_gap: px(6), border_radius: to_border_radius("Top", 4), ...node }}>
      {children}
    </Themed>
  );
}

/** A 1 px rule between a pane header's tools, reaching through the header's padding.
 *  @bevy bevy_feathers::containers::pane_header_divider */
export function pane_header_divider(): ReactElement {
  return (
    <Node width={px(1)} align_self="Stretch">
      <Themed bg={tokens.PANE_HEADER_DIVIDER} node={{ position_type: "Absolute", left: px(0), right: px(0), top: px(-6), bottom: px(-6) }} />
    </Node>
  );
}

/** A pane's body: a column, 4 px apart, 6 px padding, its bottom corners 4 px, on the Higher level.
 *  @bevy bevy_feathers::containers::pane_body */
export function pane_body(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed level="Higher" bg={tokens.PANE_BODY_BG} node={{ display: "Flex", flex_direction: "Column", row_gap: px(4), padding: UiRect.all(px(6)), border_radius: to_border_radius("Bottom", 4), ...node }}>
      {children}
    </Themed>
  );
}

/** A sub-pane: a column of a subpane_header over a subpane_body. @bevy bevy_feathers::containers::subpane */
export function subpane(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return <Node display="Flex" flex_direction="Column" align_items="Stretch" {...node}>{children}</Node>;
}

/** A sub-pane's header: a row, 10 px each side, at least HEADER_HEIGHT, its top corners 4 px, a 1 px border on three
 *  sides, on the Highest level. @bevy bevy_feathers::containers::subpane_header */
export function subpane_header(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed level="Highest" bg={tokens.SUBPANE_HEADER_BG} border={tokens.SUBPANE_HEADER_BORDER} text={tokens.SUBPANE_HEADER_TEXT} font node={{ display: "Flex", flex_direction: "Row", align_items: "Center", justify_content: "SpaceBetween", border: TOP(1), padding: UiRect.horizontal(px(10)), min_height: size.HEADER_HEIGHT, column_gap: px(4), border_radius: to_border_radius("Top", 4), ...node }}>
      {children}
    </Themed>
  );
}

/** A sub-pane's body: a column, 4 px apart, 6 px padding, a 1 px border on three sides, its bottom corners 4 px, on the
 *  Higher level. @bevy bevy_feathers::containers::subpane_body */
export function subpane_body(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed level="Higher" bg={tokens.SUBPANE_BODY_BG} border={tokens.SUBPANE_BODY_BORDER} font node={{ display: "Flex", flex_direction: "Column", border: BOTTOM(1), row_gap: px(4), padding: UiRect.all(px(6)), border_radius: to_border_radius("Bottom", 4), ...node }}>
      {children}
    </Themed>
  );
}

/** A group: a bordered column (1 px, 4 px corners) of a group_header over a group_body, on the Highest level.
 *  @bevy bevy_feathers::containers::group */
export function group(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed level="Highest" bg={tokens.GROUP_BG} border={tokens.GROUP_BORDER} node={{ display: "Flex", flex_direction: "Column", align_items: "Stretch", border: UiRect.all(px(1)), border_radius: BorderRadius.all(px(4)), ...node }}>
      {children}
    </Themed>
  );
}

/** A group's header: a row, 10 px each side, at least HEADER_HEIGHT. @bevy bevy_feathers::containers::group_header */
export function group_header(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed text={tokens.GROUP_HEADER_TEXT} font node={{ display: "Flex", flex_direction: "Row", align_items: "Center", justify_content: "SpaceBetween", padding: UiRect.horizontal(px(10)), min_height: size.HEADER_HEIGHT, column_gap: px(4), ...node }}>
      {children}
    </Themed>
  );
}

/** A group's body: a column, 4 px apart, 6 px padding. @bevy bevy_feathers::containers::group_body */
export function group_body(children?: ReactNode, node: NodeProps = {}): ReactElement {
  return (
    <Themed text={tokens.GROUP_HEADER_TEXT} font node={{ display: "Flex", flex_direction: "Column", row_gap: px(4), padding: UiRect.all(px(6)), ...node }}>
      {children}
    </Themed>
  );
}

/** A Node that grows to fill the row's free space. @bevy bevy_feathers::containers::flex_spacer */
export function flex_spacer(): ReactElement {
  return <Node flex_grow={1} />;
}
