// A controlled text input: a click focuses it, typed text goes through onChange into state and back.
// proves: html.elements.input.type_text, react-dom.controlled.input.value, react-dom.event.onChange
// tier: quick
import { useState } from "react";
import { trace } from "parity";
export const steps = [{ click: "field" }, { text: "hello" }];
export default function Case() {
  const [v, setV] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="field" type="text" className="h-9 w-56 border border-slate-400 px-2 text-sm" value={v} onChange={(e) => { trace("change", "field", e.target.value); setV(e.target.value); }} />
      <p data-testid="echo" data-value={v}>{v}</p>
    </div>
  );
}
