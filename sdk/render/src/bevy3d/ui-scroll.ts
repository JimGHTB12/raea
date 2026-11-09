// bevy_ui's scrolling as the page's CSS: a Scroll axis draws no scrollbar (bevy_ui draws none; Blitz would draw an
// overlay thumb), Node's scrollbar_width is the gutter taffy reserves beside it, ScrollPosition is the scroller's offset
// and IgnoreScroll keeps a node in place while its parent scrolls.
import { checkVal, type Overflow, type UiRect, type Val } from "./ui-val";

/** bevy_ui's IgnoreScroll: the axes on which a node keeps its place while its parent scrolls (BVec2).
 *  @bevy bevy_ui::IgnoreScroll */
export interface IgnoreScroll {
  /** it keeps its place while the parent scrolls across */
  x: boolean;
  /** it keeps its place while the parent scrolls down */
  y: boolean;
}

export interface ScrollProps { overflow?: Overflow; padding?: UiRect; scrollbar_width?: number; scroll_position?: [number, number]; ignore_scroll?: IgnoreScroll }

const grow = (v: Val, w: number) => (v.endsWith("px") ? `${parseFloat(v) + w}px` : `calc(${v} + ${w}px)`);

/** A Node's scroll fields into its CSS, after its padding, and its element's attributes. taffy reserves
 *  scrollbar_width as an inset beside a scrolling axis (on the right of one scrolling on y, below one scrolling on x),
 *  inside the border as padding is. ScrollPosition (logical px, times `scale`, the UiScale) scrolls only a Scroll axis;
 *  Blitz keeps it in the scroller's range once laid out and floors it to whole physical px, as bevy_ui's layout does (vendor/blitz-dom resolve.rs). IgnoreScroll: Blitz moves the node back by its parent's scroll. */
export function scrollCss(who: string, p: ScrollProps, s: Record<string, unknown>, attrs: Record<string, string | number>, scale = 1) {
  const o = p.overflow;
  if (o && (o.x === "Scroll" || o.y === "Scroll")) s.scrollbarWidth = "none";
  const ig = p.ignore_scroll;
  if (ig !== undefined) {
    if (typeof ig?.x !== "boolean" || typeof ig?.y !== "boolean") throw new TypeError(`${who}: ignore_scroll is { x: boolean, y: boolean }, got ${JSON.stringify(ig)}`);
    if (ig.x || ig.y) attrs["data-raea-bevy-ignore-scroll"] = `${ig.x ? "x" : ""}${ig.y ? "y" : ""}`;
  }
  const at = p.scroll_position;
  if (at !== undefined) {
    if (!Array.isArray(at) || at.length !== 2 || !at.every(Number.isFinite)) throw new TypeError(`${who}: scroll_position is [x, y] in px, got ${JSON.stringify(at)}`);
    // (in the page's px: under a UiScale's zoom, physical ones)
    const [x, y] = [at[0] * scale, at[1] * scale];
    attrs["data-raea-bevy-scroll"] = `1 ${x} ${y}`;
    attrs.scrollLeft = o?.x === "Scroll" ? x : 0;
    attrs.scrollTop = o?.y === "Scroll" ? y : 0;
  }
  const w = p.scrollbar_width;
  if (w === undefined) return;
  if (!(Number.isFinite(w) && w >= 0)) throw new RangeError(`${who}: scrollbar_width is a number of px at least 0, got ${w}`);
  if (w === 0) return;
  const pad = (k: "right" | "bottom") => (p.padding ? checkVal(who, `padding.${k}`, p.padding[k]) : "0px");
  if (o?.y === "Scroll") s.paddingRight = grow(pad("right"), w);
  if (o?.x === "Scroll") s.paddingBottom = grow(pad("bottom"), w);
}
