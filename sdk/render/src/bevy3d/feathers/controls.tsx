// bevy_feathers' controls (docs/control.feature FC0a): button.rs's FeathersButton and virtual_keyboard.rs's
// VirtualKeyboard, as Bevy spawns them, coloured from the UiTheme around them. update_button_styles picks a button's
// background by its variant and state; a render has no pointer, so a button is hovered or pressed only under input,
// and disabled when InteractionDisabled is given. FocusIndicator's ring is drawn only on the focused entity, and no
// button here takes focus.
import type { ReactNode } from "react";
import { Node, Text, type NodeProps } from "../ui";
import { BorderRadius, UiRect, px } from "../ui-val";
import { fonts, size } from "./constants";
import { InheritableText, useThemeColor } from "./theme";
import { tokens } from "./tokens";

/** a button's look. @bevy bevy_feathers::controls::ButtonVariant */
export type ButtonVariant = "Normal" | "Primary" | "Plain";

/** which corners are rounded. @bevy bevy_feathers::rounded_corners::RoundedCorners */
export type RoundedCorners = "None" | "All" | "TopLeft" | "TopRight" | "BottomRight" | "BottomLeft" | "Top" | "Right" | "Bottom" | "Left";

/** RoundedCorners::to_border_radius: `radius` px on the corners named, 0 on the rest */
export function to_border_radius(corners: RoundedCorners, radius: number): BorderRadius {
  const r = px(radius), z = px(0);
  const on = (tl: boolean, tr: boolean, br: boolean, bl: boolean) => BorderRadius.new(tl ? r : z, tr ? r : z, br ? r : z, bl ? r : z);
  const table: Record<RoundedCorners, BorderRadius> = {
    None: on(false, false, false, false), All: on(true, true, true, true),
    TopLeft: on(true, false, false, false), TopRight: on(false, true, false, false), BottomRight: on(false, false, true, false), BottomLeft: on(false, false, false, true),
    Top: on(true, true, false, false), Right: on(false, true, true, false), Bottom: on(false, false, true, true), Left: on(true, false, false, true),
  };
  if (!(corners in table)) throw new TypeError(`FeathersButton: corners is a RoundedCorners (All, Top, Left, ...), got ${JSON.stringify(corners)}`);
  return table[corners];
}

const BG = {
  Normal: [tokens.BUTTON_BG, tokens.BUTTON_BG_DISABLED],
  Primary: [tokens.BUTTON_PRIMARY_BG, tokens.BUTTON_PRIMARY_BG_DISABLED],
  Plain: [tokens.BUTTON_PLAIN_BG, tokens.BUTTON_PLAIN_BG_DISABLED],
} as const;
const TEXT = {
  Normal: [tokens.BUTTON_TEXT, tokens.BUTTON_TEXT_DISABLED],
  Primary: [tokens.BUTTON_PRIMARY_TEXT, tokens.BUTTON_PRIMARY_TEXT_DISABLED],
  Plain: [tokens.BUTTON_TEXT, tokens.BUTTON_TEXT_DISABLED],
} as const;

export interface FeathersButtonProps extends NodeProps {
  /** ButtonVariant (Normal by default) */
  variant?: ButtonVariant;
  /** RoundedCorners (All by default), each 4 px */
  corners?: RoundedCorners;
  /** InteractionDisabled */
  disabled?: boolean;
  /** the caption: a <ThemedText> takes the button's font (Fira Sans 14 px) and text colour, a plain <Text> keeps Bevy's */
  children?: ReactNode;
}

/** A feathers button: a ROW_HEIGHT node, 8 px each side, its corners 4 px, on the theme's button colour; its own Node
 *  fields (flex_grow, a min_width) patch the button's, as a bsn! patch does. @bevy bevy_feathers::controls::FeathersButton */
export function FeathersButton({ variant = "Normal", corners = "All", disabled = false, children, ...node }: FeathersButtonProps) {
  if (!(variant in BG)) throw new TypeError(`FeathersButton: variant is Normal, Primary or Plain, got ${JSON.stringify(variant)}`);
  const background = useThemeColor(BG[variant][disabled ? 1 : 0]);
  const color = useThemeColor(TEXT[variant][disabled ? 1 : 0]);
  return (
    <Node height={size.ROW_HEIGHT} justify_content="Center" align_items="Center" padding={UiRect.horizontal(px(8))} border_radius={to_border_radius(corners, 4)} background_color={background} {...node}>
      <InheritableText font={fonts.REGULAR} font_size={size.MEDIUM_FONT} weight={400} color={color}>
        {children}
      </InheritableText>
    </Node>
  );
}

export interface VirtualKeyboardProps {
  /** the keyboard's rows, each a row of key captions */
  keys: string[][];
}

/** A grid of FeathersButtons, one row of keys a row (4 px apart both ways), each key's caption a plain Text (Bevy's
 *  default font, 20 px white: feathers' font reaches only a ThemedText), each key growing to share its row.
 *  VirtualKeyPressed fires on a key under input. @bevy bevy_feathers::controls::VirtualKeyboard */
export function VirtualKeyboard({ keys }: VirtualKeyboardProps) {
  if (!Array.isArray(keys) || !keys.every((r) => Array.isArray(r) && r.every((k) => typeof k === "string"))) throw new TypeError(`VirtualKeyboard: keys is rows of strings ([["Q", "W"], ["space"]]), got ${JSON.stringify(keys)}`);
  return (
    <Node flex_direction="Column" row_gap={px(4)}>
      {keys.map((row, i) => (
        <Node key={i} flex_direction="Row" column_gap={px(4)}>
          {row.map((key, j) => (
            <FeathersButton key={j} flex_grow={1}>
              <Text>{key}</Text>
            </FeathersButton>
          ))}
        </Node>
      ))}
    </Node>
  );
}

/** A FeathersButton for a toolbar: 4 px each side and at least ROW_HEIGHT wide. @bevy bevy_feathers::controls::FeathersToolButton */
export function FeathersToolButton({ children, ...props }: FeathersButtonProps) {
  return <FeathersButton padding={UiRect.horizontal(px(4))} min_width={size.ROW_HEIGHT} {...props}>{children}</FeathersButton>;
}
