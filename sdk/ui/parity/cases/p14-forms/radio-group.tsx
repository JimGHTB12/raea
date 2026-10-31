// Radios in a group: choosing one unchooses the others.
// proves: html.elements.input.type_radio, react-dom.controlled.input.checked
import { useState } from "react";
export const steps = [{ click: "r-b" }, { click: "r-c" }];
export default function Case() {
  const [v, setV] = useState("a");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      {["a", "b", "c"].map((x) => (
        <label key={x} className="mb-1 flex items-center gap-2">
          <input data-testid={"r-" + x} type="radio" name="g" className="h-4 w-4" checked={v === x} onChange={() => setV(x)} />
          <span>{x}</span>
        </label>
      ))}
    </div>
  );
}
