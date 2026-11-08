// bevy_ui's debug overlay: GlobalUiDebugOptions over a UI, a node's own UiDebugOptions in its place, drawn by raea's
// Blitz over every UI node (vendor/blitz-paint render/bevy_debug.rs) from each node's data-raea-bevy-debug.
import { createContext, useContext, type ReactNode } from "react";
import type { Srgba } from "./colors";

/** bevy_ui's UiDebugOptions (and GlobalUiDebugOptions): which of a node's boxes to outline, how thick (px), in what
 *  colour (each node its own by default), and whether a clipped or hidden node's outline shows. Every field Bevy's
 *  default when left out. @bevy bevy_ui_render::UiDebugOptions */
export interface UiDebugOptions {
  enabled?: boolean;
  outline_border_box?: boolean;
  outline_padding_box?: boolean;
  outline_content_box?: boolean;
  line_width?: number;
  /** one colour for every outline, instead of each node's (Hsla::sequential_dispersed of its index) */
  line_color_override?: Srgba;
  /** an outline not cut by its ancestors' overflow */
  show_clipped?: boolean;
}

const Global = createContext<UiDebugOptions | null>(null);

/** bevy_ui's GlobalUiDebugOptions: the debug overlay for every node inside that has no UiDebugOptions of its own
 *  (a node's `ui_debug`). @bevy bevy_ui_render::GlobalUiDebugOptions */
export function GlobalUiDebugOptions({ children, ...options }: UiDebugOptions & { children?: ReactNode }) {
  return <Global.Provider value={options}>{children}</Global.Provider>;
}

/** bevy_color's Hsla::sequential_dispersed(index) as sRGB (s 1, l 0.5): a hue by the golden ratio, so that near
 *  indices differ most, as bevy_ui colours a node's outline by its entity's index */
export function sequentialDispersed(index: number): [number, number, number] {
  const RATIO_360 = Math.fround(360 / Math.fround(4294967295));
  const hue = Math.fround(Math.fround(Math.imul(index >>> 0, 2654435769) >>> 0) * RATIO_360);
  const k = (n: number) => (n + hue / 30) % 12;
  const f = (n: number) => 0.5 - 0.5 * Math.max(-1, Math.min(k(n) - 3, 9 - k(n), 1));
  return [f(0), f(8), f(4)];
}

/** A node's debug outline marker (data-raea-bevy-debug, "width r g b a boxes clipped"), from its own UiDebugOptions or
 *  else the GlobalUiDebugOptions it's in; nothing when neither is enabled. Called while its component renders. */
export function useDebugAttr(who: string, own: UiDebugOptions | undefined, index: number | undefined, scale: number): string | undefined {
  const global = useContext(Global);
  const o = own ?? global;
  if (!o?.enabled) return undefined;
  const w = o.line_width ?? 1;
  if (!(Number.isFinite(w) && w >= 0)) throw new RangeError(`${who}: ui_debug.line_width is px at least 0, got ${w}`);
  if (index !== undefined && !(Number.isInteger(index) && index >= 0)) throw new RangeError(`${who}: debug_color_index is a whole number at least 0, got ${index}`);
  const c = o.line_color_override;
  const [r, g, b, a] = c ? [c.red, c.green, c.blue, c.alpha] : [...sequentialDispersed(index ?? 0), 1];
  const boxes = `${(o.outline_border_box ?? true) ? "b" : ""}${o.outline_padding_box ? "p" : ""}${o.outline_content_box ? "c" : ""}`;
  return boxes ? `${w * scale} ${r} ${g} ${b} ${a} ${boxes} ${o.show_clipped ? 1 : 0}` : undefined;
}
