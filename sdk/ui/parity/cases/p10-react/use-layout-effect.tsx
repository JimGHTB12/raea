// useLayoutEffect sets state before the browser paints: the first frame drawn already shows the second render.
// proves: react.hooks.useLayoutEffect
import { useLayoutEffect, useState } from "react";
import { trace } from "parity";
export const steps = [];
export default function Case() {
  const [w, setW] = useState(40);
  useLayoutEffect(() => { trace("layout-effect", "bar"); setW(200); }, []);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <div data-testid="bar" className="h-6 bg-violet-500" style={{ width: w }} />
    </div>
  );
}
