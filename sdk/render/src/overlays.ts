// A material's overlays, drawn over its standard surface by the engine: graph paper (`grid`, raea-render's grid.rs),
// a paint look's breakup (`paint`, paint.rs), wear (`wear`, systems/wear) and a procedural surface (`procedural`,
// systems/procmat). Their props and checks; 3d.tsx's materials take them all, a Model takes `grid`.
import { toHex } from "./colour";

/** graph paper over the surface (greybox levels): lines every `cell` world units (fixed in the world, so they run on
 * across shapes), a heavier one every `major` cells (4), in `color`, and each face of a Box outlined in `edge`
 * ("" for no outlines). Its colours are the first frame's. */
export interface GridOverlay { cell: number; major?: number; color?: string; edge?: string }

/** under a paint look: the surface broken up by world-space noise of `scale` metres (1): its tone boundaries wander
 * by `breakup` (0.5), and patches of it take `color` (`amount`, 0.6, of the way): a mottled field, rust, a wall's
 * uneven paint */
export interface PaintOverlay { breakup?: number; scale?: number; color?: string; amount?: number }

/** a worn surface (systems/wear), fixed in the part's own space so it rides with a moving part: `amount` 0..1 of
 * wear, its paint chipped back to `chipColor` at edges (`chips`), `grime` low and underneath, `rust` run down in
 * streaks (each 0..1 of the amount), in blotches of `scale` metres (0.08); `seed` moves the pattern */
export interface WearOverlay { amount: number; scale?: number; chips?: number; grime?: number; rust?: number; seed?: number; chipColor?: string; grimeColor?: string; rustColor?: string }

/** the patterns a procedural surface draws (systems/procmat) */
export const PATTERNS = ["noise", "wood", "marble", "stone", "brick", "tiles", "fabric", "brushed", "plaster"] as const;

/** a surface made on the GPU (systems/procmat) in place of a flat colour: `pattern` in `colors` (two, each its own
 * look's by default) with roughness `roughA`/`roughB`, features `scale` metres across (wood 0.25, fabric 0.004), a
 * `bump` from its height (0.5); brick and tiles in cells of `tile` [w, h] metres with `gap` metre joints in
 * `gapColor`. Use over it, each 0..1 (0): `dust` on what faces up, `dirt` in joints, hollows, undersides and along the
 * floor line, `edges` worn lighter. Fixed in the part's own space; `seed` moves it. Clay reviews draw it plain. */
export interface ProceduralSurface { pattern: (typeof PATTERNS)[number]; colors?: [string, string]; scale?: number; roughA?: number; roughB?: number; bump?: number; tile?: [number, number]; gap?: number; gapColor?: string; dust?: number; dirt?: number; edges?: number; dustColor?: string; dirtColor?: string; seed?: number }

function checkProcedural(owner: string, props: Record<string, unknown>): void {
  const s = props.procedural as ProceduralSurface | undefined;
  if (s === undefined) return;
  if (!(typeof s === "object" && s !== null && PATTERNS.includes(s.pattern))) throw new TypeError(`${owner}: procedural needs a pattern (${PATTERNS.join(", ")}), got ${JSON.stringify(s)}`);
  if (props.wear !== undefined || props.grid !== undefined) throw new TypeError(`${owner}: procedural has its own use (dust, dirt, edges); it takes no wear or grid with it`);
  for (const k of ["roughA", "roughB", "dust", "dirt", "edges"] as const) if (s[k] !== undefined && !(s[k]! >= 0 && s[k]! <= 1)) throw new RangeError(`${owner}: procedural.${k} is in [0, 1], got ${s[k]}`);
  for (const k of ["scale", "gap", "bump"] as const) if (s[k] !== undefined && !(s[k]! >= 0 && Number.isFinite(s[k]))) throw new RangeError(`${owner}: procedural.${k} is 0 or more, got ${s[k]}`);
  if (s.tile !== undefined && !(Array.isArray(s.tile) && s.tile.length === 2 && s.tile.every((x) => x > 0))) throw new RangeError(`${owner}: procedural.tile is [width, height] in metres, got ${JSON.stringify(s.tile)}`);
  if (s.colors !== undefined && !(Array.isArray(s.colors) && s.colors.length === 2)) throw new TypeError(`${owner}: procedural.colors is two colours, got ${JSON.stringify(s.colors)}`);
  props.procedural = {
    ...s,
    ...(s.colors ? { colors: s.colors.map((c, i) => toHex(owner, `procedural.colors[${i}]`, c)) } : {}),
    ...Object.fromEntries((["gapColor", "dustColor", "dirtColor"] as const).filter((k) => s[k] !== undefined).map((k) => [k, toHex(owner, `procedural.${k}`, s[k])])),
  };
}

/** a `grid` prop checked, its colours as hex (MeshStandardMaterial's and Model's) */
export function checkGrid(owner: string, props: Record<string, unknown>): void {
  const grid = props.grid as GridOverlay | undefined;
  if (grid === undefined) return;
  if (!(typeof grid === "object" && grid !== null && typeof grid.cell === "number" && grid.cell > 0)) throw new TypeError(`${owner}: grid needs a cell size above 0 ({ cell: 0.5 }), got ${JSON.stringify(grid)}`);
  if (grid.major !== undefined && !(grid.major >= 1)) throw new RangeError(`${owner}: grid.major must be at least 1, got ${grid.major}`);
  props.grid = { ...grid, color: grid.color === undefined ? undefined : toHex(owner, "grid.color", grid.color), edge: grid.edge === undefined || grid.edge === "" ? grid.edge : toHex(owner, "grid.edge", grid.edge) };
}

/** a material's `grid`, `paint`, `wear` and `procedural` checked, their colours as hex */
export function checkOverlays(owner: string, props: Record<string, unknown>): void {
  checkProcedural(owner, props);
  checkGrid(owner, props);
  const pp = props.paint as PaintOverlay | undefined;
  if (pp !== undefined) {
    if (!(typeof pp === "object" && pp !== null)) throw new TypeError(`${owner}: paint is { breakup, scale, color, amount }, got ${JSON.stringify(pp)}`);
    if (pp.scale !== undefined && !(pp.scale > 0)) throw new RangeError(`${owner}: paint.scale must be above 0 metres, got ${pp.scale}`);
    props.paint = { ...pp, color: pp.color === undefined ? undefined : toHex(owner, "paint.color", pp.color) };
  }
  const w = props.wear as WearOverlay | undefined;
  if (w !== undefined && !(typeof w === "object" && w !== null && w.amount >= 0 && w.amount <= 1)) throw new RangeError(`${owner}: wear needs an amount in [0, 1] ({ amount: 0.6 }), got ${JSON.stringify(w)}`);
  if (w !== undefined) props.wear = { ...w, ...Object.fromEntries((["chipColor", "grimeColor", "rustColor"] as const).filter((k) => w[k] !== undefined).map((k) => [k, toHex(owner, `wear.${k}`, w[k])])) };
}
