// Text geometry the host lays out (docs/ts-native.feature P14a): a range's client rects, and the caret position at a
// point (caretPositionFromPoint, caretRangeFromPoint). The host answers from the page as the JS wrote it (raea-page's
// `text.rects` and `caret.at`, synchronous calls); without one (tests) the text is taken as one line in its parent's
// box at a fixed advance, enough to keep a caller's arithmetic going.
import * as wire from "../wire";
import { DOMRect_, Element_, Node_, Range_, Text_, body, computedOf, doc, hit } from "./all";

/** a character's advance where there's no host to ask: of the parent's font size */
function advance(t: Text_): number {
  const p = t.parentNode;
  const fs = p instanceof Element_ ? parseFloat(String(computedOf(p).fontSize ?? "16")) || 16 : 16;
  return fs * 0.55;
}

/** rects for one text's part [a, b): the host's, else the estimate */
function estimate(t: Text_, a: number, b: number): DOMRect_[] {
  const p = t.parentNode;
  if (!(p instanceof Element_)) return [];
  const box = p.getBoundingClientRect();
  const w = advance(t);
  // the text's start: after the texts before it in the same parent
  let before = 0;
  for (const k of p._kids) {
    if (k === t) break;
    before += (k.textContent ?? "").length;
  }
  const h = Math.max(box.height, w / 0.55) || w / 0.55;
  return [new DOMRect_(box.left + (before + a) * w, box.top, (b - a) * w, Math.min(h, (w / 0.55) * 1.25))];
}

/** A range's client rects, as Chrome's Range.getClientRects: one per line box of each text it covers (a collapsed
 *  range in a text: the caret's zero-width rect). */
export function rangeRects(r: Range_): DOMRect_[] {
  const texts = r._texts().filter(([t, a, b]) => t.isConnected && (a < b || r.collapsed));
  if (!texts.length) return [];
  const wired = texts.filter(([t]) => t._wired && t.__wid !== undefined);
  let host: number[][][] | undefined;
  if (wired.length === texts.length) {
    try {
      host = wire.callSync<number[][][]>("text.rects", { spans: wired.map(([t, a, b]) => [t.__wid, a, b]) });
    } catch {
      host = undefined;
    }
  }
  const out: DOMRect_[] = [];
  texts.forEach(([t, a, b], i) => {
    const rs = host?.[i];
    if (rs) for (const [x, y, w, h] of rs) out.push(new DOMRect_(x, y, w, h));
    else out.push(...estimate(t, a, b));
  });
  return out;
}

/** The node on the host as wire id `id`: an element or a text. */
export function nodeByWid(id: number, from: Node_ = doc): Node_ | null {
  if (from.__wid === id && from._wired) return from;
  for (const k of from._kids) {
    const n = nodeByWid(id, k);
    if (n) return n;
  }
  return null;
}

export class CaretPosition_ {
  constructor(
    public offsetNode: Node_,
    public offset: number,
  ) {}
  getClientRect(): DOMRect_ | null {
    const r = new Range_();
    r.setStart(this.offsetNode, this.offset);
    return r.getBoundingClientRect();
  }
}

/** The caret position at (x, y): the host's (its text hit testing), else the nearest of the element there's texts. */
export function caretAt(x: number, y: number): CaretPosition_ | null {
  let ans: { id: number; offset: number } | null | undefined;
  try {
    ans = wire.callSync<{ id: number; offset: number } | null>("caret.at", { x, y });
  } catch {
    ans = undefined;
  }
  if (ans) {
    const n = nodeByWid(ans.id);
    if (n instanceof Text_) return new CaretPosition_(n, Math.min(ans.offset, n.data.length));
    if (n instanceof Element_) {
      // the element's own string, or a point between its children
      if (n._inline !== undefined) return new CaretPosition_(n, 0);
      return new CaretPosition_(n, Math.min(ans.offset, n._kids.length));
    }
  }
  const el = hit(body, x, y);
  if (!el) return null;
  const t = el._kids.find((k): k is Text_ => k instanceof Text_);
  if (!t) return new CaretPosition_(el, 0);
  const [r] = estimate(t, 0, 0);
  const o = r ? Math.round((x - r.left) / advance(t)) : 0;
  return new CaretPosition_(t, Math.max(0, Math.min(t.data.length, o)));
}

export function caretRangeAt(x: number, y: number): Range_ | null {
  const p = caretAt(x, y);
  if (!p) return null;
  const r = new Range_();
  r.setStart(p.offsetNode, p.offset);
  return r;
}
