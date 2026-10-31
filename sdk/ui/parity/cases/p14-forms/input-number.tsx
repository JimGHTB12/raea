// type=number: digits accepted; the value as a string.
// proves: html.elements.input.type_number
import { useState } from "react";
export const steps = [{ click: "n" }, { text: "42" }];
export default function Case() {
  const [v, setV] = useState("");
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <input data-testid="n" type="number" className="h-9 w-56 border border-slate-400 px-2 text-sm" value={v} onChange={(e) => setV(e.target.value)} />
    </div>
  );
}
