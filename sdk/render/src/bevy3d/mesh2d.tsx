// Bevy's Mesh2d (docs/sdk.feature S2a): a 2D mesh of one of bevy_math's primitives (Circle, Rectangle, Ellipse,
// RegularPolygon) with a ColorMaterial, placed by its Transform as Bevy's 2D camera sees it: the frame's centre is the
// origin, +Y up, a unit a pixel, turns counterclockwise. It draws as <Shape2D> does (raea-sys-sprite).
import { Children, isValidElement, type ReactNode } from "react";
import { useVideoConfig } from "@raea/app/clock";
import { Shape2D, type Place2dProps, type Shape2DProps } from "../3d/sprite";
import { eulerOf } from "../3d";
import { type TransformValue } from "./all";

export interface CircleProps { radius?: number }
export interface RectangleProps { half_size?: [number, number] }
export interface EllipseProps { half_size?: [number, number] }
export interface RegularPolygonProps { circumcircle?: { radius: number }; sides?: number }
export interface ColorMaterialProps { color?: string }

const inMesh2d = (name: string) => (): ReactNode => {
  throw new TypeError(`${name} goes in a Mesh2d: <Mesh2d><${name} /><ColorMaterial color="#c84" /></Mesh2d>`);
};
/** bevy_math's Circle (radius 0.5 by default, as Bevy's) */
export const Circle = inMesh2d("Circle") as (p: CircleProps) => ReactNode;
/** bevy_math's Rectangle (half_size [0.5, 0.5] by default) */
export const Rectangle = inMesh2d("Rectangle") as (p: RectangleProps) => ReactNode;
/** bevy_math's Ellipse (half_size [1, 0.5] by default) */
export const Ellipse = inMesh2d("Ellipse") as (p: EllipseProps) => ReactNode;
/** bevy_math's RegularPolygon (circumradius 0.5, 6 sides by default) */
export const RegularPolygon = inMesh2d("RegularPolygon") as (p: RegularPolygonProps) => ReactNode;
/** bevy_sprite_render's ColorMaterial: one colour (white by default) */
export const ColorMaterial = inMesh2d("ColorMaterial") as (p: ColorMaterialProps) => ReactNode;

export interface Mesh2dProps extends Omit<Place2dProps, "x" | "y" | "rotation" | "scale" | "z"> {
  transform?: TransformValue;
  visibility?: "Inherited" | "Visible" | "Hidden";
  /** one primitive (<Circle>, <Rectangle>, <Ellipse>, <RegularPolygon>) and a <ColorMaterial> */
  children?: ReactNode;
}

/** The primitive's Shape2D: its shape, size in px and sides. */
function shapeOf(el: { type: unknown; props: unknown }): Pick<Shape2DProps, "shape" | "size" | "sides"> {
  if (el.type === Circle) {
    const d = 2 * ((el.props as CircleProps).radius ?? 0.5);
    return { shape: "circle", size: [d, d] };
  }
  if (el.type === Rectangle || el.type === Ellipse) {
    const [hx, hy] = (el.props as RectangleProps).half_size ?? (el.type === Rectangle ? [0.5, 0.5] : [1, 0.5]);
    return { shape: el.type === Rectangle ? "rect" : "ellipse", size: [2 * hx, 2 * hy] };
  }
  const p = el.props as RegularPolygonProps;
  const d = 2 * (p.circumcircle?.radius ?? 0.5);
  return { shape: "polygon", size: [d, d], sides: p.sides ?? 6 };
}

/** Bevy's Mesh2d with its Transform, a primitive and a ColorMaterial: <Shape2D> in Bevy's 2D world. */
export function Mesh2d({ transform, visibility, children, ...rest }: Mesh2dProps) {
  const { width, height } = useVideoConfig();
  let shape: Pick<Shape2DProps, "shape" | "size" | "sides"> | undefined, color: string | undefined;
  for (const c of Children.toArray(children)) {
    if (!isValidElement(c)) continue;
    if (c.type === ColorMaterial) color = (c.props as ColorMaterialProps).color ?? "#ffffff";
    else if (c.type === Circle || c.type === Rectangle || c.type === Ellipse || c.type === RegularPolygon) {
      if (shape) throw new TypeError("Mesh2d: one primitive");
      shape = shapeOf(c);
    } else throw new TypeError("Mesh2d takes a primitive (<Circle>, <Rectangle>, <Ellipse>, <RegularPolygon>) and a <ColorMaterial>");
  }
  if (!shape) throw new TypeError("Mesh2d: give it a primitive, e.g. <Circle radius={40} />");
  if (visibility === "Hidden") return null;
  const [tx, ty, tz] = transform?.translation ?? [0, 0, 0];
  const [sx, sy] = transform?.scale ?? [1, 1, 1];
  // Bevy turns counterclockwise about +Z; Shape2D's rotation is clockwise, as the page's
  const turn = transform?.rotation ? -eulerOf(transform.rotation)[2] : 0;
  return <Shape2D {...rest} {...shape} color={color ?? "#ffffff"} x={width / 2 + tx} y={height / 2 - ty} z={tz} rotation={turn} scale={sx === sy ? sx : [sx, sy]} />;
}
