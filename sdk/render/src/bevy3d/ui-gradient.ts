// bevy_ui's gradients (BackgroundGradient: LinearGradient, RadialGradient, ConicGradient with their ColorStops) as the
// page's CSS gradients, which raea's Blitz paints in the colour space each names (blitz-paint gradient.rs).
import { checkVal, colorCss, type Val } from "./ui-val";
import type { LinearRgba, Srgba } from "./colors";

type Color = Srgba | LinearRgba | string;

/** bevy_ui's ColorStop: a colour at `point` along the gradient (spaced evenly when left out, as CSS's), and `hint`, where
 *  between it and the next the colour is half way (0.5, linear, by default). @bevy bevy_ui::gradients::ColorStop */
export interface ColorStop { color: Color; point?: Val; hint?: number }
export const ColorStop = {
  new: (color: Color, point: Val): ColorStop => ({ color, point }),
  auto: (color: Color): ColorStop => ({ color }),
};

/** bevy_ui's AngularColorStop: a colour at `angle` radians round a conic gradient (spaced evenly when left out).
 *  @bevy bevy_ui::gradients::AngularColorStop */
export interface AngularColorStop { color: Color; angle?: number; hint?: number }
export const AngularColorStop = {
  new: (color: Color, angle: number): AngularColorStop => ({ color, angle }),
  auto: (color: Color): AngularColorStop => ({ color }),
};

/** bevy_ui's InterpolationColorSpace: Oklaba by default. @bevy bevy_ui::gradients::InterpolationColorSpace */
export type InterpolationColorSpace = "Oklaba" | "Oklcha" | "OklchaLong" | "Srgba" | "LinearRgba" | "Hsla" | "HslaLong";
const SPACE: Record<InterpolationColorSpace, string> = {
  Oklaba: "in oklab", Oklcha: "in oklch", OklchaLong: "in oklch longer hue", Srgba: "in srgb", LinearRgba: "in srgb-linear", Hsla: "in hsl", HslaLong: "in hsl longer hue",
};

/** bevy_ui's UiPosition: a point of the node, `anchor` from its centre (each axis -0.5 to 0.5 of its size), moved `x`
 *  and `y` from there: right and down from a left, top or centre anchor, left and up from a right or bottom one.
 *  @bevy bevy_ui::UiPosition */
export interface UiPosition {
  anchor: [number, number]; x: Val; y: Val;
  at(x: Val, y: Val): UiPosition; at_x(x: Val): UiPosition; at_y(y: Val): UiPosition;
}
const position = (anchor: [number, number], x: Val = "0px", y: Val = "0px"): UiPosition => ({
  anchor, x, y,
  at: (nx, ny) => position(anchor, nx, ny),
  at_x: (nx) => position(anchor, nx, y),
  at_y: (ny) => position(anchor, x, ny),
});
export const UiPosition = {
  anchor: (anchor: [number, number]) => position(anchor),
  CENTER: position([0, 0]), TOP: position([0, -0.5]), BOTTOM: position([0, 0.5]), LEFT: position([-0.5, 0]), RIGHT: position([0.5, 0]),
  TOP_LEFT: position([-0.5, -0.5]), TOP_RIGHT: position([0.5, -0.5]), BOTTOM_LEFT: position([-0.5, 0.5]), BOTTOM_RIGHT: position([0.5, 0.5]),
};

/** bevy_ui's RadialGradientShape (ClosestCorner by default): Bevy's ClosestSide and FarthestSide are circles to the
 *  nearest or farthest side, its ClosestCorner and FarthestCorner ellipses through the nearest or farthest sides on each
 *  axis (CSS's closest-side and farthest-side ellipses), Circle a radius, Ellipse two. @bevy bevy_ui::gradients::RadialGradientShape */
export type RadialGradientShape = "ClosestSide" | "FarthestSide" | "ClosestCorner" | "FarthestCorner" | { Circle: Val } | { Ellipse: [Val, Val] };

/** One of bevy_ui's Gradients. @bevy bevy_ui::gradients::Gradient */
export type Gradient =
  | { kind: "Linear"; angle: number; stops: ColorStop[]; color_space?: InterpolationColorSpace }
  | { kind: "Radial"; position: UiPosition; shape: RadialGradientShape; stops: ColorStop[]; color_space?: InterpolationColorSpace }
  | { kind: "Conic"; start: number; position: UiPosition; stops: AngularColorStop[]; color_space?: InterpolationColorSpace };

const TAU = 2 * Math.PI;
const linear = (angle: number) => (stops: ColorStop[]): Gradient => ({ kind: "Linear", angle, stops });
/** bevy_ui's LinearGradient: `angle` radians, 0 up and clockwise. @bevy bevy_ui::gradients::LinearGradient */
export const LinearGradient = {
  new: (angle: number, stops: ColorStop[], color_space?: InterpolationColorSpace): Gradient => ({ kind: "Linear", angle, stops, color_space }),
  degrees: (degrees: number, stops: ColorStop[]): Gradient => ({ kind: "Linear", angle: (degrees * Math.PI) / 180, stops }),
  to_top: linear(0), to_top_right: linear(TAU / 8), to_right: linear(TAU / 4), to_bottom_right: linear((3 * TAU) / 8),
  to_bottom: linear(TAU / 2), to_bottom_left: linear((5 * TAU) / 8), to_left: linear((3 * TAU) / 4), to_top_left: linear((7 * TAU) / 8),
};
/** bevy_ui's RadialGradient: centred, ClosestCorner by default. @bevy bevy_ui::gradients::RadialGradient */
export const RadialGradient = {
  new: (position: UiPosition, shape: RadialGradientShape, stops: ColorStop[], color_space?: InterpolationColorSpace): Gradient => ({ kind: "Radial", position, shape, stops, color_space }),
};
/** bevy_ui's ConicGradient: from `start` radians (0 up, clockwise) round `position`. @bevy bevy_ui::gradients::ConicGradient */
export const ConicGradient = {
  new: (position: UiPosition, stops: AngularColorStop[], start = 0, color_space?: InterpolationColorSpace): Gradient => ({ kind: "Conic", start, position, stops, color_space }),
};

/** A UiPosition as CSS's: each axis from its anchor's edge or the centre, as UiPosition::resolve places it. */
function positionCss(who: string, p: UiPosition): string {
  if (!p || !Array.isArray(p.anchor)) throw new TypeError(`${who}: a gradient's position is a UiPosition (UiPosition.CENTER), got ${JSON.stringify(p)}`);
  const axis = (a: number, v: Val, k: string) => {
    if (!(Number.isFinite(a) && a >= -0.5 && a <= 0.5)) throw new RangeError(`${who}: a UiPosition's anchor is -0.5 to 0.5 on each axis, got ${a}`);
    const off = checkVal(who, `position.${k}`, v === "auto" ? "0px" : v);
    return `calc(${50 + a * 100}% ${a > 0 ? "-" : "+"} ${off})`;
  };
  return `${axis(p.anchor[0], p.x, "x")} ${axis(p.anchor[1], p.y, "y")}`;
}

/** Stops as CSS's: each colour at its point, and a hint other than half way as CSS's midpoint between two points. */
function stopsCss(who: string, stops: (ColorStop | AngularColorStop)[], at: (s: ColorStop | AngularColorStop) => string | undefined): string {
  if (!Array.isArray(stops) || stops.length === 0) throw new TypeError(`${who}: a gradient has stops, got ${JSON.stringify(stops)}`);
  const out: string[] = [];
  stops.forEach((s, i) => {
    const p = at(s);
    out.push(p === undefined ? colorCss(who, "stop", s.color) : `${colorCss(who, "stop", s.color)} ${p}`);
    const h = s.hint ?? 0.5;
    if (i + 1 < stops.length && h !== 0.5) {
      const q = at(stops[i + 1]);
      if (!(Number.isFinite(h) && h > 0 && h < 1) || p === undefined || q === undefined) throw new TypeError(`${who}: a stop's hint is above 0 and below 1, between two stops with points, got ${h}`);
      out.push(`calc(${p} + (${q} - ${p}) * ${h})`);
    }
  });
  return out.join(", ");
}

const space = (who: string, g: { color_space?: InterpolationColorSpace }) => {
  const s = SPACE[g.color_space ?? "Oklaba"];
  if (!s) throw new TypeError(`${who}: color_space is one of ${Object.keys(SPACE).join(", ")}, got ${JSON.stringify(g.color_space)}`);
  return s;
};

function shapeCss(who: string, s: RadialGradientShape): string {
  if (s === "ClosestSide") return "circle closest-side";
  if (s === "FarthestSide") return "circle farthest-side";
  if (s === "ClosestCorner") return "ellipse closest-side";
  if (s === "FarthestCorner") return "ellipse farthest-side";
  if (typeof s === "object" && s && "Circle" in s) {
    const r = checkVal(who, "shape.Circle", s.Circle);
    if (r.endsWith("%")) throw new TypeError(`${who}: a Circle's radius in percent (of the node's width) is to do; give px, vw or vh`);
    return `circle ${r}`;
  }
  if (typeof s === "object" && s && "Ellipse" in s) return `ellipse ${checkVal(who, "shape.Ellipse", s.Ellipse[0])} ${checkVal(who, "shape.Ellipse", s.Ellipse[1])}`;
  throw new TypeError(`${who}: shape is a RadialGradientShape, got ${JSON.stringify(s)}`);
}

/** One gradient as CSS's. */
function gradientCss(who: string, g: Gradient): string {
  if (g?.kind === "Linear") {
    if (!Number.isFinite(g.angle)) throw new TypeError(`${who}: a LinearGradient's angle is radians, got ${g.angle}`);
    return `linear-gradient(${g.angle}rad ${space(who, g)}, ${stopsCss(who, g.stops, (s) => (s as ColorStop).point && checkVal(who, "stop.point", (s as ColorStop).point))})`;
  }
  if (g?.kind === "Radial") return `radial-gradient(${shapeCss(who, g.shape)} at ${positionCss(who, g.position)} ${space(who, g)}, ${stopsCss(who, g.stops, (s) => (s as ColorStop).point && checkVal(who, "stop.point", (s as ColorStop).point))})`;
  if (g?.kind === "Conic") {
    if (!Number.isFinite(g.start)) throw new TypeError(`${who}: a ConicGradient's start is radians, got ${g.start}`);
    return `conic-gradient(from ${g.start}rad at ${positionCss(who, g.position)} ${space(who, g)}, ${stopsCss(who, g.stops, (s) => ((s as AngularColorStop).angle === undefined ? undefined : `${(s as AngularColorStop).angle}rad`))})`;
  }
  throw new TypeError(`${who}: a gradient is a LinearGradient, RadialGradient or ConicGradient, got ${JSON.stringify(g)}`);
}

/** BackgroundGradient as CSS background images: Bevy draws the list's first at the bottom, CSS its last. */
export function gradientsCss(who: string, gs: Gradient[]): string {
  if (!Array.isArray(gs)) throw new TypeError(`${who}: background_gradient is a list of gradients, got ${JSON.stringify(gs)}`);
  return gs.map((g) => gradientCss(who, g)).reverse().join(", ");
}
