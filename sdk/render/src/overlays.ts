// A material's overlays, drawn over its standard surface by the engine: graph paper (`grid`, raea-render's grid.rs),
// a paint look's breakup (`paint`, paint.rs) and wear (`wear`, systems/wear). Their props and checks; 3d.tsx's
// materials take all three, a Model takes `grid`.
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

/** a `grid` prop checked, its colours as hex (MeshStandardMaterial's and Model's) */
export function checkGrid(owner: string, props: Record<string, unknown>): void {
  const grid = props.grid as GridOverlay | undefined;
  if (grid === undefined) return;
  if (!(typeof grid === "object" && grid !== null && typeof grid.cell === "number" && grid.cell > 0)) throw new TypeError(`${owner}: grid needs a cell size above 0 ({ cell: 0.5 }), got ${JSON.stringify(grid)}`);
  if (grid.major !== undefined && !(grid.major >= 1)) throw new RangeError(`${owner}: grid.major must be at least 1, got ${grid.major}`);
  props.grid = { ...grid, color: grid.color === undefined ? undefined : toHex(owner, "grid.color", grid.color), edge: grid.edge === undefined || grid.edge === "" ? grid.edge : toHex(owner, "grid.edge", grid.edge) };
}

/** a material's `grid`, `paint` and `wear` checked, their colours as hex */
export function checkOverlays(owner: string, props: Record<string, unknown>): void {
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
