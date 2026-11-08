// bevy_text's FontFeatures (OpenType features by their four-letter tag, each on with 1 or a variant's index) as CSS's
// font-feature-settings, which Blitz gives parley as bevy_text gives it.

/** bevy_text's FontFeatureTag: the registered OpenType features' tags. @bevy bevy_text::FontFeatureTag */
export const FontFeatureTag = {
  STANDARD_LIGATURES: "liga", CONTEXTUAL_LIGATURES: "clig", DISCRETIONARY_LIGATURES: "dlig", CONTEXTUAL_ALTERNATES: "calt",
  STYLISTIC_ALTERNATES: "salt", SMALL_CAPS: "smcp", CAPS_TO_SMALL_CAPS: "c2sc", SWASH: "swsh", TITLING_ALTERNATES: "titl",
  FRACTIONS: "frac", ORDINALS: "ordn", SLASHED_ZERO: "zero", SUPERSCRIPT: "sups", SUBSCRIPT: "subs",
  OLDSTYLE_FIGURES: "onum", LINING_FIGURES: "lnum", PROPORTIONAL_FIGURES: "pnum", TABULAR_FIGURES: "tnum",
  WEIGHT: "wght", WIDTH: "wdth", SLANT: "slnt",
} as const;

/** FontFeatures as CSS's font-feature-settings, or undefined when it has none. */
export function featuresCss(who: string, f: Record<string, number> | undefined): string | undefined {
  if (f === undefined) return undefined;
  if (!f || typeof f !== "object") throw new TypeError(`${who}: font_features is { [FontFeatureTag.SMALL_CAPS]: 1 }, got ${JSON.stringify(f)}`);
  const all = Object.entries(f).map(([tag, n]) => {
    if (!/^[\x20-\x7e]{4}$/.test(tag)) throw new TypeError(`${who}: a font feature's tag is four characters ("smcp"), got ${JSON.stringify(tag)}`);
    if (!Number.isInteger(n) || n < 0) throw new TypeError(`${who}: font feature ${tag} is a whole number at least 0 (1 is on), got ${n}`);
    return `"${tag}" ${n}`;
  });
  return all.length ? all.join(", ") : undefined;
}

/** bevy_text's FontWidth: a face's width as a share of its normal one (CSS's font-stretch), for a Text's font_width.
 *  @bevy bevy_text::FontWidth */
export const FontWidth = {
  ULTRA_CONDENSED: 0.5, EXTRA_CONDENSED: 0.625, CONDENSED: 0.75, SEMI_CONDENSED: 0.875, NORMAL: 1,
  SEMI_EXPANDED: 1.125, EXPANDED: 1.25, EXTRA_EXPANDED: 1.5, ULTRA_EXPANDED: 2,
} as const;
