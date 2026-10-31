// useRef: a mutable count kept across renders without causing one; shown when another state changes.
// proves: react.hooks.useRef
import { useRef, useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "bump" }, { click: "bump" }, { click: "show" }];
export default function Case() {
  const r = useRef(0);
  const [shown, setShown] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="bump" className="mr-2 bg-slate-200 px-2" onClick={() => { r.current += 1; trace("bump", "bump", r.current); }}>bump</button>
      <button data-testid="show" className="bg-slate-200 px-2" onClick={() => setShown(r.current)}>show</button>
      <p data-testid="out" data-value={String(shown)}>{shown}</p>
    </div>
  );
}
