// Events: Event and its kinds, EventTarget with capture and bubble paths (dom.ts has the overview).
import { doc, g, listensChanged, report } from "./all";

// ---------------------------------------------------------------- events

type L = { fn: any; capture: boolean; once: boolean; passive: boolean; internal: boolean; removed: boolean; signal?: any };

const listeners = new WeakMap<object, Map<string, L[]>>();

export function addListener(t: object, type: string, fn: any, opts?: any, internal = false) {
  if (!fn) return;
  const capture = typeof opts === "boolean" ? opts : !!opts?.capture;
  let m = listeners.get(t);
  if (!m) listeners.set(t, (m = new Map()));
  let ls = m.get(type);
  if (!ls) m.set(type, (ls = []));
  if (ls.some((l) => l.fn === fn && l.capture === capture)) return;
  const signal = typeof opts === "object" ? opts?.signal : undefined;
  if (signal?.aborted) return;
  const l: L = { fn, capture, once: typeof opts === "object" && !!opts?.once, passive: typeof opts === "object" && !!opts?.passive, internal, removed: false, signal };
  ls.push(l);
  if (signal) signal.addEventListener?.("abort", () => removeListener(t, type, fn, capture));
  if (!internal) listensChanged(t);
}

export function removeListener(t: object, type: string, fn: any, opts?: any) {
  const capture = typeof opts === "boolean" ? opts : !!opts?.capture;
  const ls = listeners.get(t)?.get(type);
  if (!ls) return;
  const i = ls.findIndex((l) => l.fn === fn && l.capture === capture);
  if (i < 0) return;
  ls[i].removed = true;
  ls.splice(i, 1);
  listensChanged(t);
}

export function typesOn(t: object): string[] {
  const m = listeners.get(t);
  if (!m) return [];
  return [...m].filter(([, ls]) => ls.some((l) => !l.internal)).map(([k]) => k);
}

export class EventTarget_ {
  addEventListener(type: string, fn: any, opts?: any) {
    addListener(this, String(type), fn, opts);
  }
  removeEventListener(type: string, fn: any, opts?: any) {
    removeListener(this, String(type), fn, opts);
  }
  dispatchEvent(ev: Event_): boolean {
    if (!(ev instanceof Event_)) throw new TypeError("dispatchEvent: not an Event");
    return dispatch(this, ev, false);
  }
}

export class Event_ {
  static NONE = 0;
  static CAPTURING_PHASE = 1;
  static AT_TARGET = 2;
  static BUBBLING_PHASE = 3;
  readonly NONE = 0;
  readonly CAPTURING_PHASE = 1;
  readonly AT_TARGET = 2;
  readonly BUBBLING_PHASE = 3;
  type: string;
  bubbles: boolean;
  cancelable: boolean;
  composed: boolean;
  timeStamp: number;
  isTrusted = false;
  target: any = null;
  currentTarget: any = null;
  eventPhase = 0;
  defaultPrevented = false;
  _stop = false;
  _stopNow = false;
  _passive = false;
  _path: any[] = [];
  constructor(type: string, init: any = {}) {
    if (arguments.length === 0) throw new TypeError("Event: 1 argument required");
    this.type = String(type);
    this.bubbles = !!init.bubbles;
    this.cancelable = !!init.cancelable;
    this.composed = !!init.composed;
    this.timeStamp = g.performance?.now?.() ?? 0;
  }
  get srcElement() {
    return this.target;
  }
  get returnValue() {
    return !this.defaultPrevented;
  }
  set returnValue(v: boolean) {
    if (!v) this.preventDefault();
  }
  get cancelBubble() {
    return this._stop;
  }
  set cancelBubble(v: boolean) {
    if (v) this._stop = true;
  }
  preventDefault() {
    if (this.cancelable && !this._passive) this.defaultPrevented = true;
  }
  stopPropagation() {
    this._stop = true;
  }
  stopImmediatePropagation() {
    this._stop = true;
    this._stopNow = true;
  }
  composedPath(): any[] {
    return this._path.slice();
  }
  initEvent(type: string, bubbles = false, cancelable = false) {
    this.type = type;
    this.bubbles = bubbles;
    this.cancelable = cancelable;
  }
}

const assign = (e: any, init: any, defaults: Record<string, unknown>) => {
  for (const k in defaults) e[k] = init?.[k] ?? defaults[k];
};

export class CustomEvent_ extends Event_ {
  detail: unknown;
  constructor(type: string, init: any = {}) {
    super(type, init);
    this.detail = init.detail ?? null;
  }
}

export class UIEvent_ extends Event_ {
  detail!: number;
  view: any;
  which!: number;
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { detail: 0, view: null, which: 0 });
  }
}

export class FocusEvent_ extends UIEvent_ {
  relatedTarget: any;
  constructor(type: string, init: any = {}) {
    super(type, init);
    this.relatedTarget = init.relatedTarget ?? null;
  }
}

export const MODS = { ctrlKey: false, shiftKey: false, altKey: false, metaKey: false };

function modState(this: any, k: string) {
  return k === "Control" ? this.ctrlKey : k === "Shift" ? this.shiftKey : k === "Alt" ? this.altKey : k === "Meta" ? this.metaKey : false;
}

export class MouseEvent_ extends UIEvent_ {
  clientX!: number;
  clientY!: number;
  screenX!: number;
  screenY!: number;
  pageX!: number;
  pageY!: number;
  offsetX!: number;
  offsetY!: number;
  x!: number;
  y!: number;
  movementX!: number;
  movementY!: number;
  button!: number;
  buttons!: number;
  relatedTarget: any;
  ctrlKey!: boolean;
  shiftKey!: boolean;
  altKey!: boolean;
  metaKey!: boolean;
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { clientX: 0, clientY: 0, screenX: 0, screenY: 0, movementX: 0, movementY: 0, button: 0, buttons: 0, relatedTarget: null, ...MODS });
    this.pageX = init.pageX ?? this.clientX;
    this.pageY = init.pageY ?? this.clientY;
    this.x = this.clientX;
    this.y = this.clientY;
    this.offsetX = init.offsetX ?? 0;
    this.offsetY = init.offsetY ?? 0;
  }
  getModifierState(k: string) {
    return modState.call(this, k);
  }
}

export class PointerEvent_ extends MouseEvent_ {
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { pointerId: 0, width: 1, height: 1, pressure: 0, tangentialPressure: 0, tiltX: 0, tiltY: 0, twist: 0, altitudeAngle: Math.PI / 2, azimuthAngle: 0, pointerType: "", isPrimary: false });
  }
  getCoalescedEvents() {
    return [];
  }
  getPredictedEvents() {
    return [];
  }
}

export class WheelEvent_ extends MouseEvent_ {
  static DOM_DELTA_PIXEL = 0;
  static DOM_DELTA_LINE = 1;
  static DOM_DELTA_PAGE = 2;
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { deltaX: 0, deltaY: 0, deltaZ: 0, deltaMode: 0 });
  }
}

export class KeyboardEvent_ extends UIEvent_ {
  static DOM_KEY_LOCATION_STANDARD = 0;
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { key: "", code: "", location: 0, repeat: false, isComposing: false, charCode: 0, keyCode: 0, ...MODS });
    (this as any).which = (this as any).keyCode;
  }
  getModifierState(k: string) {
    return modState.call(this, k);
  }
}

export class InputEvent_ extends UIEvent_ {
  constructor(type: string, init: any = {}) {
    super(type, init);
    assign(this, init, { data: null, inputType: "", isComposing: false, dataTransfer: null });
  }
  getTargetRanges() {
    return [];
  }
}

export class SubmitEvent_ extends Event_ {
  submitter: any;
  constructor(type: string, init: any = {}) {
    super(type, init);
    this.submitter = init.submitter ?? null;
  }
}

export class AnimationEvent_ extends Event_ {}

export class TransitionEvent_ extends Event_ {}

export class TouchEvent_ extends UIEvent_ {}

export class CompositionEvent_ extends UIEvent_ {}

export class ClipboardEvent_ extends Event_ {}

export class DragEvent_ extends MouseEvent_ {}

/** The event's path: the target, its ancestors, then document and window (a connected node's); as far up as it
 *  goes for one that isn't. */
function pathOf(t: any): any[] {
  const out: any[] = [];
  if (t === g) return [g];
  for (let n = t; n; n = n.parentNode) out.push(n);
  if (out[out.length - 1] === doc) out.push(g);
  return out;
}

function invoke(node: any, ev: Event_, capture: boolean) {
  if (ev._stop) return;
  const ls = listeners.get(node)?.get(ev.type);
  // an event handler property (`window.onload = ...`), called after the listeners, outside the capture phase
  const own = capture ? undefined : (node as any)?.[`on${ev.type}`];
  if (!ls?.length && typeof own !== "function") return;
  ev.currentTarget = node;
  for (const l of ls?.slice() ?? []) {
    if (l.removed || l.capture !== capture) continue;
    if (l.once) removeListener(node, ev.type, l.fn, l.capture);
    ev._passive = l.passive;
    try {
      if (typeof l.fn === "function") l.fn.call(node, ev);
      else l.fn.handleEvent(ev);
    } catch (e) {
      report(e);
    }
    ev._passive = false;
    if (ev._stopNow) return;
  }
  if (typeof own === "function")
    try {
      if (own.call(node, ev) === false) ev.preventDefault();
    } catch (e) {
      report(e);
    }
}

/** The DOM's dispatch: capture from window down, the target (its capture listeners, then the rest), then bubbling
 *  back up if the event bubbles. Returns false when a listener prevented its default. */
export function dispatch(target: any, ev: Event_, trusted: boolean): boolean {
  ev.isTrusted = trusted;
  ev.target = target;
  ev._stop = ev._stopNow = false;
  const path = pathOf(target);
  ev._path = path;
  for (let i = path.length - 1; i > 0; i--) {
    ev.eventPhase = 1;
    invoke(path[i], ev, true);
  }
  ev.eventPhase = 2;
  invoke(target, ev, true);
  invoke(target, ev, false);
  if (ev.bubbles)
    for (let i = 1; i < path.length; i++) {
      ev.eventPhase = 3;
      invoke(path[i], ev, false);
    }
  ev.eventPhase = 0;
  ev.currentTarget = null;
  ev._path = [];
  return !ev.defaultPrevented;
}
