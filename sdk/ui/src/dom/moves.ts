// Caret moves in an editable, as Chrome makes them (docs/ts-native.feature P14a): by character across text nodes and
// blocks, by word, to a line's ends and up or down a line from the caret's rect, and the pointer's placing, dragging
// and word and paragraph picks. Positions are (text, offset) where there's text, else (element, child index).
import { Element_, Node_, Range_, Selection_, Text_, caretAt, computedOf, rangeRects } from "./all";

type Pos = [Node_, number];

const BLOCK = new Set(["block", "flex", "grid", "list-item", "table", "flow-root"]);

export function isBlock(n: Node_): boolean {
  return n instanceof Element_ && BLOCK.has(String(computedOf(n).display ?? (["div", "p", "li", "h1", "h2", "h3", "h4", "h5", "h6", "ul", "ol", "blockquote", "pre"].includes(n.localName) ? "block" : "inline")));
}

/** the nearest block at or above `n`, up to the editing host */
export function blockOf(n: Node_, host: Element_): Element_ {
  for (let x: Node_ | null = n; x && x !== host; x = x.parentNode) if (x instanceof Element_ && isBlock(x)) return x;
  return host;
}

/** the host's texts in tree order */
export function textsIn(host: Node_): Text_[] {
  const out: Text_[] = [];
  const walk = (n: Node_) => {
    for (const k of n._kids) {
      if (k instanceof Text_) out.push(k);
      else if (k instanceof Element_ && k.getAttribute("contenteditable") !== "false") walk(k);
    }
  };
  walk(host);
  return out;
}

/** a point as a text position: in a text, or the text at or next to an element's offset (none in an empty block) */
export function toText(p: Pos, host: Element_): Pos {
  const [n, o] = p;
  if (n instanceof Text_) return p;
  const texts = textsIn(host);
  if (!texts.length) return p;
  // the first text after the point, else the last before it
  const r = new Range_();
  r.setStart(n, o);
  for (const t of texts) if (r.comparePoint(t, t.data.length) >= 0 && r.comparePoint(t, 0) >= 0) return blockOf(t, host) === blockOf(n, host) || n === host ? [t, 0] : p;
  const last = texts[texts.length - 1];
  return blockOf(last, host) === blockOf(n, host) ? [last, last.data.length] : p;
}

const surrogateHi = (c: number) => c >= 0xd800 && c <= 0xdbff;

/** one character on from `p` (dir -1 back, 1 on): within a text, else into the next text (a block's edge is a step) */
export function charStep(p: Pos, dir: -1 | 1, host: Element_): Pos {
  const [n, o] = toText(p, host);
  if (n instanceof Text_) {
    if (dir < 0 && o > 0) return [n, o - (o > 1 && surrogateHi(n.data.charCodeAt(o - 2)) ? 2 : 1)];
    if (dir > 0 && o < n.data.length) return [n, o + (surrogateHi(n.data.charCodeAt(o)) ? 2 : 1)];
    const texts = textsIn(host);
    const i = texts.indexOf(n);
    const next = texts[i + dir];
    if (!next) return [n, o];
    const sameBlock = blockOf(next, host) === blockOf(n, host);
    if (dir < 0) return [next, sameBlock ? Math.max(0, next.data.length - 1) : next.data.length];
    return [next, sameBlock ? Math.min(next.data.length, 1) : 0];
  }
  return p;
}

const wordChar = (c: string) => /[\p{L}\p{N}_]/u.test(c);

/** to the start of this or the last word (back), or the end of this or the next word (on), as Chrome on a Mac's Alt */
export function wordStep(p: Pos, dir: -1 | 1, host: Element_): Pos {
  let [n, o] = toText(p, host);
  if (!(n instanceof Text_)) return p;
  let t = n as Text_;
  const at = (i: number) => t.data[i] ?? "";
  // past what isn't a word, then through the word
  for (let seen = false; ; ) {
    const c = dir < 0 ? at(o - 1) : at(o);
    if (!c) {
      const step = charStep([t, o], dir, host);
      if (step[0] === t && step[1] === o) return [t, o];
      if (seen) return [t, o];
      [n, o] = step;
      t = n as Text_;
      continue;
    }
    if (wordChar(c)) seen = true;
    else if (seen) return [t, o];
    o += dir;
  }
}

/** the caret's rect at a position (a collapsed range's, or its text's next character's left edge) */
export function caretRect(p: Pos): { x: number; top: number; bottom: number } | null {
  const r = new Range_();
  r.setStart(p[0], p[1]);
  const rs = rangeRects(r);
  if (rs[0]) return { x: rs[0].left, top: rs[0].top, bottom: rs[0].bottom };
  const el = p[0] instanceof Element_ ? p[0] : p[0].parentElement;
  if (!el) return null;
  const b = el.getBoundingClientRect();
  return { x: b.left, top: b.top, bottom: b.bottom };
}

/** the column a run of Up and Down keeps (Chrome's goal x), cleared by any other move */
let goalX: number | null = null;

export const clearGoal = () => (goalX = null);

/** a line up or down from `p`: the position at the caret's x on the line above or below */
export function lineStep(p: Pos, dir: -1 | 1, host: Element_): Pos {
  const c = caretRect(p);
  if (!c) return p;
  const x = goalX ?? c.x;
  goalX = x;
  const h = Math.max(1, c.bottom - c.top);
  const y = dir < 0 ? c.top - h / 2 : c.bottom + h / 2;
  const hb = host.getBoundingClientRect();
  if (y < hb.top || y > hb.bottom) {
    // past the host's first or last line: to its start or end
    const texts = textsIn(host);
    if (!texts.length) return p;
    return dir < 0 ? [texts[0], 0] : [texts[texts.length - 1], texts[texts.length - 1].data.length];
  }
  const at = caretAt(x, y);
  return at && host.contains(at.offsetNode) ? [at.offsetNode, at.offset] : p;
}

/** the start or end of the caret's line (Home, End; Cmd+Left and Right on a Mac) */
export function lineEdge(p: Pos, dir: -1 | 1, host: Element_): Pos {
  const c = caretRect(p);
  if (!c) return p;
  // within the caret's own block (the host's padding has no line in it)
  const hb = blockOf(p[0], host).getBoundingClientRect();
  const at = caretAt(dir < 0 ? hb.left + 1 : hb.right - 1, (c.top + c.bottom) / 2);
  return at && host.contains(at.offsetNode) ? [at.offsetNode, at.offset] : p;
}

/** the word around a position (a double click): its start and end */
export function wordAt(p: Pos, host: Element_): [Pos, Pos] {
  const [n, o] = toText(p, host);
  if (!(n instanceof Text_)) return [p, p];
  const d = n.data;
  const inWord = wordChar(d[o] ?? "") || wordChar(d[o - 1] ?? "");
  const same = (c: string) => (inWord ? wordChar(c) : !wordChar(c) && /\S/.test(c));
  let a = o;
  let b = o;
  while (a > 0 && same(d[a - 1])) a--;
  while (b < d.length && same(d[b])) b++;
  if (a === b && b < d.length) b++;
  return [
    [n, a],
    [n, b],
  ];
}

/** the paragraph around a position (a triple click): its block's contents */
export function blockAt(p: Pos, host: Element_): [Pos, Pos] {
  const b = blockOf(p[0], host);
  const texts = textsIn(b);
  if (!texts.length) return [[b, 0], [b, b._kids.length]];
  const last = texts[texts.length - 1];
  return [
    [texts[0], 0],
    [last, last.data.length],
  ];
}

/** the selection moved (`extend`: its focus only) to a position */
export function moveTo(s: Selection_, p: Pos, extend: boolean) {
  if (extend && s.anchorNode) s.setBaseAndExtent(s.anchorNode, s.anchorOffset, p[0], p[1]);
  else s.collapse(p[0], p[1]);
}

/** Selection.modify: `alter` move or extend, `direction` forward/backward/left/right, by character, word, line or
 *  lineboundary, within the editing host the focus is in */
export function modify(s: Selection_, alter: string, direction: string, granularity: string, host: Element_ | null) {
  if (!s.focusNode || !host) return;
  const dir: -1 | 1 = direction === "backward" || direction === "left" ? -1 : 1;
  const f: Pos = [s.focusNode, s.focusOffset];
  let p: Pos = f;
  if (granularity === "character") p = charStep(f, dir, host);
  else if (granularity === "word") p = wordStep(f, dir, host);
  else if (granularity === "line") p = lineStep(f, dir, host);
  else if (granularity === "lineboundary") p = lineEdge(f, dir, host);
  if (granularity !== "line") clearGoal();
  moveTo(s, p, alter === "extend");
}

