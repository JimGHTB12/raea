// HTML drag and drop from the mouse, as Chrome runs it (docs/ts-native.feature P17), over the DOM layer's pointer
// events (dom.ts): a press on a draggable element (draggable="true", or a link or image) that moves 4 points starts a
// drag. Then dragstart on the source (prevented: no drag), and while it lasts no mouse or pointer events reach the
// page. Each move sends drag to the source, dragenter/dragleave as the element under the pointer changes, and dragover
// to it. Its handler must prevent dragover to accept a drop. The release sends drop to that element when it accepted,
// then dragend to the source, with the dropEffect agreed. One DataTransfer carries the data, writable only in
// dragstart and readable only in drop (the protected mode in between).
import * as wire from "./wire";

const g = globalThis as any;

type Mode = "rw" | "ro" | "protected";
/** DataTransfer: data by type, effectAllowed and dropEffect, items and files (none from a mouse drag here). */
export class DataTransfer_ {
  _data = new Map<string, string>();
  _mode: Mode = "rw";
  _effectAllowed = "uninitialized";
  _dropEffect = "none";
  readonly files: any = Object.assign([], { item: () => null });
  get effectAllowed() {
    return this._effectAllowed;
  }
  set effectAllowed(v: string) {
    if (this._mode === "rw" && ["none", "copy", "copyLink", "copyMove", "link", "linkMove", "move", "all", "uninitialized"].includes(v)) this._effectAllowed = v;
  }
  get dropEffect() {
    return this._dropEffect;
  }
  set dropEffect(v: string) {
    if (["none", "copy", "link", "move"].includes(v)) this._dropEffect = v;
  }
  get types() {
    return Object.freeze([...this._data.keys()]);
  }
  setData(type: string, v: string) {
    if (this._mode !== "rw") return;
    let t = String(type).toLowerCase();
    if (t === "text") t = "text/plain";
    if (t === "url") t = "text/uri-list";
    this._data.set(t, String(v));
  }
  getData(type: string) {
    if (this._mode === "protected") return "";
    let t = String(type).toLowerCase();
    if (t === "text") t = "text/plain";
    if (t === "url") return (this._data.get("text/uri-list") ?? "").split(/\r?\n/).find((l) => l && !l.startsWith("#")) ?? "";
    return this._data.get(t) ?? "";
  }
  clearData(type?: string) {
    if (this._mode !== "rw") return;
    if (type === undefined) this._data.clear();
    else this._data.delete(String(type).toLowerCase());
  }
  setDragImage(_el: unknown, _x: number, _y: number) {}
  get items() {
    const dt = this;
    const list = [...this._data.entries()].map(([type, v]) => ({ kind: "string", type, getAsString: (cb: (s: string) => void) => dt._mode !== "protected" && setTimeout(() => cb(v), 0), getAsFile: () => null }));
    return Object.assign(list, {
      add: (data: string, type: string) => dt.setData(type, data),
      remove: (i: number) => dt.clearData(list[i]?.type),
      clear: () => dt.clearData(),
    });
  }
}

let installed = false;
const THRESHOLD = 4;

function draggableOf(t: any): any {
  for (let n = t; n && n.nodeType === 1; n = n.parentNode) {
    const d = n.getAttribute?.("draggable");
    if (d === "true") return n;
    if (d === "false") return null;
    if ((n.localName === "a" && n.hasAttribute("href")) || n.localName === "img") return n;
  }
  return null;
}

/** Drag and drop on window's capture phase, once the DOM layer's classes are installed. */
export function installDnd() {
  if (installed || typeof g.addEventListener !== "function") return;
  installed = true;
  if (g.DataTransfer === undefined || !g.DataTransfer.prototype?.setData) g.DataTransfer = DataTransfer_;
  let press: { src: any; x: number; y: number } | null = null;
  let drag: { src: any; dt: DataTransfer_; over: any; accepted: boolean } | null = null;
  // a drag just ended: the release's mouseup and click don't reach the page
  let ended = false;
  const fire = (target: any, type: string, p: any, dt: DataTransfer_, cancelable: boolean) => {
    const init = { bubbles: true, cancelable, composed: true, view: g, clientX: p.clientX, clientY: p.clientY, screenX: p.screenX, screenY: p.screenY, pageX: p.clientX, pageY: p.clientY, button: 0, buttons: type === "drop" || type === "dragend" ? 0 : 1, shiftKey: p.shiftKey, ctrlKey: p.ctrlKey, altKey: p.altKey, metaKey: p.metaKey };
    const ev = new (g.DragEvent ?? g.MouseEvent)(type, init);
    try {
      Object.defineProperty(ev, "dataTransfer", { value: dt, enumerable: true, configurable: true });
    } catch {}
    const ok = target.dispatchEvent(ev);
    // to the host, which triggers it on the elements' twins as Bevy's dom::DragStart ... (docs/sdk.feature S4a)
    const path: number[] = [];
    for (let n = target; n; n = n.parentNode) if (typeof n.__wid === "number" && n.__wid > 0) path.push(n.__wid);
    if (path.length) wire.emit("drag", { type, path, prevented: !ok, stopped: !!ev._stop, x: p.clientX, y: p.clientY, dropEffect: dt._dropEffect, effectAllowed: dt._effectAllowed, types: [...dt._data.keys()] });
    return ok;
  };
  const swallow = (e: any) => {
    e.stopImmediatePropagation?.();
    e.stopPropagation?.();
  };
  g.addEventListener(
    "pointerdown",
    (e: any) => {
      press = null;
      ended = false;
      if (e.button !== 0) return;
      const src = draggableOf(e.target);
      if (src) press = { src, x: e.clientX, y: e.clientY };
    },
    true,
  );
  g.addEventListener(
    "pointermove",
    (e: any) => {
      if (drag) {
        swallow(e);
        const dt = drag.dt;
        dt._mode = "protected";
        fire(drag.src, "drag", e, dt, true);
        const t = e.target;
        if (t !== drag.over) {
          dt._dropEffect = "none";
          fire(t, "dragenter", e, dt, true);
          if (drag.over) fire(drag.over, "dragleave", e, dt, false);
          drag.over = t;
        }
        // the dropEffect a dragover starts with: what effectAllowed allows, the handler may change it
        const allowed = drag.dt._effectAllowed;
        dt._dropEffect = allowed === "uninitialized" || allowed === "all" || allowed.startsWith("copy") ? "copy" : allowed.startsWith("link") ? "link" : allowed === "move" ? "move" : "none";
        drag.accepted = !fire(t, "dragover", e, dt, true);
        if (!drag.accepted) dt._dropEffect = "none";
        return;
      }
      if (!press || !(e.buttons & 1)) return;
      if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < THRESHOLD) return;
      const dt = new DataTransfer_();
      const src = press.src;
      press = null;
      dt._mode = "rw";
      if (src.localName === "a" && src.hasAttribute("href")) {
        dt.setData("text/uri-list", src.href ?? src.getAttribute("href"));
        dt.setData("text/plain", src.href ?? src.getAttribute("href"));
      }
      if (!fire(src, "dragstart", e, dt, true)) return;
      swallow(e);
      drag = { src, dt, over: null, accepted: false };
      wire.emit("hoverFreeze", true);
    },
    true,
  );
  g.addEventListener(
    "pointerup",
    (e: any) => {
      press = null;
      if (!drag) return;
      swallow(e);
      const d = drag;
      drag = null;
      wire.emit("hoverFreeze", false);
      ended = true;
      let effect = "none";
      if (d.over && d.accepted && d.dt._dropEffect !== "none") {
        d.dt._mode = "ro";
        const prevented = !fire(d.over, "drop", e, d.dt, true);
        effect = prevented ? d.dt._dropEffect : "none";
      } else if (d.over) fire(d.over, "dragleave", e, d.dt, false);
      d.dt._mode = "protected";
      d.dt._dropEffect = effect;
      fire(d.src, "dragend", e, d.dt, false);
    },
    true,
  );
  // the click that a drag's release would otherwise make
  for (const t of ["mousemove", "mouseup", "click"])
    g.addEventListener(
      t,
      (e: any) => {
        if (drag || (ended && t !== "mousemove")) swallow(e);
      },
      true,
    );
}
