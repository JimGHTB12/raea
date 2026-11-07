// bevy_ui's Node, Text and TextSpan, drawn by the page: each a box whose CSS gives Blitz's taffy the style bevy_ui's
// convert.rs gives Bevy's, with BackgroundColor, BorderColor, ZIndex and Outline as its props. A Node outside another
// is a root: bevy_ui lays each root in an implicit viewport of its own (a grid the size of the window, its item at the
// start), so a root's box sits in one. Text is Bevy's: its default font (Bevy's FiraMono subset), 20 px (bevy_text's
// rem), white, line height 1.2, wrapped at words, drawn unsmoothed (bevy_text's coverage, not CoreText's growth).
import { Children, createContext, useContext, type CSSProperties, type ReactNode } from "react";
import { BEVY_FONT_FAMILY, bevyFontFace } from "./fonts/default";
import {
  ALIGN_CONTENT, ALIGN_ITEMS, ALIGN_SELF, DISPLAY, FLEX_DIRECTION, FLEX_WRAP, JUSTIFY_CONTENT, JUSTIFY_ITEMS, JUSTIFY_SELF, OVERFLOW_AXIS, POSITION_TYPE,
  checkVal, keyword, over, px, radiusCss, rectCss, type Backdrop, type BorderRadius, type Overflow, type OverflowClipMargin, type UiRect, type Val,
} from "./ui-val";
import type { LinearRgba, Srgba } from "./colors";
import type { TransformValue } from "./core";
import { eulerOf } from "../3d/model";
import { samplerSpec, type SamplerSpec } from "../3d/materials";
import { useVideoConfig } from "@raea/app/clock";

type Color = Srgba | LinearRgba | string;

/** bevy_ui's Node: its layout fields as Bevy names them (each Bevy's default when left out), and the components that
 *  draw it: background_color (BackgroundColor), border_color (BorderColor, every side), z_index (ZIndex). */
export interface NodeProps {
  display?: (typeof DISPLAY)[number];
  box_sizing?: "BorderBox" | "ContentBox";
  position_type?: (typeof POSITION_TYPE)[number];
  left?: Val; right?: Val; top?: Val; bottom?: Val;
  width?: Val; height?: Val; min_width?: Val; min_height?: Val; max_width?: Val; max_height?: Val;
  aspect_ratio?: number;
  flex_direction?: (typeof FLEX_DIRECTION)[number];
  flex_wrap?: (typeof FLEX_WRAP)[number];
  flex_grow?: number; flex_shrink?: number; flex_basis?: Val;
  align_items?: (typeof ALIGN_ITEMS)[number];
  align_self?: (typeof ALIGN_SELF)[number];
  align_content?: (typeof ALIGN_CONTENT)[number];
  justify_content?: (typeof JUSTIFY_CONTENT)[number];
  justify_items?: (typeof JUSTIFY_ITEMS)[number];
  justify_self?: (typeof JUSTIFY_SELF)[number];
  margin?: UiRect; padding?: UiRect; border?: UiRect;
  border_radius?: BorderRadius;
  overflow?: Overflow;
  /** the box its overflow clip clips to, grown by a margin (OverflowClipMargin.border_box().with_margin(25)) */
  overflow_clip_margin?: OverflowClipMargin;
  row_gap?: Val; column_gap?: Val;
  background_color?: Color;
  border_color?: Color;
  /** Outline: a line outside the border box, `offset` from it (0 by default) */
  outline?: { width: Val; offset?: Val; color: Color };
  z_index?: number;
  /** FixedNode: laid out as a root wherever it is in the tree (its own viewport, the window's), drawn with its parent */
  fixed?: boolean;
  children?: ReactNode;
}

// inside a Node (null at the top: a Node there is a root), and what lies under its content when known
const InUi = createContext<{ back: Backdrop } | null>(null);

const VALS = ["left", "right", "top", "bottom", "width", "height", "min_width", "min_height", "max_width", "max_height", "flex_basis", "row_gap", "column_gap"] as const;
const CSS_OF: Record<(typeof VALS)[number], keyof CSSProperties> = {
  left: "left", right: "right", top: "top", bottom: "bottom", width: "width", height: "height", min_width: "minWidth", min_height: "minHeight",
  max_width: "maxWidth", max_height: "maxHeight", flex_basis: "flexBasis", row_gap: "rowGap", column_gap: "columnGap",
};

/** A Node's props as the box's CSS: Bevy's defaults made explicit where CSS's differ (display flex, border-box,
 *  position relative, flex-shrink 1, min sizes auto; no font of the page's own), its colours blended as Bevy blends them over
 *  `back`, and the backdrop it leaves its content. */
export function nodeCss(who: string, p: NodeProps, back?: Backdrop): { css: CSSProperties; back: Backdrop } {
  const s: Record<string, unknown> = { display: "flex", boxSizing: "border-box", position: "relative", flexShrink: 1, minWidth: "auto", minHeight: "auto" };
  const kw = (k: keyof NodeProps, css: string, allowed: readonly string[]) => {
    const v = p[k];
    if (v === undefined) return;
    const w = keyword(who, k, v as string, allowed);
    if (w !== undefined) s[css] = w === "auto" ? "auto" : w;
  };
  kw("display", "display", DISPLAY);
  kw("position_type", "position", POSITION_TYPE);
  kw("flex_direction", "flexDirection", FLEX_DIRECTION);
  kw("flex_wrap", "flexWrap", FLEX_WRAP);
  kw("align_items", "alignItems", ALIGN_ITEMS);
  kw("align_self", "alignSelf", ALIGN_SELF);
  kw("align_content", "alignContent", ALIGN_CONTENT);
  kw("justify_content", "justifyContent", JUSTIFY_CONTENT);
  kw("justify_items", "justifyItems", JUSTIFY_ITEMS);
  kw("justify_self", "justifySelf", JUSTIFY_SELF);
  if (p.flex_wrap === "NoWrap") s.flexWrap = "nowrap";
  if (p.box_sizing !== undefined) s.boxSizing = keyword(who, "box_sizing", p.box_sizing, ["BorderBox", "ContentBox"]);
  for (const k of VALS) if (p[k] !== undefined) s[CSS_OF[k]] = checkVal(who, k, p[k]);
  for (const k of ["flex_grow", "flex_shrink", "aspect_ratio", "z_index"] as const) {
    const v = p[k];
    if (v !== undefined && !Number.isFinite(v)) throw new RangeError(`${who}: ${k} is a number, got ${v}`);
  }
  if (p.flex_grow !== undefined) s.flexGrow = p.flex_grow;
  if (p.flex_shrink !== undefined) s.flexShrink = p.flex_shrink;
  if (p.aspect_ratio !== undefined) s.aspectRatio = p.aspect_ratio;
  if (p.z_index !== undefined) s.zIndex = p.z_index;
  if (p.margin !== undefined) s.margin = rectCss(who, "margin", p.margin);
  if (p.padding !== undefined) s.padding = rectCss(who, "padding", p.padding);
  if (p.border !== undefined) {
    s.borderWidth = rectCss(who, "border", p.border);
    s.borderStyle = "solid";
    s.borderColor = "transparent";
  }
  if (p.border_radius !== undefined) s.borderRadius = radiusCss(who, p.border_radius);
  // (the outline is outside the node: over what is under the node, not its own background)
  if (p.outline !== undefined) {
    s.outline = `${checkVal(who, "outline.width", p.outline.width)} solid ${over(who, "outline.color", p.outline.color, back).css}`;
    if (p.outline.offset !== undefined) s.outlineOffset = checkVal(who, "outline.offset", p.outline.offset);
  }
  if (p.background_color !== undefined) ({ css: s.backgroundColor, back } = over(who, "background_color", p.background_color, back));
  // (the border over its own background, as Bevy draws it)
  if (p.border_color !== undefined) s.borderColor = over(who, "border_color", p.border_color, back).css;
  if (p.overflow !== undefined) {
    const o = p.overflow;
    for (const a of ["x", "y"] as const) if (!OVERFLOW_AXIS.includes(o?.[a])) throw new TypeError(`${who}: overflow.${a} is one of ${OVERFLOW_AXIS.join(", ")}, got ${JSON.stringify(o?.[a])}`);
    s.overflowX = o.x.toLowerCase();
    s.overflowY = o.y.toLowerCase();
  }
  const m = p.overflow_clip_margin;
  if (m !== undefined) {
    if (!["BorderBox", "PaddingBox", "ContentBox"].includes(m?.visual_box) || !(Number.isFinite(m.margin) && m.margin >= 0)) throw new TypeError(`${who}: overflow_clip_margin is OverflowClipMargin.border_box(), padding_box() or content_box(), with_margin(px >= 0), got ${JSON.stringify(m)}`);
    // CSS's overflow-clip-margin, which raea's Blitz honours for every clipping overflow, as Bevy does
    (s as Record<string, string>).overflowClipMargin = `${m.visual_box.replace("Box", "").toLowerCase()}-box ${m.margin}px`;
  }
  return { css: s as CSSProperties, back };
}

// a root's implicit viewport (bevy_ui ui_surface.rs): a grid the window's size, its one item at the start of both axes
const VIEWPORT: CSSProperties = { position: "absolute", left: 0, top: 0, width: "100%", height: "100%", display: "grid", alignItems: "start", justifyItems: "start" };

function boxOf(css: CSSProperties, back: Backdrop, children: ReactNode, root: boolean | "fixed", font?: ReactNode, attrs?: Record<string, string>) {
  const inner = (
    <div style={css} {...attrs}>
      <InUi.Provider value={{ back }}>{children}</InUi.Provider>
    </div>
  );
  return root ? <div style={root === "fixed" ? { ...VIEWPORT, position: "fixed" } : VIEWPORT}>{font}{inner}</div> : <>{font}{inner}</>;
}

/** A box of Bevy's UI, laid out as bevy_ui lays it (taffy, with Bevy's defaults). Put it over the 3D in an
 *  <AbsoluteFill> (after the <Canvas3D>): each root fills the frame as Bevy's window. @bevy bevy_ui::ui_node::Node */
export function Node({ children, fixed, ...p }: NodeProps) {
  const ui = useContext(InUi);
  const n = nodeCss("Node", p, ui?.back);
  return boxOf(n.css, n.back, children, fixed ? "fixed" : ui === null);
}

/** bevy_text's TextFont, TextColor, TextLayout and LineHeight, on a Text or a TextSpan. */
export interface TextStyleProps {
  /** a font file (staticFile("fonts/FiraSans-Bold.ttf")); Bevy's default font when left out */
  font?: string;
  /** Bevy's FontSize: a number of px, or { Px }, { Rem } (of bevy_text's 20 px rem), { Vw }, { Vh }, { VMin }, { VMax } */
  font_size?: number | { Px: number } | { Rem: number } | { Vw: number } | { Vh: number } | { VMin: number } | { VMax: number };
  /** FontWeight: 100 to 900 (400 by default) */
  weight?: number;
  style?: "Normal" | "Italic" | "Oblique";
  /** TextColor: white by default */
  color?: Color;
  /** LineHeight: { Px } or { RelativeToFont } (1.2 of the size by default) */
  line_height?: { Px: number } | { RelativeToFont: number };
  /** FontVariations: a variable font's axes by their four-letter tag, { [FontVariationTag.WEIGHT]: 300 } */
  font_variations?: Record<string, number>;
}

/** bevy_text's FontVariationTag: the registered axes' OpenType tags. @bevy bevy_text::FontVariationTag */
export const FontVariationTag = { WEIGHT: "wght", WIDTH: "wdth", SLANT: "slnt", ITALIC: "ital", OPTICAL_SIZE: "opsz" } as const;

function variationsCss(who: string, v: Record<string, number> | undefined): string | undefined {
  if (v === undefined) return undefined;
  const axes = Object.entries(v).map(([tag, n]) => {
    if (!/^[\x20-\x7e]{4}$/.test(tag)) throw new TypeError(`${who}: a font variation's tag is four characters ("wght"), got ${JSON.stringify(tag)}`);
    if (!Number.isFinite(n)) throw new TypeError(`${who}: font variation ${tag} is a number, got ${n}`);
    return `"${tag}" ${n}`;
  });
  return axes.length ? axes.join(", ") : undefined;
}

/** What bevy_text draws with a section's own glyphs (a Text's string, not its spans; a TextSpan's run). */
export interface TextRunProps {
  /** Underline and Strikethrough: a line under or through the run, as thick as the font says */
  underline?: boolean;
  strikethrough?: boolean;
  /** UnderlineColor and StrikethroughColor: the lines' colours (the text's colour by default) */
  underline_color?: Color;
  strikethrough_color?: Color;
  /** TextBackgroundColor: a box behind the run, line by line */
  text_background_color?: Color;
}

export interface TextProps extends TextStyleProps, TextRunProps, NodeProps {
  /** TextLayout's justify (Left by default) and linebreak (WordBoundary) */
  justify?: "Left" | "Center" | "Right" | "Justified" | "Start" | "End";
  linebreak?: "WordBoundary" | "AnyCharacter" | "WordOrCharacter" | "NoWrap";
}

/** A section's glyphs in its lines and background: CSS carries a decoration to every element inside it, so a Text's
 *  own are on its strings alone (Bevy's Strikethrough on a Text leaves its spans unstruck). Two colours of line are
 *  two spans, one per text-decoration-color. */
function runOf(who: string, r: TextRunProps, back: Backdrop | undefined, text: ReactNode): ReactNode {
  const col = (k: "underline_color" | "strikethrough_color" | "text_background_color") => (r[k] === undefined ? undefined : over(who, k, r[k]!, back).css);
  const [uc, sc, bg] = [col("underline_color"), col("strikethrough_color"), col("text_background_color")];
  if (!r.underline && !r.strikethrough && bg === undefined) return text;
  const line = (l: string, c: string | undefined): CSSProperties => ({ textDecorationLine: l, textDecorationThickness: "from-font", ...(c ? { textDecorationColor: c } : {}) });
  const one = r.underline && r.strikethrough && uc === sc;
  const outer: CSSProperties = { ...(bg ? { backgroundColor: bg } : {}), ...(one ? line("underline line-through", uc) : r.underline ? line("underline", uc) : !r.underline && r.strikethrough ? line("line-through", sc) : {}) };
  const struck = r.underline && r.strikethrough && !one;
  return <span style={outer}>{struck ? <span style={line("line-through", sc)}>{text}</span> : text}</span>;
}

const familyOf = (font: string) => `raea-font-${font.replace(/[^A-Za-z0-9]/g, "-")}`;

function sizeCss(who: string, f: TextStyleProps["font_size"]): string {
  if (f === undefined) return "20px";
  if (typeof f === "number") return `${checkSize(who, f)}px`;
  const [k, v] = Object.entries(f)[0] ?? [];
  const unit = { Px: "px", Rem: "rem", Vw: "vw", Vh: "vh", VMin: "vmin", VMax: "vmax" }[k as string];
  if (!unit || typeof v !== "number") throw new TypeError(`${who}: font_size is a number of px or { Px | Rem | Vw | Vh | VMin | VMax }, got ${JSON.stringify(f)}`);
  return unit === "rem" ? `${checkSize(who, v) * 20}px` : `${checkSize(who, v)}${unit}`;
}
function checkSize(who: string, n: number) {
  if (!(n > 0 && Number.isFinite(n))) throw new RangeError(`${who}: font_size is above 0, got ${n}`);
  return n;
}

/** A text style's CSS (every field Bevy's default when left out: a span takes nothing from its Text), and the
 *  @font-face its font needs. */
export function textCss(who: string, p: TextStyleProps, back?: Backdrop): { css: CSSProperties; face: string } {
  const family = p.font === undefined ? BEVY_FONT_FAMILY : familyOf(p.font);
  if (p.font !== undefined && (typeof p.font !== "string" || p.font === "")) throw new TypeError(`${who}: font is a font file (staticFile("fonts/x.ttf")), got ${JSON.stringify(p.font)}`);
  if (p.weight !== undefined && !(Number.isInteger(p.weight) && p.weight >= 1 && p.weight <= 1000)) throw new RangeError(`${who}: weight is 1 to 1000, got ${p.weight}`);
  const lh = p.line_height;
  let lineHeight: string | number = 1.2;
  if (lh !== undefined) {
    if ("Px" in lh && Number.isFinite(lh.Px)) lineHeight = `${lh.Px}px`;
    else if ("RelativeToFont" in lh && Number.isFinite(lh.RelativeToFont)) lineHeight = lh.RelativeToFont;
    else throw new TypeError(`${who}: line_height is { Px } or { RelativeToFont }, got ${JSON.stringify(lh)}`);
  }
  const css: CSSProperties = {
    fontFamily: `"${family}"`,
    fontSize: sizeCss(who, p.font_size),
    fontWeight: p.weight ?? 400,
    fontStyle: (p.style ?? "Normal").toLowerCase(),
    color: p.color === undefined ? "#ffffff" : over(who, "color", p.color, back).css,
    lineHeight,
    WebkitFontSmoothing: "antialiased",
  };
  const variations = variationsCss(who, p.font_variations);
  if (variations) css.fontVariationSettings = variations;
  const face = p.font === undefined ? bevyFontFace() : `@font-face { font-family: "${family}"; src: url("${p.font}"); font-weight: 1 1000; }`;
  return { css, face };
}

const BREAK: Record<NonNullable<TextProps["linebreak"]>, CSSProperties> = {
  WordBoundary: { whiteSpace: "pre-wrap" },
  AnyCharacter: { whiteSpace: "pre-wrap", wordBreak: "break-all" },
  WordOrCharacter: { whiteSpace: "pre-wrap", overflowWrap: "anywhere" },
  NoWrap: { whiteSpace: "pre" },
};

/** Bevy's UI text: a Node whose content is its string and its <TextSpan> children, in its TextFont, TextColor,
 *  TextLayout and LineHeight (Bevy's defaults when left out). @bevy bevy_ui::widget::text::Text */
export function Text({ children, font, font_size, weight, style, color, line_height, font_variations, justify, linebreak, underline, strikethrough, underline_color, strikethrough_color, text_background_color, ...node }: TextProps) {
  const ui = useContext(InUi);
  const n = nodeCss("Text", node, ui?.back);
  const t = textCss("Text", { font, font_size, weight, style, color, line_height, font_variations }, n.back);
  if (justify !== undefined && !["Left", "Center", "Right", "Justified", "Start", "End"].includes(justify)) throw new TypeError(`Text: justify is Left, Center, Right, Justified, Start or End, got ${JSON.stringify(justify)}`);
  if (linebreak !== undefined && !(linebreak in BREAK)) throw new TypeError(`Text: linebreak is WordBoundary, AnyCharacter, WordOrCharacter or NoWrap, got ${JSON.stringify(linebreak)}`);
  const textAlign = { Left: "left", Center: "center", Right: "right", Justified: "justify", Start: "start", End: "end" }[justify ?? "Left"] as CSSProperties["textAlign"];
  // the text itself in one block inside the Node, as bevy_text's measured leaf: its lines laid out together.
  // data-raea-bevy-text: Blitz measures the block as bevy_text does, rounded up to whole pixels (38.4 px lines are 39).
  // Its own strings carry its lines and background (runOf), inside the block: Blitz draws a line from the run's
  // inline root down, not from a block above it, as thick as the font says (bevy_text's underline_size, not em/10)
  const run = { underline, strikethrough, underline_color, strikethrough_color, text_background_color };
  const own = Children.map(children, (c) => (typeof c === "string" || typeof c === "number" ? runOf("Text", run, n.back, c) : c));
  const css = { ...n.css, ...t.css, textAlign };
  return boxOf(css, n.back, <span data-raea-bevy-text="" style={BREAK[linebreak ?? "WordBoundary"]}>{own}</span>, ui === null, <style>{t.face}</style>);
}

/** A run of a <Text> in a style of its own (Bevy's defaults, not its Text's, where left out). @bevy bevy_text::text::TextSpan */
export function TextSpan({ children, underline, strikethrough, underline_color, strikethrough_color, text_background_color, ...p }: TextStyleProps & TextRunProps & { children?: ReactNode }) {
  const back = useContext(InUi)?.back;
  const t = textCss("TextSpan", p, back);
  const run = { underline, strikethrough, underline_color, strikethrough_color, text_background_color };
  return (
    <>
      <style>{t.face}</style>
      <span style={t.css}>{runOf("TextSpan", run, back, children)}</span>
    </>
  );
}

export interface ImageNodeProps extends NodeProps {
  /** the picture: staticFile("branding/icon.png"), or one with its own sampler ({ path, sampler: { filter: "nearest" } }, Bevy's load_with_settings) */
  image: string | { path: string; sampler: SamplerSpec };
  /** its tint: white (none) by default; only white or a grey multiplies here yet (a picture's brightness) */
  color?: Srgba;
  flip_x?: boolean;
  flip_y?: boolean;
  /** TextureAtlas: the one cell of a sheet laid out by a TextureAtlasLayout to draw (the picture is the sheet) */
  texture_atlas?: { layout: TextureAtlasLayout; index: number };
  /** NodeImageMode: Sliced cuts the picture in nine by a TextureSlicer (its corners kept, its sides and centre
   *  stretched or tiled), drawn as CSS border-image with bevy_ui's scale; a sliced ImageNode holds children */
  image_mode?: { Sliced: TextureSlicer };
}

/** bevy_ui's SliceScaleMode: a side or the centre stretched, or tiled every slice x stretch_value px from its start */
export type SliceScaleMode = "Stretch" | { Tile: { stretch_value: number } };

/** bevy_sprite's TextureSlicer: the slices' insets in the picture's px, how the sides and centre scale (Stretch by
 *  default), and the corners' most scale (1 by default) */
export interface TextureSlicer { border: { left: number; right: number; top: number; bottom: number }; center_scale_mode?: SliceScaleMode; sides_scale_mode?: SliceScaleMode; max_corner_scale?: number }

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
  if (image_mode !== undefined) return SlicedNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, image_mode, ...p });
  if (texture_atlas !== undefined) return AtlasNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, ...p });
  if (typeof image !== "string" || image === "") throw new TypeError(`ImageNode: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  if (children !== undefined && children !== null && children !== false) throw new TypeError("ImageNode: takes no children here (a picture on a Node with children: put the ImageNode beside them)");
  const grey = color === undefined || (color.red === color.green && color.green === color.blue);
  if (!grey || (color && color.alpha !== 1)) throw new TypeError("ImageNode: color tints only by an opaque grey yet (brightness); a coloured or translucent tint is to do");
  const ui = useContext(InUi);
  const n = nodeCss("ImageNode", p, ui?.back);
  const flips = [flip_x && "scaleX(-1)", flip_y && "scaleY(-1)"].filter(Boolean).join(" ");
  const rendering = RENDERING[useContext(Sampler)];
  const css: CSSProperties = { ...n.css, objectFit: "fill", ...(rendering ? { imageRendering: rendering } : {}), ...(flips ? { transform: flips } : {}), ...(color && color.red !== 1 ? { filter: `brightness(${color.red})` } : {}) };
  const img = <img src={image} style={css} />;
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
  const node = <div style={n.css}><div style={box}><img src={image} style={sheet} /></div></div>;
  const root = fixed ? "fixed" : ui === null;
  return root ? <div style={root === "fixed" ? { ...VIEWPORT, position: "fixed" } : VIEWPORT}>{node}</div> : node;
}

// ImagePlugin's default sampler, for the pictures inside it (a <Node> tree's ImageNodes)
const Sampler = createContext<"Linear" | "Nearest">("Linear");
const RENDERING = { Linear: undefined, Nearest: "pixelated" } as const;

/** bevy_image's ImagePlugin: its default_sampler for every picture drawn inside it (Linear by default;
 *  ImagePlugin::default_nearest() is default_sampler="Nearest", for pixel art). @bevy bevy_image::ImagePlugin */
export function ImagePlugin({ default_sampler, children }: { default_sampler?: "Linear" | "Nearest"; children?: ReactNode }) {
  if (default_sampler !== undefined && !(default_sampler in RENDERING)) throw new TypeError(`ImagePlugin: default_sampler is Linear or Nearest, got ${JSON.stringify(default_sampler)}`);
  return <Sampler.Provider value={default_sampler ?? "Linear"}>{children}</Sampler.Provider>;
}

export interface Text2dProps extends TextStyleProps, TextRunProps {
  /** its place in Bevy's 2D world: the frame's centre the origin, +Y up, a unit a pixel, turns counterclockwise */
  transform?: TransformValue;
  /** how its lines line up with each other (TextLayout's Justify, Left by default) */
  justify?: TextProps["justify"];
  children?: ReactNode;
}

/** Text in Bevy's 2D world, its block's centre at its Transform (Text2d with Bevy's default Anchor::CENTER). bevy_text
 *  lays it out as a <Text>'s, so this is that Text, placed on the frame. Put it after the <Canvas3D> in the
 *  <AbsoluteFill>: it draws over the 2D and 3D parts, where Bevy orders it with the 2D ones by z.
 * @bevy bevy_sprite::text2d::Text2d */
export function Text2d({ transform, justify, children, ...text }: Text2dProps) {
  const { width, height } = useVideoConfig();
  const [tx, ty] = transform?.translation ?? [0, 0, 0];
  const [sx, sy] = transform?.scale ?? [1, 1, 1];
  const turn = transform?.rotation ? -eulerOf(transform.rotation)[2] : 0;
  const place: CSSProperties = { position: "absolute", left: width / 2 + tx, top: height / 2 - ty, transform: `translate(-50%, -50%) rotate(${turn}rad) scale(${sx}, ${sy})` };
  return (
    <div style={VIEWPORT}>
      <div style={place}>
        <InUi.Provider value={{ back: undefined }}>
          <Text {...text} justify={justify}>{children}</Text>
        </InUi.Provider>
      </div>
    </div>
  );
}

// an ImageNode cut in nine (NodeImageMode::Sliced): CSS border-image with its slices and `fill`, and bevy_ui's own
// corner scale and tiling on data-raea-bevy-slice (raea's Blitz, render/border_image.rs). It holds children, as a
// sliced button's label. A flipped one draws on a layer behind them (z-index -1 in the node's own stacking context,
// over its background, as Bevy draws the picture), mirrored there, so the flip doesn't turn the label.
function SlicedNode({ image, color, flip_x, flip_y, children, fixed, texture_atlas, image_mode, ...p }: ImageNodeProps) {
  if (typeof image !== "string" || image === "") throw new TypeError(`ImageNode: image is a picture file (staticFile("x.png")), got ${JSON.stringify(image)}`);
  const sl = image_mode?.Sliced;
  const b = sl?.border;
  if (!sl || !b || ![b.left, b.right, b.top, b.bottom].every((v) => Number.isFinite(v) && v >= 0)) throw new TypeError(`ImageNode: image_mode is { Sliced: { border: { left, right, top, bottom } } } (px >= 0), got ${JSON.stringify(image_mode)}`);
  if (texture_atlas !== undefined) throw new TypeError("ImageNode: a sliced picture takes no texture_atlas here yet");
  if (color !== undefined && !(color.red === 1 && color.green === 1 && color.blue === 1 && color.alpha === 1)) throw new TypeError("ImageNode: a sliced picture is untinted here yet (color white)");
  const max = sl.max_corner_scale ?? 1;
  if (!(Number.isFinite(max) && max > 0)) throw new RangeError(`ImageNode: max_corner_scale is above 0, got ${max}`);
  const mode = (m: SliceScaleMode | undefined, which: string) => {
    if (m === undefined || m === "Stretch") return "s";
    const v = typeof m === "object" ? m.Tile?.stretch_value : undefined;
    if (!(typeof v === "number" && Number.isFinite(v) && v > 0)) throw new TypeError(`ImageNode: ${which} is "Stretch" or { Tile: { stretch_value } } (above 0), got ${JSON.stringify(m)}`);
    return `t${v}`;
  };
  const marker = { "data-raea-bevy-slice": `${max} ${mode(sl.sides_scale_mode, "sides_scale_mode")} ${mode(sl.center_scale_mode, "center_scale_mode")}` };
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
  if (!flip_x && !flip_y) return boxOf({ ...n.css, ...slices }, n.back, children, root, undefined, marker);
  const flips = [flip_x && "scaleX(-1)", flip_y && "scaleY(-1)"].filter(Boolean).join(" ");
  const layer = <div key="raea-slices" style={{ position: "absolute", left: 0, top: 0, width: "100%", height: "100%", zIndex: -1, transform: flips, ...slices }} {...marker} />;
  return boxOf({ ...n.css, position: n.css.position ?? "relative", zIndex: 0 }, n.back, [layer, ...Children.toArray(children)], root);
}

/** bevy_ui's UiScale: every UI length inside times `value` (2 draws a 16 px node 32 px). Its roots lay out in a window
 *  of the frame's size over `value`, which is then scaled up from its top left, as Bevy lays out in logical px and
 *  draws in physical ones. Text2d and the 2D and 3D parts aren't UI, so they stay as they are.
 * @bevy bevy_ui::UiScale */
export function UiScale({ value, children }: { value: number; children?: ReactNode }) {
  if (!(Number.isFinite(value) && value > 0)) throw new RangeError(`UiScale: value is above 0, got ${value}`);
  return <div style={{ position: "absolute", left: 0, top: 0, width: `${100 / value}%`, height: `${100 / value}%`, transform: `scale(${value})`, transformOrigin: "0 0" }}>{children}</div>;
}
