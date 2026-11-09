// An element on the wire, React's host instances in DOM mode, and which events the host sends each element.
import * as wire from "../wire";
import type { Ev } from "../wire";
import { Control_, Element_, FOCUSABLE_TAGS, HTMLOptionElement_, HTMLSelectElement_, HTML_HIDDEN, Node_, S, SKIP_PROPS, SVG_NS, TEXT_INPUTS, Text_, attrName, attrValue, body, cssText, doc, g, hostEvent, html, htmlDoc, htmlValue, installed, isEditable, makeElement, mutated, nativelyFocusable, query, rootContainer, scrollable, tellCapture, typesOn, unwire } from "./all";

// ---------------------------------------------------------------- what an element is on the wire

/** A text field on the host (bevy_text's editor): input of a text type, and textarea. */
export function isField(el: Element_) {
  return (el.localName === "input" && TEXT_INPUTS.has((el.getAttribute("type") ?? "").toLowerCase())) || el.localName === "textarea";
}

/** React's style object as the host takes it (numbers as numbers, strings as given). */
export function wireStyle(s: Record<string, unknown>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const k in s) {
    let v = s[k];
    if (v === undefined || v === null || v === "" || typeof v === "boolean") continue;
    if (htmlDoc) {
      v = htmlValue(k, v);
      if (v === undefined) continue;
    }
    // (a custom property keeps its value as written, as React sets it: `--box: 60px` is a length, `--box: 60` isn't;
    // so does line-height, whose bare number the host takes unitless, a multiple of the font size: "56px" sent as 56
    // drew a 48px line 2688px tall)
    if (typeof v === "string" && !k.startsWith("--") && k !== "lineHeight" && /^-?\d*\.?\d+px$/.test(v.trim())) out[k] = parseFloat(v);
    else out[k] = v;
  }
  return out;
}

/** An element's kind and props as the host takes them: React's props (no handlers, children, key or ref), a text
 *  input and a textarea as the host's text field (`editableText`, its value), a natively focusable element with a
 *  tabIndex (as a browser focuses a button), style as merged with what JS set on it. */
export function wireOf(tag: string, props: Record<string, unknown>, el?: Element_): { kind: string; props: Record<string, unknown> } {
  const out: Record<string, unknown> = {};
  for (const k in props) {
    const v = props[k];
    if (k === "children" || k === "key" || k === "ref" || v === undefined || typeof v === "function" || /^on[A-Z]/.test(k) || k === "dangerouslySetInnerHTML") continue;
    out[k] = typeof v === "bigint" ? Number(v) : v;
  }
  if (el && Object.keys(el._style).length) out.style = wireStyle(el._style);
  else if (out.style && typeof out.style === "object") out.style = wireStyle(out.style as Record<string, unknown>);
  let kind = tag;
  if (htmlDoc) {
    // (`$root`: which it is, for a host that makes them the document's own; raea-page's ops.rs)
    if (tag === "html" || tag === "body") {
      kind = "div";
      out.$root = tag;
    }
    // a page's scripts, styles and the like are kept off the host's layout (a sheet html.ts made for the host stays a style)
    else if (HTML_HIDDEN.has(tag) && out["data-raea-sheet"] === undefined) {
      kind = "div";
      out.style = { display: "none" };
    }
  }
  const type = String(props.type ?? "").toLowerCase();
  if ((tag === "input" && TEXT_INPUTS.has(type)) || tag === "textarea") {
    kind = "editableText";
    if (out.value === undefined && props.defaultValue !== undefined) out.value = String(props.defaultValue);
    if (out.value !== undefined) out.value = String(out.value);
    if (tag === "textarea") out.multiline = true;
    delete out.defaultValue;
  }
  const focusableTag = tag === "button" || tag === "select" || (tag === "a" && props.href !== undefined) || (tag === "input" && type !== "hidden");
  if (out.tabIndex === undefined && focusableTag && !props.disabled) out.tabIndex = 0;
  // an editing host takes the focus and its keys, as Chrome's (editing.ts)
  const ce = props.contentEditable;
  if (out.tabIndex === undefined && (ce === true || ce === "" || ce === "true" || ce === "plaintext-only") && !(el?.parentNode instanceof Element_ && isEditable(el.parentNode))) out.tabIndex = 0;
  if (props.disabled && FOCUSABLE_TAGS.has(tag) && props.tabIndex === undefined) delete out.tabIndex;
  return { kind, props: out };
}

// ---------------------------------------------------------------- React's host instances (react.ts's DOM mode)

/** A React host instance: an element on the host already (React's createInstance sent its create). Returns the
 *  element and what to send for it. */
export function hostElement(id: number, type: string, props: Record<string, unknown>, fiber: unknown, inSvg = false): { el: Element_; kind: string; props: Record<string, unknown>; text?: string } {
  const el = makeElement(type, inSvg ? SVG_NS : undefined);
  el.__wid = id;
  el._wired = true;
  el._fiber = fiber;
  applyProps(el, {}, props, true);
  const w = wireOf(type, props, el);
  if (type === "text" && (typeof props.children === "string" || typeof props.children === "number")) el._inline = String(props.children);
  return { el, kind: w.kind, props: svgNs(el, w.props), text: el._inline };
}

/** An SVG element whose name is also a host kind (text, image, title): its create says so (`$ns`), so the host makes
 *  it in SVG's namespace; outside an <svg> those names are the host's own (systems/page/src/ops.rs in_svg). */
const SVG_AMBIGUOUS = new Set(["text", "image", "title"]);

export function svgNs(el: Element_, props: Record<string, unknown>): Record<string, unknown> {
  return el.namespaceURI === SVG_NS && SVG_AMBIGUOUS.has(el.localName) ? { ...props, $ns: "svg" } : props;
}

export function hostText(id: number, text: string): Text_ {
  const t = new Text_(text);
  t.__wid = id;
  t._wired = true;
  return t;
}

/** React's props onto the element: its attributes (as React DOM sets them), its style (React's keys over what JS set
 *  imperatively, as React DOM diffs style), value and checked, and the events it hears. */
function applyProps(el: Element_, old: Record<string, unknown>, next: Record<string, unknown>, init: boolean) {
  const svg = el.namespaceURI === SVG_NS;
  for (const k in old) {
    if (k in next || SKIP_PROPS.has(k) || /^on[A-Z]/.test(k)) continue;
    const name = attrName(k, svg);
    if (el._attrs.has(name)) {
      const was = el._attrs.get(name)!;
      el._attrs.delete(name);
      mutated(el, { type: "attributes", attributeName: name, oldValue: was });
    }
  }
  for (const k in next) {
    if (SKIP_PROPS.has(k) || /^on[A-Z]/.test(k)) continue;
    if (!init && old[k] === next[k]) continue;
    const name = attrName(k, svg);
    const v = attrValue(name, next[k]);
    const was = el._attrs.get(name) ?? null;
    if (v === null) el._attrs.delete(name);
    else el._attrs.set(name, v);
    if (was !== v && !init) mutated(el, { type: "attributes", attributeName: name, oldValue: was });
  }
  // value and checked: React sets the properties (and the attribute from defaultValue / defaultChecked)
  if (init && next.defaultValue !== undefined) el._attrs.set("value", String(next.defaultValue));
  if (next.value !== undefined && el.localName !== "input" && el.localName !== "textarea" && el.localName !== "select") el._attrs.set("value", String(next.value));
  if (el instanceof Control_) {
    if (next.value !== undefined && next.value !== null && (init || next.value !== old.value)) {
      el._value = String(next.value);
      if (el instanceof HTMLSelectElement_) for (const o of query(el, "option", false) as HTMLOptionElement_[]) o._sel = undefined;
    }
    else if (init && next.defaultValue !== undefined) el._value = String(next.defaultValue);
    if (next.checked !== undefined && (init || next.checked !== old.checked)) el._checked = !!next.checked;
    if (init && next.defaultChecked) el._attrs.set("checked", "");
  }
  // style: what React changed, over the element's current style
  const os = (old.style ?? {}) as Record<string, unknown>;
  const ns = (next.style ?? {}) as Record<string, unknown>;
  if (init || os !== ns) {
    const merged = { ...el._style };
    for (const k in os) if (!(k in ns)) delete merged[k];
    for (const k in ns) if (init || os[k] !== ns[k]) merged[k] = ns[k];
    for (const k in merged) if (merged[k] === undefined || merged[k] === null || merged[k] === "") delete merged[k];
    const before = el._attrs.get("style") ?? null;
    el._style = merged;
    const text = cssText(merged);
    if (text) el._attrs.set("style", text);
    else el._attrs.delete("style");
    if (!init && before !== (el._attrs.get("style") ?? null)) mutated(el, { type: "attributes", attributeName: "style", oldValue: before });
  }
  el._props = next;
}

/** React's update of an element's props: the mirror, and the host's props (each changed one whole, the style as
 *  merged). Returns nothing; it emits the wire ops itself. */
/** React's dangerouslySetInnerHTML: its markup as the element's children when it's new or changed (React DOM sets
 *  `innerHTML`; with no `__html` the children are left to React). */
export function innerHtmlProp(el: Element_, old: Record<string, unknown> | undefined, next: Record<string, unknown>) {
  const html = (p?: Record<string, unknown>) => (p?.dangerouslySetInnerHTML as { __html?: unknown } | undefined)?.__html;
  const now = html(next);
  if (now == null) return old && html(old) != null ? void (el.innerHTML = "") : undefined;
  if (old && html(old) === now) return;
  el.innerHTML = String(now);
}

export function hostUpdate(el: Element_, type: string, old: Record<string, unknown>, next: Record<string, unknown>, fiber: unknown) {
  if (fiber) el._fiber = fiber;
  innerHtmlProp(el, old, next);
  const styleWas = el._style;
  applyProps(el, old, next, false);
  const id = el.__wid!;
  const a = wireOf(type, old);
  const b = wireOf(type, next, el);
  delete a.props.style;
  for (const k in a.props) if (!(k in b.props) && k !== "style") wire.unset(id, k);
  for (const k in b.props) {
    if (k === "style") continue;
    if (!same(a.props[k], b.props[k])) wire.set(id, k, b.props[k]);
  }
  if (!same(wireStyle(styleWas), wireStyle(el._style))) {
    if (Object.keys(el._style).length) wire.set(id, "style", wireStyle(el._style));
    else wire.unset(id, "style");
  }
  const inline = type === "text" && (typeof next.children === "string" || typeof next.children === "number") ? String(next.children) : undefined;
  if (inline !== undefined && inline !== el._inline) {
    const was = el._inline;
    el._inline = inline;
    wire.text(id, inline);
    mutated(el, { type: "characterData", oldValue: was ?? null });
  }
  syncListens(el);
}

export function same(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== "object" || typeof b !== "object" || !a || !b) return false;
  const ka = Object.keys(a);
  if (ka.length !== Object.keys(b).length) return false;
  return ka.every((k) => same((a as any)[k], (b as any)[k]));
}

/** React placed a child (its wire op already sent): the mirror follows. */
export function mirrorPlace(p: Node_, c: Node_, before: Node_ | null) {
  if (c.parentNode) {
    const k = c.parentNode._kids;
    k.splice(k.indexOf(c), 1);
  }
  const i = before ? p._kids.indexOf(before) : -1;
  if (i < 0) p._kids.push(c);
  else p._kids.splice(i, 0, c);
  c.parentNode = p;
  mutated(p, { type: "childList", addedNodes: [c], previousSibling: c.previousSibling, nextSibling: c.nextSibling });
  if (c.isConnected) connectedTree(c);
}

/** React removed a child (its wire op already sent). */
export function mirrorRemove(p: Node_, c: Node_) {
  const i = p._kids.indexOf(c);
  if (i < 0) return;
  const prev = c.previousSibling;
  const next = c.nextSibling;
  p._kids.splice(i, 1);
  c.parentNode = null;
  unwire(c);
  mutated(p, { type: "childList", removedNodes: [c], previousSibling: prev, nextSibling: next });
  disconnected(c);
}

export function mirrorText(t: Text_, s: string) {
  t.setData(s, true);
}

export function mirrorClear(p: Node_) {
  for (const c of p._kids.slice()) mirrorRemove(p, c);
}

export function connectedTree(_n: Node_) {}

export function disconnected(n: Node_) {
  if (S.active && n.contains(S.active)) {
    // the focused element left the document: the focus goes to body, with no events (as Chrome)
    S.active = null;
  }
  if (S.pointerCapture && n.contains(S.pointerCapture)) {
    S.pointerCapture = null;
    tellCapture(null);
  }
}

// ---------------------------------------------------------------- which events the host sends each element

const WIRE_FOR: Record<string, Ev[]> = {};

for (const t of ["pointerdown", "mousedown", "click", "dblclick", "auxclick", "pointerup", "mouseup", "touchstart", "touchend"]) WIRE_FOR[t] = ["pointerdown", "pointerup"];

// the host sends a DOM root's moves on hover and while a button is held (to what captured the press, else to what's
// under the pointer)
for (const t of ["pointermove", "mousemove", "touchmove", "dragstart", "drag"]) WIRE_FOR[t] = ["pointermove"];

for (const t of ["pointerenter", "mouseenter", "pointerover", "mouseover"]) WIRE_FOR[t] = ["pointerenter"];

for (const t of ["pointerleave", "mouseleave", "pointerout", "mouseout"]) WIRE_FOR[t] = ["pointerleave"];

for (const t of ["keydown", "keyup", "keypress"]) WIRE_FOR[t] = ["keydown", "keyup"];

for (const t of ["focus", "blur", "focusin", "focusout"]) WIRE_FOR[t] = ["focus", "blur"];

for (const t of ["input", "beforeinput", "change"]) WIRE_FOR[t] = ["change", "blur"];

WIRE_FOR.contextmenu = ["contextmenu"];

WIRE_FOR.wheel = ["wheel"];

WIRE_FOR.scroll = ["scroll"];

WIRE_FOR.scrollend = ["scroll"];

/** React's handler props: each one's native event (React DOM listens for these at its root container), and those
 *  React runs on the element itself (events that don't bubble). */
const REACT: Record<string, string> = {
  Click: "click", DoubleClick: "dblclick", ContextMenu: "contextmenu", AuxClick: "auxclick",
  PointerDown: "pointerdown", PointerUp: "pointerup", PointerMove: "pointermove", PointerCancel: "pointercancel", PointerOver: "pointerover", PointerOut: "pointerout",
  GotPointerCapture: "gotpointercapture", LostPointerCapture: "lostpointercapture",
  MouseDown: "mousedown", MouseUp: "mouseup", MouseMove: "mousemove", MouseOver: "mouseover", MouseOut: "mouseout",
  KeyDown: "keydown", KeyUp: "keyup", KeyPress: "keypress",
  Focus: "focusin", Blur: "focusout", Input: "input", Change: "change", BeforeInput: "beforeinput", Submit: "submit", Reset: "reset", Invalid: "invalid",
  Wheel: "wheel", Select: "select", Copy: "copy", Cut: "cut", Paste: "paste",
  CompositionStart: "compositionstart", CompositionUpdate: "compositionupdate", CompositionEnd: "compositionend",
  TouchStart: "touchstart", TouchMove: "touchmove", TouchEnd: "touchend", TouchCancel: "touchcancel",
  DragStart: "dragstart", Drag: "drag", DragEnd: "dragend", DragEnter: "dragenter", DragLeave: "dragleave", DragOver: "dragover", Drop: "drop",
  AnimationStart: "animationstart", AnimationEnd: "animationend", AnimationIteration: "animationiteration", TransitionEnd: "transitionend", TransitionStart: "transitionstart",
  Scroll: "scroll", ScrollEnd: "scrollend", PointerEnter: "pointerenter", PointerLeave: "pointerleave", MouseEnter: "mouseenter", MouseLeave: "mouseleave", Load: "load", Error: "error", Toggle: "toggle",
  LoadedMetadata: "loadedmetadata", LoadedData: "loadeddata", Seeking: "seeking", Seeked: "seeked",
  // a video's or audio's playing (dom/media.ts, the host's player)
  Play: "play", Pause: "pause", Playing: "playing", TimeUpdate: "timeupdate", Ended: "ended", CanPlay: "canplay", CanPlayThrough: "canplaythrough",
  Waiting: "waiting", VolumeChange: "volumechange", Emptied: "emptied", DurationChange: "durationchange", RateChange: "ratechange",
};

const MEDIA_EVENTS = ["play", "pause", "playing", "timeupdate", "ended", "canplay", "canplaythrough", "waiting", "volumechange", "emptied", "durationchange", "ratechange"];
export const DIRECT = new Set(["scroll", "scrollend", "pointerenter", "pointerleave", "mouseenter", "mouseleave", "load", "error", "toggle", "invalid", "loadedmetadata", "loadeddata", "seeking", "seeked", ...MEDIA_EVENTS]);

/** native type -> the React prop names it runs (onChange is special: text fields hear it from input) */
export const REACT_BY_NATIVE = new Map<string, string[]>();

for (const [n, t] of Object.entries(REACT)) {
  const l = REACT_BY_NATIVE.get(t) ?? [];
  l.push(`on${n}`);
  REACT_BY_NATIVE.set(t, l);
}

/** void's own events: heard straight from the host, as before (a menu item's select, a menu's close) */
const VOID_DIRECT: Record<string, Ev> = { onSelect: "select", onClose: "close" };

function reactNative(prop: string): string | undefined {
  if (!/^on[A-Z]/.test(prop)) return undefined;
  const base = prop.slice(2).replace(/Capture$/, "");
  if (base === "Change") return "input";
  return REACT[base];
}

export const heard = new WeakMap<Node_, Set<Ev>>();

function hearer(el: Node_, ev: Ev) {
  return (payload?: unknown) => hostEvent(el, ev, payload);
}

/** The events the host sends this element now: what its React props and listeners need, what every focusable
 *  element hears (focus, blur, keys), a field's text, a scroller's scroll. */
function wanted(el: Element_): Set<Ev> {
  const out = new Set<Ev>();
  const add = (t: string) => WIRE_FOR[t]?.forEach((e) => out.add(e));
  for (const k in el._props) {
    if (typeof el._props[k] !== "function") continue;
    if (k in VOID_DIRECT && (el._props.menuItem !== undefined || el._props.menu !== undefined)) continue;
    const t = reactNative(k);
    if (t) {
      add(t);
      if (k.startsWith("onChange")) add("change");
    }
  }
  for (const t of typesOn(el)) add(t);
  // hover on every element: Chrome sends pointerenter and leave to each element the pointer enters, and :hover holds
  // for the whole chain under it (the host keeps Hovered only on elements that hear these)
  out.add("pointerenter");
  out.add("pointerleave");
  if (nativelyFocusable(el) || isField(el) || el.hasAttribute("tabindex")) ["focus", "blur", "keydown", "keyup"].forEach((e) => out.add(e as Ev));
  if (isField(el)) out.add("change");
  if (scrollable(el)) out.add("scroll");
  return out;
}

/** A React instance's events asked of the host (react.ts, once its create is sent). */
export const listenFor = (el: Element_) => syncListens(el);

export function syncListens(el: Element_) {
  if (!el._wired || el.__wid === undefined || !installed) return;
  if (el.__wid === 0 || el === rootContainer) return syncRoot();
  const want = wanted(el);
  // void's own direct events keep React's function as the host's listener
  for (const k in VOID_DIRECT) {
    const fn = el._props[k];
    if (typeof fn === "function" && (el._props.menuItem !== undefined || el._props.menu !== undefined)) wire.listen(el.__wid, VOID_DIRECT[k], fn as (p?: unknown) => void);
  }
  let had = heard.get(el);
  if (!had) heard.set(el, (had = new Set()));
  for (const e of had) if (!want.has(e)) wire.unlisten(el.__wid, e);
  for (const e of want) if (!had.has(e)) wire.listen(el.__wid, e, hearer(el, e));
  heard.set(el, want);
}

/** The host's root hears what document, window, html, body and the root container listen for (an input that no
 *  element under the pointer hears comes to the root, and is dispatched from the element under it). */
export function syncRoot() {
  if (!installed) return;
  const want = new Set<Ev>(["pointerdown", "pointerup"]);
  for (const t of [g, doc, html, body, rootContainer]) for (const ty of typesOn(t)) WIRE_FOR[ty]?.forEach((e) => want.add(e));
  const had = heard.get(rootContainer) ?? new Set();
  for (const e of had) if (!want.has(e)) wire.unlisten(0, e);
  for (const e of want) if (!had.has(e)) wire.listen(0, e, hearer(rootContainer, e));
  heard.set(rootContainer, want);
}

export function listensChanged(t: object) {
  if (!installed) return;
  if (t === g || t === doc || t === html || t === body || t === rootContainer) syncRoot();
  else if (t instanceof Element_) syncListens(t);
}
