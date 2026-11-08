// bevy_picking on a bevy_ui node, and bevy_ui_widgets' Popover (docs/control.feature FC0a, FC3c: ui/widgets'
// menus). A node's pointer events are Bevy's picking events as React spells them, the same set a 3D part takes
// (events3d.tsx, docs/sdk.feature S3a): Pointer<Click> is onClick, Pointer<Press> onPointerDown, Pointer<Over>
// onPointerOver. In a render they come from the composition's input timeline, in play from the pointer (a window's,
// or --input-script's): the page hit tests the pointer and fires them in a browser's order (runtime/live pointer.rs).

/** One of a Popover's placements beside its parent. @bevy bevy_ui_widgets::popover::PopoverPlacement */
export interface PopoverPlacement {
  /** PopoverSide: Bottom by default */
  side?: "Top" | "Bottom" | "Left" | "Right";
  /** PopoverAlign: which edges line up along that side, Start by default */
  align?: "Start" | "End" | "Center";
  /** the gap from the parent, in px (0 by default) */
  gap?: number;
}

/** A popup placed beside its parent's box (inside its border) after layout, at the first of `positions` the window
 *  (less `window_margin` px) hides least; absolutely positioned, as Bevy's position_popover makes it.
 *  @bevy bevy_ui_widgets::popover::Popover */
export interface Popover {
  positions: PopoverPlacement[];
  window_margin?: number;
}

/** The pointer event a handler gets: the pointer in page px and the button (0 primary, 1 middle, 2 secondary). */
export interface UiPointerEvent {
  id: number;
  kind: string;
  x: number;
  y: number;
  button?: number;
}

type On = (e: UiPointerEvent) => void;

/** A node's bevy_picking events and its Popover. */
export interface PickProps {
  popover?: Popover;
  onClick?: On;
  onPointerDown?: On;
  onPointerUp?: On;
  onPointerOver?: On;
  onPointerOut?: On;
  onPointerEnter?: On;
  onPointerLeave?: On;
  onPointerMove?: On;
}

const HANDLERS = ["onClick", "onPointerDown", "onPointerUp", "onPointerOver", "onPointerOut", "onPointerEnter", "onPointerLeave", "onPointerMove"] as const;

/** The node's handlers, for its element. */
export function handlersOf(who: string, p: PickProps): Record<string, On> {
  const out: Record<string, On> = {};
  for (const k of HANDLERS) {
    const f = p[k];
    if (f === undefined) continue;
    if (typeof f !== "function") throw new TypeError(`${who}: ${k} is a function, got ${JSON.stringify(f)}`);
    out[k] = f;
  }
  return out;
}

/** The Popover's attribute (raea's Blitz places it after layout, bevy_popover.rs), and its position made absolute. */
export function popoverAttr(who: string, p: PickProps, s: Record<string, unknown>, attrs: Record<string, string | number>) {
  const pop = p.popover;
  if (pop === undefined) return;
  if (!pop || !Array.isArray(pop.positions) || pop.positions.length === 0) throw new TypeError(`${who}: popover is { positions: [{ side, align, gap }, ...], window_margin }, got ${JSON.stringify(pop)}`);
  const margin = pop.window_margin ?? 0;
  if (!Number.isFinite(margin)) throw new RangeError(`${who}: popover.window_margin is a number of px, got ${margin}`);
  const at = pop.positions.map((q, i) => {
    const side = q.side ?? "Bottom";
    const align = q.align ?? "Start";
    const gap = q.gap ?? 0;
    if (!["Top", "Bottom", "Left", "Right"].includes(side) || !["Start", "End", "Center"].includes(align) || !Number.isFinite(gap)) {
      throw new TypeError(`${who}: popover.positions[${i}] is { side: Top|Bottom|Left|Right, align: Start|End|Center, gap: px }, got ${JSON.stringify(q)}`);
    }
    return `${side} ${align} ${gap}`;
  });
  attrs["data-raea-bevy-popover"] = [String(margin), ...at].join(";");
  s.position = "absolute";
}
