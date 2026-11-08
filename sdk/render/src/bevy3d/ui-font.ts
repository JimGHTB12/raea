// bevy_text's FontSource on a Text: a font file (its Handle), a family by name, or a generic family, resolved as
// Bevy's fontique resolves it (not as a page's CSS generic, which is Chrome's).
import { native } from "@raea/app/native";
import { BEVY_FONT_FAMILY, bevyFontFace } from "./fonts/default";

/** bevy_text's GenericFontFamily. @bevy bevy_text::GenericFontFamily */
export type GenericFontFamily = "Serif" | "SansSerif" | "Cursive" | "Fantasy" | "Monospace" | "SystemUi" | "UiSerif" | "UiSansSerif" | "UiMonospace" | "UiRounded" | "Emoji";
const GENERIC: Record<GenericFontFamily, string> = {
  Serif: "serif", SansSerif: "sans-serif", Cursive: "cursive", Fantasy: "fantasy", Monospace: "monospace", SystemUi: "system-ui",
  UiSerif: "ui-serif", UiSansSerif: "ui-sans-serif", UiMonospace: "ui-monospace", UiRounded: "ui-rounded", Emoji: "emoji",
};

/** bevy_text's FontSource: a font file (a string, staticFile("fonts/x.ttf")), { Family } by name, or { Generic }.
 *  @bevy bevy_text::FontSource */
export type FontSource = string | { Family: string } | { Generic: GenericFontFamily };
const generic = (g: GenericFontFamily) => () => ({ Generic: g }) as FontSource;
export const FontSource = {
  family: (name: string): FontSource => ({ Family: name }),
  serif: generic("Serif"), sans_serif: generic("SansSerif"), cursive: generic("Cursive"), fantasy: generic("Fantasy"), monospace: generic("Monospace"),
  system_ui: generic("SystemUi"), ui_serif: generic("UiSerif"), ui_sans_serif: generic("UiSansSerif"), ui_monospace: generic("UiMonospace"), ui_rounded: generic("UiRounded"), emoji: generic("Emoji"),
};

const familyOf = (font: string) => `raea-font-${font.replace(/[^A-Za-z0-9]/g, "-")}`;

/** A FontSource as CSS's font-family, and the @font-face a file needs (Bevy's default font when left out). */
export function fontCss(who: string, font: FontSource | undefined): { family: string; face: string } {
  if (font === undefined) return { family: `"${BEVY_FONT_FAMILY}"`, face: bevyFontFace() };
  if (typeof font === "string" && font !== "") {
    const f = familyOf(font);
    return { family: `"${f}"`, face: `@font-face { font-family: "${f}"; src: url("${font}"); font-weight: 1 1000; }` };
  }
  // (a face without a character falls to the script's fallback, as bevy_text's does, not to CSS's serif: blitz takes
  // "raea-bevy fallback" to say so)
  if (typeof font === "object" && font && "Family" in font && typeof font.Family === "string" && font.Family !== "") return { family: `${JSON.stringify(font.Family)}, "raea-bevy fallback"`, face: "" };
  // (fontique's own family for it, as Bevy resolves it, which raea registers as "raea-bevy <generic>" (raea-sys-fonts
  // bevy_generics.rs); a page's own CSS generic is Chrome's. Emoji is the page's own.)
  if (typeof font === "object" && font && "Generic" in font && GENERIC[font.Generic]) {
    const css = GENERIC[font.Generic];
    return { family: font.Generic === "Emoji" ? css : `"raea-bevy ${css}", ${css}`, face: "" };
  }
  throw new TypeError(`${who}: font is a font file (staticFile("fonts/x.ttf")), FontSource.family(name) or a generic one (FontSource.serif()), got ${JSON.stringify(font)}`);
}

/** bevy_text's FontCx, the font collection Bevy's text resolves families in: `family_names()` lists the system's
 *  families (natively, by fontique as Bevy's collection has them), sorted and each once.
 *  @bevy bevy_text::FontCx */
export const FontCx = {
  family_names: (): string[] => native("fonts.families", {}),
};

/** One page of bevy_text's glyph atlas: its texture (a PNG data: URL, which an ImageNode shows) and its size.
 *  @bevy bevy_text::FontAtlas */
export interface FontAtlas {
  texture: string;
  width: number;
  height: number;
}

/** bevy_text's FontAtlasSet, the glyph atlases Bevy's text is drawn from, for one face at one size:
 *  `atlases(font, font_size, text)` is the pages a Text of `text` in the font file `font` at `font_size` px fills,
 *  each glyph rasterized and packed by bevy_text's own code (natively) in the order its layout meets them. A text that
 *  grows a character at a time fills them as its last version does.
 *  @bevy bevy_text::FontAtlasSet */
export const FontAtlasSet = {
  atlases: (font: string, font_size: number, text: string): FontAtlas[] =>
    native("fonts.atlases", { font, size: font_size, text }).map((a) => ({ texture: a.src, width: a.width, height: a.height })),
};
