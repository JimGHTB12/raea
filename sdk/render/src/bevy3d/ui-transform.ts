// bevy_ui's UiTransform (a node turned, scaled and moved about its centre after layout, which it doesn't change) and
// BoxShadow (drop shadows under a node) as the page's CSS transform and box-shadow, each shadowed node marked
// data-raea-bevy-shadow with its BoxShadowSamples, which raea's Blitz paints as bevy_ui's box_shadow.wesl does.
import { createContext, createElement, type ReactNode } from "react";
import { checkVal, colorCss, type Val } from "./ui-val";

/** bevy_math's Rot2: a rotation by its cosine and sine; positive turns a UI node clockwise. @bevy bevy_math::Rot2 */
export interface Rot2 { cos: number; sin: number }
const rot = (r: number): Rot2 => {
  if (!Number.isFinite(r)) throw new RangeError(`Rot2: an angle is a finite number, got ${r}`);
  return { cos: Math.fround(Math.cos(r)), sin: Math.fround(Math.sin(r)) };
};
export const Rot2 = {
  IDENTITY: { cos: 1, sin: 0 } as Rot2,
  radians: (r: number): Rot2 => rot(r),
  degrees: (d: number): Rot2 => rot((d * Math.PI) / 180),
  /** a fraction of a full turn (0.25 is a quarter) */
  turn_fraction: (f: number): Rot2 => rot(f * 2 * Math.PI),
};

/** bevy_ui's Val2: a translation's x and y (a percent of the node's own width and height; auto is 0).
 *  @bevy bevy_ui::ui_transform::Val2 */
export interface Val2 { x: Val; y: Val }
export const Val2 = {
  ZERO: { x: "0px", y: "0px" } as Val2,
  px: (x: number, y: number): Val2 => ({ x: `${x}px`, y: `${y}px` }),
  percent: (x: number, y: number): Val2 => ({ x: `${x}%`, y: `${y}%` }),
  new: (x: Val, y: Val): Val2 => ({ x, y }),
};

/** bevy_ui's UiTransform: the node scaled, then turned, then moved, about its centre, after layout.
 *  @bevy bevy_ui::ui_transform::UiTransform */
export interface UiTransform { translation?: Val2; scale?: [number, number]; rotation?: Rot2 }
export const UiTransform = {
  IDENTITY: {} as UiTransform,
  from_translation: (translation: Val2): UiTransform => ({ translation }),
  from_rotation: (rotation: Rot2): UiTransform => ({ rotation }),
  from_scale: (scale: [number, number]): UiTransform => ({ scale }),
};

/** A UiTransform as CSS's transform (its origin is CSS's default, the box's centre, as Bevy's), or undefined at identity. */
export function transformCss(who: string, t: UiTransform): string | undefined {
  if (!t || typeof t !== "object") throw new TypeError(`${who}: ui_transform is a UiTransform (UiTransform.from_rotation(Rot2.degrees(45))), got ${JSON.stringify(t)}`);
  const parts: string[] = [];
  if (t.translation !== undefined) {
    const v = (k: "x" | "y") => (t.translation![k] === "auto" ? "0px" : checkVal(who, `ui_transform.translation.${k}`, t.translation![k]));
    const [x, y] = [v("x"), v("y")];
    if (parseFloat(x) !== 0 || parseFloat(y) !== 0) parts.push(`translate(${x}, ${y})`);
  }
  if (t.rotation !== undefined) {
    const { cos, sin } = t.rotation;
    if (!Number.isFinite(cos) || !Number.isFinite(sin)) throw new TypeError(`${who}: ui_transform.rotation is a Rot2 (Rot2.degrees(45)), got ${JSON.stringify(t.rotation)}`);
    const a = Math.atan2(sin, cos);
    if (a !== 0) parts.push(`rotate(${a}rad)`);
  }
  if (t.scale !== undefined) {
    const [sx, sy] = t.scale;
    if (!Number.isFinite(sx) || !Number.isFinite(sy)) throw new TypeError(`${who}: ui_transform.scale is [x, y], got ${JSON.stringify(t.scale)}`);
    if (sx !== 1 || sy !== 1) parts.push(`scale(${sx}, ${sy})`);
  }
  return parts.length ? parts.join(" ") : undefined;
}

/** bevy_ui's ShadowStyle: one drop shadow's colour, offset, spread and blur. @bevy bevy_ui::ShadowStyle */
export interface ShadowStyle { color: unknown; x_offset: Val; y_offset: Val; spread_radius: Val; blur_radius: Val }
/** bevy_ui's BoxShadow: drop shadows under a node, drawn back to front. @bevy bevy_ui::BoxShadow */
export type BoxShadow = ShadowStyle[];
export const BoxShadow = {
  new: (color: unknown, x_offset: Val, y_offset: Val, spread_radius: Val, blur_radius: Val): BoxShadow => [{ color, x_offset, y_offset, spread_radius, blur_radius }],
};

const pxOf = (who: string, k: string, v: Val) => {
  const s = checkVal(who, k, v);
  if (!s.endsWith("px")) throw new TypeError(`${who}: ${k} in px is what a shadow takes for now (a percent of the node's size is to do), got ${s}`);
  return parseFloat(s);
};

/** A BoxShadow as CSS's box-shadow. Bevy's blur radius is its Gaussian's standard deviation, CSS's twice that. Bevy
 *  grows the shadow's width by spread_radius (half each side) and scales its height and corners by the same ratio,
 *  where CSS's spread grows every side by it; the spread is sent as it is, and raea's Blitz spreads a marked node's
 *  shadow by Bevy's rule (vendor/blitz-paint box_shadow.rs). */
export function shadowCss(who: string, b: BoxShadow): string {
  if (!Array.isArray(b)) throw new TypeError(`${who}: box_shadow is a BoxShadow (BoxShadow.new(color, px(30), px(30), px(0), px(4))), got ${JSON.stringify(b)}`);
  // CSS lists the shadow on top first, Bevy draws its list back to front
  return [...b].reverse().map((s, i) => {
    const k = `box_shadow[${b.length - 1 - i}]`;
    const blur = pxOf(who, `${k}.blur_radius`, s.blur_radius);
    if (blur < 0) throw new RangeError(`${who}: ${k}.blur_radius is at least 0, got ${blur}`);
    return `${pxOf(who, `${k}.x_offset`, s.x_offset)}px ${pxOf(who, `${k}.y_offset`, s.y_offset)}px ${blur * 2}px ${pxOf(who, `${k}.spread_radius`, s.spread_radius)}px ${colorCss(who, `${k}.color`, s.color)}`;
  }).join(", ");
}

/** the BoxShadowSamples around a node: 4, bevy_ui_render's default, outside one */
export const ShadowSamplesCx = createContext(4);

export interface BoxShadowSamplesProps {
  /** steps each shadow's blur is summed in (4) */
  samples: number;
  /** the UI it applies to */
  children?: ReactNode;
}

/** bevy_ui_render's BoxShadowSamples: the steps each shadow's blur is summed in down its height (4 by default). Bevy sets
 *  it on a UI camera; here it wraps the UI that camera draws. @bevy bevy_ui_render::BoxShadowSamples */
export function BoxShadowSamples({ samples, children }: BoxShadowSamplesProps) {
  if (!(Number.isInteger(samples) && samples >= 0)) throw new RangeError(`BoxShadowSamples: samples is a whole number, 0 or more, got ${samples}`);
  return createElement(ShadowSamplesCx.Provider, { value: samples }, children);
}
