// CSS properties the engine's page (Blitz, through stylo) reads that csstype doesn't type yet, typed as raea's own so
// `style={{ … }}` takes them (tools/sdk-coverage counts them as typed). React's CSSProperties is csstype's Properties,
// so the augmentation reaches every style prop. Each is stylo's longhand (vendor/stylo/properties/longhands.toml) and
// its CSS spec's grammar.
import type { Globals, Property } from "csstype";

export interface RaeaCSSProperties<TLength = (string & {}) | 0> {
  /** which baseline of a box aligns it in its line: its first line's or its last's (css-inline-3) */
  baselineSource?: Globals | "auto" | "first" | "last" | undefined;
  /** a flex container's line count when it balances its items across lines (css-flexbox-2): a positive integer */
  flexLineCount?: Globals | number | undefined;
  /** a mask image's horizontal position, as background-position-x takes it (css-masking-1) */
  maskPositionX?: Property.BackgroundPositionX<TLength> | undefined;
  /** a mask image's vertical position, as background-position-y takes it (css-masking-1) */
  maskPositionY?: Property.BackgroundPositionY<TLength> | undefined;
  /** how far an underline or line-through reaches past (or short of) the text at each end: auto, or one or two
   *  lengths (css-text-decor-4) */
  textDecorationInset?: Globals | TLength | "auto" | (string & {}) | undefined;
}

declare module "csstype" {
  interface StandardLonghandProperties<TLength, TTime> extends RaeaCSSProperties<TLength> {}
}
