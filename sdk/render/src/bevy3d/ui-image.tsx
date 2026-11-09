// bevy_ui's ImageNode, drawn by the page: a picture in a Node's box, one cell of a sheet (TextureAtlas), or cut in
// nine (NodeImageMode::Sliced), each sampled as ImagePlugin's default sampler or its own says.
import { Children, createContext, useContext, type CSSProperties, type ReactNode } from "react";
import { InUi, UiScaleCx, VIEWPORT, boxOf, nodeCss, type NodeProps } from "./ui";
import { over, px } from "./ui-val";
import { LinearRgba, Srgba } from "./colors";
import { samplerSpec, type SamplerSpec } from "../3d/materials";

type Color = Srgba | LinearRgba | string;

export interface ImageNodeProps extends NodeProps {
  /** the picture: staticFile("branding/icon.png"), or one with its own sampler ({ path, sampler: { filter: "nearest" } }, Bevy's load_with_settings) */
  image: string | { path: string; sampler: SamplerSpec };
  /** its tint: white (none) by default; only white or a grey multiplies here yet (a picture's brightness) */
  color?: Srgba;
  flip_x?: boolean;
  flip_y?: boolean;
  /** TextureAtlas: the one cell of a sheet laid out by a TextureAtlasLayout to draw (the picture is the sheet) */
  texture_atlas?: { layout: TextureAtlasLayout; index: number };
  /** NodeImageMode: "Auto" (the default) sizes the node by the picture and draws it contained in the content box,
   *  its aspect kept; "Stretch" gives the node no size of its own and stretches the picture over its content box;
   *  Sliced cuts the picture in nine by a TextureSlicer (its corners kept, its sides and centre stretched or tiled),
   *  drawn as CSS border-image with bevy_ui's scale; a sliced ImageNode holds children */
  image_mode?: "Auto" | "Stretch" | { Sliced: TextureSlicer };
}

/** bevy_ui's SliceScaleMode: a side or the centre stretched, or tiled every slice x stretch_value px from its start */
export type SliceScaleMode = "Stretch" | { Tile: { stretch_value: number } };

/** bevy_sprite's TextureSlicer: the slices' insets in the picture's px, how the sides and centre scale (Stretch by
 *  default), and the corners' most scale (1 by default) */
export interface TextureSlicer {
  /** the slices' insets, in the picture's px */
  border: { left: number; right: number; top: number; bottom: number };
  /** how the centre scales (Stretch) */
  center_scale_mode?: SliceScaleMode;
  /** how the sides scale (Stretch) */
  sides_scale_mode?: SliceScaleMode;
  /** how much the corners may scale at most (1) */
  max_corner_scale?: number;
}

/** bevy_image's TextureAtlasLayout: a sheet's size and its cells' rects in px. @bevy bevy_image::TextureAtlasLayout */
export interface TextureAtlasLayout { size: [number, number]; textures: { x: number; y: number; w: number; h: number }[] }
export const TextureAtlasLayout = {
  /** cells of `tile` px, `columns` by `rows`, row by row from the top left; padding between cells and an offset
   *  before the first, as Bevy's from_grid has them (none by default) */
  from_grid(tile: [number, number], columns: number, rows: number, padding?: [number, number], offset?: [number, number]): TextureAtlasLayout {
    const [w, h] = tile, [px_, py] = padding ?? [0, 0], [ox, oy] = offset ?? [0, 0];
    for (const n of [w, h, columns, rows]) if (!(Number.isInteger(n) && n > 0)) throw new RangeError(`TextureAtlasLayout.from_grid: tile, columns and rows are whole numbers above 0, got ${JSON.stringify([tile, columns, rows])}`);
    const textures = [];
    for (let r = 0; r < rows; r++) for (let c = 0; c < columns; c++) textures.push({ x: ox + c * (w + px_), y: oy + r * (h + py), w, h });
    // (Bevy's size: (tile + padding) * grid - padding, its padding between cells only; not the offset)
    return { size: [columns * (w + px_) - px_, rows * (h + py) - py], textures };
  },
};

/** bevy_ui's ImageNode: a Node showing a picture stretched over its content box (inside its padding and border),
 *  sized by the picture when its width or height is auto, as bevy_ui's ImageNodeSize measures it. No children (an
 *  <img>). @bevy bevy_ui::widget::image::ImageNode */
export function ImageNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, image_mode, ...p }: ImageNodeProps) {
  // a picture with its own sampler: its filter is how it's drawn here (a UI picture is never wrapped or mipmapped)
  const outer = useContext(Sampler);
  if (typeof image === "object" && image !== null) {
    const s = samplerSpec("ImageNode: image", image.sampler);
    if (s.wrap !== undefined || s.anisotropy !== undefined) throw new TypeError("ImageNode: an image's sampler takes filter (and mipmap) here; a UI picture isn't wrapped");
    const filter = s.filter === "nearest" ? "Nearest" : s.filter === "linear" ? "Linear" : outer;
    return <Sampler.Provider value={filter}><ImageNode image={image.path} color={color} flip_x={flip_x} flip_y={flip_y} fixed={fixed} texture_atlas={texture_atlas} image_mode={image_mode} {...p}>{children}</ImageNode></Sampler.Provider>;
  }
  if (image_mode !== undefined && image_mode !== "Auto" && image_mode !== "Stretch" && typeof image_mode?.Sliced !== "object") throw new TypeError(`ImageNode: image_mode is "Auto", "Stretch" or { Sliced }, got ${JSON.stringify(image_mode)}`);
  if (typeof image_mode === "object") return SlicedNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, image_mode, ...p });
  if (texture_atlas !== undefined) return AtlasNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, ...p });
  if (typeof image !== "string" || image === "") throw new TypeError(`ImageNode: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  if (children !== undefined && children !== null && children !== false) throw new TypeError("ImageNode: takes no children here (a picture on a Node with children: put the ImageNode beside them)");
  const grey = color === undefined || (color.red === color.green && color.green === color.blue);
  if (!grey || (color && color.alpha !== 1)) throw new TypeError("ImageNode: color tints only by an opaque grey yet (brightness); a coloured or translucent tint is to do");
  const ui = useContext(InUi);
  const n = nodeCss("ImageNode", p, ui?.back);
  const flips = [flip_x && "scaleX(-1)", flip_y && "scaleY(-1)"].filter(Boolean).join(" ");
  const rendering = RENDERING[useContext(Sampler)];
  const look: CSSProperties = { ...(rendering ? { imageRendering: rendering } : {}), ...(flips ? { transform: flips } : {}), ...(color && color.red !== 1 ? { filter: `brightness(${color.red})` } : {}) };
  // Auto: contained, its aspect kept, as bevy_ui draws it; Stretch: no picture size for layout, the picture over the
  // content box as a background
  const img = image_mode === "Stretch"
    ? <div style={{ ...n.css, ...look, backgroundImage: `url("${image}")`, backgroundSize: "100% 100%", backgroundOrigin: "content-box", backgroundClip: "content-box", backgroundRepeat: "no-repeat" }} {...n.attrs} />
    : <img src={image} style={{ ...n.css, objectFit: "contain", ...look }} {...n.attrs} />;
  const root = fixed ? "fixed" : ui === null;
  return root ? <div style={root === "fixed" ? { ...VIEWPORT, position: "fixed" } : VIEWPORT}>{img}</div> : img;
}

// an ImageNode of one atlas cell: the sheet scaled so the cell fills the content box (Bevy's UVs are the cell's rect
// over the layout's size), the rest clipped; sized by the cell when its width and height are auto, as Bevy measures it
function AtlasNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, ...p }: ImageNodeProps) {
  if (typeof image !== "string" || image === "") throw new TypeError(`ImageNode: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  if (children !== undefined && children !== null && children !== false) throw new TypeError("ImageNode: takes no children here (a picture on a Node with children: put the ImageNode beside them)");
  const { layout, index } = texture_atlas!;
  const cell = layout?.textures?.[index];
  if (!cell) throw new RangeError(`ImageNode: texture_atlas.index ${index} is not one of its layout's ${layout?.textures?.length ?? 0} cells`);
  if (color !== undefined && !(color.red === color.green && color.green === color.blue && color.alpha === 1)) throw new TypeError("ImageNode: color tints only by an opaque grey yet (brightness); a coloured or translucent tint is to do");
  const ui = useContext(InUi);
  const n = nodeCss("ImageNode", { width: px(cell.w), height: px(cell.h), ...p }, ui?.back);
  const flips = [flip_x && "scaleX(-1)", flip_y && "scaleY(-1)"].filter(Boolean).join(" ");
  const pct = (a: number, b: number) => `${(a / b) * 100}%`;
  const rendering = RENDERING[useContext(Sampler)];
  const sheet: CSSProperties = {
    position: "absolute", maxWidth: "none", ...(rendering ? { imageRendering: rendering } : {}),
    left: pct(-cell.x, cell.w), top: pct(-cell.y, cell.h), width: pct(layout.size[0], cell.w), height: pct(layout.size[1], cell.h),
    ...(color && color.red !== 1 ? { filter: `brightness(${color.red})` } : {}),
  };
  const box: CSSProperties = { position: "relative", overflow: "hidden", width: "100%", height: "100%", ...(flips ? { transform: flips } : {}) };
  const node = <div style={n.css} {...n.attrs}><div style={box}><img src={image} style={sheet} /></div></div>;
  const root = fixed ? "fixed" : ui === null;
  return root ? <div style={root === "fixed" ? { ...VIEWPORT, position: "fixed" } : VIEWPORT}>{node}</div> : node;
}

// ImagePlugin's default sampler, for the pictures inside it (a <Node> tree's ImageNodes)
const Sampler = createContext<"Linear" | "Nearest">("Linear");
const RENDERING = { Linear: undefined, Nearest: "pixelated" } as const;

export interface ImagePluginProps {
  /** "Linear" (the default) or "Nearest" (pixel art) */
  default_sampler?: "Linear" | "Nearest";
  /** the UI and sprites whose pictures it samples */
  children?: ReactNode;
}

/** bevy_image's ImagePlugin: its default_sampler for every picture drawn inside it (Linear by default;
 *  ImagePlugin::default_nearest() is default_sampler="Nearest", for pixel art). @bevy bevy_image::ImagePlugin */
export function ImagePlugin({ default_sampler, children }: ImagePluginProps) {
  if (default_sampler !== undefined && !(default_sampler in RENDERING)) throw new TypeError(`ImagePlugin: default_sampler is Linear or Nearest, got ${JSON.stringify(default_sampler)}`);
  return <Sampler.Provider value={default_sampler ?? "Linear"}>{children}</Sampler.Provider>;
}

// an ImageNode cut in nine (NodeImageMode::Sliced): CSS border-image with its slices and `fill`, and bevy_ui's own
// corner scale and tiling on data-raea-bevy-slice (raea's Blitz, render/border_image.rs). It holds children, as a
// sliced button's label. A flipped one draws on a layer behind them (z-index -1 in the node's own stacking context,
// over its background, as Bevy draws the picture), mirrored there, so the flip doesn't turn the label.
function SlicedNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, image_mode, ...p }: ImageNodeProps) {
  if (typeof image !== "string" || image === "") throw new TypeError(`ImageNode: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  const sl = typeof image_mode === "object" ? image_mode.Sliced : undefined;
  const b = sl?.border;
  if (!sl || !b || ![b.left, b.right, b.top, b.bottom].every((v) => Number.isFinite(v) && v >= 0)) throw new TypeError(`ImageNode: image_mode is { Sliced: { border: { left, right, top, bottom } } } (px >= 0), got ${JSON.stringify(image_mode)}`);
  // a sliced atlas cell: its rect in the layout, which Blitz scales to the sheet as Bevy's atlas UVs are
  let cellOf = "";
  if (texture_atlas !== undefined) {
    const { layout, index } = texture_atlas;
    const c = layout?.textures?.[index];
    if (!c) throw new RangeError(`ImageNode: texture_atlas.index ${index} is not one of its layout's ${layout?.textures?.length ?? 0} cells`);
    cellOf = ` cell ${c.x} ${c.y} ${c.w} ${c.h} of ${layout.size[0]} ${layout.size[1]}`;
  }
  if (color !== undefined && !(color.red === 1 && color.green === 1 && color.blue === 1 && color.alpha === 1)) throw new TypeError("ImageNode: a sliced picture is untinted here yet (color white)");
  const max = sl.max_corner_scale ?? 1;
  if (!(Number.isFinite(max) && max > 0)) throw new RangeError(`ImageNode: max_corner_scale is above 0, got ${max}`);
  // (lengths in the page's px: under a UiScale's zoom, times it, as Bevy scales them by the scale factor)
  const scale = useContext(UiScaleCx);
  const mode = (m: SliceScaleMode | undefined, which: string) => {
    if (m === undefined || m === "Stretch") return "s";
    const v = typeof m === "object" ? m.Tile?.stretch_value : undefined;
    if (!(typeof v === "number" && Number.isFinite(v) && v > 0)) throw new TypeError(`ImageNode: ${which} is "Stretch" or { Tile: { stretch_value } } (above 0), got ${JSON.stringify(m)}`);
    return `t${v * scale}`;
  };
  const marker = { "data-raea-bevy-slice": `${max * scale} ${mode(sl.sides_scale_mode, "sides_scale_mode")} ${mode(sl.center_scale_mode, "center_scale_mode")}${cellOf}` };
  const ui = useContext(InUi);
  const n = nodeCss("ImageNode", p, ui?.back);
  const rendering = RENDERING[useContext(Sampler)];
  const slices: CSSProperties = {
    borderImageSource: `url("${image}")`,
    borderImageSlice: `${b.top} ${b.right} ${b.bottom} ${b.left} fill`,
    // CSS's own widths (the slices at max_corner_scale); Blitz scales them as bevy_ui does by the marker
    borderImageWidth: `${b.top * max}px ${b.right * max}px ${b.bottom * max}px ${b.left * max}px`,
    ...(rendering ? { imageRendering: rendering } : {}),
  };
  const root = fixed ? "fixed" : ui === null;
  if (!flip_x && !flip_y) return boxOf({ ...n.css, ...slices }, n.back, children, root, undefined, { ...n.attrs, ...marker });
  const flips = [flip_x && "scaleX(-1)", flip_y && "scaleY(-1)"].filter(Boolean).join(" ");
  const layer = <div key="raea-slices" style={{ position: "absolute", left: 0, top: 0, width: "100%", height: "100%", zIndex: -1, transform: flips, ...slices }} {...marker} />;
  return boxOf({ ...n.css, position: n.css.position ?? "relative" }, n.back, [layer, ...Children.toArray(children)], root, undefined, n.attrs);
}

/** bevy_ui's InlineImage: a picture in a <Text>'s line, as an inline box on its baseline (bevy_text's parley lays it
 *  out so, and Blitz's parley here), `width` and `height` px (the other by its aspect, or its own size), tinted by
 *  `color` (each pixel times it: the picture multiplied over the tint, its own alpha the mask).
 * @bevy bevy_ui::widget::InlineImage */
export function InlineImage({ image, color, width, height }: { image: string; color?: Color; width?: number; height?: number }) {
  if (typeof image !== "string" || image === "") throw new TypeError(`InlineImage: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  for (const [k, v] of [["width", width], ["height", height]] as const) if (v !== undefined && !(Number.isFinite(v) && v > 0)) throw new RangeError(`InlineImage: ${k} is px above 0, got ${v}`);
  const url = `url("${image}")`;
  // the tint opaque, multiplied under the picture, and its alpha the box's opacity (each pixel's alpha times it)
  const s = color instanceof LinearRgba ? color.to_srgba() : color instanceof Srgba ? color : undefined;
  if (color !== undefined && s === undefined) throw new TypeError(`InlineImage: color is an Srgba or LinearRgba, got ${JSON.stringify(color)}`);
  const tint = s && over("InlineImage", "color", new Srgba(s.red, s.green, s.blue, 1), undefined).css;
  const tinted: CSSProperties = tint === undefined ? {} : { backgroundImage: `${url}, linear-gradient(${tint}, ${tint})`, backgroundSize: "100% 100%", backgroundBlendMode: "multiply", maskImage: url, maskSize: "100% 100%" };
  if (s && s.alpha < 1) tinted.opacity = Math.max(0, s.alpha);
  return (
    <span style={{ display: "inline-block", ...tinted }}>
      <img src={image} style={{ display: "block", width: width === undefined ? undefined : `${width}px`, height: height === undefined ? undefined : `${height}px`, opacity: tint === undefined ? 1 : 0 }} />
    </span>
  );
}
