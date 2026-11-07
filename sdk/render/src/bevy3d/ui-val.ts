// bevy_ui's values as the page's CSS: Val (px, percent, vw, vh, vmin, vmax, auto), UiRect, BorderRadius, and Node's
// enums. A Bevy UI tree is drawn by the page (Blitz lays it out with taffy, as bevy_ui does), so each value is the CSS
// that gives taffy the style bevy_ui's convert.rs gives it.
import { LinearRgba, Srgba } from "./colors";

/** A bevy_ui Val as CSS: px(12) is "12px", percent(50) "50%", auto "auto". */
export type Val = string;

const VAL = /^(auto|-?\d+(\.\d+)?(e[-+]?\d+)?(px|%|vw|vh|vmin|vmax))$/;
const num = (what: string, n: number) => {
  if (!Number.isFinite(n)) throw new RangeError(`${what}: a finite number, got ${n}`);
  return n;
};

/** Bevy's Val::Px: logical pixels */
export const px = (n: number): Val => `${num("px", n)}px`;
/** Bevy's Val::Percent: of the parent's size on that axis (padding and margin: its width, as CSS) */
export const percent = (n: number): Val => `${num("percent", n)}%`;
export const vw = (n: number): Val => `${num("vw", n)}vw`;
export const vh = (n: number): Val => `${num("vh", n)}vh`;
export const vmin = (n: number): Val => `${num("vmin", n)}vmin`;
export const vmax = (n: number): Val => `${num("vmax", n)}vmax`;
/** Bevy's Val::Auto */
export const auto: Val = "auto";

export function checkVal(who: string, k: string, v: unknown): Val {
  if (typeof v !== "string" || !VAL.test(v)) throw new TypeError(`${who}: ${k} is a Val (px(12), percent(50), vw(10), auto), got ${JSON.stringify(v)}`);
  return v;
}

/** Bevy's UiRect: a Val for each side (0 by default). */
export interface UiRect { left: Val; right: Val; top: Val; bottom: Val }
const Z = "0px";
export const UiRect = {
  all: (v: Val): UiRect => ({ left: v, right: v, top: v, bottom: v }),
  axes: (horizontal: Val, vertical: Val): UiRect => ({ left: horizontal, right: horizontal, top: vertical, bottom: vertical }),
  horizontal: (v: Val): UiRect => ({ left: v, right: v, top: Z, bottom: Z }),
  vertical: (v: Val): UiRect => ({ left: Z, right: Z, top: v, bottom: v }),
  new: (left: Val, right: Val, top: Val, bottom: Val): UiRect => ({ left, right, top, bottom }),
  left: (v: Val): UiRect => ({ left: v, right: Z, top: Z, bottom: Z }),
  right: (v: Val): UiRect => ({ left: Z, right: v, top: Z, bottom: Z }),
  top: (v: Val): UiRect => ({ left: Z, right: Z, top: v, bottom: Z }),
  bottom: (v: Val): UiRect => ({ left: Z, right: Z, top: Z, bottom: v }),
  /** every side auto (UiRect::AUTO: a margin that centres) */
  AUTO: { left: "auto", right: "auto", top: "auto", bottom: "auto" } as UiRect,
  ZERO: { left: Z, right: Z, top: Z, bottom: Z } as UiRect,
};

/** A UiRect as CSS's four values (top right bottom left). */
export function rectCss(who: string, k: string, r: UiRect): string {
  if (typeof r !== "object" || r === null) throw new TypeError(`${who}: ${k} is a UiRect (UiRect.all(px(4))), got ${JSON.stringify(r)}`);
  return (["top", "right", "bottom", "left"] as const).map((s) => checkVal(who, `${k}.${s}`, r[s])).join(" ");
}

/** Bevy's CornerRadius: a corner's x and y radii, an elliptical corner (a Val alone is a circular one). */
export interface CornerRadius { x: Val; y: Val }
export const CornerRadius = {
  new: (x: Val, y: Val): CornerRadius => ({ x, y }),
  all: (v: Val): CornerRadius => ({ x: v, y: v }),
  circular: (v: Val): CornerRadius => ({ x: v, y: "auto" }),
};
type Corner = Val | CornerRadius;

/** Bevy's BorderRadius: a Val (circular) or a CornerRadius (elliptical) for each corner (0 by default). */
export interface BorderRadius { top_left: Corner; top_right: Corner; bottom_right: Corner; bottom_left: Corner }
export const BorderRadius = {
  all: (v: Corner): BorderRadius => ({ top_left: v, top_right: v, bottom_right: v, bottom_left: v }),
  new: (top_left: Corner, top_right: Corner, bottom_right: Corner, bottom_left: Corner): BorderRadius => ({ top_left, top_right, bottom_right, bottom_left }),
  /** each corner circular, in px (f32::MAX's a corner as round as the node allows) */
  px: (top_left: number, top_right: number, bottom_right: number, bottom_left: number): BorderRadius => ({ top_left: px(top_left), top_right: px(top_right), bottom_right: px(bottom_right), bottom_left: px(bottom_left) }),
  top: (v: Corner): BorderRadius => ({ top_left: v, top_right: v, bottom_right: Z, bottom_left: Z }),
  bottom: (v: Corner): BorderRadius => ({ top_left: Z, top_right: Z, bottom_right: v, bottom_left: v }),
  left: (v: Corner): BorderRadius => ({ top_left: v, top_right: Z, bottom_right: Z, bottom_left: v }),
  right: (v: Corner): BorderRadius => ({ top_left: Z, top_right: v, bottom_right: v, bottom_left: Z }),
  MAX: { top_left: "9999px", top_right: "9999px", bottom_right: "9999px", bottom_left: "9999px" } as BorderRadius,
};

/** A Val in px of `of` px (a percent of it), or undefined for a viewport unit, which the page resolves. */
const pxOf = (v: Val, of: number) => (v.endsWith("%") ? (parseFloat(v) / 100) * of : v.endsWith("px") ? parseFloat(v) : undefined);

/** CSS border-radius as Bevy resolves the corners (CornerRadius::resolve): a circular one of the node's shorter side,
 *  clamped to half of it, an elliptical one's x of its width and y of its height, each clamped to half that. With the
 *  node's size in px (`size`, its border box), they're resolved so: CSS would instead shrink every corner together
 *  when two overlap (a round corner beside a square one, which Bevy draws half the node wide). */
export function radiusCss(who: string, r: BorderRadius, size?: [number, number]): string {
  if (typeof r !== "object" || r === null) throw new TypeError(`${who}: border_radius is a BorderRadius (BorderRadius.all(px(8))), got ${JSON.stringify(r)}`);
  const at = (v: Val, of: number) => {
    const n = pxOf(v, of);
    return n === undefined ? v : `${Math.min(Math.max(n, 0), of / 2)}px`;
  };
  const corners = (["top_left", "top_right", "bottom_right", "bottom_left"] as const).map((s): [string, string] => {
    const c = r[s];
    const k = `border_radius.${s}`;
    const [x, y] = typeof c === "object" && c !== null ? [checkVal(who, `${k}.x`, c.x), checkVal(who, `${k}.y`, c.y)] : [checkVal(who, k, c), "auto"];
    if (x === "auto" && y === "auto") return [Z, Z];
    if (x === "auto" || y === "auto") {
      const v = x === "auto" ? y : x;
      const m = size ? at(v, Math.min(...size)) : v;
      return [m, m];
    }
    return size ? [at(x, size[0]), at(y, size[1])] : [x, y];
  });
  const xs = corners.map((c) => c[0]).join(" "), ys = corners.map((c) => c[1]).join(" ");
  return xs === ys ? xs : `${xs} / ${ys}`;
}

/** A Bevy enum variant as CSS's keyword (FlexStart: flex-start, SpaceBetween: space-between); Default is CSS's
 *  normal, which taffy reads as bevy_ui's None, so it sends nothing. */
export function keyword(who: string, k: string, v: string, allowed: readonly string[]): string | undefined {
  if (!allowed.includes(v)) throw new TypeError(`${who}: ${k} is one of ${allowed.join(", ")}, got ${JSON.stringify(v)}`);
  return v === "Default" ? undefined : v.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase();
}

export const ALIGN_ITEMS = ["Default", "Start", "End", "FlexStart", "FlexEnd", "Center", "Baseline", "Stretch"] as const;
export const ALIGN_SELF = ["Auto", ...ALIGN_ITEMS.slice(1)] as const;
export const JUSTIFY_ITEMS = ["Default", "Start", "End", "Center", "Baseline", "Stretch"] as const;
export const JUSTIFY_SELF = ["Auto", "Start", "End", "Center", "Baseline", "Stretch"] as const;
export const JUSTIFY_CONTENT = ["Default", "Start", "End", "FlexStart", "FlexEnd", "Center", "Stretch", "SpaceBetween", "SpaceEvenly", "SpaceAround"] as const;
export const ALIGN_CONTENT = JUSTIFY_CONTENT;
export const FLEX_DIRECTION = ["Row", "Column", "RowReverse", "ColumnReverse"] as const;
export const FLEX_WRAP = ["NoWrap", "Wrap", "WrapReverse"] as const;
export const DISPLAY = ["Flex", "Grid", "Block", "None"] as const;
export const POSITION_TYPE = ["Relative", "Absolute"] as const;
export const OVERFLOW_AXIS = ["Visible", "Clip", "Hidden", "Scroll"] as const;

/** Bevy's Overflow: each axis Visible, Clip, Hidden or Scroll. */
export interface Overflow { x: (typeof OVERFLOW_AXIS)[number]; y: (typeof OVERFLOW_AXIS)[number] }
export const Overflow = {
  visible: (): Overflow => ({ x: "Visible", y: "Visible" }),
  clip: (): Overflow => ({ x: "Clip", y: "Clip" }),
  clip_x: (): Overflow => ({ x: "Clip", y: "Visible" }),
  clip_y: (): Overflow => ({ x: "Visible", y: "Clip" }),
  hidden: (): Overflow => ({ x: "Hidden", y: "Hidden" }),
  scroll: (): Overflow => ({ x: "Scroll", y: "Scroll" }),
  scroll_x: (): Overflow => ({ x: "Scroll", y: "Visible" }),
  scroll_y: (): Overflow => ({ x: "Visible", y: "Scroll" }),
};

/** A bevy_color colour as CSS: Srgba or LinearRgba (its sRGB), or a CSS colour string as written. */
export function colorCss(who: string, k: string, c: unknown): string {
  if (typeof c === "string") return c;
  const s = c instanceof LinearRgba ? c.to_srgba() : c instanceof Srgba ? c : undefined;
  if (!s) throw new TypeError(`${who}: ${k} is an Srgba, a LinearRgba or a CSS colour, got ${JSON.stringify(c)}`);
  const b = (v: number) => Math.round(Math.max(0, Math.min(1, v)) * 255);
  return `rgba(${b(s.red)}, ${b(s.green)}, ${b(s.blue)}, ${Math.max(0, Math.min(1, s.alpha))})`;
}

/** What lies under a part of the UI, in linear light, when it's known: an opaque Node colour, or translucent ones over
 *  one. Bevy draws its UI onto an sRGB target, so a translucent colour blends in linear light, as a page with bevy_ui in it
 *  now does too (data-raea-blend). Over a known backdrop the colour is sent as the opaque colour Bevy's blend makes (white
 *  at 0.2 over srgb 0.1 green is 126; sRGB blending made 72); over the 3D (unknown) it's sent as it is. */
export type Backdrop = [number, number, number] | undefined;

const toLinear = (c: number) => (c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4));
const toSrgb = (c: number) => (c <= 0.0031308 ? c * 12.92 : 1.055 * Math.pow(c, 1 / 2.4) - 0.055);

/** A colour over `back` as Bevy blends it: its CSS, and the backdrop it leaves. */
export function over(who: string, k: string, c: unknown, back: Backdrop): { css: string; back: Backdrop } {
  const s = c instanceof LinearRgba ? c.to_srgba() : c instanceof Srgba ? c : undefined;
  if (!s) return { css: colorCss(who, k, c), back: undefined };
  const a = Math.max(0, Math.min(1, s.alpha));
  const lin: [number, number, number] = [s.red, s.green, s.blue].map((v) => toLinear(Math.max(0, Math.min(1, v)))) as [number, number, number];
  if (a >= 1) return { css: colorCss(who, k, s), back: lin };
  if (!back) return { css: colorCss(who, k, s), back: undefined };
  const mixed = lin.map((v, i) => a * v + (1 - a) * back[i]) as [number, number, number];
  const [r, g, b] = mixed.map(toSrgb);
  return { css: colorCss(who, k, new Srgba(r, g, b, 1)), back: mixed };
}

/** Bevy's OverflowClipMargin: the box an overflow clip clips to, grown by a margin in px (the padding box by default). */
export interface OverflowClipMargin { visual_box: "BorderBox" | "PaddingBox" | "ContentBox"; margin: number }
const clipMargin = (visual_box: OverflowClipMargin["visual_box"]) => ({ visual_box, margin: 0, with_margin(margin: number) { return { visual_box, margin }; } });
export const OverflowClipMargin = {
  DEFAULT: { visual_box: "PaddingBox", margin: 0 } as OverflowClipMargin,
  border_box: () => clipMargin("BorderBox"),
  padding_box: () => clipMargin("PaddingBox"),
  content_box: () => clipMargin("ContentBox"),
};
