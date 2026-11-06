// TreeWalker and NodeFilter, and the Document.
import * as wire from "../wire";
import { Comment_, CustomEvent_, DocumentFragment_, Range_, caretAt, caretRangeAt, implementation, selection, Element_, Event_, HTML_NS, KeyboardEvent_, MouseEvent_, Node_, S, Text_, body, g, head, hit, html, makeElement, nodeList, query } from "./all";

// ---------------------------------------------------------------- TreeWalker

export const NodeFilter_ = {
  FILTER_ACCEPT: 1,
  FILTER_REJECT: 2,
  FILTER_SKIP: 3,
  SHOW_ALL: 0xffffffff,
  SHOW_ELEMENT: 0x1,
  SHOW_ATTRIBUTE: 0x2,
  SHOW_TEXT: 0x4,
  SHOW_COMMENT: 0x80,
  SHOW_DOCUMENT: 0x100,
  SHOW_DOCUMENT_FRAGMENT: 0x400,
};

export class TreeWalker_ {
  currentNode: Node_;
  constructor(
    public root: Node_,
    public whatToShow: number = 0xffffffff,
    public filter: any = null,
  ) {
    this.currentNode = root;
  }
  _accept(n: Node_): number {
    if (!(this.whatToShow & (1 << (n.nodeType - 1)))) return 3;
    if (!this.filter) return 1;
    const r = typeof this.filter === "function" ? this.filter(n) : this.filter.acceptNode(n);
    return r ?? 1;
  }
  parentNode(): Node_ | null {
    for (let n = this.currentNode; n && n !== this.root; ) {
      n = n.parentNode!;
      if (n && this._accept(n) === 1) return (this.currentNode = n);
    }
    return null;
  }
  firstChild() {
    return this._child(true);
  }
  lastChild() {
    return this._child(false);
  }
  _child(first: boolean): Node_ | null {
    let n: Node_ | null = first ? this.currentNode.firstChild : this.currentNode.lastChild;
    while (n) {
      const r = this._accept(n);
      if (r === 1) return (this.currentNode = n);
      if (r === 3) {
        const c: Node_ | null = first ? n.firstChild : n.lastChild;
        if (c) {
          n = c;
          continue;
        }
      }
      for (;;) {
        const sib: Node_ | null = first ? n!.nextSibling : n!.previousSibling;
        if (sib) {
          n = sib;
          break;
        }
        const p: Node_ | null = n!.parentNode;
        if (!p || p === this.root || p === this.currentNode) return null;
        n = p;
      }
    }
    return null;
  }
  nextSibling() {
    return this._sibling(true);
  }
  previousSibling() {
    return this._sibling(false);
  }
  _sibling(next: boolean): Node_ | null {
    let n: Node_ | null = this.currentNode;
    if (n === this.root) return null;
    for (;;) {
      let sib: Node_ | null = next ? n!.nextSibling : n!.previousSibling;
      while (sib) {
        n = sib;
        const r = this._accept(n);
        if (r === 1) return (this.currentNode = n);
        sib = next ? n.firstChild : n.lastChild;
        if (r === 2 || !sib) sib = next ? n.nextSibling : n.previousSibling;
      }
      n = n!.parentNode;
      if (!n || n === this.root) return null;
      if (this._accept(n) === 1) return null;
    }
  }
  nextNode(): Node_ | null {
    let n: Node_ = this.currentNode;
    let r = 1;
    for (;;) {
      while (r !== 2 && n.firstChild) {
        n = n.firstChild;
        r = this._accept(n);
        if (r === 1) return (this.currentNode = n);
      }
      let sib: Node_ | null = null;
      for (let t: Node_ | null = n; t; t = t.parentNode) {
        if (t === this.root) return null;
        sib = t.nextSibling;
        if (sib) break;
      }
      if (!sib) return null;
      n = sib;
      r = this._accept(n);
      if (r === 1) return (this.currentNode = n);
    }
  }
  previousNode(): Node_ | null {
    let n: Node_ = this.currentNode;
    while (n !== this.root) {
      let sib = n.previousSibling;
      while (sib) {
        n = sib;
        let r = this._accept(n);
        while (r !== 2 && n.lastChild) {
          n = n.lastChild;
          r = this._accept(n);
        }
        if (r === 1) return (this.currentNode = n);
        sib = n.previousSibling;
      }
      if (n === this.root || !n.parentNode) return null;
      n = n.parentNode;
      if (this._accept(n) === 1) return (this.currentNode = n);
    }
    return null;
  }
}

// ---------------------------------------------------------------- document

// document.fonts where the host loads the page's @font-face faces itself (raea-page's Blitz page says so with
// `__ui_fonts`): load() and ready settle once none is still loading, as a FontFaceSet's do. Elsewhere it's absent,
// and a page that checks for it draws at once (the bevy_ui page has its faces before the page runs).
export const fontWaits: (() => void)[] = [];

const fontsLoaded = () =>
  new Promise<void>((done) => {
    fontWaits.push(done);
    if (fontWaits.length === 1) wire.emit("fonts.wait", true);
  });

// FontFace (the CSS Font Loading API): a face made in script becomes an @font-face rule in one sheet of the page's
// own (<style data-raea-fonts> in body), which the host loads as it loads any sheet's faces; load() and `loaded`
// settle as document.fonts does, once nothing is still loading. Excalidraw makes its fonts this way.
const faces = new Set<FontFace_>();
let faceSheet: Element_ | null = null;

const cssString = (s: string) => `"${s.replace(/["\\]/g, "\\$&")}"`;

function writeFaces() {
  const css = [...faces]
    .map((f) => `@font-face{font-family:${cssString(f.family)};src:${f._src};font-style:${f.style};font-weight:${f.weight};font-stretch:${f.stretch};unicode-range:${f.unicodeRange};font-display:${f.display}}`)
    .join("\n");
  if (!faceSheet) {
    faceSheet = makeElement("style");
    faceSheet.setAttribute("data-raea-fonts", "");
    body.appendChild(faceSheet);
  }
  if (faceSheet.textContent !== css) faceSheet.textContent = css;
}

/** A font's bytes as a data URL, for a face made from an ArrayBuffer or a typed array. */
function bytesUrl(src: ArrayBuffer | ArrayBufferView): string {
  const b = src instanceof ArrayBuffer ? new Uint8Array(src) : new Uint8Array(src.buffer, src.byteOffset, src.byteLength);
  let s = "";
  for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000));
  return `url(data:font/ttf;base64,${btoa(s)})`;
}

type FaceDescriptors = { style?: string; weight?: string; stretch?: string; unicodeRange?: string; display?: string; featureSettings?: string };

export class FontFace_ {
  family: string;
  style = "normal";
  weight = "normal";
  stretch = "normal";
  unicodeRange = "U+0-10FFFF";
  display = "auto";
  featureSettings = "normal";
  variant = "normal";
  status: "unloaded" | "loading" | "loaded" | "error" = "unloaded";
  /** the rule's src: the url()/local() list as given, or the bytes as a data URL */
  _src: string;
  loaded: Promise<FontFace_>;
  private _done!: (f: FontFace_) => void;
  constructor(family: string, source: string | ArrayBuffer | ArrayBufferView, descriptors: FaceDescriptors = {}) {
    this.family = String(family).replace(/^["']|["']$/g, "");
    for (const k of Object.keys(descriptors) as (keyof FaceDescriptors)[]) if (descriptors[k] !== undefined) (this as any)[k] = String(descriptors[k]);
    this._src = typeof source === "string" ? source : bytesUrl(source);
    this.loaded = new Promise((done) => (this._done = done));
    // a face from bytes is loaded once made, as in Chrome (its data is already here)
    if (typeof source !== "string") this.load();
  }
  load(): Promise<FontFace_> {
    if (this.status === "unloaded") {
      this.status = "loading";
      // a face loads by being in the page's sheet, so one loaded by hand joins it as add() would
      faces.add(this);
      writeFaces();
      void fontsLoaded().then(() => {
        this.status = "loaded";
        this._done(this);
        fontSet._loaded(this);
      });
    }
    return this.loaded;
  }
}

/** The families a CSS font shorthand names ("bold 20px Excalifont, sans-serif" -> Excalifont, sans-serif). */
function familiesOf(font: string): string[] {
  const m = font.match(/(?:^|\s)[\d.]+(?:px|pt|em|rem|%|vh|vw)(?:\/\S+)?\s+(.+)$/);
  return (m ? m[1] : font).split(",").map((f) => f.trim().replace(/^["']|["']$/g, "")).filter(Boolean);
}

type FontsListener = ((e: { type: string; fontfaces: FontFace_[] }) => void) | { handleEvent(e: unknown): void };

class FontFaceSet_ {
  onloading: unknown = null;
  onloadingdone: unknown = null;
  onloadingerror: unknown = null;
  private listeners = new Map<string, Set<FontsListener>>();
  private done: FontFace_[] = [];
  get status() {
    return [...faces].some((f) => f.status === "loading") ? "loading" : "loaded";
  }
  get ready() {
    return fontsLoaded().then(() => this);
  }
  /** As in Chrome: the faces of the font's families start loading, and the promise has those loaded. */
  load(font: string, _text?: string) {
    const names = new Set(familiesOf(font));
    const mine = [...faces].filter((f) => names.has(f.family));
    return Promise.all(mine.map((f) => f.load())).then(() => mine.filter((f) => f.status === "loaded"));
  }
  /** False while a face of one of the font's families is in the set but not loaded, as in Chrome; Excalidraw waits on
   *  this, and measured text in the fallback when it always said true. */
  check(font: string, _text?: string) {
    const names = new Set(familiesOf(font));
    const mine = [...faces].filter((f) => names.has(f.family));
    return !mine.length || mine.some((f) => f.status === "loaded");
  }
  /** A face finished loading: one loadingdone for the faces done in this turn, which is how Excalidraw learns to
   *  redraw text it drew in the fallback. */
  _loaded(f: FontFace_) {
    this.done.push(f);
    if (this.done.length > 1) return;
    queueMicrotask(() => {
      const fontfaces = this.done;
      this.done = [];
      const e = { type: "loadingdone", fontfaces };
      if (typeof this.onloadingdone === "function") (this.onloadingdone as (e: unknown) => void)(e);
      for (const l of [...(this.listeners.get("loadingdone") ?? [])]) typeof l === "function" ? l(e) : l.handleEvent(e);
    });
  }
  add(f: FontFace_) {
    if (!faces.has(f)) {
      faces.add(f);
      writeFaces();
    }
    return this;
  }
  delete(f: FontFace_) {
    const had = faces.delete(f);
    if (had) writeFaces();
    return had;
  }
  clear() {
    faces.clear();
    writeFaces();
  }
  get size() {
    return faces.size;
  }
  forEach(fn: (f: FontFace_, k: FontFace_, set: FontFaceSet_) => void) {
    for (const f of faces) fn(f, f, this);
  }
  has(f: FontFace_) {
    return faces.has(f);
  }
  values() {
    return faces.values();
  }
  [Symbol.iterator]() {
    return faces.values();
  }
  addEventListener(type: string, l: FontsListener | null) {
    if (!l) return;
    if (!this.listeners.has(type)) this.listeners.set(type, new Set());
    this.listeners.get(type)!.add(l);
  }
  removeEventListener(type: string, l: FontsListener | null) {
    if (l) this.listeners.get(type)?.delete(l);
  }
}

const fontSet = new FontFaceSet_();

export class Document_ extends Node_ {
  nodeType = 9;
  readyState = "complete";
  visibilityState = "visible";
  hidden = false;
  compatMode = "CSS1Compat";
  characterSet = "UTF-8";
  contentType = "text/html";
  dir = "";
  cookie = "";
  title = "";
  get nodeName() {
    return "#document";
  }
  get ownerDocument() {
    return null;
  }
  get fonts() {
    return (globalThis as { __ui_fonts?: boolean }).__ui_fonts ? fontSet : undefined;
  }
  get documentElement() {
    return html;
  }
  get body() {
    return body;
  }
  get head() {
    return head;
  }
  get defaultView() {
    return g;
  }
  get activeElement(): Element_ | null {
    return S.active && S.active.isConnected ? S.active : body;
  }
  get scrollingElement() {
    return html;
  }
  get fullscreenElement() {
    return null;
  }
  get pointerLockElement() {
    return null;
  }
  get location() {
    return g.location;
  }
  get textContent(): string | null {
    return null;
  }
  set textContent(_v: string | null) {}
  createElement(tag: string, _opts?: unknown) {
    return makeElement(String(tag).toLowerCase());
  }
  createElementNS(ns: string | null, tag: string) {
    return makeElement(tag, ns ?? HTML_NS);
  }
  createTextNode(t: string) {
    return new Text_(t);
  }
  createComment(t: string) {
    return new Comment_(t);
  }
  createDocumentFragment() {
    return new DocumentFragment_();
  }
  createTreeWalker(root: Node_, what?: number, filter?: any) {
    return new TreeWalker_(root, what ?? 0xffffffff, filter ?? null);
  }
  createEvent(kind: string) {
    const C = /mouse/i.test(kind) ? MouseEvent_ : /key/i.test(kind) ? KeyboardEvent_ : /custom/i.test(kind) ? CustomEvent_ : Event_;
    return new C("");
  }
  getElementById(id: string): Element_ | null {
    return query(this, `#${cssEscape(id)}`, true)[0] ?? null;
  }
  querySelector(s: string) {
    return query(this, s, true)[0] ?? null;
  }
  querySelectorAll(s: string) {
    return nodeList(query(this, s, false));
  }
  getElementsByTagName(t: string) {
    return this.querySelectorAll(t);
  }
  getElementsByClassName(c: string) {
    return html.getElementsByClassName(c);
  }
  elementFromPoint(x: number, y: number) {
    return hit(body, x, y) ?? (x >= 0 && y >= 0 && x < S.vw && y < S.vh ? body : null);
  }
  elementsFromPoint(x: number, y: number) {
    const out: Element_[] = [];
    for (let n: Node_ | null = this.elementFromPoint(x, y); n instanceof Element_; n = n.parentNode) out.push(n);
    return out;
  }
  hasFocus() {
    return true;
  }
  get implementation() {
    return implementation;
  }
  getSelection() {
    return selection;
  }
  createRange() {
    const r = new Range_();
    r.setStart(this, 0);
    return r;
  }
  caretPositionFromPoint(x: number, y: number) {
    return caretAt(x, y);
  }
  caretRangeFromPoint(x: number, y: number) {
    return caretRangeAt(x, y);
  }
  execCommand() {
    return false;
  }
  exitFullscreen() {
    return Promise.resolve();
  }
}

export function cssEscape(s: string): string {
  return String(s).replace(/([^a-zA-Z0-9_\u00a0-\uffff-])/g, "\\$1").replace(/^(\d)/, "\\3$1 ");
}
