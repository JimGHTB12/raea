// bevy_feathers' constants.rs (docs/control.feature FC0a): its fonts and sizes. Its fonts are its own embedded files,
// which raea bundles whole (core/fonts: Fira Sans and Fira Mono, as bevy_feathers 0.20.0 ships them), so each is
// named by its family here: Bevy resolves a font handle to its family too, and the TextFont's weight and style pick
// the face.
import { FontSource } from "../ui-font";
import { px } from "../ui-val";

/** bevy_feathers' fonts, by family (the face by the TextFont's weight and style). @bevy bevy_feathers::constants::fonts */
export const fonts = {
  REGULAR: FontSource.family("Fira Sans"),
  ITALIC: FontSource.family("Fira Sans"),
  BOLD: FontSource.family("Fira Sans"),
  BOLD_ITALIC: FontSource.family("Fira Sans"),
  MONO: FontSource.family("Fira Mono"),
};

/** bevy_feathers' sizes. @bevy bevy_feathers::constants::size */
export const size = {
  ROW_HEIGHT: px(24),
  CHECKBOX_SIZE: px(18),
  HEADER_HEIGHT: px(30),
  RADIO_SIZE: px(18),
  TOGGLE_WIDTH: px(32),
  TOGGLE_HEIGHT: px(18),
  HEADER_FONT: { Px: 16 },
  MEDIUM_FONT: { Px: 14 },
  COMPACT_FONT: { Px: 13 },
  SMALL_FONT: { Px: 12 },
  EXTRA_SMALL_FONT: { Px: 11 },
} as const;
