// Range and Selection as the DOM spec keeps them (docs/ts-native.feature P14a): boundary points in tree order, the
// selection's range live (moved by the tree's mutations as Chrome moves it), selectionchange queued as a task, and
// the selection mirrored to the host so it paints (with a caret in a focused editable).
import * as wire from "../wire";
import { Comment_, DOMRect_, DocumentFragment_, Element_, Event_, Node_, S, Text_, dispatch, doc, nodeList, rangeRects, run } from "./all";

// ---------------------------------------------------------------- boundary points

/** A node's length as boundary points count it: a text's or comment's characters (UTF-16), else its children. */
export function nodeLength(n: Node_): number {
  return n instanceof Text_ || n instanceof Comment_ ? n.data.length : n._kids.length;
}

export function indexOf(n: Node_): number {
  return n.parentNode ? n.parentNode._kids.indexOf(n) : 0;
}

/** Where (a, ao) is against (b, bo) in tree order: -1 before, 0 equal, 1 after (DOM's boundary point position). */
export function cmpPoint(a: Node_, ao: number, b: Node_, bo: number): number {
  if (a === b) return ao === bo ? 0 : ao < bo ? -1 : 1;
  const pos = a.compareDocumentPosition(b);
  // b precedes a: the answer the other way round
  if (pos & 2 && !(pos & 8)) return -cmpPoint(b, bo, a, ao);
  if (pos & 8) return -cmpPoint(b, bo, a, ao);
  // a is b's ancestor: b's ancestor that's a's child against ao
  if (pos & 16) {
    let c: Node_ = b;
    while (c.parentNode !== a) c = c.parentNode!;
    return indexOf(c) < ao ? 1 : -1;
  }
  return -1;
}

function check(n: Node_, o: number) {
  if (n.nodeType === 10) throw new Error("InvalidNodeTypeError: a doctype has no boundary points");
  if (o < 0 || o > nodeLength(n)) throw new Error(`IndexSizeError: offset ${o} is past the node's length ${nodeLength(n)}`);
}

// ---------------------------------------------------------------- Range

/** Ranges the tree's mutations move: the selection's (a range made by createRange keeps where it was set, which is
 *  all ProseMirror needs of one: it sets both ends before each read). */
const live = new Set<Range_>();

export class Range_ {
  static START_TO_START = 0;
  static START_TO_END = 1;
  static END_TO_END = 2;
  static END_TO_START = 3;
  startContainer: Node_;
  startOffset = 0;
  endContainer: Node_;
  endOffset = 0;
  /** the selection this is the range of: its changes are the selection's */
  _sel: Selection_ | null = null;
  constructor() {
    this.startContainer = this.endContainer = doc;
  }
  get collapsed() {
    return this.startContainer === this.endContainer && this.startOffset === this.endOffset;
  }
  get commonAncestorContainer(): Node_ {
    for (let n: Node_ | null = this.startContainer; n; n = n.parentNode) if (n.contains(this.endContainer)) return n;
    return doc;
  }
  _set(start: boolean, n: Node_, o: number) {
    check(n, o);
    if (start) {
      this.startContainer = n;
      this.startOffset = o;
      if (this.endContainer.getRootNode() !== n.getRootNode() || cmpPoint(n, o, this.endContainer, this.endOffset) > 0) [this.endContainer, this.endOffset] = [n, o];
    } else {
      this.endContainer = n;
      this.endOffset = o;
      if (this.startContainer.getRootNode() !== n.getRootNode() || cmpPoint(n, o, this.startContainer, this.startOffset) < 0) [this.startContainer, this.startOffset] = [n, o];
    }
    this._sel?._changed(true);
  }
  setStart(n: Node_, o: number) {
    this._set(true, n, o);
  }
  setEnd(n: Node_, o: number) {
    this._set(false, n, o);
  }
  setStartBefore(n: Node_) {
    this._set(true, n.parentNode!, indexOf(n));
  }
  setStartAfter(n: Node_) {
    this._set(true, n.parentNode!, indexOf(n) + 1);
  }
  setEndBefore(n: Node_) {
    this._set(false, n.parentNode!, indexOf(n));
  }
  setEndAfter(n: Node_) {
    this._set(false, n.parentNode!, indexOf(n) + 1);
  }
  collapse(toStart = false) {
    if (toStart) [this.endContainer, this.endOffset] = [this.startContainer, this.startOffset];
    else [this.startContainer, this.startOffset] = [this.endContainer, this.endOffset];
    this._sel?._changed(true);
  }
  selectNode(n: Node_) {
    const p = n.parentNode;
    if (!p) throw new Error("InvalidNodeTypeError: the node has no parent");
    const i = indexOf(n);
    this._both(p, i, p, i + 1);
  }
  selectNodeContents(n: Node_) {
    this._both(n, 0, n, nodeLength(n));
  }
  _both(a: Node_, ao: number, b: Node_, bo: number) {
    this.startContainer = a;
    this.startOffset = ao;
    this.endContainer = b;
    this.endOffset = bo;
    this._sel?._changed(true);
  }
  cloneRange(): Range_ {
    const r = new Range_();
    r._both(this.startContainer, this.startOffset, this.endContainer, this.endOffset);
    return r;
  }
  detach() {}
  compareBoundaryPoints(how: number, other: Range_): number {
    const [a, ao] = how === 0 || how === 3 ? [this.startContainer, this.startOffset] : [this.endContainer, this.endOffset];
    const [b, bo] = how === 0 || how === 1 ? [other.startContainer, other.startOffset] : [other.endContainer, other.endOffset];
    return cmpPoint(a, ao, b, bo);
  }
  comparePoint(n: Node_, o: number): number {
    check(n, o);
    if (cmpPoint(n, o, this.startContainer, this.startOffset) < 0) return -1;
    if (cmpPoint(n, o, this.endContainer, this.endOffset) > 0) return 1;
    return 0;
  }
  isPointInRange(n: Node_, o: number): boolean {
    if (n.getRootNode() !== this.startContainer.getRootNode()) return false;
    return this.comparePoint(n, o) === 0;
  }
  intersectsNode(n: Node_): boolean {
    const p = n.parentNode;
    if (!p) return true;
    const i = indexOf(n);
    return cmpPoint(p, i, this.endContainer, this.endOffset) < 0 && cmpPoint(p, i + 1, this.startContainer, this.startOffset) > 0;
  }
  /** the text nodes the range covers, each with the part of it covered: [node, from, to] */
  _texts(): [Text_, number, number][] {
    const out: [Text_, number, number][] = [];
    const { startContainer: s, startOffset: so, endContainer: e, endOffset: eo } = this;
    if (s === e && s instanceof Text_) return [[s, so, eo]];
    const root = this.commonAncestorContainer;
    const walk = (n: Node_) => {
      for (const k of n._kids) {
        if (k instanceof Text_) {
          const from = k === s ? so : 0;
          const to = k === e ? eo : k.data.length;
          if (k === s || k === e || (cmpPoint(k, 0, s, so) >= 0 && cmpPoint(k, k.data.length, e, eo) <= 0)) {
            if (k === s || k === e || this.intersectsNode(k)) out.push([k, from, to]);
          }
        } else walk(k);
      }
    };
    if (root instanceof Text_) return [[root, so, eo]];
    walk(root);
    return out;
  }
  toString(): string {
    return this._texts()
      .map(([t, a, b]) => t.data.slice(a, b))
      .join("");
  }
  getClientRects(): DOMRect_[] {
    return nodeList(rangeRects(this));
  }
  getBoundingClientRect(): DOMRect_ {
    const rs = rangeRects(this);
    if (!rs.length) return new DOMRect_(0, 0, 0, 0);
    let [l, t, r, b] = [Infinity, Infinity, -Infinity, -Infinity];
    for (const x of rs) {
      l = Math.min(l, x.left);
      t = Math.min(t, x.top);
      r = Math.max(r, x.right);
      b = Math.max(b, x.bottom);
    }
    return new DOMRect_(l, t, r - l, b - t);
  }
  /** the range's contents removed: a text's covered part cut, the nodes wholly inside taken out, and the range
   *  collapsed where they were (DOM's deleteContents, for one container or across siblings' subtrees) */
  deleteContents() {
    if (this.collapsed) return;
    const { startContainer: s, startOffset: so, endContainer: e, endOffset: eo } = this;
    if (s === e && s instanceof Text_) {
      s.deleteData(so, eo - so);
      return;
    }
    // where the range collapses: the start, or after the start's ancestor that's under the end's
    let [ns, no] = [s, so];
    if (!s.contains(e)) {
      let r: Node_ = s;
      while (r.parentNode && !r.parentNode.contains(e)) r = r.parentNode;
      [ns, no] = [r.parentNode!, indexOf(r) + 1];
    }
    // the nodes wholly inside whose parent isn't
    const inside = (k: Node_) => cmpPoint(k.parentNode!, indexOf(k), s, so) >= 0 && cmpPoint(k.parentNode!, indexOf(k) + 1, e, eo) <= 0;
    const whole: Node_[] = [];
    const walk = (n: Node_) => {
      for (const k of n._kids) {
        if (inside(k)) whole.push(k);
        else if (k.contains(s) || k.contains(e) || this.intersectsNode(k)) walk(k);
      }
    };
    walk(this.commonAncestorContainer);
    if (s instanceof Text_) s.deleteData(so, s.data.length - so);
    for (const n of whole) n.parentNode?.removeChild(n);
    if (e instanceof Text_) e.deleteData(0, eo);
    no = Math.min(no, nodeLength(ns));
    this._both(ns, no, ns, no);
  }
  insertNode(n: Node_) {
    const { startContainer: s, startOffset: so } = this;
    let parent: Node_;
    let before: Node_ | null;
    if (s instanceof Text_) {
      parent = s.parentNode!;
      before = so === 0 ? s : so === s.data.length ? s.nextSibling : s.splitText(so);
    } else {
      parent = s;
      before = s._kids[so] ?? null;
    }
    const grow = this.collapsed;
    parent.insertBefore(n, before);
    if (grow) this.setEnd(parent, indexOf(n) + (n instanceof DocumentFragment_ ? 0 : 1));
  }
  cloneContents(): DocumentFragment_ {
    const f = new DocumentFragment_();
    f.appendChild(doc.createTextNode(this.toString()));
    return f;
  }
  extractContents(): DocumentFragment_ {
    const f = this.cloneContents();
    this.deleteContents();
    return f;
  }
  surroundContents(n: Node_) {
    const f = this.extractContents();
    this.insertNode(n);
    n.appendChild(f);
    this.selectNode(n);
  }
}

// ---------------------------------------------------------------- the tree's mutations, on live ranges

/** A child list change (from nodes.ts's mutated): `added` put in at `at`, or `removed` taken out of `at`. */
export function rangesChildList(parent: Node_, at: number, added: number, removed: Node_ | null) {
  for (const r of live) {
    for (const start of [true, false]) {
      let n = start ? r.startContainer : r.endContainer;
      let o = start ? r.startOffset : r.endOffset;
      if (removed && removed.contains(n)) [n, o] = [parent, at];
      else if (n === parent && removed && o > at) o--;
      else if (n === parent && added && o > at) o += added;
      if (start) [r.startContainer, r.startOffset] = [n, o];
      else [r.endContainer, r.endOffset] = [n, o];
    }
    r._sel?._changed(false);
  }
}

/** A text's data replaced from `at`, `count` characters by `len` new ones (DOM's replace data). */
export function rangesText(t: Text_, at: number, count: number, len: number) {
  for (const r of live) {
    let moved = false;
    for (const start of [true, false]) {
      const n = start ? r.startContainer : r.endContainer;
      if (n !== t) continue;
      let o = start ? r.startOffset : r.endOffset;
      if (o > at && o <= at + count) o = at;
      else if (o > at + count) o += len - count;
      else continue;
      moved = true;
      if (start) r.startOffset = o;
      else r.endOffset = o;
    }
    if (moved) r._sel?._changed(false);
  }
}

/** A text split at `at` into it and `next` (DOM's split a Text node): points past the split go to the new node. */
export function rangesSplit(t: Text_, at: number, next: Text_) {
  for (const r of live) {
    if (r.startContainer === t && r.startOffset > at) [r.startContainer, r.startOffset] = [next, r.startOffset - at];
    if (r.endContainer === t && r.endOffset > at) [r.endContainer, r.endOffset] = [next, r.endOffset - at];
  }
}

export const liveRanges = () => live.size > 0;

// ---------------------------------------------------------------- Selection

export class Selection_ {
  _range: Range_ | null = null;
  /** anchor before focus */
  _forward = true;
  get rangeCount() {
    return this._range ? 1 : 0;
  }
  get anchorNode() {
    return this._range ? (this._forward ? this._range.startContainer : this._range.endContainer) : null;
  }
  get anchorOffset() {
    return this._range ? (this._forward ? this._range.startOffset : this._range.endOffset) : 0;
  }
  get focusNode() {
    return this._range ? (this._forward ? this._range.endContainer : this._range.startContainer) : null;
  }
  get focusOffset() {
    return this._range ? (this._forward ? this._range.endOffset : this._range.startOffset) : 0;
  }
  get baseNode() {
    return this.anchorNode;
  }
  get baseOffset() {
    return this.anchorOffset;
  }
  get extentNode() {
    return this.focusNode;
  }
  get extentOffset() {
    return this.focusOffset;
  }
  get isCollapsed() {
    return !this._range || this._range.collapsed;
  }
  get type() {
    return !this._range ? "None" : this._range.collapsed ? "Caret" : "Range";
  }
  get direction() {
    return !this._range || this._range.collapsed ? "none" : this._forward ? "forward" : "backward";
  }
  getRangeAt(i: number): Range_ {
    if (i !== 0 || !this._range) throw new Error(`IndexSizeError: no range at ${i}`);
    return this._range;
  }
  _adopt(r: Range_ | null, forward = true) {
    if (this._range && this._range !== r) {
      this._range._sel = null;
      live.delete(this._range);
    }
    this._range = r;
    this._forward = forward;
    if (r) {
      r._sel = this;
      live.add(r);
    }
    this._changed(true);
  }
  addRange(r: Range_) {
    if (this._range || !r.startContainer.isConnected) return;
    this._adopt(r);
  }
  removeRange(r: Range_) {
    if (r !== this._range) throw new Error("NotFoundError: not the selection's range");
    this._adopt(null);
  }
  removeAllRanges() {
    if (this._range) this._adopt(null);
  }
  empty() {
    this.removeAllRanges();
  }
  collapse(n: Node_ | null, o = 0) {
    if (!n) return this.removeAllRanges();
    check(n, o);
    if (!n.isConnected) return;
    const r = new Range_();
    r._both(n, o, n, o);
    this._adopt(r);
  }
  setPosition(n: Node_ | null, o = 0) {
    this.collapse(n, o);
  }
  collapseToStart() {
    if (!this._range) throw new Error("InvalidStateError: no selection");
    this.collapse(this._range.startContainer, this._range.startOffset);
  }
  collapseToEnd() {
    if (!this._range) throw new Error("InvalidStateError: no selection");
    this.collapse(this._range.endContainer, this._range.endOffset);
  }
  extend(n: Node_, o = 0) {
    if (!this._range) throw new Error("InvalidStateError: no selection to extend");
    check(n, o);
    this.setBaseAndExtent(this.anchorNode!, this.anchorOffset, n, o);
  }
  setBaseAndExtent(a: Node_, ao: number, f: Node_, fo: number) {
    check(a, ao);
    check(f, fo);
    if (!a.isConnected || !f.isConnected) return;
    const r = new Range_();
    const fwd = cmpPoint(a, ao, f, fo) <= 0;
    if (fwd) r._both(a, ao, f, fo);
    else r._both(f, fo, a, ao);
    this._adopt(r, fwd);
  }
  selectAllChildren(n: Node_) {
    this.setBaseAndExtent(n, 0, n, nodeLength(n));
  }
  containsNode(n: Node_, partly = false): boolean {
    const r = this._range;
    if (!r || !n.parentNode) return false;
    const p = n.parentNode;
    const i = indexOf(n);
    if (partly) return r.intersectsNode(n);
    return cmpPoint(p, i, r.startContainer, r.startOffset) >= 0 && cmpPoint(p, i + 1, r.endContainer, r.endOffset) <= 0;
  }
  deleteFromDocument() {
    this._range?.deleteContents();
  }
  /** Selection.modify: moved or extended by a character, word or line (editing.ts's moves) */
  modify(alter: string, direction: string, granularity: string) {
    (globalThis as { __raea_modify?: (s: Selection_, a: string, d: string, g: string) => void }).__raea_modify?.(this, alter, direction, granularity);
  }
  toString() {
    return this._range?.toString() ?? "";
  }
  /** The selection changed: selectionchange queued (one per task, as Chrome coalesces them), and the host told
   *  what to paint. `now` is a change made through the API rather than a mutation's move. */
  _changed(_now: boolean) {
    scheduleChange();
  }
}

export const selection = new Selection_();

let queued = false;

let lastSent = "";

function scheduleChange() {
  if (queued) return;
  queued = true;
  setTimeout(() => {
    queued = false;
    mirror();
    run(() => dispatch(doc, new Event_("selectionchange", { bubbles: false, cancelable: false }), true));
  }, 0);
}

/** A boundary point as the host takes it: a text's wire id and UTF-16 offset, or an element's wire id and the wire id
 *  of the child it's before (null: at its end). Unwired points have none. */
function hostPoint(n: Node_, o: number): { id: number; offset?: number; before?: number | null } | null {
  if (!n._wired || n.__wid === undefined) return null;
  if (n instanceof Text_) return { id: n.__wid, offset: o };
  if (!(n instanceof Element_)) return null;
  // the element's own string (React's text content) is one text: its offset is in it
  if (n._inline !== undefined) return { id: n.__wid, offset: o ? n._inline.length : 0 };
  for (let i = o; i < n._kids.length; i++) if (n._kids[i]._wired) return { id: n.__wid, before: n._kids[i].__wid };
  return { id: n.__wid, before: null };
}

/** What the host paints: the selection's two ends, and whether a caret shows (collapsed in a focused editable). */
export function mirror() {
  const s = selection;
  const a = s.anchorNode ? hostPoint(s.anchorNode, s.anchorOffset) : null;
  const f = s.focusNode ? hostPoint(s.focusNode, s.focusOffset) : null;
  const editable = !!S.active && (S.active as { isContentEditable?: boolean }).isContentEditable === true && !!s.focusNode && S.active.contains(s.focusNode);
  const msg = a && f ? { anchor: a, focus: f, caret: editable && s.isCollapsed } : null;
  const json = JSON.stringify(msg);
  if (json === lastSent) return;
  lastSent = json;
  try {
    wire.callSync("selection.set", msg);
  } catch {
    // a host without selection painting (the bevy_ui page)
  }
}

/** The host said the selection's endpoints moved off the tree (a fresh mount): forget it. */
export function resetSelection() {
  selection._adopt(null);
  lastSent = "";
}
