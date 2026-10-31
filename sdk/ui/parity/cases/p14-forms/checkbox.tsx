// A controlled checkbox toggled by clicks, and by clicking its label.
// proves: html.elements.input.type_checkbox, react-dom.controlled.input.checked, css.selectors.checked
// tier: quick
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "box" }, { click: "label" }];
export default function Case() {
  const [on, setOn] = useState(false);
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <label data-testid="label" className="flex w-48 items-center gap-2">
        <input data-testid="box" type="checkbox" className="h-5 w-5" checked={on} onChange={(e) => { trace("change", "box", e.target.checked); setOn(e.target.checked); }} />
        <span>agree</span>
      </label>
    </div>
  );
}
