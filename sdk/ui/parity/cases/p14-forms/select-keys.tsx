// A select changed from the keyboard: focused by Tab, a letter picks the option it starts (type-ahead; on macOS an
// arrow key on a closed select opens its menu instead).
// proves: html.elements.select, react-dom.controlled.select.value
import { useState } from "react";
export const steps = [{ key: "Tab" }, { key: "b" }];
export default function Case() {
  const [v, setV] = useState("red");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <select data-testid="sel" className="h-9 w-40 border border-slate-400 text-sm" value={v} onChange={(e) => setV(e.target.value)}>
        <option value="red">red</option>
        <option value="green">green</option>
        <option value="blue">blue</option>
      </select>
    </div>
  );
}
