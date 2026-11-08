// bevy_feathers' menus (menu.rs; docs/control.feature FC0a): a FeathersMenu holds a menu button and its popup. At rest
// the popup is spawned hidden (Visibility::Hidden, absolute, so it takes no room); a FeathersLazyMenu spawns its popup
// only on opening, so it holds the button alone. Opening, placing the popup (Popover) and the items' hover need input.
import type { ReactElement, ReactNode } from "react";
import { Node, type NodeProps } from "../ui";
import { BoxShadow } from "../ui-transform";
import { Srgba } from "../colors";
import { UiRect, px } from "../ui-val";
import { fonts, size } from "./constants";
import { FeathersButton, FeathersToolButton, to_border_radius, type RoundedCorners } from "./controls";
import { icon } from "./display";
import { icons } from "./icons";
import { InheritableText, ThemeContext, useThemeColor } from "./theme";
import { tokens } from "./tokens";

/** A menu: a ROW_HEIGHT node holding its button and popup. @bevy bevy_feathers::controls::FeathersMenu */
export function FeathersMenu({ children, ...node }: NodeProps) {
  return <Node height={size.ROW_HEIGHT} justify_content="Stretch" align_items="Stretch" {...node}>{children}</Node>;
}

/** A menu whose popup is made when it opens (`popup`, here unused at rest): its button alone.
 *  @bevy bevy_feathers::controls::FeathersLazyMenu */
export function FeathersLazyMenu({ popup, children, ...node }: NodeProps & { popup?: () => ReactNode }) {
  if (popup !== undefined && typeof popup !== "function") throw new TypeError(`FeathersLazyMenu: popup is a function making the popup, got ${JSON.stringify(popup)}`);
  return <FeathersMenu {...node}>{children}</FeathersMenu>;
}

export interface FeathersMenuButtonProps extends NodeProps {
  corners?: RoundedCorners;
  /** a chevron after the caption (true by default) */
  arrow?: boolean;
  /** the caption */
  children?: ReactNode;
}

/** A menu's button: a Normal FeathersButton, its caption, then a spacer and a down chevron.
 *  @bevy bevy_feathers::controls::FeathersMenuButton */
export function FeathersMenuButton({ corners, arrow = true, children, ...node }: FeathersMenuButtonProps) {
  return (
    <FeathersButton variant="Normal" corners={corners} {...node}>
      {children}
      {arrow ? <><Node flex_grow={1} />{icon(icons.CHEVRON_DOWN)}</> : null}
    </FeathersButton>
  );
}

/** A menu's tool button: a FeathersToolButton, its caption, 2 px, then a down chevron.
 *  @bevy bevy_feathers::controls::FeathersMenuToolButton */
export function FeathersMenuToolButton({ corners, arrow = true, children, ...node }: FeathersMenuButtonProps) {
  return (
    <FeathersToolButton variant="Normal" corners={corners} {...node}>
      {children}
      {arrow ? <><Node min_width={px(2)} />{icon(icons.CHEVRON_DOWN)}</> : null}
    </FeathersToolButton>
  );
}

function Popup({ children, ...node }: NodeProps) {
  return (
    <Node position_type="Absolute" display="Flex" flex_direction="Column" justify_content="Stretch" align_items="Stretch" border={UiRect.all(px(1))} padding={UiRect.axes(px(0), px(4))} border_radius={to_border_radius("All", 4)} background_color={useThemeColor(tokens.MENU_BG)} border_color={useThemeColor(tokens.MENU_BORDER)} box_shadow={BoxShadow.new(new Srgba(0, 0, 0, 0.9), px(0), px(0), px(1), px(4))} global_z_index={100} visibility="Hidden" {...node}>
      {children}
    </Node>
  );
}

/** A menu's popup: a bordered column of items on the Floating level, hidden until the menu opens.
 *  @bevy bevy_feathers::controls::FeathersMenuPopup */
export function FeathersMenuPopup(props: NodeProps) {
  return <ThemeContext level="Floating"><Popup {...props} /></ThemeContext>;
}

/** A menu item: a ROW_HEIGHT row, 8 px each side, its caption in MENUITEM_TEXT. @bevy bevy_feathers::controls::FeathersMenuItem */
export function FeathersMenuItem({ children, ...node }: NodeProps) {
  const color = useThemeColor(tokens.MENUITEM_TEXT);
  return (
    <Node height={size.ROW_HEIGHT} min_width={size.ROW_HEIGHT} justify_content="Start" align_items="Center" padding={UiRect.horizontal(px(8))} background_color={useThemeColor(tokens.MENU_BG)} {...node}>
      <InheritableText font={fonts.REGULAR} font_size={size.MEDIUM_FONT} weight={400} color={color}>{children}</InheritableText>
    </Node>
  );
}

/** A 1 px rule between menu items. @bevy bevy_feathers::controls::FeathersMenuDivider */
export function FeathersMenuDivider(): ReactElement {
  return <Node height={px(1)} justify_content="Start" align_self="Stretch" margin={UiRect.vertical(px(2))} background_color={useThemeColor(tokens.MENU_BORDER)} />;
}
