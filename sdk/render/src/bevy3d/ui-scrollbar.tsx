// bevy_ui_widgets' Scrollbar and ScrollbarThumb. The track is a Node laid out as any other; the thumb is not laid out
// (Bevy gives it no Node) but sized and placed in the track after layout from its target's ScrollPosition, as
// bevy_ui_widgets' update_scrollbar_thumb does (vendor/blitz-dom bevy_scroll.rs).
import { useContext, type ReactNode } from "react";
import { boxOf, InUi, nodeCss, UiScaleCx, type NodeProps } from "./ui";
import type { BorderRadius, UiRect } from "./ui-val";

/** bevy_ui_widgets' ControlOrientation. @bevy bevy_ui_widgets::ControlOrientation */
export type ControlOrientation = "Horizontal" | "Vertical";

export interface ScrollbarProps extends NodeProps {
  /** the Name of the Node it scrolls (an overflow Scroll one) */
  target: string;
  orientation: ControlOrientation;
  /** the thumb's least length, px */
  min_thumb_length: number;
  /** its ScrollbarThumb */
  children?: ReactNode;
}

/** A scrollbar's track: its ScrollbarThumb inside shows how much of its target is in view and where.
 *  @bevy bevy_ui_widgets::Scrollbar */
export function Scrollbar({ target, orientation, min_thumb_length, children, ...p }: ScrollbarProps) {
  if (typeof target !== "string" || target === "" || /\s/.test(target)) throw new TypeError(`Scrollbar: target is a Node's name with no spaces, got ${JSON.stringify(target)}`);
  if (orientation !== "Horizontal" && orientation !== "Vertical") throw new TypeError(`Scrollbar: orientation is "Horizontal" or "Vertical", got ${JSON.stringify(orientation)}`);
  if (!(Number.isFinite(min_thumb_length) && min_thumb_length >= 0)) throw new RangeError(`Scrollbar: min_thumb_length is a number of px at least 0, got ${min_thumb_length}`);
  const ui = useContext(InUi);
  if (ui === null) throw new Error("Scrollbar: goes inside a Node");
  const scale = useContext(UiScaleCx);
  const n = nodeCss("Scrollbar", p, ui.back, scale);
  // (the least length in the page's px: under a UiScale's zoom, physical ones)
  n.attrs["data-raea-bevy-scrollbar"] = `${orientation === "Vertical" ? "v" : "h"} ${min_thumb_length * scale} ${target}`;
  return boxOf(n.css, n.back, children, false, undefined, n.attrs);
}

export interface ScrollbarThumbProps {
  border_radius?: BorderRadius;
  /** the border's width on each side */
  border?: UiRect;
  /** BackgroundColor */
  background_color?: NodeProps["background_color"];
  /** BorderColor, every side */
  border_color?: NodeProps["background_color"];
}

/** A Scrollbar's thumb: the track's width across it, its length the share of the target in view.
 *  @bevy bevy_ui_widgets::ScrollbarThumb */
export function ScrollbarThumb({ border_radius, border, background_color, border_color }: ScrollbarThumbProps) {
  const ui = useContext(InUi);
  if (ui === null) throw new Error("ScrollbarThumb: goes inside a Scrollbar");
  const n = nodeCss("ScrollbarThumb", { border_radius, border, background_color, border_color }, ui.back, useContext(UiScaleCx));
  // (out of the track's layout: Blitz gives it its size and place)
  const css = { ...n.css, position: "absolute" as const, left: 0, top: 0 };
  return <div style={css} {...n.attrs} data-raea-bevy-thumb="1" />;
}
