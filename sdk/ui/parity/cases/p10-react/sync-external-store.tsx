// useSyncExternalStore: a store outside React, subscribed to; a click writes it.
// proves: react.hooks.useSyncExternalStore
import { useSyncExternalStore } from "react";
let value = 0;
const subs = new Set<() => void>();
const store = {
  subscribe(f: () => void) { subs.add(f); return () => subs.delete(f); },
  get: () => value,
  set(v: number) { value = v; subs.forEach((f) => f()); },
};
export const steps = [{ click: "inc" }, { click: "inc" }];
export default function Case() {
  const v = useSyncExternalStore(store.subscribe, store.get);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="inc" className="bg-slate-200 px-2" onClick={() => store.set(v + 1)}>inc</button>
      <div data-testid="bar" data-value={String(v)} className="mt-2 h-6 bg-orange-400" style={{ width: 40 + v * 40 }} />
    </div>
  );
}
