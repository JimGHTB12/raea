// The effects' order: insertion, layout and passive effects, parent and child, on mount, on update (each cleanup
// before its effect runs again) and on unmount.
// proves: react.hooks.useInsertionEffect, react.hooks.useLayoutEffect, react.hooks.useEffect
import { useEffect, useInsertionEffect, useLayoutEffect, useState } from "react";
import { trace } from "parity";
function Fx({ name, n }: { name: string; n: number }) {
  useInsertionEffect(() => {
    trace("insertion", name, n);
    return () => trace("insertion-cleanup", name, n);
  }, [n]);
  useLayoutEffect(() => {
    trace("layout", name, n);
    return () => trace("layout-cleanup", name, n);
  }, [n]);
  useEffect(() => {
    trace("effect", name, n);
    return () => trace("effect-cleanup", name, n);
  }, [n]);
  return null;
}
function Parent({ n }: { n: number }) {
  return (
    <div data-testid="parent" className="h-10 w-48 bg-sky-200">
      <Fx name="child" n={n} />
      <Fx name="parent" n={n} />
    </div>
  );
}
export const steps = [{ click: "bump" }, { click: "drop" }];
export default function Case() {
  const [n, setN] = useState(0);
  const [on, setOn] = useState(true);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div className="flex gap-2">
        <button data-testid="bump" className="bg-slate-200 px-2" onClick={() => setN(n + 1)}>bump</button>
        <button data-testid="drop" className="bg-slate-200 px-2" onClick={() => setOn(false)}>drop</button>
      </div>
      {on && <Parent n={n} />}
    </div>
  );
}
