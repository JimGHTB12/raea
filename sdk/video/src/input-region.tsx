// A box of a page with input of its own: several panels in one page (the phone's compact shell, docs/platform.feature),
// each hearing the pointer as it would as a page of its own (apps/void/src/space_ui.rs gives each panel its own page).
import { useContext, useMemo, type ReactNode } from "react";
import { ClockContext, useClockTick, type FrameInput, type InputTimeline } from "@raea/app/clock";

type Box = { x: number; y: number; w: number; h: number; scale: number };
type Entry = Partial<FrameInput> & { live?: boolean };

/** The timeline as a box hears it: the pointer moved into the box's own points (page pixels over `scale`), and only
 *  while it's the box's: from a press inside it to the release, wherever the finger goes meanwhile, or hovering over
 *  it with nothing pressed. Each frame's entry is made once, so a game's fold sees the same entries frame to frame. */
function boxed(timeline: InputTimeline, b: Box): InputTimeline {
  const made = new Map<number, Entry | undefined>();
  const held = new Map<number, boolean>();
  const inside = (p: { x: number; y: number }) => p.x >= b.x && p.y >= b.y && p.x < b.x + b.w && p.y < b.y + b.h;
  // whether the box holds the pointer at frame f: pressed inside it and not let go since
  const holds = (f: number): boolean => {
    let g = f;
    // the frames back to the press (or to the start) are walked once, then remembered
    while (g > 0 && !held.has(g - 1) && (timeline[g - 1] as Entry | undefined)?.pointer?.down) g--;
    for (; g <= f; g++) {
      if (held.has(g)) continue;
      const p = (timeline[g] as Entry | undefined)?.pointer;
      const before = g > 0 ? (timeline[g - 1] as Entry | undefined)?.pointer?.down : false;
      held.set(g, !!p?.down && (before ? (held.get(g - 1) ?? false) : inside(scaled(p))));
    }
    return held.get(f) ?? false;
  };
  const scaled = (p: { x: number; y: number }) => ({ x: p.x / b.scale, y: p.y / b.scale });
  const at = (f: number): Entry | undefined => {
    if (made.has(f)) return made.get(f);
    const e = timeline[f] as Entry | undefined;
    let out = e;
    if (e?.pointer) {
      const p = scaled(e.pointer);
      const mine = e.pointer.down ? holds(f) : inside(p) && !holds(f - 1);
      out = { ...e, pointer: mine ? { x: p.x - b.x, y: p.y - b.y, down: e.pointer.down } : undefined };
      if (!mine) delete out.pointer;
    }
    made.set(f, out);
    // a live game keeps 4096 frames of input (games.ts LIVE_KEEP); so does this
    made.delete(f - 4096);
    held.delete(f - 4096);
    return out;
  };
  return new Proxy(timeline, {
    get: (t, k) => (typeof k === "string" && /^\d+$/.test(k) ? at(Number(k)) : Reflect.get(t, k)),
  });
}

/**
 * Its children hear the pointer in their own box (x, y, w, h in points; the page's pixels are `scale` points each), as
 * a panel does when it's a page of its own: a press inside the box is the box's until it's let go. Outside the box,
 * and while another box holds a press, they see no pointer; keys reach every box.
 */
export function InputRegion({ x, y, w, h, scale = 1, children }: { x: number; y: number; w: number; h: number; scale?: number; children?: ReactNode }) {
  const clock = useContext(ClockContext);
  // its copy is a new object each frame, so its children hear the frame through it: it hears it from the Clock
  useClockTick(clock);
  const input = useMemo(() => (clock ? boxed(clock.input, { x, y, w, h, scale }) : null), [clock?.input, x, y, w, h, scale]);
  if (!clock || !input) return <>{children}</>;
  return <ClockContext.Provider value={{ ...clock, input }}>{children}</ClockContext.Provider>;
}
