// Images, media, style, iframe, canvas and SVG elements, and the tag-to-class table that makes elements.
import * as wire from "../wire";
import { canvasClass } from "../canvas";
import { DocumentFragment_, Element_, Event_, HTMLAnchorElement_, HTMLButtonElement_, HTMLDivElement_, HTMLElement_, HTMLFormElement_, HTMLInputElement_, HTMLLIElement_, HTMLLabelElement_, HTMLOptionElement_, HTMLSelectElement_, HTMLSpanElement_, HTMLTextAreaElement_, HTMLUListElement_, MediaNews, Node_, S, SVG_NS, SVG_TAGS, Text_, byWid, dispatchDirect, focusTo, g, pageUrl, transition } from "./all";

export class HTMLImageElement_ extends HTMLElement_ {
  _media?: MediaNews;
  get src() {
    return this.getAttribute("src") ?? "";
  }
  set src(v: string) {
    this.setAttribute("src", pageUrl(String(v)));
  }
  get complete() {
    return !this.getAttribute("src") || this._media?.type === "load" || this._media?.type === "error";
  }
  get naturalWidth() {
    return this._media?.w ?? 0;
  }
  get naturalHeight() {
    return this._media?.h ?? 0;
  }
  get currentSrc() {
    return this.complete ? (this.getAttribute("src") ?? "") : "";
  }
  decode() {
    return Promise.resolve();
  }
}

/** video and audio: the source's size and duration as the host decoded them; paused (the host plays nothing yet),
 *  muted as set, and currentTime, whose set seeks the host's decoder (`__seek`: the time and a count). */
export class HTMLMediaElement_ extends HTMLElement_ {
  _media?: MediaNews;
  get src() {
    return this.getAttribute("src") ?? "";
  }
  set src(v: string) {
    this.setAttribute("src", pageUrl(String(v)));
  }
  _time = 0;
  _seeks = 0;
  _muted?: boolean;
  get paused() {
    return true;
  }
  get muted() {
    return this._muted ?? (this._props.muted === true || this.hasAttribute("muted"));
  }
  set muted(v: boolean) {
    this._muted = !!v;
  }
  get duration() {
    return this._media?.duration ?? NaN;
  }
  get readyState() {
    const t = this._media?.type;
    return t === "loadeddata" || t === "seeked" ? 2 : t === "loadedmetadata" ? 1 : 0;
  }
  get currentTime() {
    return this._time;
  }
  set currentTime(t: number) {
    this._time = Number(t) || 0;
    this._extra.__seek = [this._time, ++this._seeks];
    if (this._wired && this.__wid !== undefined) wire.set(this.__wid, "__seek", this._extra.__seek);
    dispatchDirect(this, new Event_("seeking", { bubbles: false }));
  }
  get videoWidth() {
    return this._media?.w ?? 0;
  }
  get videoHeight() {
    return this._media?.h ?? 0;
  }
  get currentSrc() {
    return this.getAttribute("src") ?? "";
  }
}

/** A media element's news from the host: its state, then the event at it. */
export function mediaNews(v: MediaNews & { id: number }) {
  const el = byWid(v.id) as (HTMLImageElement_ | HTMLMediaElement_) | null;
  if (!el || !v.type) return;
  el._media = { ...el._media, ...v, w: v.w ?? el._media?.w, h: v.h ?? el._media?.h, duration: v.duration ?? el._media?.duration };
  if (el instanceof HTMLMediaElement_ && typeof v.time === "number") el._time = v.time;
  dispatchDirect(el, new Event_(v.type, { bubbles: false }));
}

export class HTMLStyleElement_ extends HTMLElement_ {
  get sheet() {
    return { cssRules: [], insertRule: () => 0, deleteRule: () => {} };
  }
}

export class HTMLBodyElement_ extends HTMLElement_ {}

export class HTMLHtmlElement_ extends HTMLElement_ {}

export class HTMLHeadElement_ extends HTMLElement_ {}

export class HTMLIFrameElement_ extends HTMLElement_ {
  /** the frame's window as its parent sees it, raea-page's proxy (RAEA_PAGE_IFRAMES, docs/blitz-bevy.feature B5c);
   *  undefined without it, as before */
  get contentWindow(): unknown {
    return g.__raea_frames?.window(this);
  }
}

export class HTMLDialogElement_ extends HTMLElement_ {}

/** canvas: getContext("2d") draws on the host (src/canvas.ts) */
export const HTMLCanvasElement_ = canvasClass(HTMLElement_);

export class SVGElement_ extends Element_ {
  get ownerSVGElement() {
    return this.closest("svg");
  }
  focus() {
    focusTo(this, true);
  }
  blur() {
    if (S.active === this) transition(null, true);
  }
  get tabIndex() {
    return Number(this.getAttribute("tabindex") ?? -1);
  }
  get dataset() {
    return super.dataset;
  }
  /** the transform attribute's functions (SVGAnimatedTransformList: the CSS `transform` property is not in it) */
  get transform() {
    const items = (this.getAttribute("transform") ?? "").match(/[a-zA-Z]+\([^)]*\)/g) ?? [];
    const list = { length: items.length, numberOfItems: items.length, getItem: (i: number) => items[i] };
    return { baseVal: list, animVal: list };
  }
}

export class SVGSVGElement_ extends SVGElement_ {}

const TAG_CLASS: Record<string, typeof HTMLElement_> = {
  input: HTMLInputElement_,
  textarea: HTMLTextAreaElement_,
  button: HTMLButtonElement_,
  select: HTMLSelectElement_,
  option: HTMLOptionElement_,
  form: HTMLFormElement_,
  a: HTMLAnchorElement_,
  label: HTMLLabelElement_,
  div: HTMLDivElement_,
  span: HTMLSpanElement_,
  ul: HTMLUListElement_,
  ol: HTMLUListElement_,
  li: HTMLLIElement_,
  img: HTMLImageElement_,
  video: HTMLMediaElement_,
  audio: HTMLMediaElement_,
  style: HTMLStyleElement_,
  body: HTMLBodyElement_,
  html: HTMLHtmlElement_,
  head: HTMLHeadElement_,
  iframe: HTMLIFrameElement_,
  canvas: HTMLCanvasElement_ as unknown as typeof HTMLElement_,
  dialog: HTMLDialogElement_,
};

export function makeElement(tag: string, ns?: string): Element_ {
  const svg = ns === SVG_NS || (ns === undefined && SVG_TAGS.has(tag) && tag !== "text" && tag !== "image" && tag !== "title");
  if (svg) return tag === "svg" ? new SVGSVGElement_(tag, SVG_NS) : new SVGElement_(tag, SVG_NS);
  const C = TAG_CLASS[tag.toLowerCase()] ?? HTMLElement_;
  return new C(tag);
}

export function cloneOf(n: Node_, deep: boolean): Node_ {
  let c: Node_;
  if (n instanceof Text_) c = new Text_(n.data);
  else if (n instanceof Element_) {
    const e = makeElement(n.localName, n.namespaceURI);
    for (const [k, v] of n._attrs) e._attrs.set(k, v);
    e._extra = { ...n._props, ...n._extra };
    e._style = { ...n._style };
    c = e;
  } else c = new DocumentFragment_();
  if (deep) for (const k of n._kids) c.appendChild(cloneOf(k, true));
  return c;
}
