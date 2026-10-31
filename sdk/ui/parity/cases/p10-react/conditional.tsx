// Conditional rendering: a panel shown and hidden by state (&& and a ternary).
// proves: react.hooks.useState
// tier: quick
import { useState } from "react";
export const steps = [{ click: "toggle" }, { click: "toggle" }];
export default function Case() {
  const [open, setOpen] = useState(false);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <button data-testid="toggle" className="bg-slate-200 px-2" onClick={() => setOpen(!open)}>{open ? "close" : "open"}</button>
      {open && <div data-testid="panel" className="mt-2 h-24 w-64 bg-indigo-200" />}
      {open ? <p data-testid="on">shown</p> : <p data-testid="off">hidden</p>}
    </div>
  );
}
