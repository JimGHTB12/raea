// The wire between React (react.ts) and the page that turns ops into its document (raea-page's ops.rs applies them to
// Blitz's DOM). Typed and versioned: each batch is `{v, ops}`, and the host refuses a version it doesn't speak.
// Coalescing, the handler registry and the host bindings live here once.
//
// Ops (JS -> host), one batch per flush:
//   clear                              everything under the root goes (a fresh mount)
//   create  {id, kind, props, text?}   an element; `text` is a text element's string
//   append  {parent, child}            child last under parent (moved there if it was elsewhere)
//   insert  {parent, child, before}    child before `before`
//   remove  {id}                       the element and everything in it go
//   update  {id, set?, unset?, text?}  props set (each replaces its whole value, `style` too), props unset, a text
//                                      element's string; at most one per element per batch
//   listen  {id, events}               the element's events start coming back
//   unlisten {id, events}              ...and stop
// Messages (host -> JS), delivered synchronously by `__ui_deliver(json)`:
//   {t: "event", id, event, payload}   an element's event, to its listener
//   {t: "named", name, value}          a host event (resize, scan, agent, keydown, frame, action; `media`: an img's,
//                                      video's or audio's {id, type, w?, h?, duration?, time?}, fired at the element)
//   {t: "answer", id, ok, value|error} the host's answer to a call
//   {t: "ecs", key, value, error?}     a watched ECS read's new answer (src/watch.rs; @raea/ui's ecs.ts)
// Asks (JS -> host): `__ui_emit(name, json)` (send, blur, focus, ticks, actions, actionDone; a host's own), and
// `__ui_call(id, name, json)`, answered later by an `answer`, and `__ui_sync(name, json)`, answered as it returns.

/** The op protocol's version; the host module checks it on every batch (tsx.rs OPS_VERSION). */
export const OPS_VERSION = 1;

/** An element event's name on the wire: the host's own names (DOM's event types), not a driver's prop names. */
export type Ev =
  | "pointerdown"
  | "pointermove"
  | "pointerup"
  | "pointerenter"
  | "pointerleave"
  | "contextmenu"
  | "wheel"
  | "scroll"
  | "change"
  | "keydown"
  | "focus"
  | "blur"
  | "select"
  | "close"
  | "keyup";

/** The handler props both drivers take (`onPointerDown={fn}`) and the event each listens for. */
export const EVENTS: Readonly<Record<string, Ev>> = {
  onPointerDown: "pointerdown",
  onPointerMove: "pointermove",
  onPointerUp: "pointerup",
  onPointerEnter: "pointerenter",
  onPointerLeave: "pointerleave",
  onContextMenu: "contextmenu",
  onWheel: "wheel",
  onScroll: "scroll",
  onChange: "change",
  onKeyDown: "keydown",
  onFocus: "focus",
  onBlur: "blur",
  onSelect: "select",
  onClose: "close",
};

export type Props = Record<string, unknown>;

export type Op =
  | { op: "clear" }
  | { op: "create"; id: number; kind: string; props: Props; text?: string }
  | { op: "append"; parent: number; child: number }
  | { op: "insert"; parent: number; child: number; before: number }
  | { op: "remove"; id: number }
  | { op: "update"; id: number; set?: Props; unset?: string[]; text?: string }
  | { op: "listen"; id: number; events: Ev[] }
  | { op: "unlisten"; id: number; events: Ev[] };

export type Inbound =
  | { t: "event"; id: number; event: Ev; payload: unknown }
  | { t: "named"; name: string; value: unknown }
  | { t: "answer"; id: number; ok: true; value: unknown }
  | { t: "answer"; id: number; ok: false; error: string }
  | { t: "ecs"; key: string; value: unknown; error?: string };

/** What the host gives the JS: the three calls out (installed as globals before the bundle runs). */
type Host = {
  __ui_flush?: (json: string) => void;
  __ui_emit?: (name: string, json: string) => void;
  __ui_call?: (id: number, name: string, json: string) => void;
  __ui_sync?: (name: string, json: string) => string;
  __ui_deliver?: (json: string) => void;
};
const g = globalThis as unknown as Host;

// ---------------------------------------------------------------- the emitter

let pending: Op[] = [];
// an element's update, listen and unlisten within one batch, each one op (merged as they come)
let updates = new Map<number, Extract<Op, { op: "update" }>>();
let listens = new Map<number, Extract<Op, { op: "listen" }>>();
let unlistens = new Map<number, Extract<Op, { op: "unlisten" }>>();
let nextId = 1; // 0 is the root
// ops pushed outside a handler (a call's answer awaited, a timer) go out at the next microtask
let inDispatch = 0;
let scheduled = false;
/** How many ops the JS has written, ever: geometry asks whether the page changed since it was last laid out. */
let written = 0;
export const writes = () => written;
function pend(op: Op) {
  written++;
  pending.push(op);
  if (inDispatch || scheduled) return;
  scheduled = true;
  queueMicrotask(() => {
    scheduled = false;
    if (!inDispatch) flush();
  });
}

/** A fresh wire id (the host maps it to its Entity). */
export const allocId = (): number => nextId++;
/** The root's wire id: what a tree mounts under. */
export const ROOT = 0;

export const clear = () => void pend({ op: "clear" });
export function create(id: number, kind: string, props: Props, text?: string) {
  pend(text === undefined ? { op: "create", id, kind, props } : { op: "create", id, kind, props, text });
}
export const append = (parent: number, child: number) => void pend({ op: "append", parent, child });
export function place(parent: number, child: number, before: number | undefined) {
  pend(before === undefined ? { op: "append", parent, child } : { op: "insert", parent, child, before });
}
export function remove(id: number) {
  pend({ op: "remove", id });
  drop(id);
}

function upd(id: number) {
  let o = updates.get(id);
  if (!o) {
    o = { op: "update", id };
    updates.set(id, o);
    pend(o);
  }
  return o;
}
/** A prop set on an element (merged with its other changes this batch into one update). */
export function set(id: number, k: string, v: unknown) {
  const o = upd(id);
  (o.set ??= {})[k] = v;
  if (o.unset) o.unset = o.unset.filter((x) => x !== k);
}
/** A prop taken off an element. */
export function unset(id: number, k: string) {
  const o = upd(id);
  if (o.set) delete o.set[k];
  (o.unset ??= []).includes(k) || o.unset.push(k);
}
/** A text element's string. */
export const text = (id: number, t: string) => void (upd(id).text = t);

// ---------------------------------------------------------------- listeners (closures stay in JS; names cross)

type Fn = (payload?: unknown) => void;
const handlers = new Map<number, Partial<Record<Ev, Fn>>>();

/** `fn` hears the element's `ev`; the host is told only when the element starts listening for it. */
export function listen(id: number, ev: Ev, fn: Fn) {
  let hs = handlers.get(id);
  if (!hs) handlers.set(id, (hs = {}));
  const had = ev in hs;
  hs[ev] = fn;
  if (had) return;
  const un = unlistens.get(id);
  if (un?.events.includes(ev)) {
    un.events = un.events.filter((e) => e !== ev);
    return;
  }
  let o = listens.get(id);
  if (!o) {
    o = { op: "listen", id, events: [] };
    listens.set(id, o);
    pend(o);
  }
  o.events.push(ev);
}
/** The element stops hearing `ev`. */
export function unlisten(id: number, ev: Ev) {
  const hs = handlers.get(id);
  if (!hs || !(ev in hs)) return;
  delete hs[ev];
  const l = listens.get(id);
  if (l?.events.includes(ev)) {
    l.events = l.events.filter((e) => e !== ev);
    return;
  }
  let o = unlistens.get(id);
  if (!o) {
    o = { op: "unlisten", id, events: [] };
    unlistens.set(id, o);
    pend(o);
  }
  o.events.push(ev);
}
/** An element gone: its listeners with it (the host forgets them as it removes the element). */
export const drop = (id: number) => void handlers.delete(id);

/** The batch so far, sent to the host as one `{v, ops}`; nothing is sent when nothing changed. */
export function flush() {
  if (!pending.length) return;
  const ops = pending.filter((o) => !((o.op === "listen" || o.op === "unlisten") && !o.events.length));
  pending = [];
  updates = new Map();
  listens = new Map();
  unlistens = new Map();
  if (ops.length) g.__ui_flush?.(JSON.stringify({ v: OPS_VERSION, ops }));
}

// ---------------------------------------------------------------- what comes back

const named = new Map<string, Set<(v: any) => void>>();
const calls = new Map<number, { ok: (v: unknown) => void; err: (e: Error) => void }>();
const watched = new Map<string, (value: unknown, error?: string) => void>();
let callId = 1;
/** How the mounted driver runs a handler: inside its batch (the core) or its flushSync (React), its ops flushed. */
let dispatch: (fn: () => void) => void = (fn) => {
  fn();
  flush();
};
export function setDispatch(d: (fn: () => void) => void) {
  dispatch = d;
}

/** A host event (resize, scan, agent, keydown, frame, action); returns the unlisten. */
export function on<T = unknown>(name: string, cb: (v: T) => void): () => void {
  let s = named.get(name);
  if (!s) named.set(name, (s = new Set()));
  s.add(cb);
  return () => void s.delete(cb);
}
/** A BRP read the host runs every frame (src/watch.rs); `cb` hears each answer that differs from the last. Returns
 *  the unwatch. */
export function watch(key: string, method: string, params: unknown, cb: (value: unknown, error?: string) => void): () => void {
  watched.set(key, cb);
  emit("watch", { key, method, params });
  return () => {
    if (watched.get(key) !== cb) return;
    watched.delete(key);
    emit("unwatch", { key });
  };
}
/** An ask of the host (send, blur, focus, ticks, ...). */
export const emit = (name: string, value: unknown = null): void => void g.__ui_emit?.(name, JSON.stringify(value ?? null));
/** A call to the host: settles the frame its answer arrives. */
export function call<T = unknown>(name: string, args: unknown = null): Promise<T> {
  const id = callId++;
  return new Promise<T>((ok, err) => {
    calls.set(id, { ok: ok as (v: unknown) => void, err });
    g.__ui_call?.(id, name, JSON.stringify(args ?? null));
  });
}

/** A host call answered before it returns (the host's `__ui_sync`, raea-runtime-js src/calls.rs), for a web API that returns
 *  at once with what only the host has (a canvas's pixels). Throws the host's error; undefined where there's no host. */
export function callSync<T = unknown>(name: string, args: unknown = null): T | undefined {
  if (typeof g.__ui_sync !== "function") return undefined;
  // what's been written so far goes first, so the host answers from the page as the JS sees it (getComputedStyle)
  flush();
  const r = JSON.parse(g.__ui_sync(name, JSON.stringify(args ?? null))) as { ok?: T; err?: string };
  if (r.err !== undefined) throw new Error(r.err);
  return r.ok;
}

/** One message from the host, run now: a handler's writes and their ops are flushed before this returns. */
export function deliver(m: Inbound) {
  switch (m.t) {
    case "event": {
      const fn = handlers.get(m.id)?.[m.event];
      if (fn) run(() => (m.payload === null || m.payload === undefined ? fn() : fn(m.payload)));
      break;
    }
    case "named": {
      const s = named.get(m.name);
      if (s?.size) run(() => s.forEach((cb) => cb(m.value)));
      break;
    }
    case "ecs": {
      const cb = watched.get(m.key);
      if (cb) run(() => cb(m.value, m.error));
      break;
    }
    case "answer": {
      const c = calls.get(m.id);
      if (!c) break;
      calls.delete(m.id);
      if (m.ok) c.ok(m.value);
      else c.err(new Error(m.error));
      break;
    }
  }
}
function run(fn: () => void) {
  inDispatch++;
  try {
    dispatch(() => {
      try {
        fn();
      } catch (e) {
        console.error(`[ui] a handler threw: ${(e as Error)?.stack ?? e}`);
      }
    });
  } finally {
    inDispatch--;
  }
}
/** Run `fn` as a handler is run (a mount): its ops wait for the flush the driver makes at its end. */
export function quiet<T>(fn: () => T): T {
  inDispatch++;
  try {
    return fn();
  } finally {
    inDispatch--;
  }
}
g.__ui_deliver = (json: string) => deliver(JSON.parse(json));

/** Back to nothing mounted: ids, listeners and the batch (a test's next driver, a fresh mount). */
export function restart() {
  pending = [];
  updates = new Map();
  listens = new Map();
  unlistens = new Map();
  handlers.clear();
  watched.clear();
  nextId = 1;
}
