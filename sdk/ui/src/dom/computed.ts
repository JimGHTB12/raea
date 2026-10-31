// getComputedStyle and DOMRect.
import * as wire from "../wire";
import { Element_, Node_, cssMatrix, cssProp, effStyle, htmlDoc, kebab, styleString } from "./all";

// ---------------------------------------------------------------- getComputedStyle

const INLINE_TAGS = new Set(["span", "a", "b", "i", "em", "strong", "code", "label", "small", "abbr", "kbd", "img", "svg", "text"]);

const IBLOCK_TAGS = new Set(["button", "input", "select", "textarea"]);

type Computed = Record<string, string>;

/** What Chrome would compute for the properties the libraries read (display, position, overflow, visibility,
 *  opacity, transform, animation and transition names, sizes, paddings, borders), from the element's inline style,
 *  its Tailwind classes and its box; the rest as Chrome's initial values. */
/** An element's own pointer-events (its style, else its last pointer-events-* class), undefined when it inherits. */
function ownPointerEvents(el: Element_): string | undefined {
  const v = el._style.pointerEvents as string | undefined;
  if (v && v !== "inherit") return v;
  const cls = typeof el.className === "string" ? el.className.split(/\s+/) : [];
  for (let i = cls.length - 1; i >= 0; i--) if (cls[i] === "pointer-events-none" || cls[i] === "pointer-events-auto") return cls[i].slice(15);
  return undefined;
}

/** pointer-events as Chrome computes it: inherited from the nearest element up that sets it (body's too). */
export function pointerEventsOf(el: Element_): string {
  for (let n: Node_ | null = el; n instanceof Element_; n = n.parentNode) {
    const v = ownPointerEvents(n);
    if (v) return v;
  }
  return "auto";
}

export function computedOf(el: Element_): Computed {
  const s = effStyle(el);
  const cls = el.className.split(/\s+/);
  const has = (c: string) => cls.includes(c);
  const pick = (map: Record<string, string>, fallback: string) => {
    for (let i = cls.length - 1; i >= 0; i--) if (map[cls[i]] !== undefined) return map[cls[i]];
    return fallback;
  };
  const tag = el.localName;
  let display = (s.display as string) ?? pick({ hidden: "none", block: "block", flex: "flex", "inline-flex": "inline-flex", grid: "grid", "inline-grid": "inline-grid", inline: "inline", "inline-block": "inline-block", contents: "contents", table: "table" }, INLINE_TAGS.has(tag) ? "inline" : IBLOCK_TAGS.has(tag) ? "inline-block" : tag === "li" ? "list-item" : "block");
  if (el.hasAttribute("hidden") || tag === "style" || tag === "script" || tag === "head") display = "none";
  const position = (s.position as string) ?? pick({ static: "static", relative: "relative", absolute: "absolute", fixed: "fixed", sticky: "sticky" }, "static");
  const ov = (axis: "x" | "y") =>
    (s[axis === "x" ? "overflowX" : "overflowY"] as string) ??
    (s.overflow as string) ??
    pick(
      { "overflow-auto": "auto", "overflow-scroll": "scroll", "overflow-hidden": "hidden", "overflow-clip": "clip", "overflow-visible": "visible", [`overflow-${axis}-auto`]: "auto", [`overflow-${axis}-scroll`]: "scroll", [`overflow-${axis}-hidden`]: "hidden", [`overflow-${axis}-clip`]: "clip", [`overflow-${axis}-visible`]: "visible" },
      "visible",
    );
  const b = el._box();
  const px = (v: number | undefined) => `${v ?? 0}px`;
  const out: Computed = {
    display,
    position,
    overflowX: ov("x"),
    overflowY: ov("y"),
    visibility: (s.visibility as string) ?? (has("invisible") ? "hidden" : "visible"),
    opacity: s.opacity !== undefined ? String(s.opacity) : "1",
    transform: htmlDoc ? cssMatrix(s.transform, el._box()) : ((s.transform as string) ?? "none"),
    translate: (s.translate as string) ?? "none",
    scale: (s.scale as string) ?? "none",
    rotate: (s.rotate as string) ?? "none",
    transformBox: (s.transformBox as string) ?? "view-box",
    zIndex: s.zIndex !== undefined ? String(s.zIndex) : "auto",
    pointerEvents: pointerEventsOf(el),
    direction: (s.direction as string) ?? "ltr",
    writingMode: "horizontal-tb",
    boxSizing: "border-box",
    animationName: (s.animationName as string) ?? "none",
    animationDuration: "0s",
    animationDelay: "0s",
    animationPlayState: "running",
    transitionProperty: "all",
    transitionDuration: (s.transitionDuration as string) ?? "0s",
    transitionDelay: "0s",
    willChange: (s.willChange as string) ?? "auto",
    contain: "none",
    containerType: "normal",
    backdropFilter: "none",
    filter: (s.filter as string) ?? "none",
    perspective: "none",
    cursor: (s.cursor as string) ?? "auto",
    width: b ? px(b[2]) : "auto",
    height: b ? px(b[3]) : "auto",
    paddingLeft: px(b?.[8]),
    paddingTop: px(b?.[9]),
    paddingRight: px(b?.[10]),
    paddingBottom: px(b?.[11]),
    borderLeftWidth: px(b?.[12]),
    borderTopWidth: px(b?.[13]),
    borderRightWidth: px(b?.[14]),
    borderBottomWidth: px(b?.[15]),
    marginLeft: "0px",
    marginTop: "0px",
    marginRight: "0px",
    marginBottom: "0px",
    top: (s.top as string) !== undefined ? styleString("top", s.top) : "auto",
    left: (s.left as string) !== undefined ? styleString("left", s.left) : "auto",
    right: "auto",
    bottom: "auto",
    fontSize: "16px",
    lineHeight: "normal",
    scrollbarGutter: "auto",
    scrollPaddingTop: "auto",
    scrollBehavior: "auto",
    color: (s.color as string) ?? "rgb(0, 0, 0)",
    backgroundColor: (s.backgroundColor as string) ?? "rgba(0, 0, 0, 0)",
  };
  out.overflow = out.overflowX === out.overflowY ? out.overflowX : `${out.overflowX} ${out.overflowY}`;
  // an HTML page's other declared values, as written (GSAP reads the property it tweens from here)
  if (htmlDoc) for (const k in s) if (!(k in out) || k === "fontSize" || k === "lineHeight" || /^margin/.test(k) || k === "right" || k === "bottom") out[k] = styleString(k, s[k]);
  return out;
}

export function getComputedStyle(el: Element_, _pseudo?: string | null): any {
  if (!(el instanceof Element_)) throw new TypeError("getComputedStyle: not an Element");
  const c = computedOf(el);
  // with the host's cascade on, each value is stylo's for the element as the JS has written it so far; this cascade's
  // answers where the host has none (an unwired element, a call outside a step)
  const host = htmlDoc && el._wired && el.__wid !== undefined ? el.__wid : undefined;
  const ask = (prop: string): string | undefined => {
    if (host === undefined) return undefined;
    const name = prop.startsWith("--") ? prop : kebab(prop);
    try {
      const v = wire.callSync<Record<string, string>>("style.computed", { id: host, names: [name] })?.[name];
      return v ? v : undefined;
    } catch {
      return undefined;
    }
  };
  const get = (name: string) => ask(cssProp(name)) ?? c[cssProp(name)] ?? "";
  return new Proxy(c, {
    get(t, p) {
      if (p === "getPropertyValue") return get;
      if (p === "getPropertyPriority") return () => "";
      if (p === "length") return Object.keys(t).length;
      if (p === "item") return (i: number) => kebab(Object.keys(t)[i] ?? "");
      if (typeof p !== "string") return undefined;
      return ask(p) ?? t[p] ?? "";
    },
  });
}

// ---------------------------------------------------------------- DOMRect

export class DOMRectReadOnly_ {
  constructor(
    public x = 0,
    public y = 0,
    public width = 0,
    public height = 0,
  ) {}
  get top() {
    return Math.min(this.y, this.y + this.height);
  }
  get left() {
    return Math.min(this.x, this.x + this.width);
  }
  get right() {
    return Math.max(this.x, this.x + this.width);
  }
  get bottom() {
    return Math.max(this.y, this.y + this.height);
  }
  toJSON() {
    const { x, y, width, height, top, left, right, bottom } = this;
    return { x, y, width, height, top, left, right, bottom };
  }
  static fromRect(r: any = {}) {
    return new this(r.x, r.y, r.width, r.height);
  }
}

export class DOMRect_ extends DOMRectReadOnly_ {}
