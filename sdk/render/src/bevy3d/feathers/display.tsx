// bevy_feathers' display (label.rs) and theme.rs's colour components (docs/control.feature FC0a). caption is a
// ThemedText, which takes the font and colour the widget around it propagates (Bevy's defaults outside one); label,
// label_dim and label_small are Text in feathers' own font and the theme's text colour, which no widget's propagation
// overrides (PropagateOver<TextFont>, and ThemeTextColor's PropagateOver<TextColor>). ThemeBackgroundColor,
// ThemeBorderColor and ThemeTextColor colour the one element inside from the UiTheme, as the components do the entity
// they're on.
import { cloneElement, isValidElement, type ReactElement } from "react";
import { Text } from "../ui";
import { fonts, size } from "./constants";
import { ThemedText, useThemeColor } from "./theme";
import { tokens, type ThemeToken } from "./tokens";

/** a ThemedText of `text`. @bevy bevy_feathers::display::caption */
export function caption(text: string): ReactElement {
  return <ThemedText>{text}</ThemedText>;
}

function Label({ text, token, font_size }: { text: string; token: ThemeToken; font_size: { Px: number } }) {
  return <Text font={fonts.REGULAR} font_size={font_size} weight={400} color={useThemeColor(token)}>{text}</Text>;
}

/** `text` in Fira Sans 14 px, TEXT_MAIN. @bevy bevy_feathers::display::label */
export function label(text: string): ReactElement {
  return <Label text={text} token={tokens.TEXT_MAIN} font_size={size.MEDIUM_FONT} />;
}

/** `text` in Fira Sans 14 px, TEXT_DIM. @bevy bevy_feathers::display::label_dim */
export function label_dim(text: string): ReactElement {
  return <Label text={text} token={tokens.TEXT_DIM} font_size={size.MEDIUM_FONT} />;
}

/** `text` in Fira Sans 11 px, TEXT_MAIN. @bevy bevy_feathers::display::label_small */
export function label_small(text: string): ReactElement {
  return <Label text={text} token={tokens.TEXT_MAIN} font_size={size.EXTRA_SMALL_FONT} />;
}

function only(who: string, children: unknown): ReactElement<Record<string, unknown>> {
  if (!isValidElement(children)) throw new TypeError(`${who}: holds one element (a <Node> or <Text>) to colour`);
  return children as ReactElement<Record<string, unknown>>;
}

/** The element inside on the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeBackgroundColor */
export function ThemeBackgroundColor({ token, children }: { token: ThemeToken; children: ReactElement }) {
  return cloneElement(only("ThemeBackgroundColor", children), { background_color: useThemeColor(token) });
}

/** The element inside bordered in the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeBorderColor */
export function ThemeBorderColor({ token, children }: { token: ThemeToken; children: ReactElement }) {
  return cloneElement(only("ThemeBorderColor", children), { border_color: useThemeColor(token) });
}

/** The Text inside in the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeTextColor */
export function ThemeTextColor({ token, children }: { token: ThemeToken; children: ReactElement }) {
  return cloneElement(only("ThemeTextColor", children), { color: useThemeColor(token) });
}
