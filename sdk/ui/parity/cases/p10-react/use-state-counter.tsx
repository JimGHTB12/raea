// A counter: useState, a click handler, the count re-rendered as text and as data-value.
// proves: react.hooks.useState, react-dom.event.onClick, react-dom.element.div, react-dom.element.span
// tier: quick
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "inc" }, { click: "inc" }];
export default function Case() {
  const [n, setN] = useState(0);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="inc" className="rounded bg-sky-600 px-3 py-1 text-white" onClick={() => { trace("click", "inc", n); setN(n + 1); }}>add</button>
      <span data-testid="count" data-value={String(n)} className="ml-3">{n}</span>
    </div>
  );
}
