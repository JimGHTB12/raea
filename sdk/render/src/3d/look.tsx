// Looks and passes, and the scene's props.
import { useCurrentFrame, useVideoConfig } from "@raea/app/clock";
import type { World } from "@raea/app/ecs";
import { toHex } from "../colour";
import { AmbientLight, DirectionalLight, Mesh, MeshStandardMaterial, PerspectiveCamera, node3d } from "./all";

export interface LookProps {
  /** "site" (raea.io's grain: each cell's light ordered-dithered into the site's six greys), "pixel" (each cell
   * its colour) or "ink" (two colours, as Return of the Obra Dinn: each cell's light 8x8-Bayer-dithered into `paper`
   * or `ink`, with a line of ink where the light steps by more than `line`), "paint" (an editorial illustration: every
   * surface lit in `bands` flat tones by the suns, hard cast shadows as shapes, ragged edges, paper grain, a flat
   * sky) or "cartoon" (paint's preset for a cartoon: a flat fill and one shade, a thick even `outline` that boils) */
  mode?: "site" | "pixel" | "ink" | "paint" | "cartoon";
  /** a cell's size in the frame's pixels (6 by default) */
  pixelSize?: number;
  /** the site's dark greys (the default) or its light ones */
  theme?: "dark" | "light";
  /** how much of each cell's hue its grey keeps, 0 (greys only) to 1 (the default) */
  color?: number;
  /** the site look's ink: light = lift + light x gain (0 and 1 by default, so black stays black) */
  lift?: number;
  gain?: number;
  /** the ink look's ink and paper, hex colours (a warm black and an old paper by default) */
  ink?: string;
  paper?: string;
  /** the ink look's line: how big a step in a cell's light (0 to 1) from a neighbour's draws ink there (0, none, by
   * default; about 0.15 rings figures and edges without picking up texture) */
  line?: number;
  /** the ink look's split: light below it is ink, above it paper (0.5 by default) */
  threshold?: number;
  /** how wide a band of light around `threshold` is dithered (1, the whole range, by default; 0.1 gives hard ink and
   * paper with a thin dithered edge between, a silhouette's look) */
  band?: number;
  /** paint, cartoon: how many flat tones the suns' light makes on a surface, 1-8 (3 for paint: shadow, mid and lit;
   * 2 for cartoon: shade and fill); keyed per frame */
  bands?: number;
  /** paint, cartoon: the darkest tone (turned from the sun, or in a cast shadow), as a colour the surface's own is
   * multiplied by (a cool grey-blue by default; a warm blush for a cartoon) */
  shadow?: string;
  /** paint: the tone between shadow and full light, a colour the surface's own is multiplied by (a warm pink-grey) */
  mid?: string;
  /** paint, cartoon: paper grain, 0 (none) to 1 (0.3 for paint, 0.06 for cartoon); a fixed pattern; keyed */
  grain?: number;
  /** paint, cartoon: how far edges (silhouettes, shadows, lines) wander, in pixels (2 for paint, 1.5 for cartoon; 0
   * for clean edges); keyed */
  rag?: number;
  /** how wide the wander's waves are, in pixels (24) */
  ragScale?: number;
  /** the wander's seed (0); the same seed paints the same edges */
  seed?: number;
  /** a new wander every this many frames: the line boils (0: a fixed wander, paint's default; 2, on twos, cartoon's) */
  boil?: number;
  /** where nothing's drawn: one flat colour, or [top, bottom] for a two-tone gradient (the Canvas3D's background
   * otherwise) */
  sky?: string | [string, string];
  /** colours every pixel snaps to (nearest by Oklab, lightness counted double), up to 16; none by default */
  palette?: string[];
  /** cartoon: the outline's colour (a dark brown, "#2b1d14"; "none" for no line) and width in pixels (6) */
  outline?: string;
  outlineWidth?: number;
  /** paint: ambient occlusion as one band, the surface's tone times `ao` (a grey-violet): on walls up to `aoHeight`
   * metres above `ground` (0), its top torn by noise (a contact shadow, ground bounce's edge), and where an
   * <AmbientOcclusion> finds a crease darker than `ssao` (0 to 1; 0.6 is a fair start) */
  ao?: string;
  aoHeight?: number;
  ground?: number;
  ssao?: number;
  /** paint: the water's height; whatever stands in it is ringed with broken foam (`foamColor`, white) up to `foam`
   * metres (0.25) above it */
  water?: number;
  foam?: number;
  foamColor?: string;
  /** paint: how far the cast shadows' edges tear (0 to 0.5; 0, ruled, by default) and the tears' size in metres (1.5) */
  edge?: number;
  edgeScale?: number;
  /** paint: a textured surface's colour taken as its texture's average over a patch `washSize` across (in uv, 0.05)
   * instead of its fine, photographic detail, 0 (the texture as is) to 1: scanned models painted */
  wash?: number;
  washSize?: number;
  /** paint: brush strokes, each surface's colour lightened and darkened in short world-space strokes, 0 to 1, each
   * stroke about `strokeScale` metres (0.08) long */
  stroke?: number;
  strokeScale?: number;
  /** paint: colour pushed past (above 1) or toward grey (below 1) its own saturation (1) */
  saturate?: number;
  /** paint: cut-out leaves and blades (materials with alpha) shaded as their plant's mass, 0 to 1: normals bent up
   * so a clump takes the light as one shape, colour graded from a dark root to a light tip over `foliageHeight`
   * metres above `ground` (0.6) */
  foliage?: number;
  foliageHeight?: number;
  /** paint: low foliage (cut-out blades and petals within twice `foliageHeight` of `ground`) settled toward one colour with
   * distance, from `calmFrom` to `calmTo` metres from the camera (8 to 40), by `calmAmount` (0.85): far grass and
   * flowers, finer than a pixel, read as a field instead of speckle (pixel art keeps its distance quiet) */
  calm?: string;
  calmFrom?: number;
  calmTo?: number;
  calmAmount?: number;
  /** paint: every surface drawn as pixel art fixed to the world, the frame still full resolution: colour, texture,
   * shadow and strokes taken once per cell of a world grid `texel` metres across (0.03: a hand's width is three
   * cells), so the pixels sit on the meshes and don't crawl as the camera moves */
  texel?: number;
  /** paint: each texel's own shade, lighter or darker by up to this (0.08), so a flat colour shows its pixels */
  texelJitter?: number;
  /** paint: cloud shadows, shapes of shade drifting over everything lit, cut hard like cast shadows: how much of the
   * land they cover (0 to 1; 0.4 is a fair sky), their size in metres (40), and how far the wind has carried them
   * ([x, z] metres; move it with the frame to drift them) */
  cloudShadows?: number;
  cloudShadowSize?: number;
  cloudShadowDrift?: [number, number];
  /** paint: wind in the cut-out foliage near the ground (grass, flowers): how far a blade's tip bends, in metres (0,
   * still; 0.08 a breeze, 0.25 a gale), the way it blows ([x, z], [1, 0] by default), and how long it has blown in
   * seconds (the frame's time, by default) that gusts roll across the field with. Outlines and shadows bend too */
  wind?: number;
  windDirection?: [number, number];
  windTime?: number;
  /** paint: air lying low over the land, in this colour (a hex colour, the sky's at the horizon, never white; none
   * by default). It starts `hazeStart` metres out (8) so the foreground keeps its colour, thickens with distance
   * (`hazeDensity` per metre, 0.02) and thins with height (`hazeHeight` metres, 3), drains a thing's saturation
   * before its value (the distance goes cool and soft, not pale), never covers more than `hazeMost` (0.7), and
   * looking toward the sun glows `hazeSun` (a warm colour; none by default) */
  haze?: string;
  hazeDensity?: number;
  hazeHeight?: number;
  hazeStart?: number;
  hazeMost?: number;
  hazeSun?: string;
  /** composition passes (a part's `pass`) the look leaves exactly as drawn: no rag, grain, palette or sky over them,
   * and their surfaces drawn with their own materials, not the paint's (a page on a <Screen> crisp behind a painted
   * body; the body keeps its paint where it covers the page) */
  skip?: string[];
  /** each named pass's own paint (LookPass: rag, grain, palette, bands, edge, stroke, saturate, wash); keyed */
  passes?: Record<string, LookPass>;
}

/** One composition pass's own paint (<Look passes>): the frame's rag, grain and palette, and its surfaces' bands, torn
 * shadow edge, strokes, saturation and wash; what it leaves out is the Look's. */
export interface LookPass {
  rag?: number;
  grain?: number;
  /** false: its pixels aren't snapped to the palette */
  palette?: boolean;
  bands?: number;
  edge?: number;
  stroke?: number;
  saturate?: number;
  wash?: number;
}

const PASS_OK = /^[A-Za-z][A-Za-z0-9_-]{0,31}$/;

/** A part's composition pass, checked: a name, letters first (up to 32 of letters, digits, _ and -). */
export function checkPass(owner: string, pass: unknown): void {
  if (pass !== undefined && !(typeof pass === "string" && PASS_OK.test(pass))) throw new TypeError(`${owner}: pass is a name like "ui" (letters first, up to 32 letters, digits, _ or -), got ${JSON.stringify(pass)}`);
}

/** An effect's `skip` (pass names) and `passes` (a share, or a Look's own settings, by name), checked. */
export function checkShares(owner: string, skip: unknown, passes: unknown, settings = false): void {
  if (skip !== undefined) {
    if (!Array.isArray(skip)) throw new TypeError(`${owner}: skip is a list of pass names (["ui"]), got ${JSON.stringify(skip)}`);
    for (const n of skip) checkPass(owner, n);
  }
  if (passes === undefined) return;
  if (typeof passes !== "object" || passes === null || Array.isArray(passes)) throw new TypeError(`${owner}: passes maps a pass's name to ${settings ? "its own settings" : "its share, 0 to 1"}, got ${JSON.stringify(passes)}`);
  for (const [n, v] of Object.entries(passes)) {
    checkPass(owner, n);
    if (settings) {
      if (typeof v !== "object" || v === null) throw new TypeError(`${owner}: passes.${n} is that pass's own settings ({ rag: 0, grain: 0 }), got ${JSON.stringify(v)}`);
      for (const [k, x] of Object.entries(v)) {
        if (k === "palette") {
          if (typeof x !== "boolean") throw new TypeError(`${owner}: passes.${n}.palette is true or false, got ${JSON.stringify(x)}`);
        } else if (!["rag", "grain", "bands", "edge", "stroke", "saturate", "wash"].includes(k)) throw new TypeError(`${owner}: passes.${n} takes rag, grain, palette, bands, edge, stroke, saturate and wash, got ${k}`);
        else if (!(typeof x === "number" && Number.isFinite(x) && x >= 0)) throw new RangeError(`${owner}: passes.${n}.${k} must be a number 0 or more, got ${JSON.stringify(x)}`);
      }
    } else if (!(typeof v === "number" && v >= 0 && v <= 1)) throw new RangeError(`${owner}: passes.${n} is a share from 0 to 1, got ${JSON.stringify(v)}`);
  }
}

const PAINT = { bands: 3, grain: 0.3, rag: 2, boil: 0 };

const CARTOON = { bands: 2, grain: 0.06, rag: 1.5, boil: 2, shadow: "#f2c7b8" };

/** The whole frame drawn as a model's look draws its model: every part of the scene in the one grain, its bloom,
 * fog and blur inside it. Put it inside the Canvas3D. mode="paint" or "cartoon" draws every standard material as
 * paint (engine/render paint.rs):
 * @bevy bevy_pbr::mesh_material::MeshMaterial3d<bevy_pbr::extended_material::ExtendedMaterial<bevy_pbr::pbr_material::StandardMaterial, raea_render::engine::paint::Paint>>
 */
export function Look(props: LookProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (props.mode === "paint" || props.mode === "cartoon") return paintLook(props, frame, fps);
  if (props.mode !== undefined && props.mode !== "site" && props.mode !== "pixel" && props.mode !== "ink") throw new RangeError(`Look: mode is "site", "pixel", "ink", "paint" or "cartoon", got ${props.mode}`);
  if (props.pixelSize !== undefined && !(props.pixelSize >= 1)) throw new RangeError(`Look: pixelSize must be 1 or more, got ${props.pixelSize}`);
  if (props.band !== undefined && !(props.band > 0)) throw new RangeError(`Look: band must be above 0, got ${props.band}`);
  if (props.line !== undefined && !(props.line >= 0 && props.line <= 1)) throw new RangeError(`Look: line must be between 0 and 1, got ${props.line}`);
  if (props.color !== undefined && !(props.color >= 0 && props.color <= 1)) throw new RangeError(`Look: color must be between 0 and 1, got ${props.color}`);
  checkShares("Look", props.skip, props.passes, true);
  return node3d("Look", props as Record<string, unknown>);
}

/** <Look mode="paint" | "cartoon">: the look's element (mode "paint" for the engine, the boil folded into the seed so
 * the engine never reads a clock) and, for a cartoon, its outline. */
function paintLook(props: LookProps, frame: number, fps: number) {
  const frameTime = frame / fps;
  const cartoon = props.mode === "cartoon";
  const d: Record<string, unknown> = cartoon ? CARTOON : PAINT;
  const p = { ...d, ...props } as LookProps & typeof PAINT;
  if (!(Number.isInteger(p.bands) && p.bands >= 1 && p.bands <= 8)) throw new RangeError(`Look: bands must be a whole number from 1 to 8, got ${p.bands}`);
  for (const k of ["grain", "rag", "ragScale", "seed", "boil", "outlineWidth", "aoHeight", "ssao", "foam", "edge", "edgeScale", "wash", "washSize", "stroke", "strokeScale", "saturate", "foliage", "foliageHeight", "calmFrom", "calmTo", "calmAmount", "texel", "texelJitter", "cloudShadows", "cloudShadowSize", "wind", "windTime", "hazeDensity", "hazeHeight", "hazeStart", "hazeMost"] as const) {
    const v = p[k];
    if (v !== undefined && !(typeof v === "number" && Number.isFinite(v) && v >= 0)) throw new RangeError(`Look: ${k} must be a number 0 or more, got ${JSON.stringify(v)}`);
  }
  if (p.grain !== undefined && p.grain > 1) throw new RangeError(`Look: grain must be from 0 to 1, got ${p.grain}`);
  if (p.ragScale !== undefined && p.ragScale < 1) throw new RangeError(`Look: ragScale must be 1 pixel or more, got ${p.ragScale}`);
  if (p.pixelSize !== undefined && !(p.pixelSize >= 1)) throw new RangeError(`Look: pixelSize must be 1 or more, got ${p.pixelSize}`);
  if (p.palette !== undefined && !(Array.isArray(p.palette) && p.palette.length >= 1 && p.palette.length <= 16)) throw new RangeError(`Look: palette must be 1 to 16 colours, got ${JSON.stringify(p.palette)}`);
  if (p.sky !== undefined && Array.isArray(p.sky) && p.sky.length !== 2) throw new RangeError(`Look: sky is one colour or [top, bottom], got ${JSON.stringify(p.sky)}`);
  checkShares("Look", p.skip, p.passes, true);
  const boil = p.boil ?? 0;
  const seed = Math.round(p.seed ?? 0) + (boil > 0 ? Math.floor(frame / boil) : 0);
  const look: Record<string, unknown> = {
    mode: "paint", pixelSize: p.pixelSize, bands: p.bands, grain: p.grain, rag: p.rag, ragScale: p.ragScale, seed,
    shadow: p.shadow === undefined ? undefined : toHex("Look", "shadow", p.shadow),
    mid: p.mid === undefined ? undefined : toHex("Look", "mid", p.mid),
    sky: p.sky === undefined ? undefined : Array.isArray(p.sky) ? p.sky.map((c) => toHex("Look", "sky", c)) : toHex("Look", "sky", p.sky),
    palette: p.palette?.map((c) => toHex("Look", "palette", c)),
    ao: p.ao === undefined ? undefined : toHex("Look", "ao", p.ao),
    foamColor: p.foamColor === undefined ? undefined : toHex("Look", "foamColor", p.foamColor),
    calm: p.calm === undefined ? undefined : toHex("Look", "calm", p.calm), haze: p.haze === undefined ? undefined : toHex("Look", "haze", p.haze), hazeDensity: p.hazeDensity, hazeHeight: p.hazeHeight, hazeStart: p.hazeStart, hazeMost: p.hazeMost,
    hazeSun: p.hazeSun === undefined ? undefined : toHex("Look", "hazeSun", p.hazeSun), calmFrom: p.calmFrom, calmTo: p.calmTo, calmAmount: p.calmAmount, texel: p.texel, texelJitter: p.texelJitter, cloudShadows: p.cloudShadows, cloudShadowSize: p.cloudShadowSize,
    cloudShadowX: p.cloudShadowDrift?.[0], cloudShadowZ: p.cloudShadowDrift?.[1],
    wind: p.wind, windX: p.windDirection?.[0], windZ: p.windDirection?.[1], windTime: p.windTime ?? (p.wind ? frameTime : undefined),
    aoHeight: p.aoHeight, ground: p.ground, ssao: p.ssao, water: p.water, foam: p.foam, edge: p.edge, edgeScale: p.edgeScale,
    wash: p.wash, washSize: p.washSize, stroke: p.stroke, strokeScale: p.strokeScale, saturate: p.saturate, foliage: p.foliage, foliageHeight: p.foliageHeight,
    skip: p.skip?.length ? p.skip : undefined, passes: p.passes && Object.keys(p.passes).length ? p.passes : undefined,
  };
  if (!cartoon || p.outline === "none") return node3d("Look", look);
  const line = toHex("Look", "outline", p.outline ?? "#2b1d14");
  return (
    <>
      {node3d("Look", look)}
      {node3d("Outline", { color: line, width: p.outlineWidth ?? 6, depth: 0.04, normal: 0 })}
    </>
  );
}

export interface SceneProps {
  /** the world whose rows are drawn; its systems run up to the current frame */
  world: World;
}

/** A world drawn at the current frame: its `mesh` rows (shape, x y z, rot_x rot_y rot_z in radians, scale, color,
 * rough, metal), `camera` (x y z, look_x look_y look_z, fov), `light` (x y z, intensity) and `ambient` (intensity)
 * rows as the 3D elements above. Inside a Canvas3D. */
export function Scene({ world }: SceneProps) {
  const frame = useCurrentFrame();
  const rows = world.at(Math.max(0, Math.floor(frame))).rows();
  const n = (r: Record<string, unknown>, k: string, d: number) => (typeof r[k] === "number" ? (r[k] as number) : d);
  return (
    <>
      {rows.map((r) => {
        const key = `${r.kind}-${r.id}`;
        switch (r.kind) {
          case "camera":
            return <PerspectiveCamera key={key} position={[n(r, "x", 0), n(r, "y", 0), n(r, "z", 5)]} fov={n(r, "fov", 50)} />;
          case "light":
            return <DirectionalLight key={key} position={[n(r, "x", 0), n(r, "y", 1), n(r, "z", 0)]} intensity={n(r, "intensity", 1)} />;
          case "ambient":
            return <AmbientLight key={key} intensity={n(r, "intensity", 1)} />;
          case "mesh":
            if (typeof r.shape !== "string") return null;
            return (
              <Mesh key={key} shape={r.shape} position={[n(r, "x", 0), n(r, "y", 0), n(r, "z", 0)]} rotation={[n(r, "rot_x", 0), n(r, "rot_y", 0), n(r, "rot_z", 0)]} scale={n(r, "scale", 1)}>
                <MeshStandardMaterial color={typeof r.color === "string" ? r.color : "#cccccc"} roughness={n(r, "rough", 0.5)} metalness={n(r, "metal", 0)} />
              </Mesh>
            );
          default:
            return null;
        }
      })}
    </>
  );
}

/** A world's entities drawn by the engine (no React element per entity): every entity with a Mesh component ({shape:
 * a Manifold expression or a Solid's expr, color, roughness, metalness}) and a Transform ({x, y, z, rx, ry, rz in
 * radians, scale}) is a mesh, placed in world space (a child's Transform is in its parent's). Give it the world at the
 * frame: `<Entities world={game.at(useCurrentFrame())} />`. Needs the engine's tables. */
export function Entities({ world }: { world: { scene(): unknown[] } }) {
  return node3d("Entities", { scene: world.scene() });
}
