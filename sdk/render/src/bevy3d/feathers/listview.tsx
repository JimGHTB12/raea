// bevy_feathers' list view (listview.rs, scrollbar.rs; docs/control.feature FC0a): a column of FeathersListRows in a
// vertical scroll area beside a 6 px scrollbar track in a 14 px gutter. In Bevy 0.20 the list view hands the scrollbar
// its scroll area as #inner through FeathersScrollbarProps, which resolves to no entity; so update_scrollbar_thumb never
// sizes the thumb (Bevy draws none) and update_scrollbar_visibility never hides the track or drops the gutter, whether the
// rows overflow or not. raea draws the same: the track, no thumb, the gutter kept. Selection, hover and scrolling need
// input.
import type { ReactNode } from "react";
import { Node, type NodeProps } from "../ui";
import { Scrollbar, ScrollbarThumb, type ControlOrientation } from "../ui-scrollbar";
import { BorderRadius, Overflow, UiRect, px } from "../ui-val";
import { fonts, size } from "./constants";
import { InheritableText, useThemeColor } from "./theme";
import { tokens } from "./tokens";

const GUTTER = 14;

export interface FeathersScrollbarProps extends NodeProps {
  /** the Name of the Node it scrolls */
  target: string;
  /** which way it scrolls: Vertical or Horizontal */
  orientation?: ControlOrientation;
  /** hidden: the auto_hide decision, made by the list view (Bevy makes it after layout) */
  hidden?: boolean;
}

/** A thin scrollbar: a track on SCROLLBAR_BG with 3 px corners, its thumb on SCROLLBAR_THUMB, at least 8 px long.
 *  @bevy bevy_feathers::controls::FeathersScrollbar */
export function FeathersScrollbar({ target, orientation = "Vertical", hidden = false, ...node }: FeathersScrollbarProps) {
  return (
    <Scrollbar target={target} orientation={orientation} min_thumb_length={8} border_radius={BorderRadius.all(px(3))} background_color={useThemeColor(tokens.SCROLLBAR_BG)} visibility={hidden ? "Hidden" : "Inherited"} {...node}>
      <ScrollbarThumb border_radius={BorderRadius.all(px(3))} background_color={useThemeColor(tokens.SCROLLBAR_THUMB)} />
    </Scrollbar>
  );
}

export interface FeathersListViewProps extends NodeProps {
  /** its FeathersListRows */
  children?: ReactNode;
}

/** A list box: its rows in a scroll area, a scrollbar track beside them.
 *  @bevy bevy_feathers::controls::FeathersListView */
export function FeathersListView({ children, ...node }: FeathersListViewProps) {
  const track = useThemeColor(tokens.SCROLLBAR_BG);
  return (
    <Node display="Flex" flex_direction="Column" align_items="Stretch" justify_content="Start" padding={UiRect.right(px(GUTTER))} {...node}>
      <Node display="Flex" flex_direction="Column" align_items="Stretch" justify_content="Start" overflow={Overflow.scroll_y()}>
        {children}
      </Node>
      <Node position_type="Absolute" right={px(4)} top={px(0)} bottom={px(0)} width={px(6)} border_radius={BorderRadius.all(px(3))} background_color={track} />
    </Node>
  );
}

export interface FeathersListRowProps extends NodeProps {
  /** Selected */
  selected?: boolean;
  /** InteractionDisabled */
  disabled?: boolean;
  /** its caption */
  children?: ReactNode;
}

/** A list row: at least ROW_HEIGHT, 8 px each side and 2 px above and below, on LISTROW_BG_SELECTED when selected, its
 *  caption in Fira Sans 14 px. @bevy bevy_feathers::controls::FeathersListRow */
export function FeathersListRow({ selected = false, disabled = false, children, ...node }: FeathersListRowProps) {
  const bg = useThemeColor(!disabled && selected ? tokens.LISTROW_BG_SELECTED : tokens.LISTROW_BG);
  const text = useThemeColor(disabled ? tokens.LISTROW_TEXT_DISABLED : tokens.LISTROW_TEXT);
  return (
    <Node min_height={size.ROW_HEIGHT} min_width={size.ROW_HEIGHT} display="Flex" flex_direction="Row" justify_content="Start" align_items="Center" padding={UiRect.axes(px(8), px(2))} background_color={bg} {...node}>
      <InheritableText font={fonts.REGULAR} font_size={{ Px: 14 }} weight={400} color={text}>{children}</InheritableText>
    </Node>
  );
}
