// useEffect with deps: it runs again only when its dependency changes, and its cleanup runs first.
// proves: react.hooks.useEffect
import { useEffect, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "a" }, { click: "b" }, { click: "a" }];
export default function Case() {
  const [a, setA] = useState(0);
  const [b, setB] = useState(0);
  useEffect(() => { trace("effect", "a", a); return () => trace("cleanup", "a", a); }, [a]);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="a" className="mr-2 bg-slate-200 px-2" onClick={() => setA(a + 1)}>a {a}</button>
      <button data-testid="b" className="bg-slate-200 px-2" onClick={() => setB(b + 1)}>b {b}</button>
    </div>
  );
}
