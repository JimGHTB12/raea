// What the UI's roots are drawn through: bevy_ui's UiScale, the UI camera's Viewport and UiTargetCamera.
import type { ReactNode } from "react";
import { UiScaleCx } from "./ui";

export interface UiScaleProps {
  /** how many times as big the UI is drawn (UiScale) */
  value: number;
  /** the UI it scales */
  children?: ReactNode;
}

/** bevy_ui's UiScale: every UI length inside times `value` (2 draws a 16 px node 32 px), as CSS's zoom: laid out
 *  and its text shaped at that size, in physical px, as Bevy lays out (a 3 px padding at 1.25 is 4 px, not 3 drawn 1.25
 *  times as big). Text2d and the 2D and 3D parts aren't UI, so they stay as they are.
 * @bevy bevy_ui::UiScale */
export function UiScale({ value, children }: UiScaleProps) {
  if (!(Number.isFinite(value) && value > 0)) throw new RangeError(`UiScale: value is above 0, got ${value}`);
  return (
    <div style={{ position: "absolute", left: 0, top: 0, width: "100%", height: "100%", zoom: value }}>
      <UiScaleCx.Provider value={value}>{children}</UiScaleCx.Provider>
    </div>
  );
}

export interface ViewportProps {
  /** the region's top-left corner, px from the window's */
  physical_position: [number, number];
  /** the region's width and height, px */
  physical_size: [number, number];
  /** the cameras and UI drawn into the region */
  children?: ReactNode;
}

/** The UI camera's viewport (bevy_camera's Viewport on the Camera2d the UI draws through): its roots lay out in this
 *  region of the window, `physical_position` from its top left and `physical_size` big (px, at a scale factor of 1),
 *  and are clipped to it. @bevy bevy_camera::Viewport */
export function Viewport({ physical_position, physical_size, children }: ViewportProps) {
  const ok = (v: unknown, min: number) => Array.isArray(v) && v.length === 2 && v.every((n) => Number.isInteger(n) && n >= min);
  if (!ok(physical_position, 0) || !ok(physical_size, 1)) throw new RangeError(`Viewport: physical_position is [x, y] >= 0 and physical_size [w, h] >= 1 in whole px, got ${JSON.stringify({ physical_position, physical_size })}`);
  const [[left, top], [width, height]] = [physical_position, physical_size];
  return <div style={{ position: "absolute", left, top, width, height, overflow: "hidden" }}>{children}</div>;
}

export interface UiTargetCameraProps {
  /** the camera, by its order, these roots draw with (0) */
  order?: number;
  /** the UI roots that camera draws */
  children?: ReactNode;
}

/** The roots inside drawn by their own Camera2d (bevy_ui's UiTargetCamera): a layer over the whole window, stacked by
 *  the camera's `order` as Bevy draws its cameras, lowest first. Cameras on one window share its main texture, which
 *  only the first of them clears (bevy_render's ViewTarget, its first call), so the window's clear is the layered
 *  <Canvas3D clearColor>'s and a later camera's own clear colour shows nowhere. Roots outside any are the default
 *  camera's (the highest order). @bevy bevy_ui::UiTargetCamera */
export function UiTargetCamera({ order = 0, children }: UiTargetCameraProps) {
  if (!Number.isInteger(order)) throw new RangeError(`UiTargetCamera: order is a whole number, got ${order}`);
  return <div style={{ position: "absolute", left: 0, top: 0, width: "100%", height: "100%", zIndex: order, isolation: "isolate" }}>{children}</div>;
}
