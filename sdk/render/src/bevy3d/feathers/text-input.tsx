// bevy_feathers' text input (text_input.rs; docs/control.feature FC0a): a FeathersTextInputContainer, the rounded field
// on TEXT_INPUT_BG handing feathers' compact font and TEXT_INPUT_TEXT to what's inside, holding a FeathersTextInput, an
// EditableText in that font. Typing, the caret's focus and the focus ring need input.
import type { ReactNode } from "react";
import { Node, type NodeProps } from "../ui";
import { EditableText, type EditableTextProps } from "../ui-edit";
import { BorderRadius, UiRect, px } from "../ui-val";
import { fonts, size } from "./constants";
import { InheritableText, useInheritedColor, useThemeColor } from "./theme";
import { tokens } from "./tokens";

export interface FeathersTextInputContainerProps extends NodeProps {
  /** its FeathersTextInput, and anything beside it */
  children?: ReactNode;
}

/** A text field's frame: ROW_HEIGHT high, 3 px each side, 4 px corners, growing to fill its row.
 *  @bevy bevy_feathers::controls::FeathersTextInputContainer */
export function FeathersTextInputContainer({ children, ...node }: FeathersTextInputContainerProps) {
  const bg = useThemeColor(tokens.TEXT_INPUT_BG);
  const text = useThemeColor(tokens.TEXT_INPUT_TEXT);
  return (
    <Node height={size.ROW_HEIGHT} display="Flex" justify_content="Center" align_items="Center" padding={UiRect.new(px(3), px(3), px(0), px(0))} flex_grow={1} border_radius={BorderRadius.all(px(4))} column_gap={px(4)} background_color={bg} {...node}>
      <InheritableText font={fonts.REGULAR} font_size={size.COMPACT_FONT} weight={400} color={text}>{children}</InheritableText>
    </Node>
  );
}

export interface FeathersTextInputProps extends Omit<EditableTextProps, "color"> {
  /** visible_width: its width in "0"s; it grows to fill the field when left out */
  visible_width?: number;
  /** max_characters: how many characters typing may leave in it */
  max_characters?: number;
}

/** A one-line text input in feathers' compact font, its colour the field's. @bevy bevy_feathers::controls::FeathersTextInput */
export function FeathersTextInput({ visible_width, max_characters, children = "", font = fonts.REGULAR, font_size = size.COMPACT_FONT, weight = 400, ...node }: FeathersTextInputProps) {
  if (max_characters !== undefined && !(Number.isInteger(max_characters) && max_characters > 0)) throw new RangeError(`FeathersTextInput: max_characters is a whole number above 0, got ${max_characters}`);
  const inherited = useInheritedColor();
  const own = useThemeColor(tokens.TEXT_INPUT_TEXT);
  const color = inherited ?? own;
  return (
    <EditableText flex_grow={visible_width === undefined ? 1 : 0} visible_width={visible_width ?? null} cursor_width={0.3} linebreak="NoWrap" font={font} font_size={font_size} weight={weight} color={color} {...node}>
      {children}
    </EditableText>
  );
}
