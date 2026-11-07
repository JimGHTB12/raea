// 2D over the 3D: sprites and flat shapes, drawn over the Canvas3D's 3D by a 2D camera of its own (raea-sys-sprite).
import { useId } from "react";
import { node3d } from "./core";
import { usePointer2d, type Flat, type PointerHandlers3d } from "../events3d";

/** Where a 2D part is: composition px from the frame's top left, y down (as the page's), its centre there; and its
 * pointer and drag handlers, called as a 3D part's are (2D parts are picked first, the top one by z). */
export interface Place2dProps extends PointerHandlers3d {
  name?: string;
  x?: number;
  y?: number;
  /** radians, clockwise (as CSS rotate) */
  rotation?: number;
  /** 1 by default; [x, y] to stretch */
  scale?: number | [number, number];
  /** draw order among the 2D parts: higher draws over lower (0 by default) */
  z?: number;
  /** 0 to 1 (1 by default) */
  opacity?: number;
}

export interface SpriteProps extends Place2dProps {
  /** a picture in the media folder: staticFile("hero.png") */
  src: string;
  /** size in px (the picture's, or one cell's, by default); one given, the other follows the picture's shape */
  width?: number;
  height?: number;
  /** a sprite sheet: the picture is `[columns, rows]` cells of equal size, read left to right, top to bottom */
  grid?: [number, number];
  /** with `grid`: which cell shows (0 is the top left); animate it to play the sheet: frame={Math.floor(f / 3)} */
  frame?: number;
  flipX?: boolean;
  flipY?: boolean;
  /** a tint multiplied over the picture (white by default) */
  color?: string;
  /** Bevy's SpriteImageMode::Tiled: the picture repeats across the sprite's width (tileX) or height (tileY) instead of
   * stretching to it, a tile every `tileStretch` x the picture's size (1 by default): a floor, a fence, a wall. Animate
   * width and height to show more or fewer tiles */
  tileX?: boolean;
  tileY?: boolean;
  tileStretch?: number;
}

/** A picture in 2D, over the 3D: Bevy's Sprite, drawn as a quad with its sprite material, a sprite sheet's cells cut
 * by a texture atlas (raea-sys-sprite). Put it inside the Canvas3D.
 * @bevy bevy_sprite::sprite::Sprite
 * @bevy bevy_sprite_render::sprite_mesh::sprite_mesh_material::SpriteMeshMaterial
 * @bevy bevy_sprite_render::mesh2d::material::MeshMaterial2d<bevy_sprite_render::sprite_mesh::sprite_mesh_material::SpriteMeshMaterial>
 * @bevy bevy_image::texture_atlas::TextureAtlasLayout */
export function Sprite(props: SpriteProps) {
  if (typeof props.src !== "string" || props.src === "") throw new TypeError("Sprite: src must be a picture (staticFile(\"x.png\"))");
  const gr = props.grid;
  if (gr !== undefined && !(Array.isArray(gr) && gr.length === 2 && gr.every((n) => Number.isInteger(n) && n >= 1))) throw new TypeError(`Sprite: grid is [columns, rows], two whole numbers of 1 or more, got ${JSON.stringify(gr)}`);
  if (props.frame !== undefined && !(Number.isFinite(props.frame) && props.frame >= 0)) throw new RangeError(`Sprite: frame is a cell of the grid, 0 or more, got ${props.frame}`);
  for (const k of ["width", "height"] as const) if (props[k] !== undefined && !(Number.isFinite(props[k]) && props[k] > 0)) throw new RangeError(`Sprite: ${k} is px above 0, got ${props[k]}`);
  if (props.tileStretch !== undefined && !(Number.isFinite(props.tileStretch) && props.tileStretch > 0)) throw new RangeError(`Sprite: tileStretch is above 0, got ${props.tileStretch}`);
  if (props.tileStretch !== undefined && !props.tileX && !props.tileY) throw new TypeError("Sprite: tileStretch is for a tiled sprite: give tileX or tileY");
  check2d("Sprite", props);
  const id = useId();
  const handled = Object.keys(props).some((k) => /^on[A-Z]/.test(k));
  if (handled && (props.width === undefined || props.height === undefined)) throw new TypeError("Sprite: give width and height to pick it (an on* handler needs its size, which the picture alone doesn't give here)");
  usePointer2d(props.name ?? id, flatOf(props, props.width ?? 0, props.height ?? 0, false), props as unknown as Record<string, unknown>);
  return node3d("Sprite", props as unknown as Record<string, unknown>);
}

export interface Shape2DProps extends Place2dProps {
  /** a rect by default; a polygon has `sides` (5 by default) */
  shape?: "rect" | "circle" | "ellipse" | "triangle" | "polygon";
  /** [width, height] in px ([100, 100] by default); a circle's and a polygon's diameter is the width */
  size?: [number, number];
  sides?: number;
  /** a CSS colour (white by default) */
  color?: string;
}

/** A flat shape in 2D, over the 3D, in one colour: Bevy's Mesh2d with a ColorMaterial (raea-sys-sprite). Put it
 * inside the Canvas3D.
 * @bevy bevy_sprite_render::mesh2d::color_material::ColorMaterial
 * @bevy bevy_sprite_render::mesh2d::material::MeshMaterial2d<bevy_sprite_render::mesh2d::color_material::ColorMaterial>
 * @bevy bevy_mesh::components::Mesh2d */
export function Shape2D(props: Shape2DProps) {
  const shapes = ["rect", "circle", "ellipse", "triangle", "polygon"];
  if (props.shape !== undefined && !shapes.includes(props.shape)) throw new TypeError(`Shape2D: shape is one of ${shapes.join(", ")}, got ${JSON.stringify(props.shape)}`);
  const s = props.size;
  if (s !== undefined && !(Array.isArray(s) && s.length === 2 && s.every((n) => Number.isFinite(n) && n > 0))) throw new TypeError(`Shape2D: size is [width, height], two numbers above 0, got ${JSON.stringify(s)}`);
  if (props.sides !== undefined && !(Number.isInteger(props.sides) && props.sides >= 3)) throw new RangeError(`Shape2D: sides is a whole number of 3 or more, got ${props.sides}`);
  check2d("Shape2D", props);
  const id = useId();
  const [w, h] = props.size ?? [100, 100];
  const round = props.shape === "circle" || props.shape === "ellipse" || props.shape === "polygon";
  // a circle's and a polygon's diameter is the width; a polygon is picked as its circle
  usePointer2d(props.name ?? id, flatOf(props, w, props.shape === "circle" || props.shape === "polygon" ? w : h, round), props as unknown as Record<string, unknown>);
  return node3d("Shape2D", props as unknown as Record<string, unknown>);
}

/** A 2D part's place for picking: its size times its scale, about its centre. */
function flatOf(p: Place2dProps, w: number, h: number, round: boolean): Flat {
  const [sx, sy] = Array.isArray(p.scale) ? p.scale : [p.scale ?? 1, p.scale ?? 1];
  return { x: p.x ?? 0, y: p.y ?? 0, w: w * sx, h: h * sy, rotation: p.rotation ?? 0, z: p.z ?? 0, round };
}

function check2d(owner: string, p: Place2dProps): void {
  for (const k of ["x", "y", "rotation", "z"] as const) if (p[k] !== undefined && !Number.isFinite(p[k])) throw new RangeError(`${owner}: ${k} must be a number, got ${p[k]}`);
  if (p.opacity !== undefined && !(Number.isFinite(p.opacity) && p.opacity >= 0 && p.opacity <= 1)) throw new RangeError(`${owner}: opacity is 0 to 1, got ${p.opacity}`);
  const sc = p.scale;
  if (sc !== undefined && !(Number.isFinite(sc) || (Array.isArray(sc) && sc.length === 2 && sc.every(Number.isFinite)))) throw new TypeError(`${owner}: scale is a number or [x, y], got ${JSON.stringify(sc)}`);
}
