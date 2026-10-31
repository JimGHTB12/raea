// Hooks, and the render's current component.
import { Context, Instance, S, render } from "./all";

/** function components rendered (or reused) so far: a host subtree that added none holds no component */






// this pass's context reads and rendered components, in render order: an instance's are the run its render added
// (each kept at its size, with its own length: emptied with length = 0, a log's storage was let go and grown again
// every frame, a few KB a frame for a game's hundreds of components)
export const readLog: any[] = [];

export const codeLog: number[] = [];

// Each hook takes its slot first and makes its state only when the slot is new: an init closure handed to a shared
// helper was made at every call, every frame, for every hook of every component (lightspeed Z3)
export function slot(): number {
  if (!S.current) throw new Error("a hook was called outside a component (raea lite)");
  return S.hookIndex++;
}

export function useContext<T>(ctx: Context<T>): T {
  const v = S.contexts.has(ctx._id) ? S.contexts.get(ctx._id) : ctx._default;
  if (S.current) {
    readLog[S.readN++] = ctx;
    readLog[S.readN++] = v;
  }
  return v;
}

interface Cell {
  value: any;
  set: (n: any) => void;
  /** what useState hands back: one pair per cell, its value written each render (a pair is read where it's given) */
  pair: [any, (n: any) => void];
  dispatch?: (a: any) => void;
}

function cell(init: any): Cell {
  const owner = S.current!;
  const c: Cell = { value: typeof init === "function" ? init() : init, set: null as any, pair: null as any };
  c.set = (n: any) => {
    const next = typeof n === "function" ? n(c.value) : n;
    if (!Object.is(next, c.value)) {
      c.value = next;
      S.dirty = true;
      // it and every component above it render again (none may reuse its last output)
      for (let p: Instance | null = owner; p; p = p.parent) p.stale = true;
      if (S.rerender) queueMicrotask(() => S.rerender && S.dirty && S.rerender());
    }
  };
  c.pair = [c.value, c.set];
  return c;
}

export function useState<T>(init: T | (() => T)): [T, (v: T | ((p: T) => T)) => void] {
  const i = slot();
  const h = S.current!.hooks;
  let c: Cell = h[i];
  if (c === undefined) h.push((c = cell(init)));
  c.pair[0] = c.value;
  return c.pair;
}

export function useReducer(reducer: any, init: any, initFn?: any): [any, (a: any) => void] {
  const i = slot();
  const h = S.current!.hooks;
  let c: Cell = h[i];
  if (c === undefined) {
    h.push((c = cell(initFn ? () => initFn(init) : init)));
    c.pair = [c.value, null as any];
  }
  // the dispatch reads the reducer of the render it was last handed out by, as React's does
  c.dispatch = c.dispatch ?? ((a: any) => c.set((p: any) => (c as any).reducer(p, a)));
  (c as any).reducer = reducer;
  c.pair[0] = c.value;
  c.pair[1] = c.dispatch;
  return c.pair;
}

export const same = (a?: any[], b?: any[]) => {
  if (!a || !b || a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (!Object.is(a[i], b[i])) return false;
  return true;
};

interface Memo {
  deps: any[] | undefined;
  value: any;
}

export function useMemo<T>(f: () => T, deps?: any[]): T {
  const i = slot();
  const h = S.current!.hooks;
  let m: Memo = h[i];
  if (m === undefined) h.push((m = { deps, value: f() }));
  else if (!deps || !same(m.deps, deps)) {
    m.value = f();
    m.deps = deps;
  }
  return m.value;
}

export function useCallback<T>(f: T, deps?: any[]): T {
  const i = slot();
  const h = S.current!.hooks;
  let m: Memo = h[i];
  if (m === undefined) h.push((m = { deps, value: f }));
  else if (!deps || !same(m.deps, deps)) {
    m.value = f;
    m.deps = deps;
  }
  return m.value;
}

export function useRef<T>(init: T): { current: T } {
  const i = slot();
  const h = S.current!.hooks;
  let r = h[i];
  if (r === undefined) h.push((r = { current: init }));
  return r;
}

export function useId(): string {
  const i = slot();
  const h = S.current!.hooks;
  if (h.length <= i) h.push(`:r${(S.idCounter++).toString(36)}:`);
  return h[i];
}

interface Fx {
  deps: any[] | undefined;
  init: boolean;
}

function effect(f: () => void | (() => void), deps?: any[]) {
  const inst = S.current!;
  const i = slot();
  let v: Fx = inst.hooks[i];
  if (v === undefined) inst.hooks.push((v = { deps: undefined, init: false }));
  if (!v.init || !deps || !same(v.deps, deps)) {
    v.deps = deps;
    v.init = true;
    inst.effects.push(() => {
      const old = inst.cleanups.get(i);
      if (old) old();
      const c = f();
      if (typeof c === "function") inst.cleanups.set(i, c);
      else inst.cleanups.delete(i);
    });
  }
}

export const useEffect = effect;

export const useLayoutEffect = effect;

export const useInsertionEffect = effect;

export function useSyncExternalStore<T>(subscribe: (cb: () => void) => () => void, get: () => T): T {
  const [, force] = useState(0);
  effect(() => subscribe(() => force((n: number) => n + 1)), [subscribe]);
  return get();
}

export function useImperativeHandle() {}

export function useDebugValue() {}

export function useTransition(): [boolean, (f: () => void) => void] {
  return [false, (f) => f()];
}

export function useDeferredValue<T>(v: T): T {
  return v;
}

export function memo(c: any) {
  return c;
}

export function forwardRef(render: (props: any, ref: any) => any) {
  return (props: any) => render(props, props.ref ?? null);
}

export function startTransition(f: () => void) {
  f();
}

export class Component<P = any, S = any> {
  props: P;
  state!: S;
  constructor(props: P) {
    this.props = props;
  }
  setState() {
    throw new Error("class components aren't in raea lite (live games): write a function component");
  }
}

export const version = "19.2.7-raea-lite";
