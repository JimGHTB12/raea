// bevy_ui's scrolling as the page's CSS: a Scroll axis draws no scrollbar (bevy_ui draws none; Blitz would draw an
// overlay thumb), Node's scrollbar_width is the gutter taffy reserves beside it, and IgnoreScroll.
import { checkVal, type Overflow, type UiRect, type Val } from "./ui-val";

/** bevy_ui's IgnoreScroll: the axes on which a node keeps its place while its parent scrolls (BVec2).
 *  @bevy bevy_ui::IgnoreScroll */
export interface IgnoreScroll { x: boolean; y: boolean }

const grow = (v: Val, w: number) => (v.endsWith("px") ? `${parseFloat(v) + w}px` : `calc(${v} + ${w}px)`);

/** A Node's scroll fields into its CSS, after its padding. taffy reserves scrollbar_width as an inset beside a
 *  scrolling axis (on the right of one scrolling on y, below one scrolling on x), inside the border as padding is. A
 *  page here is never scrolled (no ScrollPosition yet), so IgnoreScroll keeps every node where it already is. */
export function scrollCss(who: string, p: { overflow?: Overflow; padding?: UiRect; scrollbar_width?: number; ignore_scroll?: IgnoreScroll }, s: Record<string, unknown>) {
  const o = p.overflow;
  if (o && (o.x === "Scroll" || o.y === "Scroll")) s.scrollbarWidth = "none";
  if (p.ignore_scroll !== undefined && (typeof p.ignore_scroll?.x !== "boolean" || typeof p.ignore_scroll?.y !== "boolean")) throw new TypeError(`${who}: ignore_scroll is { x: boolean, y: boolean }, got ${JSON.stringify(p.ignore_scroll)}`);
  const w = p.scrollbar_width;
  if (w === undefined) return;
  if (!(Number.isFinite(w) && w >= 0)) throw new RangeError(`${who}: scrollbar_width is a number of px at least 0, got ${w}`);
  if (w === 0) return;
  const pad = (k: "right" | "bottom") => (p.padding ? checkVal(who, `padding.${k}`, p.padding[k]) : "0px");
  if (o?.y === "Scroll") s.paddingRight = grow(pad("right"), w);
  if (o?.x === "Scroll") s.paddingBottom = grow(pad("bottom"), w);
}
