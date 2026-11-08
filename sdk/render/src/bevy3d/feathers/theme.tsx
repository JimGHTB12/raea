// bevy_feathers' theme.rs and font_styles.rs (docs/control.feature FC0a): the theme a widget looks its colours up in
// (the UiTheme resource, by token and the surface level it sits on), and the font and text colour a widget hands down
// to its ThemedText. In Bevy both are Propagate<TextFont> and Propagate<TextColor> filtered With<ThemedText>: only a
// ThemedText takes them, so a plain <Text> inside a feathers button keeps Bevy's default font, size and colour, as
// Bevy draws it. Here they're React context, read by <ThemedText>.
import { createContext, useContext, type ReactNode } from "react";
import { Srgba } from "../colors";
import { Text, type TextProps, type TextStyleProps } from "../ui";
import type { ThemeProps, SurfaceLevel } from "./dark_theme";
import type { ThemeToken } from "./tokens";

/** palettes::basic::FUCHSIA, what a theme gives for a token it has no colour for */
const FUCHSIA = new Srgba(1, 0, 1, 1);

/** ThemeProps::default(): no colours, so every token is FUCHSIA, as Bevy draws a feathers widget with no UiTheme */
const EMPTY: ThemeProps = { semantic_base: {}, semantic_overrides: {}, token_assignments: {} };

const ThemeCx = createContext<ThemeProps>(EMPTY);
const LevelCx = createContext<SurfaceLevel>("Base");

/** Bevy's UiTheme resource: the theme every feathers widget inside looks its colours up in (create_dark_theme()).
 *  @bevy bevy_feathers::theme::UiTheme */
export function UiTheme({ theme, children }: { theme: ThemeProps; children?: ReactNode }) {
  return <ThemeCx.Provider value={theme}>{children}</ThemeCx.Provider>;
}

/** the surface level the widgets inside sit on, which picks the theme's overrides for it.
 *  @bevy bevy_feathers::theme::ThemeContext */
export function ThemeContext({ level, children }: { level: SurfaceLevel; children?: ReactNode }) {
  return <LevelCx.Provider value={level}>{children}</LevelCx.Provider>;
}

/** ThemeProps::context_color: a token's colour on a surface level, the level's override first, FUCHSIA when the theme
 *  has none */
export function context_color(theme: ThemeProps, token: ThemeToken, level: SurfaceLevel = "Base"): Srgba {
  const semantic = theme.token_assignments[token];
  if (semantic === undefined) return FUCHSIA;
  const c = theme.semantic_overrides[level]?.[semantic] ?? theme.semantic_base[semantic];
  return c ? c.to_srgba() : FUCHSIA;
}

/** a token's colour in the UiTheme and surface level around the caller */
export function useThemeColor(token: ThemeToken): Srgba {
  return context_color(useContext(ThemeCx), token, useContext(LevelCx));
}

/** the TextFont and TextColor a widget propagates to its ThemedText (InheritableFont, InheritableThemeTextColor) */
type Inherited = Pick<TextStyleProps, "font" | "font_size" | "weight" | "color">;
const InheritedCx = createContext<Inherited>({});

/** InheritableFont and InheritableThemeTextColor on a widget: the font, size, weight and colour its ThemedText take
 *  (each replacing the ThemedText's own, as Propagate inserts the component). */
export function InheritableText({ children, ...style }: Inherited & { children?: ReactNode }) {
  const outer = useContext(InheritedCx);
  return <InheritedCx.Provider value={{ ...outer, ...Object.fromEntries(Object.entries(style).filter(([, v]) => v !== undefined)) }}>{children}</InheritedCx.Provider>;
}

/** A Text with ThemedText: it takes the font and colour the feathers widget around it propagates.
 *  @bevy bevy_feathers::theme::ThemedText */
export function ThemedText(props: TextProps) {
  return <Text {...props} {...useContext(InheritedCx)} />;
}
