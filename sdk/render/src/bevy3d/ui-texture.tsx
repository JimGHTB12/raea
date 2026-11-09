// docs/control.feature FC0a: bevy_ui rendered into an image a 3D material samples (Bevy's ui/render_ui_to_texture: a
// Camera2d whose RenderTarget is an Image, its UI roots under UiTargetCamera, the Image a base_color_texture). The
// image is a <Screen>'s: UiTexture lays its UI out as a screen's picture (screenContent: off the frame's view, painted
// on its own and handed to raea-render by name each frame), and a material naming it samples that screen's image
// (raea-sys-screen's named_image: `raea-screen:name@WxH`), with no screen quad of its own.
import type { ReactNode } from "react";
import { screenContent } from "../screen";

/** An image the UI inside a <UiTexture> is drawn into, `width` x `height` px (bevy_image's Image::new_target_texture):
 *  pass it to a material's texture field, e.g. StandardMaterial's base_color_texture. The name tells one target
 *  from another (and from the composition's <Screen>s). @bevy bevy_image::Image::new_target_texture */
export function targetImage(name: string, width: number, height: number): string {
  if (!/^[A-Za-z0-9_-]+$/.test(name)) throw new TypeError(`targetImage: a name is letters, digits, _ and -, got ${JSON.stringify(name)}`);
  if (![width, height].every((n) => Number.isInteger(n) && n >= 1 && n <= 8192)) throw new RangeError(`targetImage: width and height are whole px from 1 to 8192, got ${width} x ${height}`);
  return `raea-screen:${name}@${width}x${height}`;
}

export interface UiTextureProps {
  /** the image's name, as a material or ImageNode uses it */
  target: string;
  /** the UI roots drawn into it */
  children?: ReactNode;
}

/** The UI roots inside drawn into `target` instead of the window (a Camera2d with RenderTarget::Image, the roots'
 *  UiTargetCamera): laid out in a box the image's size, see-through where they draw nothing, and nowhere in the frame
 *  itself. A root's 100% is the image's size. @bevy bevy_camera::RenderTarget::Image */
export function UiTexture({ target, children }: UiTextureProps) {
  const m = /^raea-screen:([A-Za-z0-9_-]+)@(\d+)x(\d+)$/.exec(target);
  if (!m) throw new TypeError(`UiTexture: target is a targetImage(name, width, height), got ${JSON.stringify(target)}`);
  return screenContent(m[1], { resolution: [Number(m[2]), Number(m[3])], crt: false, background: "transparent", children });
}
