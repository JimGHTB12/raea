// bevy_feathers' colour swatch and colour slider (color_swatch.rs, color_slider.rs, alpha_pattern.rs; docs/control.feature
// FC0a). Bevy draws the checkerboard under a translucent colour with a UiMaterial (alpha_pattern.wesl: 8 px squares of
// linear 0.2 and 0.6, a corner of four at the node's centre); raea has no UiMaterial yet, so here it's the node's CSS
// background, a conic gradient tiled every 16 px from the centre, which the node's corners clip as the shader's
// sd_rounded_box does. A slider's thumb is placed as update_slider_pos places it after layout, from its value alone.
import { useContext, type CSSProperties, type ReactNode } from "react";
import { InUi, Node, UiScaleCx, boxOf, nodeCss, type NodeProps } from "../ui";
import { LinearRgba, Srgba } from "../colors";
import { Hsla } from "../colorspaces";
import { Okhsla } from "../okhsla";
import { ColorStop, LinearGradient, type Gradient, type InterpolationColorSpace } from "../ui-gradient";
import { GridPlacement } from "../ui-grid";
import { BorderRadius, UiRect, percent, px } from "../ui-val";
import { UiTransform, Val2 } from "../ui-transform";
import { size } from "./constants";
import { to_border_radius, type RoundedCorners } from "./controls";

/** a colour a colour control takes */
export type FeathersColor = Srgba | LinearRgba | Hsla | Okhsla;

const srgbaOf = (c: FeathersColor): Srgba => (c instanceof Srgba ? c : c.to_srgba());
const hslaOf = (c: FeathersColor): Hsla => (c instanceof Hsla ? c : Hsla.from(srgbaOf(c)));
const okhslaOf = (c: FeathersColor): Okhsla => (c instanceof Okhsla ? c : Okhsla.from(c instanceof Hsla ? c : c instanceof LinearRgba ? c : srgbaOf(c)));
const withAlpha = (c: FeathersColor, a: number): FeathersColor => (c instanceof Srgba ? c.with_alpha(a) : c instanceof LinearRgba ? new LinearRgba(c.red, c.green, c.blue, a) : c instanceof Hsla ? new Hsla(c.hue, c.saturation, c.lightness, a) : c.with_alpha(a));

/** alpha_pattern.wesl's checkerboard: linear 0.2 (sRGB 124) below right of the centre and above left, 0.6 (203) beside */
const CHECKER: CSSProperties = {
  backgroundImage: "conic-gradient(rgb(203, 203, 203) 0deg 90deg, rgb(124, 124, 124) 90deg 180deg, rgb(203, 203, 203) 180deg 270deg, rgb(124, 124, 124) 270deg 360deg)",
  backgroundSize: "16px 16px",
  backgroundPosition: "50% 50%",
  backgroundRepeat: "repeat",
};

/** a Node with feathers' AlphaPattern material under its children */
function AlphaPattern({ children, ...p }: NodeProps & { children?: ReactNode }) {
  if (p.background_color !== undefined || p.background_gradient !== undefined) throw new TypeError("AlphaPattern: the pattern is the node's background; colour a child");
  const ui = useContext(InUi);
  const n = nodeCss("AlphaPattern", p, ui?.back, useContext(UiScaleCx));
  // the pattern is opaque: what's inside blends over its squares, which differ, so it leaves no one backdrop
  return boxOf({ ...n.css, ...CHECKER }, undefined, children, ui === null, undefined, n.attrs);
}

export interface FeathersColorSwatchProps extends Omit<NodeProps, "border_radius"> {
  /** ColorSwatchValue: the colour shown over the checkerboard */
  color?: FeathersColor;
  /** how much of its right side shows the colour opaque, in percent */
  opaque_color_percentage?: number;
  corners?: RoundedCorners;
  /** the corners' radius in px (5 by default), on the corners `corners` names */
  border_radius?: number;
}

/** A colour swatch: ROW_HEIGHT high, at least as wide, its colour over the alpha pattern, the right
 *  `opaque_color_percentage` of it the colour made opaque. @bevy bevy_feathers::controls::FeathersColorSwatch */
export function FeathersColorSwatch({ color, opaque_color_percentage = 0, corners = "All", border_radius = 5, ...node }: FeathersColorSwatchProps) {
  // ColorSwatchValue's default is Color::default (white); update_swatch_color paints both parts from it
  const c = srgbaOf(color ?? new Srgba(1, 1, 1, 1));
  const radius = to_border_radius(corners, border_radius);
  return (
    <AlphaPattern height={size.ROW_HEIGHT} min_width={size.ROW_HEIGHT} border_radius={radius} {...node}>
      <Node position_type="Absolute" left={px(0)} top={px(0)} bottom={px(0)} right={px(0)} border_radius={radius} background_color={c} />
      {opaque_color_percentage > 0 && (
        <Node position_type="Absolute" left={percent(100 - Math.max(0, Math.min(100, opaque_color_percentage)))} top={px(0)} bottom={px(0)} right={px(0)} border_radius={to_border_radius("Right", border_radius)} background_color={c.with_alpha(1)} z_index={1} />
      )}
    </AlphaPattern>
  );
}

/** Which channel of a colour a FeathersColorSlider sets. @bevy bevy_feathers::controls::ColorChannel */
export type ColorChannel = "Red" | "Green" | "Blue" | "HslHue" | "HslSaturation" | "HslLightness" | "Alpha" | "OkhslHue" | "OkhslSaturation" | "OkhslLightness";
const CHANNELS: readonly ColorChannel[] = ["Red", "Green", "Blue", "HslHue", "HslSaturation", "HslLightness", "Alpha", "OkhslHue", "OkhslSaturation", "OkhslLightness"];

/** ColorChannel::range: 0 to 360 for a hue, 0 to 1 for the rest. */
export function channel_range(channel: ColorChannel): { start: number; end: number } {
  return channel === "HslHue" || channel === "OkhslHue" ? { start: 0, end: 360 } : { start: 0, end: 1 };
}

/** ColorChannel::gradient_ends: the slider track's colours at its start, middle and end, the channel at 0, half and
 *  full with the base colour's other channels. */
export function gradient_ends(channel: ColorChannel, base: FeathersColor): [FeathersColor, FeathersColor, FeathersColor] {
  const rgb = () => srgbaOf(base);
  const three = <T,>(f: (v: number) => T): [T, T, T] => [f(0), f(0.5), f(1)];
  switch (channel) {
    case "Red": return three((v) => new Srgba(v, rgb().green, rgb().blue));
    case "Green": return three((v) => new Srgba(rgb().red, v, rgb().blue));
    case "Blue": return three((v) => new Srgba(rgb().red, rgb().green, v));
    case "HslHue": return [new Hsla(0.0001, 1, 0.5), new Hsla(180, 1, 0.5), new Hsla(360 - 0.0001, 1, 0.5)];
    case "HslSaturation": { const h = hslaOf(base); return three((v) => new Hsla(h.hue, v, h.lightness)); }
    case "HslLightness": { const h = hslaOf(base); return three((v) => new Hsla(h.hue, h.saturation, v)); }
    case "OkhslHue": return [new Okhsla(0.0001, 1, 0.5), new Okhsla(180, 1, 0.5), new Okhsla(360 - 0.0001, 1, 0.5)];
    case "OkhslSaturation": { const o = okhslaOf(base); return three((v) => new Okhsla(o.hue, v, o.lightness)); }
    case "OkhslLightness": { const o = okhslaOf(base); return three((v) => new Okhsla(o.hue, o.saturation, v)); }
    case "Alpha": return three((v) => withAlpha(base, v));
  }
}

/** update_track_color's interpolation space: the channel's own, or for Alpha the base colour's */
function spaceOf(channel: ColorChannel, base: FeathersColor): InterpolationColorSpace {
  if (channel === "Red" || channel === "Green" || channel === "Blue") return "Srgba";
  if (channel.startsWith("Hsl")) return "Hsla";
  if (channel.startsWith("Okhsl")) return "Okhsla";
  return base instanceof LinearRgba ? "LinearRgba" : base instanceof Hsla ? "Hsla" : base instanceof Okhsla ? "Srgba" : "Srgba";
}

/** a stop's colour as the gradient takes it: an Hsla gradient's as CSS hsl(), its hue kept as Bevy keeps it (the stop
 *  turned to linear and back to Hsla on the CPU, where 359.9999 stays 359.9999, which CSS's rgb() would round to red) */
function stopColor(c: FeathersColor, space: InterpolationColorSpace): Srgba | string {
  if (space !== "Hsla") return srgbaOf(c);
  const h = Hsla.from(LinearRgba.from(srgbaOf(c)));
  return `hsl(${h.hue} ${h.saturation * 100}% ${h.lightness * 100}% / ${h.alpha})`;
}

/** Color::NONE: transparent black in sRGB */
const NONE = new Srgba(0, 0, 0, 0);
const SLIDER_HEIGHT = 16;
const TRACK_PADDING = 3;
const TRACK_RADIUS = SLIDER_HEIGHT * 0.5 - TRACK_PADDING;
const THUMB_SIZE = SLIDER_HEIGHT - 2;

export interface FeathersColorSliderProps extends NodeProps {
  /** SliderValue: the channel's value, in its range */
  value?: number;
  channel?: ColorChannel;
  /** SliderBaseColor: the colour the track shows the channel of (white by default) */
  base_color?: FeathersColor;
}

/** A colour slider: a 16 px track of the channel's gradient over the alpha pattern, a 14 px ringed thumb at the value.
 *  @bevy bevy_feathers::controls::FeathersColorSlider */
export function FeathersColorSlider({ value = 0, channel = "Alpha", base_color = new Srgba(1, 1, 1, 1), ...node }: FeathersColorSliderProps) {
  if (!CHANNELS.includes(channel)) throw new TypeError(`FeathersColorSlider: channel is one of ${CHANNELS.join(", ")}, got ${JSON.stringify(channel)}`);
  if (!Number.isFinite(value)) throw new RangeError(`FeathersColorSlider: value is a number, got ${value}`);
  const [start, middle, end] = gradient_ends(channel, base_color);
  const space = spaceOf(channel, base_color);
  const half = (from: FeathersColor, angle: (stops: ColorStop[]) => Gradient): Gradient => ({
    ...angle([ColorStop.new(stopColor(from, space), px(0)), ColorStop.new(stopColor(from, space), px(THUMB_SIZE * 0.5)), ColorStop.new(stopColor(middle, space), percent(50)), ColorStop.new(stopColor(NONE, space), percent(50)), ColorStop.new(stopColor(NONE, space), percent(100))]),
    color_space: space,
  });
  const range = channel_range(channel);
  const t = range.end > range.start ? (value - range.start) / (range.end - range.start) : 0.5;
  const cell = { grid_row: GridPlacement.start(1), grid_column: GridPlacement.start(1) };
  const track = { ...cell, margin: UiRect.vertical(px(TRACK_PADDING)), border_radius: BorderRadius.all(px(TRACK_RADIUS)) };
  return (
    <Node display="Grid" height={px(SLIDER_HEIGHT)} flex_grow={1} {...node}>
      <AlphaPattern {...track} />
      <Node {...track} background_gradient={[half(start, LinearGradient.to_right), half(end, LinearGradient.to_left)]} />
      <Node {...cell} margin={UiRect.horizontal(px(THUMB_SIZE * 0.5))}>
        {/* update_slider_pos: x = t * the track's width - half the thumb */}
        <Node position_type="Absolute" left={percent(t * 100)} top={percent(50)} width={px(THUMB_SIZE)} height={px(THUMB_SIZE)} border={UiRect.all(px(2))} border_radius={BorderRadius.MAX} border_color={new Srgba(1, 1, 1, 1)} outline={{ width: px(1), offset: px(0), color: new Srgba(0, 0, 0, 1) }} ui_transform={UiTransform.from_translation(Val2.new(px(-THUMB_SIZE * 0.5), percent(-50)))} />
      </Node>
    </Node>
  );
}
