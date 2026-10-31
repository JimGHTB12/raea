// TreeWalker and NodeFilter, and the Document.
import * as wire from "../wire";
import { Comment_, CustomEvent_, DOMRect_, DocumentFragment_, Element_, Event_, HTML_NS, KeyboardEvent_, MouseEvent_, Node_, S, Text_, body, g, head, hit, html, makeElement, nodeList, query } from "./all";

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

class FontFaceSet_ {
  status = "loaded";
  get ready() {
    return fontsLoaded().then(() => this);
  }
  load(_font: string, _text?: string) {
    return fontsLoaded().then(() => []);
  }
  check(_font: string, _text?: string) {
    return true;
  }
  get size() {
    return 0;
  }
  forEach() {}
  has() {
    return false;
  }
  addEventListener() {}
  removeEventListener() {}
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
  getSelection() {
    return g.getSelection();
  }
  createRange() {
    return { setStart() {}, setEnd() {}, selectNodeContents() {}, collapse() {}, getBoundingClientRect: () => new DOMRect_(), getClientRects: () => [], cloneContents: () => new DocumentFragment_(), commonAncestorContainer: body };
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
