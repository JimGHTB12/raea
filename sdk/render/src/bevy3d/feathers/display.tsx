// bevy_feathers' display (label.rs) and theme.rs's colour components (docs/control.feature FC0a). caption is a
// ThemedText, which takes the font and colour the widget around it propagates (Bevy's defaults outside one); label,
// label_dim and label_small are Text in feathers' own font and the theme's text colour, which no widget's propagation
// overrides (PropagateOver<TextFont>, and ThemeTextColor's PropagateOver<TextColor>). ThemeBackgroundColor,
// ThemeBorderColor and ThemeTextColor colour the one element inside from the UiTheme, as the components do the entity
// they're on.
import { cloneElement, isValidElement, useContext, type CSSProperties, type ReactElement } from "react";
import { InUi, Text, nodeCss } from "../ui";
import { LinearRgba, Srgba } from "../colors";
import { over, px } from "../ui-val";
import { fonts, size } from "./constants";
import type { FeathersIcon } from "./icons";
import { ThemedText, useInheritedColor, useThemeColor } from "./theme";
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

export interface ThemeBackgroundColorProps {
  /** the theme colour's token */
  token: ThemeToken;
  /** the one element it colours */
  children: ReactElement;
}

/** The element inside on the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeBackgroundColor */
export function ThemeBackgroundColor({ token, children }: ThemeBackgroundColorProps) {
  return cloneElement(only("ThemeBackgroundColor", children), { background_color: useThemeColor(token) });
}

export interface ThemeBorderColorProps {
  /** the theme colour's token */
  token: ThemeToken;
  /** the one element it borders */
  children: ReactElement;
}

/** The element inside bordered in the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeBorderColor */
export function ThemeBorderColor({ token, children }: ThemeBorderColorProps) {
  return cloneElement(only("ThemeBorderColor", children), { border_color: useThemeColor(token) });
}

export interface ThemeTextColorProps {
  /** the theme colour's token */
  token: ThemeToken;
  /** the Text it colours */
  children: ReactElement;
}

/** The Text inside in the theme's colour for `token`. @bevy bevy_feathers::theme::ThemeTextColor */
export function ThemeTextColor({ token, children }: ThemeTextColorProps) {
  return cloneElement(only("ThemeTextColor", children), { color: useThemeColor(token) });
}

/** an icon 14 px high (its width by its aspect), its white shape multiplied by `tint`: the picture over the tint, its
 *  own alpha the mask, as bevy_ui multiplies an ImageNode's pixels by its color */
function Icon({ icon: i, tint }: { icon: FeathersIcon; tint: Srgba }) {
  if (!i || typeof i.image !== "string" || !(i.width > 0 && i.height > 0)) throw new TypeError(`icon: takes one of feathers' icons (icons.CHEVRON_DOWN), got ${JSON.stringify(i)}`);
  const ui = useContext(InUi);
  const n = nodeCss("icon", { width: px((14 * i.width) / i.height), height: px(14) }, ui?.back);
  const url = `url("${i.image}")`;
  const c = over("icon", "color", new Srgba(tint.red, tint.green, tint.blue, 1), undefined).css;
  const look: CSSProperties = { backgroundImage: `${url}, linear-gradient(${c}, ${c})`, backgroundSize: "100% 100%", backgroundBlendMode: "multiply", maskImage: url, maskSize: "100% 100%" };
  if (tint.alpha < 1) look.opacity = Math.max(0, tint.alpha);
  return <div style={{ ...n.css, ...look }} {...n.attrs} />;
}

function ThemedIcon({ icon: i }: { icon: FeathersIcon }) {
  const c = useInheritedColor();
  const tint = c === undefined ? new Srgba(1, 1, 1, 1) : c instanceof Srgba ? c : c instanceof LinearRgba ? c.to_srgba() : undefined;
  if (tint === undefined) throw new TypeError(`icon: the text colour around it is an Srgba or LinearRgba, got ${JSON.stringify(c)}`);
  return <Icon icon={i} tint={tint} />;
}

/** One of feathers' icons, 14 px high, tinted by the text colour the widget around it propagates (white outside one),
 *  as a ThemedIcon. @bevy bevy_feathers::display::icon */
export function icon(i: FeathersIcon): ReactElement {
  return <ThemedIcon icon={i} />;
}

/** One of feathers' icons, 14 px high, untinted. @bevy bevy_feathers::display::icon_untinted */
export function icon_untinted(i: FeathersIcon): ReactElement {
  return <Icon icon={i} tint={new Srgba(1, 1, 1, 1)} />;
}
