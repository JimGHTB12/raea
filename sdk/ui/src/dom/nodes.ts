// Nodes: Node, the tree's mutations, attributes and props, and the style object.
import * as wire from "../wire";
import { Document_, Element_, EventTarget_, MutationRecord_, body, cloneOf, connectedTree, disconnected, doc, htmlDoc, liveRanges, observers, query, queueRecord, rangesChildList, rangesSplit, rangesText, rootContainer, svgNs, syncListens, wireOf } from "./all";

// ---------------------------------------------------------------- nodes

export const HTML_NS = "http://www.w3.org/1999/xhtml";

export const SVG_NS = "http://www.w3.org/2000/svg";

export const SVG_TAGS = new Set(["svg", "path", "circle", "rect", "line", "polyline", "polygon", "ellipse", "g", "defs", "linearGradient", "radialGradient", "stop", "clipPath", "mask", "use", "symbol", "text", "tspan", "pattern", "filter", "foreignObject", "image", "marker", "title", "desc"]);

export function mutated(target: Node_, rec: Partial<MutationRecord_>) {
  // the selection's range moved as the tree changed (selection.ts): a child list's index from its siblings now
  if (liveRanges() && rec.type === "childList") {
    const prev = rec.previousSibling;
    const at = prev ? target._kids.indexOf(prev) + 1 : 0;
    rangesChildList(target, at, rec.addedNodes?.length ?? 0, rec.removedNodes?.[0] ?? null);
  }
  if (observers.size) queueRecord(target, rec);
}

export class Node_ extends EventTarget_ {
  static ELEMENT_NODE = 1;
  static ATTRIBUTE_NODE = 2;
  static TEXT_NODE = 3;
  static COMMENT_NODE = 8;
  static DOCUMENT_NODE = 9;
  static DOCUMENT_FRAGMENT_NODE = 11;
  static DOCUMENT_POSITION_DISCONNECTED = 1;
  static DOCUMENT_POSITION_PRECEDING = 2;
  static DOCUMENT_POSITION_FOLLOWING = 4;
  static DOCUMENT_POSITION_CONTAINS = 8;
  static DOCUMENT_POSITION_CONTAINED_BY = 16;
  readonly ELEMENT_NODE = 1;
  readonly TEXT_NODE = 3;
  readonly DOCUMENT_NODE = 9;
  readonly DOCUMENT_FRAGMENT_NODE = 11;
  nodeType = 1;
  parentNode: Node_ | null = null;
  _kids: Node_[] = [];
  /** the wire id, once on the host */
  __wid: number | undefined;
  /** on the host (created there, and attached if it has a parent there) */
  _wired = false;
  get nodeName(): string {
    return "";
  }
  get ownerDocument(): Document_ | null {
    return doc;
  }
  get childNodes(): Node_[] {
    return nodeList(this._kids);
  }
  get firstChild() {
    return this._kids[0] ?? null;
  }
  get lastChild() {
    return this._kids[this._kids.length - 1] ?? null;
  }
  get parentElement(): Element_ | null {
    const p = this.parentNode;
    return p instanceof Element_ ? p : null;
  }
  get nextSibling(): Node_ | null {
    const k = this.parentNode?._kids;
    return k ? (k[k.indexOf(this) + 1] ?? null) : null;
  }
  get previousSibling(): Node_ | null {
    const k = this.parentNode?._kids;
    return k ? (k[k.indexOf(this) - 1] ?? null) : null;
  }
  get isConnected(): boolean {
    let n: Node_ | null = this;
    while (n.parentNode) n = n.parentNode;
    return n === doc;
  }
  get nodeValue(): string | null {
    return null;
  }
  set nodeValue(_v: string | null) {}
  hasChildNodes() {
    return this._kids.length > 0;
  }
  getRootNode(): Node_ {
    let n: Node_ = this;
    while (n.parentNode) n = n.parentNode;
    return n;
  }
  contains(other: Node_ | null): boolean {
    for (let n = other; n; n = n.parentNode) if (n === this) return true;
    return false;
  }
  compareDocumentPosition(other: Node_): number {
    if (other === this) return 0;
    if (this.getRootNode() !== other.getRootNode()) return 1 | 32 | 2;
    if (this.contains(other)) return 16 | 4;
    if (other.contains(this)) return 8 | 2;
    const a = ancestry(this);
    const b = ancestry(other);
    let i = 0;
    while (a[i] === b[i]) i++;
    const k = a[i - 1]._kids;
    return k.indexOf(a[i]) < k.indexOf(b[i]) ? 4 : 2;
  }
  get textContent(): string | null {
    return this._kids.map((k) => k.textContent ?? "").join("");
  }
  set textContent(v: string | null) {
    for (const k of this._kids.slice()) this.removeChild(k);
    if (v) this.appendChild(doc.createTextNode(String(v)));
  }
  appendChild<T extends Node_>(c: T): T {
    return this.insertBefore(c, null);
  }
  insertBefore<T extends Node_>(c: T, before: Node_ | null): T {
    if (c instanceof DocumentFragment_) {
      for (const k of c._kids.slice()) this.insertBefore(k, before);
      return c;
    }
    if (c.contains(this)) throw new Error("HierarchyRequestError: the new child contains the parent");
    if (c.parentNode) c.parentNode.removeChild(c);
    const i = before ? this._kids.indexOf(before) : -1;
    if (before && i < 0) throw new Error("NotFoundError: the node before which to insert isn't a child of this node");
    if (i < 0) this._kids.push(c);
    else this._kids.splice(i, 0, c);
    c.parentNode = this;
    // on the host: a wired parent takes the child (made there first if it isn't)
    const pw = wireParent(this);
    if (pw !== undefined) {
      materialize(c);
      wire.place(pw, c.__wid!, before?._wired ? before.__wid : nextWired(this, c));
    }
    mutated(this, { type: "childList", addedNodes: [c], previousSibling: c.previousSibling, nextSibling: c.nextSibling });
    if (c.isConnected) connectedTree(c);
    return c;
  }
  removeChild<T extends Node_>(c: T): T {
    const i = this._kids.indexOf(c);
    if (i < 0) throw new Error("NotFoundError: not a child of this node");
    const prev = c.previousSibling;
    const next = c.nextSibling;
    this._kids.splice(i, 1);
    c.parentNode = null;
    if (c._wired && c.__wid !== undefined && c.__wid !== 0) {
      wire.remove(c.__wid);
      unwire(c);
    }
    mutated(this, { type: "childList", removedNodes: [c], previousSibling: prev, nextSibling: next });
    disconnected(c);
    return c;
  }
  replaceChild<T extends Node_>(c: Node_, old: T): T {
    this.insertBefore(c, old);
    return this.removeChild(old);
  }
  remove() {
    this.parentNode?.removeChild(this);
  }
  append(...nodes: (Node_ | string)[]) {
    for (const n of nodes) this.appendChild(typeof n === "string" ? doc.createTextNode(n) : n);
  }
  prepend(...nodes: (Node_ | string)[]) {
    const first = this.firstChild;
    for (const n of nodes) this.insertBefore(typeof n === "string" ? doc.createTextNode(n) : n, first);
  }
  replaceChildren(...nodes: (Node_ | string)[]) {
    for (const k of this._kids.slice()) this.removeChild(k);
    this.append(...nodes);
  }
  cloneNode(deep = false): Node_ {
    return cloneOf(this, deep);
  }
  isSameNode(o: Node_) {
    return o === this;
  }
  isEqualNode(o: Node_) {
    return o === this;
  }
  normalize() {}
}

export function nodeList<T>(a: T[]): T[] {
  const l = a.slice() as any;
  l.item = (i: number) => l[i] ?? null;
  return l;
}

function ancestry(n: Node_): Node_[] {
  const out: Node_[] = [];
  for (let x: Node_ | null = n; x; x = x.parentNode) out.unshift(x);
  return out;
}

/** The wire id a child of `n` goes under on the host (body and the root container are the host's root), or none
 *  when `n` isn't on the host (head, a detached tree). */
function wireParent(n: Node_): number | undefined {
  if (n === body && !htmlDoc) return 0;
  if (n._wired && n.__wid !== undefined) return n.__wid;
  return undefined;
}

/** The wired sibling after `c` (where `c` goes before on the host). */
function nextWired(p: Node_, c: Node_): number | undefined {
  const k = p._kids;
  for (let i = k.indexOf(c) + 1; i < k.length; i++) if (k[i]._wired) return k[i].__wid;
  return undefined;
}

export function unwire(n: Node_) {
  if (n !== rootContainer && n.__wid !== 0) n._wired = false;
  for (const k of n._kids) unwire(k);
}

/** A node made in JS put on the host: created there with its subtree (its listeners asked for). */
export function materialize(n: Node_) {
  if (n._wired) return;
  if (n.__wid === undefined) n.__wid = wire.allocId();
  if (n instanceof Text_) wire.create(n.__wid, "text", {}, n.data);
  else if (n instanceof Element_) {
    const { kind, props } = wireOf(n.localName, { ...n._props, ...n._extra }, n);
    wire.create(n.__wid, kind, svgNs(n, props));
  } else return;
  n._wired = true;
  if (n instanceof Element_) syncListens(n);
  for (const k of n._kids) {
    materialize(k);
    wire.append(n.__wid, k.__wid!);
  }
}

export class Text_ extends Node_ {
  nodeType = 3;
  _d: string;
  constructor(data = "") {
    super();
    this._d = String(data);
  }
  get data() {
    return this._d;
  }
  set data(v: string) {
    this.setData(v ?? "");
  }
  get nodeName() {
    return "#text";
  }
  get nodeValue() {
    return this.data;
  }
  set nodeValue(v: string | null) {
    this.setData(v ?? "");
  }
  get textContent() {
    return this.data;
  }
  set textContent(v: string | null) {
    this.setData(v ?? "");
  }
  get wholeText() {
    return this.data;
  }
  get length() {
    return this.data.length;
  }
  /** The data replaced (DOM's replace data): `count` characters from `at` by `v`, the live ranges moved with it.
   *  A whole replace (nodeValue, data, textContent) is from 0 over all of it. */
  setData(v: string, fromReact = false, at = 0, count = this.data.length) {
    const old = this._d;
    this._d = String(v);
    if (!fromReact && this._wired && this.__wid !== undefined) wire.text(this.__wid, this.data);
    if (liveRanges()) rangesText(this, at, count, this.data.length - old.length + count);
    mutated(this, { type: "characterData", oldValue: old });
  }
  substringData(at: number, count: number) {
    this._at(at);
    return this.data.slice(at, at + count);
  }
  replaceData(at: number, count: number, v: string) {
    this._at(at);
    count = Math.min(count, this.data.length - at);
    this.setData(this.data.slice(0, at) + v + this.data.slice(at + count), false, at, count);
  }
  insertData(at: number, v: string) {
    this.replaceData(at, 0, v);
  }
  deleteData(at: number, count: number) {
    this.replaceData(at, count, "");
  }
  appendData(v: string) {
    this.replaceData(this.data.length, 0, v);
  }
  _at(at: number) {
    if (at < 0 || at > this.data.length) throw new Error(`IndexSizeError: offset ${at} is past the text's length ${this.data.length}`);
  }
  /** The text split at `at`: what's after goes to a new text node after this one (DOM's split a Text node). */
  splitText(at: number): Text_ {
    this._at(at);
    const next = new Text_(this.data.slice(at));
    const p = this.parentNode;
    if (p) p.insertBefore(next, this.nextSibling);
    if (liveRanges()) rangesSplit(this, at, next);
    this.replaceData(at, this.data.length - at, "");
    return next;
  }
}

export class Comment_ extends Node_ {
  nodeType = 8;
  data: string;
  constructor(d = "") {
    super();
    this.data = d;
  }
  get nodeName() {
    return "#comment";
  }
  get textContent() {
    return this.data;
  }
}

export class DocumentFragment_ extends Node_ {
  nodeType = 11;
  get nodeName() {
    return "#document-fragment";
  }
  querySelector(s: string) {
    return query(this, s, true)[0] ?? null;
  }
  querySelectorAll(s: string) {
    return nodeList(query(this, s, false));
  }
}

export class ShadowRoot_ extends DocumentFragment_ {}

// ---------------------------------------------------------------- attributes and props

const BOOL_ATTRS = new Set(["disabled", "hidden", "checked", "readonly", "required", "multiple", "selected", "open", "inert", "novalidate", "formnovalidate", "autofocus", "default", "defer", "async", "loop", "muted", "controls", "playsinline", "reversed", "itemscope", "allowfullscreen", "nomodule"]);

const PROP_ATTR: Record<string, string> = { className: "class", htmlFor: "for", tabIndex: "tabindex", acceptCharset: "accept-charset", httpEquiv: "http-equiv" };

export const ATTR_PROP: Record<string, string> = { class: "className", for: "htmlFor", tabindex: "tabIndex", readonly: "readOnly", maxlength: "maxLength", minlength: "minLength", autocomplete: "autoComplete", contenteditable: "contentEditable", spellcheck: "spellCheck", autofocus: "autoFocus", inputmode: "inputMode", enterkeyhint: "enterKeyHint" };

export const SKIP_PROPS = new Set(["children", "key", "ref", "dangerouslySetInnerHTML", "suppressContentEditableWarning", "suppressHydrationWarning", "defaultValue", "defaultChecked", "value", "checked", "style"]);

/** React's camelCase SVG props whose attribute is kebab-case */
export const kebab = (s: string) => s.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

const SVG_KEEP = new Set(["viewBox", "preserveAspectRatio", "gradientUnits", "gradientTransform", "patternUnits", "clipPathUnits", "markerWidth", "markerHeight", "refX", "refY", "pathLength", "stdDeviation", "baseFrequency", "numOctaves", "textLength", "lengthAdjust", "spreadMethod", "maskUnits"]);

export function attrName(prop: string, svg: boolean): string {
  if (PROP_ATTR[prop]) return PROP_ATTR[prop];
  if (prop.startsWith("data-") || prop.startsWith("aria-")) return prop;
  if (svg) return SVG_KEEP.has(prop) ? prop : prop === "xlinkHref" ? "xlink:href" : kebab(prop);
  return prop.toLowerCase();
}

export function attrValue(name: string, v: unknown): string | null {
  if (v === null || v === undefined || typeof v === "function" || typeof v === "symbol") return null;
  if (typeof v === "boolean") {
    if (name.startsWith("aria-") || name.startsWith("data-") || name === "draggable" || name === "spellcheck" || name === "contenteditable") return String(v);
    return BOOL_ATTRS.has(name) && v ? "" : null;
  }
  return String(v);
}

export function cssText(style: Record<string, unknown>): string {
  return Object.entries(style)
    .filter(([, v]) => v !== null && v !== undefined && v !== "" && typeof v !== "boolean")
    .map(([k, v]) => `${k.startsWith("--") ? k : kebab(k).replace(/^(webkit|moz|ms)-/, "-$1-")}: ${typeof v === "number" && v !== 0 && !UNITLESS.has(k) ? `${v}px` : v};`)
    .join(" ");
}

const UNITLESS = new Set(["opacity", "zIndex", "flex", "flexGrow", "flexShrink", "order", "fontWeight", "lineHeight", "zoom", "gridRow", "gridColumn", "gridArea", "columnCount", "fillOpacity", "strokeOpacity", "strokeWidth", "aspectRatio", "scale", "orphans", "widows", "tabSize"]);

// ---------------------------------------------------------------- the style object

export function cssProp(name: string): string {
  if (name.startsWith("--")) return name;
  return name.replace(/^-(webkit|moz|ms)-/, (_m, p) => `${p}-`).replace(/-([a-z])/g, (_m, c) => c.toUpperCase());
}

export function makeStyle(el: Element_): any {
  const api = {
    setProperty(name: string, value: string | null, _prio?: string) {
      el._setStyle(cssProp(name), value === null || value === "" ? undefined : String(value));
    },
    getPropertyValue(name: string) {
      const v = el._style[cssProp(name)];
      return v === undefined ? "" : styleString(cssProp(name), v);
    },
    removeProperty(name: string) {
      const k = cssProp(name);
      const old = api.getPropertyValue(name);
      el._setStyle(k, undefined);
      return old;
    },
    getPropertyPriority: () => "",
    item: (i: number) => kebab(Object.keys(el._style)[i] ?? ""),
  };
  return new Proxy(api, {
    // `"transform" in el.style`, as libraries feature-test a property (GSAP's CSSPlugin)
    has(t, p) {
      return typeof p === "string" && (p in t || p === "cssText" || p.startsWith("--") || /^[a-z][a-zA-Z]*$/.test(p));
    },
    get(t, p) {
      if (typeof p !== "string") return undefined;
      if (p in t) return (t as any)[p];
      if (p === "cssText") return cssText(el._style);
      if (p === "length") return Object.keys(el._style).length;
      const v = el._style[p];
      return v === undefined ? "" : styleString(p, v);
    },
    set(_t, p, v) {
      if (typeof p !== "string") return true;
      if (p === "cssText") {
        el._replaceStyle(parseCss(String(v)));
        return true;
      }
      el._setStyle(p, v === null || v === "" ? undefined : v);
      return true;
    },
  });
}

export function styleString(k: string, v: unknown): string {
  return typeof v === "number" && v !== 0 && !UNITLESS.has(k) ? `${v}px` : String(v);
}

export function parseCss(s: string): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const d of s.split(";")) {
    const i = d.indexOf(":");
    if (i < 0) continue;
    const k = d.slice(0, i).trim();
    if (k) out[cssProp(k)] = d.slice(i + 1).trim();
  }
  return out;
}
