// Portions ported from HyperFrames (Apache-2.0, https://github.com/heygen-com/hyperframes); see NOTICE.
// The parts of the the HTML composition format runtime a render depends on (docs/ts-native.feature P27), as engine/html's runtime.rs
// ported them from the composition format's packages/core/src/runtime (startResolver.ts, timeline.ts, clipWindow.ts): `data-start`
// expressions (absolute, `id`, `id + 2`, `id - 0.5`) offset by the host composition, durations from data-duration /
// data-end / media / sub-timelines, frame-snapped clip windows, the ancestor chain rule, and display:none for hidden
// in-flow leaf clips.

const FRAME_BOUNDARY_EPSILON = 1e-3;
const TERMINAL_EPSILON_SECONDS = 1e-6;

type El = Element & { style: any };

function sameInstant(a: number, b: number) {
  if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b;
  return Math.abs(a - b) <= 4 * Number.EPSILON * Math.max(Math.abs(a), Math.abs(b), 1);
}
export function snapToFrame(t: number, fps: number) {
  const grid = Number.isFinite(fps) && fps > 0 ? fps : 30;
  const pos = t * grid;
  const nearest = Math.round(pos);
  return Math.abs(pos - nearest) <= FRAME_BOUNDARY_EPSILON ? nearest / grid : t;
}
export function clipVisibleAt(t: number, start: number, end: number, compDuration: number) {
  const started = t >= start || sameInstant(t, start);
  const inWindow = started && t < end && !sameInstant(t, end);
  return inWindow || (started && compDuration > 0 && end >= compDuration - TERMINAL_EPSILON_SECONDS);
}

type StartExpr = { abs: number } | { ref: string; off: number };
const validRef = (s: string) => s.length > 0 && /^[A-Za-z0-9_.:-]+$/.test(s);
function num(s: string | null | undefined): number | undefined {
  if (s === null || s === undefined) return undefined;
  const t = s.trim();
  if (!t) return undefined;
  const v = Number(t);
  return Number.isFinite(v) ? v : undefined;
}
export function parseStart(raw: string | null): StartExpr | undefined {
  const s = raw?.trim();
  if (!s) return undefined;
  const v = num(s);
  if (v !== undefined) return { abs: v };
  if (validRef(s)) return { ref: s, off: 0 };
  const m = /^(.*?)([+-])\s*([\d.]+)$/.exec(s);
  if (!m) return undefined;
  const ref = m[1].trim();
  if (!validRef(ref)) return undefined;
  const mag = parseFloat(m[3]);
  return { ref, off: m[2] === "-" ? -mag : mag };
}

/** Where each `<video>` / `<audio>` sits on the composition's clock: [start, end) by element id. */
export type MediaWindows = Map<string, [number, number]>;

export class Resolver {
  private starts = new Map<Element, number | null>();
  private durs = new Map<Element, number | null>();
  private visiting = new Set<Element>();
  constructor(
    private doc: Document,
    private timelineDurations: Map<string, number>,
    private media: MediaWindows,
  ) {}
  private closestComp(from: Element | null): Element | null {
    for (let e = from; e; e = e.parentElement) if (e.hasAttribute("data-composition-id")) return e;
    return null;
  }
  private findRef(id: string): Element | null {
    for (const e of Array.from(this.doc.querySelectorAll("[id]"))) if (e.id === id) return e;
    for (const e of Array.from(this.doc.querySelectorAll("[data-composition-id]"))) if (e.getAttribute("data-composition-id") === id) return e;
    return null;
  }
  duration(el: Element): number | undefined {
    if (this.durs.has(el)) return this.durs.get(el) ?? undefined;
    let r: number | undefined;
    for (const a of ["data-duration", "data-hf-authored-duration"]) {
      const d = num(el.getAttribute(a));
      if (d !== undefined && d > 0) {
        r = d;
        break;
      }
    }
    if (r === undefined) {
      const start = this.start(el, 0);
      for (const a of ["data-end", "data-hf-authored-end"]) {
        const end = num(el.getAttribute(a));
        if (end !== undefined && end > start) {
          r = end - start;
          break;
        }
      }
    }
    if (r === undefined && (el.localName === "video" || el.localName === "audio") && el.id) {
      const w = this.media.get(el.id);
      if (w && w[1] > w[0]) r = w[1] - w[0];
    }
    if (r === undefined) {
      const cid = el.getAttribute("data-composition-id");
      const d = cid ? this.timelineDurations.get(cid) : undefined;
      if (d !== undefined && Number.isFinite(d) && d > 0) r = d;
    }
    this.durs.set(el, r ?? null);
    return r;
  }
  private hostOffset(el: Element, fallback: number): number {
    const comp = el.hasAttribute("data-composition-id") ? this.closestComp(el.parentElement) : this.closestComp(el);
    return comp ? this.start(comp, fallback) : 0;
  }
  start(el: Element, fallback: number): number {
    if (this.starts.has(el)) return this.starts.get(el) ?? fallback;
    if (this.visiting.has(el)) return fallback;
    this.visiting.add(el);
    const expr = parseStart(el.getAttribute("data-start"));
    let result: number;
    if (!expr) {
      let r: number | null = null;
      const p = el.parentElement;
      if (el.hasAttribute("data-composition-id") && p && (p.hasAttribute("data-composition-src") || p.hasAttribute("data-composition-id") || p.hasAttribute("data-composition-file"))) r = this.start(p, fallback);
      this.starts.set(el, r);
      result = r ?? fallback;
    } else if ("abs" in expr) {
      const host = this.hostOffset(el, fallback);
      const media = el.localName === "video" || el.localName === "audio";
      const global = media && host > 0 && !el.hasAttribute("data-hf-auto-start") && (el.getAttribute("data-hf-media-start-basis") ?? "").trim().toLowerCase() === "global";
      result = global ? Math.max(expr.abs, 0) : Math.max(host + Math.max(expr.abs, 0), 0);
      this.starts.set(el, result);
    } else {
      const target = this.findRef(expr.ref);
      if (!target) {
        this.starts.set(el, null);
        result = fallback;
      } else {
        const ts = this.start(target, 0);
        const d = this.duration(target);
        result = Math.max(d !== undefined && d > 0 ? ts + d + expr.off : ts + expr.off, 0);
        this.starts.set(el, result);
      }
    }
    this.visiting.delete(el);
    return result;
  }
}

type Timed = { el: El; start: number; end: number; origin: number; ancestors: number[]; inFlowLeaf: boolean; hiddenAttr: boolean };

export class Timing {
  timed: Timed[] = [];
  private applied: (boolean | undefined)[] = [];
  constructor(
    doc: Document,
    root: Element | null,
    fps: number,
    public compDuration: number,
    timelineDurations: Map<string, number>,
    media: MediaWindows,
    positionOf: (el: Element) => string,
  ) {
    const r = new Resolver(doc, timelineDurations, media);
    const nodes = Array.from(doc.querySelectorAll("[data-start], [data-hidden]")).filter((e) => e !== root && !["script", "style", "link", "meta"].includes(e.localName));
    const index = new Map<Element, number>(nodes.map((n, i) => [n, i]));
    for (const node of nodes) {
      const start = r.start(node, 0);
      let duration = r.duration(node);
      const cid = node.getAttribute("data-composition-id");
      if (cid) {
        const authored = ["data-duration", "data-end", "data-hf-authored-duration", "data-hf-authored-end"].some((a) => node.hasAttribute(a));
        if (!authored && !(duration !== undefined && duration > 0)) {
          const d = timelineDurations.get(cid);
          duration = d !== undefined && Number.isFinite(d) && d > 0 ? d : undefined;
        }
      }
      const end = duration !== undefined && duration > 0 ? start + duration : Infinity;
      const ancestors: number[] = [];
      for (let p = node.parentElement; p && p !== root; p = p.parentElement) {
        const i = index.get(p);
        if (i !== undefined && p.hasAttribute("data-start")) ancestors.push(i);
      }
      const leaf = !node.querySelector("[data-start]");
      const pos = positionOf(node);
      this.timed.push({
        el: node as El,
        start: snapToFrame(start, fps),
        end: Number.isFinite(end) ? snapToFrame(end, fps) : end,
        origin: start,
        ancestors,
        inFlowLeaf: leaf && (pos === "static" || pos === "relative" || pos === "sticky" || pos === ""),
        hiddenAttr: node.hasAttribute("data-hidden"),
      });
    }
  }
  visible(i: number, t: number) {
    const e = this.timed[i];
    if (e.hiddenAttr) return false;
    return clipVisibleAt(t, e.start, e.end, this.compDuration) && e.ancestors.every((a) => !this.timed[a].hiddenAttr && clipVisibleAt(t, this.timed[a].start, this.timed[a].end, this.compDuration));
  }
  /** Visibility (and display for in-flow leaves) for time t. */
  apply(t: number) {
    for (let i = 0; i < this.timed.length; i++) {
      const v = this.visible(i, t);
      if (this.applied[i] === v) continue;
      this.applied[i] = v;
      const e = this.timed[i];
      if (e.hiddenAttr) {
        e.el.style.setProperty("display", "none");
        continue;
      }
      e.el.style.setProperty("visibility", v ? "visible" : "hidden");
      if (e.inFlowLeaf) {
        if (v) e.el.style.removeProperty("display");
        else e.el.style.setProperty("display", "none");
      }
    }
  }
}
